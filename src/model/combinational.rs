use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::{Config, Xlen};
use crate::machine::core::CoreState;
use crate::machine::memory::Memory;
use crate::machine::registers::RegisterFile;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::riscv::compressed;
use crate::riscv::decode;
use crate::riscv::isa::InstrId;

/// Results of combinational logic for one core.
pub struct CombinationalResult {
    /// The decoded instruction ID
    pub instruction_id: NodeId,
    /// The value to write to rd (register data flow)
    pub rd_value: NodeId,
    /// The rd address (which register to write)
    pub rd_addr: NodeId,
    /// Next PC value (control flow)
    pub next_pc: NodeId,
    /// Memory write address (if store)
    pub store_addr: NodeId,
    /// Memory write value (if store)
    pub store_value: NodeId,
    /// Memory write width (1/2/4/8, as constant node)
    pub store_width: NodeId,
    /// Whether this instruction writes to a register
    pub writes_rd: NodeId,
    /// Whether this instruction writes to memory
    pub writes_memory: NodeId,
    /// Whether this instruction is an ecall
    pub is_ecall: NodeId,
    /// Fetched instruction register
    pub ir: NodeId,
    /// Is this a compressed instruction?
    pub is_compressed: NodeId,
    /// Various condition flags for properties
    pub division_by_zero: NodeId,
    pub invalid_address: NodeId,
}

/// Generate the combinational logic for one core:
/// fetch instruction, decode, compute data flow (rd value) and control flow (next PC).
pub fn rotor_combinational(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    core: &CoreState,
) -> CombinationalResult {
    let bool_sid = sorts.sid_boolean;
    let mw_sid = sorts.sid_machine_word;

    // ===== FETCH =====
    // Read instruction from code segment at current PC
    let ir = builder.read(
        sorts.sid_code_word,
        core.code_segment_state,
        core.pc_state,
        Some("fetch instruction".to_string()),
    );

    // Check if compressed instruction (bits [1:0] != 0b11)
    let low2 = builder.slice(sorts.sid_2bit, ir, 1, 0, None);
    let three = builder.constd(sorts.sid_2bit, 3, None);
    let is_full = builder.eq_node(bool_sid, low2, three, None);
    let is_compressed = builder.not(bool_sid, is_full, Some("is compressed?".to_string()));

    // Extract compressed instruction (lower 16 bits)
    let c_ir = builder.slice(sorts.sid_half_word, ir, 15, 0, Some("compressed IR".to_string()));

    // ===== DECODE =====
    let full_instr_id = decode::decode_instruction(builder, sorts, consts, config, ir);

    // Decode compressed if C extension enabled
    let instruction_id = if config.enable_c {
        let c_instr_id = compressed::decode_compressed(builder, sorts, consts, c_ir);
        builder.ite(
            sorts.sid_instruction_id,
            is_compressed,
            c_instr_id,
            full_instr_id,
            Some("select instruction ID".to_string()),
        )
    } else {
        full_instr_id
    };

    // ===== REGISTER READ =====
    let rd_addr = decode::extract_rd(builder, sorts, ir);
    let rs1_addr = decode::extract_rs1(builder, sorts, ir);
    let rs2_addr = decode::extract_rs2(builder, sorts, ir);

    let rs1_val = RegisterFile::load_register(builder, sorts, core.register_file_state, rs1_addr, Some("rs1 value".to_string()));
    let rs2_val = RegisterFile::load_register(builder, sorts, core.register_file_state, rs2_addr, Some("rs2 value".to_string()));

    // ===== IMMEDIATES =====
    let i_imm = decode::extract_i_imm(builder, sorts, config, ir);
    let s_imm = decode::extract_s_imm(builder, sorts, config, ir);
    let sb_imm = decode::extract_sb_imm(builder, sorts, consts, config, ir);
    let u_imm = decode::extract_u_imm(builder, sorts, config, ir);
    let uj_imm = decode::extract_uj_imm(builder, sorts, consts, config, ir);

    // ===== DATA FLOW (compute rd value) =====
    let _unknown_id = consts.nid_instr_id(InstrId::Unknown);
    let pc_plus_4 = builder.add(mw_sid, core.pc_state, consts.nid_instruction_size, Some("PC + 4".to_string()));
    let pc_plus_2 = builder.add(mw_sid, core.pc_state, consts.nid_compressed_instruction_size, Some("PC + 2".to_string()));
    let next_seq_pc = if config.enable_c {
        builder.ite(mw_sid, is_compressed, pc_plus_2, pc_plus_4, Some("next sequential PC".to_string()))
    } else {
        pc_plus_4
    };

    // Compute address for loads/stores: rs1 + I-immediate (or S-immediate for stores)
    let load_addr = builder.add(mw_sid, rs1_val, i_imm, Some("rs1 + I-imm (load addr)".to_string()));
    let store_addr = builder.add(mw_sid, rs1_val, s_imm, Some("rs1 + S-imm (store addr)".to_string()));

    // Select the appropriate memory segment for load address
    let load_memory = core.segmentation.select_segment(
        builder,
        sorts,
        load_addr,
        core.data_segment_state,
        core.heap_segment_state,
        core.stack_segment_state,
        sorts.sid_data_state, // Using data_state sort as representative array sort
    );

    // ===== ALU OPERATIONS =====
    // R-type results
    let add_result = builder.add(mw_sid, rs1_val, rs2_val, None);
    let sub_result = builder.sub(mw_sid, rs1_val, rs2_val, None);
    let sll_result = builder.sll(mw_sid, rs1_val, rs2_val, None);
    let slt_result = {
        let cmp = builder.slt(bool_sid, rs1_val, rs2_val, None);
        builder.ite(mw_sid, cmp, consts.nid_machine_word_1, consts.nid_machine_word_0, Some("slt result".to_string()))
    };
    let sltu_result = {
        let cmp = builder.ult(bool_sid, rs1_val, rs2_val, None);
        builder.ite(mw_sid, cmp, consts.nid_machine_word_1, consts.nid_machine_word_0, Some("sltu result".to_string()))
    };
    let xor_result = builder.xor_node(mw_sid, rs1_val, rs2_val, None);
    let srl_result = builder.srl(mw_sid, rs1_val, rs2_val, None);
    let sra_result = builder.sra(mw_sid, rs1_val, rs2_val, None);
    let or_result = builder.or_node(mw_sid, rs1_val, rs2_val, None);
    let and_result = builder.and_node(mw_sid, rs1_val, rs2_val, None);

    // I-type immediate results
    let addi_result = builder.add(mw_sid, rs1_val, i_imm, None);
    let slti_result = {
        let cmp = builder.slt(bool_sid, rs1_val, i_imm, None);
        builder.ite(mw_sid, cmp, consts.nid_machine_word_1, consts.nid_machine_word_0, None)
    };
    let sltiu_result = {
        let cmp = builder.ult(bool_sid, rs1_val, i_imm, None);
        builder.ite(mw_sid, cmp, consts.nid_machine_word_1, consts.nid_machine_word_0, None)
    };
    let xori_result = builder.xor_node(mw_sid, rs1_val, i_imm, None);
    let ori_result = builder.or_node(mw_sid, rs1_val, i_imm, None);
    let andi_result = builder.and_node(mw_sid, rs1_val, i_imm, None);

    // Shift immediate - extract shamt from I-immediate
    let shamt_mask = if config.xlen == Xlen::X64 {
        builder.constd(mw_sid, 0x3F, None) // 6-bit shamt for RV64
    } else {
        builder.constd(mw_sid, 0x1F, None) // 5-bit shamt for RV32
    };
    let shamt = builder.and_node(mw_sid, i_imm, shamt_mask, Some("shift amount".to_string()));
    let slli_result = builder.sll(mw_sid, rs1_val, shamt, None);
    let srli_result = builder.srl(mw_sid, rs1_val, shamt, None);
    let srai_result = builder.sra(mw_sid, rs1_val, shamt, None);

    // LUI: upper immediate
    let lui_result = u_imm;

    // AUIPC: PC + upper immediate
    let auipc_result = builder.add(mw_sid, core.pc_state, u_imm, Some("PC + U-imm".to_string()));

    // JAL/JALR: store return address (next_seq_pc)
    let jal_rd_value = next_seq_pc;
    let jalr_rd_value = next_seq_pc;

    // Load results - build for each load type
    let lb_result = Memory::load_value(builder, sorts, consts, load_memory, load_addr, 1, true, config);
    let lbu_result = Memory::load_value(builder, sorts, consts, load_memory, load_addr, 1, false, config);
    let lh_result = Memory::load_value(builder, sorts, consts, load_memory, load_addr, 2, true, config);
    let lhu_result = Memory::load_value(builder, sorts, consts, load_memory, load_addr, 2, false, config);
    let lw_result = Memory::load_value(builder, sorts, consts, load_memory, load_addr, 4, true, config);

    let lwu_result = if config.xlen == Xlen::X64 {
        Memory::load_value(builder, sorts, consts, load_memory, load_addr, 4, false, config)
    } else {
        consts.nid_machine_word_0
    };

    let ld_result = if config.xlen == Xlen::X64 {
        Memory::load_value(builder, sorts, consts, load_memory, load_addr, 8, false, config)
    } else {
        consts.nid_machine_word_0
    };

    // M extension results
    let mul_result = builder.mul(mw_sid, rs1_val, rs2_val, None);
    let div_result = builder.sdiv(mw_sid, rs1_val, rs2_val, None);
    let divu_result = builder.udiv(mw_sid, rs1_val, rs2_val, None);
    let rem_result = builder.srem(mw_sid, rs1_val, rs2_val, None);
    let remu_result = builder.urem(mw_sid, rs1_val, rs2_val, None);

    // Division by zero check
    let rs2_is_zero = builder.eq_node(bool_sid, rs2_val, consts.nid_machine_word_0, None);
    let is_div = {
        let id_div = consts.nid_instr_id(InstrId::Div);
        let id_divu = consts.nid_instr_id(InstrId::Divu);
        let id_rem = consts.nid_instr_id(InstrId::Rem);
        let id_remu = consts.nid_instr_id(InstrId::Remu);
        let d1 = builder.eq_node(bool_sid, instruction_id, id_div, None);
        let d2 = builder.eq_node(bool_sid, instruction_id, id_divu, None);
        let d3 = builder.eq_node(bool_sid, instruction_id, id_rem, None);
        let d4 = builder.eq_node(bool_sid, instruction_id, id_remu, None);
        let a = builder.or_node(bool_sid, d1, d2, None);
        let b = builder.or_node(bool_sid, d3, d4, None);
        builder.or_node(bool_sid, a, b, Some("is div/rem instruction?".to_string()))
    };
    let division_by_zero = builder.and_node(bool_sid, is_div, rs2_is_zero, Some("division by zero?".to_string()));

    // Invalid address check for loads/stores
    let load_valid = core.segmentation.is_valid_read_address(builder, sorts, load_addr);
    let store_valid = core.segmentation.is_valid_write_address(builder, sorts, store_addr);
    let is_load_instr = {
        let ids = [InstrId::Lb, InstrId::Lh, InstrId::Lw, InstrId::Lbu, InstrId::Lhu];
        let mut acc = consts.nid_false;
        for id in ids {
            let eq = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
            acc = builder.or_node(bool_sid, acc, eq, None);
        }
        if config.xlen == Xlen::X64 {
            let ld = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Ld), None);
            let lwu = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Lwu), None);
            acc = builder.or_node(bool_sid, acc, ld, None);
            acc = builder.or_node(bool_sid, acc, lwu, None);
        }
        acc
    };
    let is_store_instr = {
        let ids = [InstrId::Sb, InstrId::Sh, InstrId::Sw];
        let mut acc = consts.nid_false;
        for id in ids {
            let eq = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
            acc = builder.or_node(bool_sid, acc, eq, None);
        }
        if config.xlen == Xlen::X64 {
            let sd = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Sd), None);
            acc = builder.or_node(bool_sid, acc, sd, None);
        }
        acc
    };
    let not_load_valid = builder.not(bool_sid, load_valid, None);
    let load_invalid = builder.and_node(bool_sid, is_load_instr, not_load_valid, None);
    let not_store_valid = builder.not(bool_sid, store_valid, None);
    let store_invalid = builder.and_node(bool_sid, is_store_instr, not_store_valid, None);
    let invalid_address = builder.or_node(bool_sid, load_invalid, store_invalid, Some("invalid memory access?".to_string()));

    // ===== BUILD RD VALUE ITE CHAIN =====
    let mut rd_value = consts.nid_machine_word_0;

    // Loads
    if config.xlen == Xlen::X64 {
        let is_ld = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Ld), None);
        rd_value = builder.ite(mw_sid, is_ld, ld_result, rd_value, None);
        let is_lwu = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Lwu), None);
        rd_value = builder.ite(mw_sid, is_lwu, lwu_result, rd_value, None);
    }
    let load_instrs = [
        (InstrId::Lw, lw_result),
        (InstrId::Lhu, lhu_result),
        (InstrId::Lh, lh_result),
        (InstrId::Lbu, lbu_result),
        (InstrId::Lb, lb_result),
    ];
    for (id, val) in load_instrs {
        let cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
        rd_value = builder.ite(mw_sid, cond, val, rd_value, None);
    }

    // M extension
    if config.enable_m {
        let m_instrs = [
            (InstrId::Remu, remu_result),
            (InstrId::Rem, rem_result),
            (InstrId::Divu, divu_result),
            (InstrId::Div, div_result),
            (InstrId::Mul, mul_result),
        ];
        for (id, val) in m_instrs {
            let cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
            rd_value = builder.ite(mw_sid, cond, val, rd_value, None);
        }
    }

    // R-type
    let r_instrs = [
        (InstrId::And, and_result),
        (InstrId::Or, or_result),
        (InstrId::Sra, sra_result),
        (InstrId::Srl, srl_result),
        (InstrId::Xor, xor_result),
        (InstrId::Sltu, sltu_result),
        (InstrId::Slt, slt_result),
        (InstrId::Sll, sll_result),
        (InstrId::Sub, sub_result),
        (InstrId::Add, add_result),
    ];
    for (id, val) in r_instrs {
        let cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
        rd_value = builder.ite(mw_sid, cond, val, rd_value, None);
    }

    // I-type immediates
    let i_instrs = [
        (InstrId::Srai, srai_result),
        (InstrId::Srli, srli_result),
        (InstrId::Slli, slli_result),
        (InstrId::Andi, andi_result),
        (InstrId::Ori, ori_result),
        (InstrId::Xori, xori_result),
        (InstrId::Sltiu, sltiu_result),
        (InstrId::Slti, slti_result),
        (InstrId::Addi, addi_result),
    ];
    for (id, val) in i_instrs {
        let cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
        rd_value = builder.ite(mw_sid, cond, val, rd_value, None);
    }

    // U-type
    let lui_cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Lui), None);
    rd_value = builder.ite(mw_sid, lui_cond, lui_result, rd_value, None);
    let auipc_cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Auipc), None);
    rd_value = builder.ite(mw_sid, auipc_cond, auipc_result, rd_value, None);

    // JAL / JALR (write return address to rd)
    let jal_cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Jal), None);
    rd_value = builder.ite(mw_sid, jal_cond, jal_rd_value, rd_value, None);
    let jalr_cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Jalr), None);
    rd_value = builder.ite(mw_sid, jalr_cond, jalr_rd_value, rd_value, None);

    // ===== CONTROL FLOW (compute next PC) =====
    // Branch conditions
    let beq_cond = builder.eq_node(bool_sid, rs1_val, rs2_val, None);
    let bne_cond = builder.neq(bool_sid, rs1_val, rs2_val, None);
    let blt_cond = builder.slt(bool_sid, rs1_val, rs2_val, None);
    let bge_cond = builder.sgte(bool_sid, rs1_val, rs2_val, None);
    let bltu_cond = builder.ult(bool_sid, rs1_val, rs2_val, None);
    let bgeu_cond = builder.ugte(bool_sid, rs1_val, rs2_val, None);

    let branch_target = builder.add(mw_sid, core.pc_state, sb_imm, Some("branch target".to_string()));
    let jal_target = builder.add(mw_sid, core.pc_state, uj_imm, Some("JAL target".to_string()));
    let jalr_target = {
        let raw = builder.add(mw_sid, rs1_val, i_imm, None);
        // Clear LSB per JALR spec
        let mask = builder.constd(mw_sid, !1u64, None);
        builder.and_node(mw_sid, raw, mask, Some("JALR target (LSB cleared)".to_string()))
    };

    // Build next_pc ITE chain
    let mut next_pc = next_seq_pc;

    // Branches
    let branch_instrs = [
        (InstrId::Bgeu, bgeu_cond),
        (InstrId::Bltu, bltu_cond),
        (InstrId::Bge, bge_cond),
        (InstrId::Blt, blt_cond),
        (InstrId::Bne, bne_cond),
        (InstrId::Beq, beq_cond),
    ];
    for (id, cond) in branch_instrs {
        let is_this = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
        let taken = builder.and_node(bool_sid, is_this, cond, None);
        next_pc = builder.ite(mw_sid, taken, branch_target, next_pc, None);
    }

    // JAL
    let is_jal = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Jal), None);
    next_pc = builder.ite(mw_sid, is_jal, jal_target, next_pc, None);

    // JALR
    let is_jalr = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Jalr), None);
    next_pc = builder.ite(mw_sid, is_jalr, jalr_target, next_pc, None);

    // ===== WRITES FLAGS =====
    // Does this instruction write to rd?
    let writes_rd = {
        // Everything except branches, stores, and ecall writes to rd
        let _not_branch = builder.not(bool_sid, is_load_instr, None); // placeholder
        // Simpler: stores and branches don't write rd
        let no_write = builder.or_node(bool_sid, is_store_instr, consts.nid_false, None);
        let is_branch_any = {
            let mut acc = consts.nid_false;
            for id in [InstrId::Beq, InstrId::Bne, InstrId::Blt, InstrId::Bge, InstrId::Bltu, InstrId::Bgeu] {
                let eq = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
                acc = builder.or_node(bool_sid, acc, eq, None);
            }
            acc
        };
        let is_ecall_node = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Ecall), None);
        let no_rd = builder.or_node(bool_sid, no_write, is_branch_any, None);
        let no_rd = builder.or_node(bool_sid, no_rd, is_ecall_node, None);
        builder.not(bool_sid, no_rd, Some("instruction writes rd?".to_string()))
    };

    let is_ecall = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Ecall), Some("is ecall?".to_string()));

    // Store width (as machine word for convenience)
    let store_width = {
        let w1 = consts.nid_machine_word_1;
        let w2 = consts.nid_machine_word_2;
        let w4 = consts.nid_machine_word_4;
        let w8 = consts.nid_machine_word_8;
        let mut sw = consts.nid_machine_word_0;
        let sb_cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Sb), None);
        sw = builder.ite(mw_sid, sb_cond, w1, sw, None);
        let sh_cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Sh), None);
        sw = builder.ite(mw_sid, sh_cond, w2, sw, None);
        let sw_cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Sw), None);
        sw = builder.ite(mw_sid, sw_cond, w4, sw, None);
        if config.xlen == Xlen::X64 {
            let sd_cond = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(InstrId::Sd), None);
            sw = builder.ite(mw_sid, sd_cond, w8, sw, None);
        }
        sw
    };

    let store_value = rs2_val;

    CombinationalResult {
        instruction_id,
        rd_value,
        rd_addr,
        next_pc,
        store_addr,
        store_value,
        store_width,
        writes_rd,
        writes_memory: is_store_instr,
        is_ecall,
        ir,
        is_compressed,
        division_by_zero,
        invalid_address,
    }
}
