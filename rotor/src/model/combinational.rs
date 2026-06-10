use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::{Config, Xlen};
use crate::machine::core::CoreState;
use crate::machine::kernel::KernelState;
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
    /// Memory load address (rs1 + I-imm)
    pub load_addr: NodeId,
    /// Whether this instruction is a load
    pub is_load: NodeId,
    /// Memory write address (rs1 + S-imm)
    pub store_addr: NodeId,
    /// Memory write value (if store)
    pub store_value: NodeId,
    /// Memory write width (1/2/4/8, as constant node)
    pub store_width: NodeId,
    /// Whether this instruction writes to a register
    pub writes_rd: NodeId,
    /// Whether this instruction writes to memory (alias for is_store)
    pub writes_memory: NodeId,
    /// Whether this instruction is an ecall
    pub is_ecall: NodeId,
    /// Fetched instruction register
    pub ir: NodeId,
    /// Is this a compressed instruction?
    pub is_compressed: NodeId,
    /// Whether the decoder treats this instruction as Unknown
    pub is_unknown_instruction: NodeId,
    /// Whether the compressed decoder produced Unknown (only meaningful when
    /// is_compressed is true; always false when the C extension is disabled).
    pub is_unknown_compressed: NodeId,
    /// Whether this is a compressed load (CLw/CLd/CLwsp/CLdsp).
    pub is_compressed_load: NodeId,
    /// Whether this is a compressed store (CSw/CSd/CSwsp/CSdsp).
    pub is_compressed_store: NodeId,
    /// Various per-property condition flags (see properties.rs)
    pub division_by_zero: NodeId,
    pub signed_division_overflow: NodeId,
    /// Generic load/store-validity flag — true iff a load OR store this step
    /// targets an address outside the valid read/write segments.
    pub invalid_address: NodeId,
    /// Granular load-side address invalidity (active only on loads).
    pub load_invalid_address: NodeId,
    /// Granular store-side address invalidity (active only on stores).
    pub store_invalid_address: NodeId,
    /// All combinational kernel/syscall signals for this step
    /// (C rotor's kernel_combinational).
    pub kernel: crate::machine::kernel::KernelFlows,
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
    let c_ir = builder.slice(
        sorts.sid_half_word,
        ir,
        15,
        0,
        Some("compressed IR".to_string()),
    );

    // ===== DECODE =====
    let full_instr_id = decode::decode_instruction(builder, sorts, consts, config, ir);

    // Decode compressed if C extension enabled. We keep c_instr_id around so
    // properties.rs can flag illegal-compressed-instruction precisely.
    let c_instr_id = if config.enable_c {
        compressed::decode_compressed(builder, sorts, consts, c_ir)
    } else {
        // When C ext is off, treat the "compressed id" as Unknown — never used
        // because the is_compressed_* flags below are gated on config.enable_c.
        consts.nid_instr_id(InstrId::Unknown)
    };
    let instruction_id = if config.enable_c {
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

    let rs1_val = RegisterFile::load_register(
        builder,
        sorts,
        core.register_file_state,
        rs1_addr,
        Some("rs1 value".to_string()),
    );
    let rs2_val = RegisterFile::load_register(
        builder,
        sorts,
        core.register_file_state,
        rs2_addr,
        Some("rs2 value".to_string()),
    );

    // ===== IMMEDIATES =====
    let i_imm = decode::extract_i_imm(builder, sorts, config, ir);
    let s_imm = decode::extract_s_imm(builder, sorts, config, ir);
    let sb_imm = decode::extract_sb_imm(builder, sorts, consts, config, ir);
    let u_imm = decode::extract_u_imm(builder, sorts, config, ir);
    let uj_imm = decode::extract_uj_imm(builder, sorts, consts, config, ir);

    // ===== DATA FLOW (compute rd value) =====
    let _unknown_id = consts.nid_instr_id(InstrId::Unknown);
    let pc_plus_4 = builder.add(
        mw_sid,
        core.pc_state,
        consts.nid_instruction_size,
        Some("PC + 4".to_string()),
    );
    let pc_plus_2 = builder.add(
        mw_sid,
        core.pc_state,
        consts.nid_compressed_instruction_size,
        Some("PC + 2".to_string()),
    );
    let next_seq_pc = if config.enable_c {
        builder.ite(
            mw_sid,
            is_compressed,
            pc_plus_2,
            pc_plus_4,
            Some("next sequential PC".to_string()),
        )
    } else {
        pc_plus_4
    };

    // Compute address for loads/stores: rs1 + I-immediate (or S-immediate for stores)
    let load_addr = builder.add(
        mw_sid,
        rs1_val,
        i_imm,
        Some("rs1 + I-imm (load addr)".to_string()),
    );
    let store_addr = builder.add(
        mw_sid,
        rs1_val,
        s_imm,
        Some("rs1 + S-imm (store addr)".to_string()),
    );

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
        builder.ite(
            mw_sid,
            cmp,
            consts.nid_machine_word_1,
            consts.nid_machine_word_0,
            Some("slt result".to_string()),
        )
    };
    let sltu_result = {
        let cmp = builder.ult(bool_sid, rs1_val, rs2_val, None);
        builder.ite(
            mw_sid,
            cmp,
            consts.nid_machine_word_1,
            consts.nid_machine_word_0,
            Some("sltu result".to_string()),
        )
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
        builder.ite(
            mw_sid,
            cmp,
            consts.nid_machine_word_1,
            consts.nid_machine_word_0,
            None,
        )
    };
    let sltiu_result = {
        let cmp = builder.ult(bool_sid, rs1_val, i_imm, None);
        builder.ite(
            mw_sid,
            cmp,
            consts.nid_machine_word_1,
            consts.nid_machine_word_0,
            None,
        )
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
    let lb_result = Memory::load_value(
        builder,
        sorts,
        consts,
        load_memory,
        load_addr,
        1,
        true,
        config,
    );
    let lbu_result = Memory::load_value(
        builder,
        sorts,
        consts,
        load_memory,
        load_addr,
        1,
        false,
        config,
    );
    let lh_result = Memory::load_value(
        builder,
        sorts,
        consts,
        load_memory,
        load_addr,
        2,
        true,
        config,
    );
    let lhu_result = Memory::load_value(
        builder,
        sorts,
        consts,
        load_memory,
        load_addr,
        2,
        false,
        config,
    );
    let lw_result = Memory::load_value(
        builder,
        sorts,
        consts,
        load_memory,
        load_addr,
        4,
        true,
        config,
    );

    let lwu_result = if config.xlen == Xlen::X64 {
        Memory::load_value(
            builder,
            sorts,
            consts,
            load_memory,
            load_addr,
            4,
            false,
            config,
        )
    } else {
        consts.nid_machine_word_0
    };

    let ld_result = if config.xlen == Xlen::X64 {
        Memory::load_value(
            builder,
            sorts,
            consts,
            load_memory,
            load_addr,
            8,
            false,
            config,
        )
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
    let division_by_zero = builder.and_node(
        bool_sid,
        is_div,
        rs2_is_zero,
        Some("division by zero?".to_string()),
    );

    // Invalid address check for loads/stores
    let load_valid = core
        .segmentation
        .is_valid_read_address(builder, sorts, load_addr);
    let store_valid = core
        .segmentation
        .is_valid_write_address(builder, sorts, store_addr);
    let is_load_instr = {
        let ids = [
            InstrId::Lb,
            InstrId::Lh,
            InstrId::Lw,
            InstrId::Lbu,
            InstrId::Lhu,
        ];
        let mut acc = consts.nid_false;
        for id in ids {
            let eq = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
            acc = builder.or_node(bool_sid, acc, eq, None);
        }
        if config.xlen == Xlen::X64 {
            let ld = builder.eq_node(
                bool_sid,
                instruction_id,
                consts.nid_instr_id(InstrId::Ld),
                None,
            );
            let lwu = builder.eq_node(
                bool_sid,
                instruction_id,
                consts.nid_instr_id(InstrId::Lwu),
                None,
            );
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
            let sd = builder.eq_node(
                bool_sid,
                instruction_id,
                consts.nid_instr_id(InstrId::Sd),
                None,
            );
            acc = builder.or_node(bool_sid, acc, sd, None);
        }
        acc
    };
    let not_load_valid = builder.not(bool_sid, load_valid, None);
    let load_invalid_address = builder.and_node(
        bool_sid,
        is_load_instr,
        not_load_valid,
        Some("load at invalid address?".to_string()),
    );
    let not_store_valid = builder.not(bool_sid, store_valid, None);
    let store_invalid_address = builder.and_node(
        bool_sid,
        is_store_instr,
        not_store_valid,
        Some("store at invalid address?".to_string()),
    );
    let invalid_address = builder.or_node(
        bool_sid,
        load_invalid_address,
        store_invalid_address,
        Some("invalid memory access?".to_string()),
    );

    // Signed division overflow: rs1 == INT_MIN AND rs2 == -1 AND (DIV or REM)
    let int_min_val: u64 = if config.xlen == Xlen::X64 {
        0x8000_0000_0000_0000
    } else {
        0x8000_0000
    };
    let int_min = builder.constd(mw_sid, int_min_val, Some("INT_MIN".to_string()));
    let minus_one_val: u64 = if config.xlen == Xlen::X64 { !0u64 } else { 0xFFFF_FFFF };
    let minus_one = builder.constd(mw_sid, minus_one_val, Some("-1".to_string()));
    let rs1_is_int_min = builder.eq_node(bool_sid, rs1_val, int_min, None);
    let rs2_is_minus_one = builder.eq_node(bool_sid, rs2_val, minus_one, None);
    let is_signed_div = {
        let id_div = consts.nid_instr_id(InstrId::Div);
        let id_rem = consts.nid_instr_id(InstrId::Rem);
        let a = builder.eq_node(bool_sid, instruction_id, id_div, None);
        let b = builder.eq_node(bool_sid, instruction_id, id_rem, None);
        builder.or_node(bool_sid, a, b, Some("is signed div/rem?".to_string()))
    };
    let overflow_args = builder.and_node(bool_sid, rs1_is_int_min, rs2_is_minus_one, None);
    let signed_division_overflow = builder.and_node(
        bool_sid,
        is_signed_div,
        overflow_args,
        Some("signed division overflow?".to_string()),
    );

    // Unknown / illegal instruction detection
    let unknown_id = consts.nid_instr_id(InstrId::Unknown);
    let is_unknown_instruction = builder.eq_node(
        bool_sid,
        instruction_id,
        unknown_id,
        Some("decoder returned Unknown".to_string()),
    );

    // Compressed-decoder-specific Unknown — only meaningful when C ext is on
    // and the current instruction is in compressed form.
    let is_unknown_compressed = if config.enable_c {
        let c_unknown = builder.eq_node(bool_sid, c_instr_id, unknown_id, None);
        builder.and_node(
            bool_sid,
            is_compressed,
            c_unknown,
            Some("compressed decoder returned Unknown".to_string()),
        )
    } else {
        consts.nid_false
    };

    // Identify compressed loads and stores so properties.rs can emit per-instruction
    // address-validity bad nodes for them. The C extension splits loads into
    // {CLw, CLd, CLwsp, CLdsp} and stores into {CSw, CSd, CSwsp, CSdsp}.
    let is_compressed_load = if config.enable_c {
        let ids = [
            InstrId::CLw, InstrId::CLd, InstrId::CLwsp, InstrId::CLdsp,
        ];
        let mut acc = consts.nid_false;
        for id in ids {
            let eq = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
            acc = builder.or_node(bool_sid, acc, eq, None);
        }
        builder.and_node(bool_sid, is_compressed, acc, Some("compressed load?".to_string()))
    } else {
        consts.nid_false
    };
    let is_compressed_store = if config.enable_c {
        let ids = [
            InstrId::CSw, InstrId::CSd, InstrId::CSwsp, InstrId::CSdsp,
        ];
        let mut acc = consts.nid_false;
        for id in ids {
            let eq = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
            acc = builder.or_node(bool_sid, acc, eq, None);
        }
        builder.and_node(bool_sid, is_compressed, acc, Some("compressed store?".to_string()))
    } else {
        consts.nid_false
    };

    // ===== BUILD RD VALUE ITE CHAIN =====
    let mut rd_value = consts.nid_machine_word_0;

    // Loads
    if config.xlen == Xlen::X64 {
        let is_ld = builder.eq_node(
            bool_sid,
            instruction_id,
            consts.nid_instr_id(InstrId::Ld),
            None,
        );
        rd_value = builder.ite(mw_sid, is_ld, ld_result, rd_value, None);
        let is_lwu = builder.eq_node(
            bool_sid,
            instruction_id,
            consts.nid_instr_id(InstrId::Lwu),
            None,
        );
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
    let lui_cond = builder.eq_node(
        bool_sid,
        instruction_id,
        consts.nid_instr_id(InstrId::Lui),
        None,
    );
    rd_value = builder.ite(mw_sid, lui_cond, lui_result, rd_value, None);
    let auipc_cond = builder.eq_node(
        bool_sid,
        instruction_id,
        consts.nid_instr_id(InstrId::Auipc),
        None,
    );
    rd_value = builder.ite(mw_sid, auipc_cond, auipc_result, rd_value, None);

    // JAL / JALR (write return address to rd)
    let jal_cond = builder.eq_node(
        bool_sid,
        instruction_id,
        consts.nid_instr_id(InstrId::Jal),
        None,
    );
    rd_value = builder.ite(mw_sid, jal_cond, jal_rd_value, rd_value, None);
    let jalr_cond = builder.eq_node(
        bool_sid,
        instruction_id,
        consts.nid_instr_id(InstrId::Jalr),
        None,
    );
    rd_value = builder.ite(mw_sid, jalr_cond, jalr_rd_value, rd_value, None);

    // ===== CONTROL FLOW (compute next PC) =====
    // Branch conditions
    let beq_cond = builder.eq_node(bool_sid, rs1_val, rs2_val, None);
    let bne_cond = builder.neq(bool_sid, rs1_val, rs2_val, None);
    let blt_cond = builder.slt(bool_sid, rs1_val, rs2_val, None);
    let bge_cond = builder.sgte(bool_sid, rs1_val, rs2_val, None);
    let bltu_cond = builder.ult(bool_sid, rs1_val, rs2_val, None);
    let bgeu_cond = builder.ugte(bool_sid, rs1_val, rs2_val, None);

    let branch_target = builder.add(
        mw_sid,
        core.pc_state,
        sb_imm,
        Some("branch target".to_string()),
    );
    let jal_target = builder.add(
        mw_sid,
        core.pc_state,
        uj_imm,
        Some("JAL target".to_string()),
    );
    let jalr_target = {
        let raw = builder.add(mw_sid, rs1_val, i_imm, None);
        // Clear LSB per JALR spec
        let mask = builder.constd(mw_sid, !1u64, None);
        builder.and_node(
            mw_sid,
            raw,
            mask,
            Some("JALR target (LSB cleared)".to_string()),
        )
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
    let is_jal = builder.eq_node(
        bool_sid,
        instruction_id,
        consts.nid_instr_id(InstrId::Jal),
        None,
    );
    next_pc = builder.ite(mw_sid, is_jal, jal_target, next_pc, None);

    // JALR
    let is_jalr = builder.eq_node(
        bool_sid,
        instruction_id,
        consts.nid_instr_id(InstrId::Jalr),
        None,
    );
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
            for id in [
                InstrId::Beq,
                InstrId::Bne,
                InstrId::Blt,
                InstrId::Bge,
                InstrId::Bltu,
                InstrId::Bgeu,
            ] {
                let eq = builder.eq_node(bool_sid, instruction_id, consts.nid_instr_id(id), None);
                acc = builder.or_node(bool_sid, acc, eq, None);
            }
            acc
        };
        let is_ecall_node = builder.eq_node(
            bool_sid,
            instruction_id,
            consts.nid_instr_id(InstrId::Ecall),
            None,
        );
        let no_rd = builder.or_node(bool_sid, no_write, is_branch_any, None);
        let no_rd = builder.or_node(bool_sid, no_rd, is_ecall_node, None);
        builder.not(bool_sid, no_rd, Some("instruction writes rd?".to_string()))
    };

    let is_ecall = builder.eq_node(
        bool_sid,
        instruction_id,
        consts.nid_instr_id(InstrId::Ecall),
        Some("is ecall?".to_string()),
    );

    // ===== KERNEL COMBINATIONAL (C rotor kernel_combinational) =====
    // All per-step syscall signals: brk/openat/read/write decode, the brk
    // validity check, the read-progress helpers, and the read return value.
    let kernel = KernelState::kernel_combinational(
        builder,
        sorts,
        consts,
        config,
        &core.kernel,
        core.register_file_state,
        core.segmentation.heap_end,
        is_ecall,
    );

    // Kernel control-flow stall (C rotor rotor.c:11037-11051):
    // PC freezes forever on exit, and while a read syscall still has more
    // than one byte to deliver (the read executes one byte per transition).
    let next_pc = {
        let ongoing_read = builder.and_node(
            bool_sid,
            kernel.is_read,
            kernel.more_than_one_readable_byte_to_read,
            Some("ongoing read system call".to_string()),
        );
        let exit_or_read = builder.or_node(
            bool_sid,
            kernel.is_exit,
            ongoing_read,
            Some("ongoing exit or read system call".to_string()),
        );
        let stall = builder.and_node(
            bool_sid,
            is_ecall,
            exit_or_read,
            Some("active system call stalls PC".to_string()),
        );
        builder.ite(
            mw_sid,
            stall,
            core.pc_state,
            next_pc,
            Some("update program counter unless in kernel mode".to_string()),
        )
    };

    // Store width (as machine word for convenience)
    let store_width = {
        let w1 = consts.nid_machine_word_1;
        let w2 = consts.nid_machine_word_2;
        let w4 = consts.nid_machine_word_4;
        let w8 = consts.nid_machine_word_8;
        let mut sw = consts.nid_machine_word_0;
        let sb_cond = builder.eq_node(
            bool_sid,
            instruction_id,
            consts.nid_instr_id(InstrId::Sb),
            None,
        );
        sw = builder.ite(mw_sid, sb_cond, w1, sw, None);
        let sh_cond = builder.eq_node(
            bool_sid,
            instruction_id,
            consts.nid_instr_id(InstrId::Sh),
            None,
        );
        sw = builder.ite(mw_sid, sh_cond, w2, sw, None);
        let sw_cond = builder.eq_node(
            bool_sid,
            instruction_id,
            consts.nid_instr_id(InstrId::Sw),
            None,
        );
        sw = builder.ite(mw_sid, sw_cond, w4, sw, None);
        if config.xlen == Xlen::X64 {
            let sd_cond = builder.eq_node(
                bool_sid,
                instruction_id,
                consts.nid_instr_id(InstrId::Sd),
                None,
            );
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
        load_addr,
        is_load: is_load_instr,
        store_addr,
        store_value,
        store_width,
        writes_rd,
        writes_memory: is_store_instr,
        is_ecall,
        ir,
        is_compressed,
        is_unknown_instruction,
        is_unknown_compressed,
        is_compressed_load,
        is_compressed_store,
        division_by_zero,
        signed_division_overflow,
        invalid_address,
        load_invalid_address,
        store_invalid_address,
        kernel,
    }
}
