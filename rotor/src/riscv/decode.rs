use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::Config;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::riscv::isa::InstrId;

/// Extract opcode (bits [6:0]) from instruction register
pub fn extract_opcode(builder: &mut Btor2Builder, sorts: &MachineSorts, ir: NodeId) -> NodeId {
    builder.slice(sorts.sid_opcode, ir, 6, 0, Some("opcode".to_string()))
}

/// Extract funct3 (bits [14:12]) from instruction register
pub fn extract_funct3(builder: &mut Btor2Builder, sorts: &MachineSorts, ir: NodeId) -> NodeId {
    builder.slice(sorts.sid_funct3, ir, 14, 12, Some("funct3".to_string()))
}

/// Extract funct7 (bits [31:25]) from instruction register
pub fn extract_funct7(builder: &mut Btor2Builder, sorts: &MachineSorts, ir: NodeId) -> NodeId {
    builder.slice(sorts.sid_funct7, ir, 31, 25, Some("funct7".to_string()))
}

/// Extract funct6 (bits [31:26]) - used for 64-bit shifts
pub fn extract_funct6(builder: &mut Btor2Builder, sorts: &MachineSorts, ir: NodeId) -> NodeId {
    builder.slice(sorts.sid_funct6, ir, 31, 26, Some("funct6".to_string()))
}

/// Extract rd field (bits [11:7])
pub fn extract_rd(builder: &mut Btor2Builder, sorts: &MachineSorts, ir: NodeId) -> NodeId {
    builder.slice(
        sorts.sid_register_address,
        ir,
        11,
        7,
        Some("rd".to_string()),
    )
}

/// Extract rs1 field (bits [19:15])
pub fn extract_rs1(builder: &mut Btor2Builder, sorts: &MachineSorts, ir: NodeId) -> NodeId {
    builder.slice(
        sorts.sid_register_address,
        ir,
        19,
        15,
        Some("rs1".to_string()),
    )
}

/// Extract rs2 field (bits [24:20])
pub fn extract_rs2(builder: &mut Btor2Builder, sorts: &MachineSorts, ir: NodeId) -> NodeId {
    builder.slice(
        sorts.sid_register_address,
        ir,
        24,
        20,
        Some("rs2".to_string()),
    )
}

/// Extract I-immediate (bits [31:20], sign-extended)
pub fn extract_i_imm(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    config: &Config,
    ir: NodeId,
) -> NodeId {
    let imm12 = builder.slice(sorts.sid_12bit, ir, 31, 20, Some("I-imm[11:0]".to_string()));
    builder.sext(
        sorts.sid_machine_word,
        imm12,
        config.machine_word_bits() - 12,
        Some("sign-extend I-immediate".to_string()),
    )
}

/// Extract S-immediate (bits [31:25] | [11:7], sign-extended)
pub fn extract_s_imm(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    config: &Config,
    ir: NodeId,
) -> NodeId {
    let imm11_5 = builder.slice(sorts.sid_7bit, ir, 31, 25, Some("S-imm[11:5]".to_string()));
    let imm4_0 = builder.slice(sorts.sid_5bit, ir, 11, 7, Some("S-imm[4:0]".to_string()));
    let imm12 = builder.concat(
        sorts.sid_12bit,
        imm11_5,
        imm4_0,
        Some("S-imm[11:0]".to_string()),
    );
    builder.sext(
        sorts.sid_machine_word,
        imm12,
        config.machine_word_bits() - 12,
        Some("sign-extend S-immediate".to_string()),
    )
}

/// Extract SB-immediate (branch immediate, sign-extended)
pub fn extract_sb_imm(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    ir: NodeId,
) -> NodeId {
    let bit12 = builder.slice(
        sorts.sid_boolean,
        ir,
        31,
        31,
        Some("SB-imm[12]".to_string()),
    );
    let bit11 = builder.slice(sorts.sid_boolean, ir, 7, 7, Some("SB-imm[11]".to_string()));
    let bits10_5 = builder.slice(sorts.sid_6bit, ir, 30, 25, Some("SB-imm[10:5]".to_string()));
    let bits4_1 = builder.slice(sorts.sid_4bit, ir, 11, 8, Some("SB-imm[4:1]".to_string()));

    let top2 = builder.concat(sorts.sid_2bit, bit12, bit11, None);
    let mid = builder.concat(sorts.sid_8bit, top2, bits10_5, None);
    let upper = builder.concat(sorts.sid_12bit, mid, bits4_1, None);
    let zero_bit = consts.nid_false;
    let imm13 = builder.concat(sorts.sid_13bit, upper, zero_bit, None);

    builder.sext(
        sorts.sid_machine_word,
        imm13,
        config.machine_word_bits() - 13,
        Some("sign-extend SB-immediate".to_string()),
    )
}

/// Extract U-immediate (bits [31:12] << 12)
pub fn extract_u_imm(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    config: &Config,
    ir: NodeId,
) -> NodeId {
    let imm20 = builder.slice(
        sorts.sid_20bit,
        ir,
        31,
        12,
        Some("U-imm[31:12]".to_string()),
    );
    let zeros = builder.constd(sorts.sid_12bit, 0, Some("12 zero bits".to_string()));
    let imm32 = builder.concat(
        sorts.sid_single_word,
        imm20,
        zeros,
        Some("U-imm << 12".to_string()),
    );

    if config.machine_word_bits() > 32 {
        builder.sext(
            sorts.sid_machine_word,
            imm32,
            config.machine_word_bits() - 32,
            Some("sign-extend U-immediate".to_string()),
        )
    } else {
        imm32
    }
}

/// Extract UJ-immediate (JAL immediate, sign-extended)
pub fn extract_uj_imm(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    ir: NodeId,
) -> NodeId {
    let bit20 = builder.slice(
        sorts.sid_boolean,
        ir,
        31,
        31,
        Some("UJ-imm[20]".to_string()),
    );
    let bits19_12 = builder.slice(
        sorts.sid_8bit,
        ir,
        19,
        12,
        Some("UJ-imm[19:12]".to_string()),
    );
    let bit11 = builder.slice(
        sorts.sid_boolean,
        ir,
        20,
        20,
        Some("UJ-imm[11]".to_string()),
    );
    let bits10_1 = builder.slice(
        sorts.sid_10bit,
        ir,
        30,
        21,
        Some("UJ-imm[10:1]".to_string()),
    );

    let top = builder.concat(sorts.sid_9bit, bit20, bits19_12, None);
    let mid = builder.concat(sorts.sid_10bit, top, bit11, None);
    let upper = builder.concat(sorts.sid_20bit, mid, bits10_1, None);
    let zero_bit = consts.nid_false;
    let imm21 = builder.concat(sorts.sid_21bit, upper, zero_bit, None);

    builder.sext(
        sorts.sid_machine_word,
        imm21,
        config.machine_word_bits() - 21,
        Some("sign-extend UJ-immediate".to_string()),
    )
}

/// Decode instruction: given the fetched IR, produce the instruction ID.
pub fn decode_instruction(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    ir: NodeId,
) -> NodeId {
    let opcode = extract_opcode(builder, sorts, ir);
    let funct3 = extract_funct3(builder, sorts, ir);
    let funct7 = extract_funct7(builder, sorts, ir);

    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    // Decode LUI
    let lui_opcode = builder.constd(sorts.sid_opcode, super::isa::opcodes::LUI as u64, None);
    let is_lui = builder.eq_node(
        bool_sid,
        opcode,
        lui_opcode,
        Some("opcode == LUI".to_string()),
    );
    let nid_lui = consts.nid_instr_id(InstrId::Lui);

    // Decode AUIPC
    let auipc_opcode = builder.constd(sorts.sid_opcode, super::isa::opcodes::AUIPC as u64, None);
    let is_auipc = builder.eq_node(bool_sid, opcode, auipc_opcode, None);
    let nid_auipc = consts.nid_instr_id(InstrId::Auipc);

    // Decode JAL
    let jal_opcode = builder.constd(sorts.sid_opcode, super::isa::opcodes::JAL as u64, None);
    let is_jal = builder.eq_node(bool_sid, opcode, jal_opcode, None);
    let nid_jal = consts.nid_instr_id(InstrId::Jal);

    // Decode JALR
    let jalr_opcode = builder.constd(sorts.sid_opcode, super::isa::opcodes::JALR as u64, None);
    let is_jalr = builder.eq_node(bool_sid, opcode, jalr_opcode, None);
    let nid_jalr = consts.nid_instr_id(InstrId::Jalr);

    // Decode SYSTEM (ECALL)
    let sys_opcode = builder.constd(sorts.sid_opcode, super::isa::opcodes::SYSTEM as u64, None);
    let is_ecall = builder.eq_node(bool_sid, opcode, sys_opcode, None);
    let nid_ecall = consts.nid_instr_id(InstrId::Ecall);

    // Decode groups
    let branch_id = decode_branches(builder, sorts, consts, opcode, funct3);
    let load_id = decode_loads(builder, sorts, consts, config, opcode, funct3);
    let store_id = decode_stores(builder, sorts, consts, config, opcode, funct3);
    let imm_id = decode_op_imm(builder, sorts, consts, config, opcode, funct3, funct7, ir);
    let op_id = decode_op(builder, sorts, consts, config, opcode, funct3, funct7);

    let imm32_id = if config.xlen == crate::config::Xlen::X64 {
        decode_op_imm_32(builder, sorts, consts, opcode, funct3, funct7)
    } else {
        nid_unknown
    };

    let op32_id = if config.xlen == crate::config::Xlen::X64 {
        decode_op_32(builder, sorts, consts, config, opcode, funct3, funct7)
    } else {
        nid_unknown
    };

    // Build ITE cascade
    let mut result = nid_unknown;
    result = builder.ite(id_sid, is_ecall, nid_ecall, result, None);

    if config.xlen == crate::config::Xlen::X64 {
        let has_op32 = builder.neq(bool_sid, op32_id, nid_unknown, None);
        result = builder.ite(id_sid, has_op32, op32_id, result, None);
        let has_imm32 = builder.neq(bool_sid, imm32_id, nid_unknown, None);
        result = builder.ite(id_sid, has_imm32, imm32_id, result, None);
    }

    let has_op = builder.neq(bool_sid, op_id, nid_unknown, None);
    result = builder.ite(id_sid, has_op, op_id, result, None);
    let has_imm = builder.neq(bool_sid, imm_id, nid_unknown, None);
    result = builder.ite(id_sid, has_imm, imm_id, result, None);
    let has_store = builder.neq(bool_sid, store_id, nid_unknown, None);
    result = builder.ite(id_sid, has_store, store_id, result, None);
    let has_load = builder.neq(bool_sid, load_id, nid_unknown, None);
    result = builder.ite(id_sid, has_load, load_id, result, None);
    let has_branch = builder.neq(bool_sid, branch_id, nid_unknown, None);
    result = builder.ite(id_sid, has_branch, branch_id, result, None);

    result = builder.ite(id_sid, is_jalr, nid_jalr, result, None);
    result = builder.ite(id_sid, is_jal, nid_jal, result, None);
    result = builder.ite(id_sid, is_auipc, nid_auipc, result, None);
    result = builder.ite(
        id_sid,
        is_lui,
        nid_lui,
        result,
        Some("instruction decode".to_string()),
    );

    result
}

fn decode_funct3_group(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    opcode: NodeId,
    funct3: NodeId,
    expected_opcode: u64,
    mappings: &[(u32, InstrId)],
    comment: &str,
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let opc = builder.constd(sorts.sid_opcode, expected_opcode, None);
    let is_group = builder.eq_node(bool_sid, opcode, opc, None);

    let mut result = nid_unknown;
    for (f3_val, instr) in mappings.iter().rev() {
        let f3 = builder.constd(sorts.sid_funct3, *f3_val as u64, None);
        let m = builder.eq_node(bool_sid, funct3, f3, None);
        let nid = consts.nid_instr_id(*instr);
        result = builder.ite(id_sid, m, nid, result, None);
    }

    builder.ite(
        id_sid,
        is_group,
        result,
        nid_unknown,
        Some(comment.to_string()),
    )
}

fn decode_branches(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    opcode: NodeId,
    funct3: NodeId,
) -> NodeId {
    decode_funct3_group(
        builder,
        sorts,
        consts,
        opcode,
        funct3,
        super::isa::opcodes::BRANCH as u64,
        &[
            (super::isa::funct3::BEQ, InstrId::Beq),
            (super::isa::funct3::BNE, InstrId::Bne),
            (super::isa::funct3::BLT, InstrId::Blt),
            (super::isa::funct3::BGE, InstrId::Bge),
            (super::isa::funct3::BLTU, InstrId::Bltu),
            (super::isa::funct3::BGEU, InstrId::Bgeu),
        ],
        "branch decode",
    )
}

fn decode_loads(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    opcode: NodeId,
    funct3: NodeId,
) -> NodeId {
    let mut mappings = vec![
        (super::isa::funct3::LB, InstrId::Lb),
        (super::isa::funct3::LH, InstrId::Lh),
        (super::isa::funct3::LW, InstrId::Lw),
        (super::isa::funct3::LBU, InstrId::Lbu),
        (super::isa::funct3::LHU, InstrId::Lhu),
    ];
    if config.xlen == crate::config::Xlen::X64 {
        mappings.push((super::isa::funct3::LD, InstrId::Ld));
        mappings.push((super::isa::funct3::LWU, InstrId::Lwu));
    }
    decode_funct3_group(
        builder,
        sorts,
        consts,
        opcode,
        funct3,
        super::isa::opcodes::LOAD as u64,
        &mappings,
        "load decode",
    )
}

fn decode_stores(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    opcode: NodeId,
    funct3: NodeId,
) -> NodeId {
    let mut mappings = vec![
        (super::isa::funct3::SB, InstrId::Sb),
        (super::isa::funct3::SH, InstrId::Sh),
        (super::isa::funct3::SW, InstrId::Sw),
    ];
    if config.xlen == crate::config::Xlen::X64 {
        mappings.push((super::isa::funct3::SD, InstrId::Sd));
    }
    decode_funct3_group(
        builder,
        sorts,
        consts,
        opcode,
        funct3,
        super::isa::opcodes::STORE as u64,
        &mappings,
        "store decode",
    )
}

fn decode_op_imm(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    opcode: NodeId,
    funct3: NodeId,
    funct7: NodeId,
    ir: NodeId,
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let imm_opcode = builder.constd(sorts.sid_opcode, super::isa::opcodes::OP_IMM as u64, None);
    let is_imm = builder.eq_node(bool_sid, opcode, imm_opcode, None);

    let simple_imms = [
        (super::isa::funct3::ADD_SUB, InstrId::Addi),
        (super::isa::funct3::SLT, InstrId::Slti),
        (super::isa::funct3::SLTU, InstrId::Sltiu),
        (super::isa::funct3::XOR, InstrId::Xori),
        (super::isa::funct3::OR, InstrId::Ori),
        (super::isa::funct3::AND, InstrId::Andi),
    ];

    let mut result = nid_unknown;

    let f7_zero = builder.constd(sorts.sid_funct7, super::isa::funct7::ZERO as u64, None);
    let f7_sra = builder.constd(sorts.sid_funct7, super::isa::funct7::SUB_SRA as u64, None);

    // SLLI
    let f3_sll = builder.constd(sorts.sid_funct3, super::isa::funct3::SLL as u64, None);
    let is_sll_f3 = builder.eq_node(bool_sid, funct3, f3_sll, None);
    let is_slli = if config.xlen == crate::config::Xlen::X64 {
        let funct6 = extract_funct6(builder, sorts, ir);
        let f6_zero = builder.constd(sorts.sid_funct6, 0, None);
        let is_f6_zero = builder.eq_node(bool_sid, funct6, f6_zero, None);
        builder.and_node(bool_sid, is_sll_f3, is_f6_zero, None)
    } else {
        let is_f7_zero = builder.eq_node(bool_sid, funct7, f7_zero, None);
        builder.and_node(bool_sid, is_sll_f3, is_f7_zero, None)
    };
    let nid_slli = consts.nid_instr_id(InstrId::Slli);
    result = builder.ite(id_sid, is_slli, nid_slli, result, None);

    // SRLI/SRAI
    let f3_sr = builder.constd(sorts.sid_funct3, super::isa::funct3::SRL_SRA as u64, None);
    let is_sr_f3 = builder.eq_node(bool_sid, funct3, f3_sr, None);

    if config.xlen == crate::config::Xlen::X64 {
        let funct6 = extract_funct6(builder, sorts, ir);
        let f6_zero = builder.constd(sorts.sid_funct6, 0, None);
        let f6_sra = builder.constd(sorts.sid_funct6, 0b010000, None);

        let is_f6_zero = builder.eq_node(bool_sid, funct6, f6_zero, None);
        let is_srli = builder.and_node(bool_sid, is_sr_f3, is_f6_zero, None);
        let nid_srli = consts.nid_instr_id(InstrId::Srli);
        result = builder.ite(id_sid, is_srli, nid_srli, result, None);

        let is_f6_sra = builder.eq_node(bool_sid, funct6, f6_sra, None);
        let is_srai = builder.and_node(bool_sid, is_sr_f3, is_f6_sra, None);
        let nid_srai = consts.nid_instr_id(InstrId::Srai);
        result = builder.ite(id_sid, is_srai, nid_srai, result, None);
    } else {
        let is_f7_zero = builder.eq_node(bool_sid, funct7, f7_zero, None);
        let is_srli = builder.and_node(bool_sid, is_sr_f3, is_f7_zero, None);
        let nid_srli = consts.nid_instr_id(InstrId::Srli);
        result = builder.ite(id_sid, is_srli, nid_srli, result, None);

        let is_f7_sra = builder.eq_node(bool_sid, funct7, f7_sra, None);
        let is_srai = builder.and_node(bool_sid, is_sr_f3, is_f7_sra, None);
        let nid_srai = consts.nid_instr_id(InstrId::Srai);
        result = builder.ite(id_sid, is_srai, nid_srai, result, None);
    }

    for (f3_val, instr) in simple_imms.iter().rev() {
        let f3 = builder.constd(sorts.sid_funct3, *f3_val as u64, None);
        let m = builder.eq_node(bool_sid, funct3, f3, None);
        let nid = consts.nid_instr_id(*instr);
        result = builder.ite(id_sid, m, nid, result, None);
    }

    builder.ite(
        id_sid,
        is_imm,
        result,
        nid_unknown,
        Some("OP-IMM decode".to_string()),
    )
}

fn decode_op(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    opcode: NodeId,
    funct3: NodeId,
    funct7: NodeId,
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let op_opcode = builder.constd(sorts.sid_opcode, super::isa::opcodes::OP as u64, None);
    let is_op = builder.eq_node(bool_sid, opcode, op_opcode, None);

    let f7_zero = builder.constd(sorts.sid_funct7, super::isa::funct7::ZERO as u64, None);
    let f7_sub = builder.constd(sorts.sid_funct7, super::isa::funct7::SUB_SRA as u64, None);
    let f7_mul = builder.constd(sorts.sid_funct7, super::isa::funct7::MULDIV as u64, None);

    let is_f7_zero = builder.eq_node(bool_sid, funct7, f7_zero, None);
    let is_f7_sub = builder.eq_node(bool_sid, funct7, f7_sub, None);
    let is_f7_mul = builder.eq_node(bool_sid, funct7, f7_mul, None);

    let mut result = nid_unknown;

    let r_zero = [
        (super::isa::funct3::ADD_SUB, InstrId::Add),
        (super::isa::funct3::SLL, InstrId::Sll),
        (super::isa::funct3::SLT, InstrId::Slt),
        (super::isa::funct3::SLTU, InstrId::Sltu),
        (super::isa::funct3::XOR, InstrId::Xor),
        (super::isa::funct3::SRL_SRA, InstrId::Srl),
        (super::isa::funct3::OR, InstrId::Or),
        (super::isa::funct3::AND, InstrId::And),
    ];
    for (f3_val, instr) in r_zero.iter() {
        let f3 = builder.constd(sorts.sid_funct3, *f3_val as u64, None);
        let is_f3 = builder.eq_node(bool_sid, funct3, f3, None);
        let cond = builder.and_node(bool_sid, is_f7_zero, is_f3, None);
        let nid = consts.nid_instr_id(*instr);
        result = builder.ite(id_sid, cond, nid, result, None);
    }

    // SUB, SRA
    let f3_add = builder.constd(sorts.sid_funct3, super::isa::funct3::ADD_SUB as u64, None);
    let is_f3_add = builder.eq_node(bool_sid, funct3, f3_add, None);
    let is_sub = builder.and_node(bool_sid, is_f7_sub, is_f3_add, None);
    result = builder.ite(
        id_sid,
        is_sub,
        consts.nid_instr_id(InstrId::Sub),
        result,
        None,
    );

    let f3_sr = builder.constd(sorts.sid_funct3, super::isa::funct3::SRL_SRA as u64, None);
    let is_f3_sr = builder.eq_node(bool_sid, funct3, f3_sr, None);
    let is_sra = builder.and_node(bool_sid, is_f7_sub, is_f3_sr, None);
    result = builder.ite(
        id_sid,
        is_sra,
        consts.nid_instr_id(InstrId::Sra),
        result,
        None,
    );

    // M extension
    if config.enable_m {
        let m = [
            (super::isa::funct3::MUL, InstrId::Mul),
            (super::isa::funct3::MULH, InstrId::Mulh),
            (super::isa::funct3::MULHSU, InstrId::Mulhsu),
            (super::isa::funct3::MULHU, InstrId::Mulhu),
            (super::isa::funct3::DIV, InstrId::Div),
            (super::isa::funct3::DIVU, InstrId::Divu),
            (super::isa::funct3::REM, InstrId::Rem),
            (super::isa::funct3::REMU, InstrId::Remu),
        ];
        for (f3_val, instr) in m.iter() {
            let f3 = builder.constd(sorts.sid_funct3, *f3_val as u64, None);
            let is_f3 = builder.eq_node(bool_sid, funct3, f3, None);
            let cond = builder.and_node(bool_sid, is_f7_mul, is_f3, None);
            result = builder.ite(id_sid, cond, consts.nid_instr_id(*instr), result, None);
        }
    }

    builder.ite(
        id_sid,
        is_op,
        result,
        nid_unknown,
        Some("OP decode".to_string()),
    )
}

fn decode_op_imm_32(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    opcode: NodeId,
    funct3: NodeId,
    funct7: NodeId,
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let opc = builder.constd(
        sorts.sid_opcode,
        super::isa::opcodes::OP_IMM_32 as u64,
        None,
    );
    let is_imm32 = builder.eq_node(bool_sid, opcode, opc, None);

    let f7_zero = builder.constd(sorts.sid_funct7, super::isa::funct7::ZERO as u64, None);
    let f7_sra = builder.constd(sorts.sid_funct7, super::isa::funct7::SUB_SRA as u64, None);
    let is_f7_zero = builder.eq_node(bool_sid, funct7, f7_zero, None);
    let is_f7_sra = builder.eq_node(bool_sid, funct7, f7_sra, None);

    let mut result = nid_unknown;

    let f3_add = builder.constd(sorts.sid_funct3, super::isa::funct3::ADD_SUB as u64, None);
    let is_addiw = builder.eq_node(bool_sid, funct3, f3_add, None);
    result = builder.ite(
        id_sid,
        is_addiw,
        consts.nid_instr_id(InstrId::Addiw),
        result,
        None,
    );

    let f3_sll = builder.constd(sorts.sid_funct3, super::isa::funct3::SLL as u64, None);
    let is_f3_sll = builder.eq_node(bool_sid, funct3, f3_sll, None);
    let is_slliw = builder.and_node(bool_sid, is_f3_sll, is_f7_zero, None);
    result = builder.ite(
        id_sid,
        is_slliw,
        consts.nid_instr_id(InstrId::Slliw),
        result,
        None,
    );

    let f3_sr = builder.constd(sorts.sid_funct3, super::isa::funct3::SRL_SRA as u64, None);
    let is_f3_sr = builder.eq_node(bool_sid, funct3, f3_sr, None);
    let is_srliw = builder.and_node(bool_sid, is_f3_sr, is_f7_zero, None);
    result = builder.ite(
        id_sid,
        is_srliw,
        consts.nid_instr_id(InstrId::Srliw),
        result,
        None,
    );
    let is_sraiw = builder.and_node(bool_sid, is_f3_sr, is_f7_sra, None);
    result = builder.ite(
        id_sid,
        is_sraiw,
        consts.nid_instr_id(InstrId::Sraiw),
        result,
        None,
    );

    builder.ite(
        id_sid,
        is_imm32,
        result,
        nid_unknown,
        Some("OP-IMM-32 decode".to_string()),
    )
}

fn decode_op_32(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    opcode: NodeId,
    funct3: NodeId,
    funct7: NodeId,
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let opc = builder.constd(sorts.sid_opcode, super::isa::opcodes::OP_32 as u64, None);
    let is_op32 = builder.eq_node(bool_sid, opcode, opc, None);

    let f7_zero = builder.constd(sorts.sid_funct7, super::isa::funct7::ZERO as u64, None);
    let f7_sub = builder.constd(sorts.sid_funct7, super::isa::funct7::SUB_SRA as u64, None);
    let f7_mul = builder.constd(sorts.sid_funct7, super::isa::funct7::MULDIV as u64, None);
    let is_f7_zero = builder.eq_node(bool_sid, funct7, f7_zero, None);
    let is_f7_sub = builder.eq_node(bool_sid, funct7, f7_sub, None);
    let is_f7_mul = builder.eq_node(bool_sid, funct7, f7_mul, None);

    let mut result = nid_unknown;

    for (f3_val, instr) in [
        (super::isa::funct3::ADD_SUB, InstrId::Addw),
        (super::isa::funct3::SLL, InstrId::Sllw),
        (super::isa::funct3::SRL_SRA, InstrId::Srlw),
    ] {
        let f3 = builder.constd(sorts.sid_funct3, f3_val as u64, None);
        let is_f3 = builder.eq_node(bool_sid, funct3, f3, None);
        let cond = builder.and_node(bool_sid, is_f7_zero, is_f3, None);
        result = builder.ite(id_sid, cond, consts.nid_instr_id(instr), result, None);
    }

    let f3_add = builder.constd(sorts.sid_funct3, super::isa::funct3::ADD_SUB as u64, None);
    let is_f3_add = builder.eq_node(bool_sid, funct3, f3_add, None);
    let is_subw = builder.and_node(bool_sid, is_f7_sub, is_f3_add, None);
    result = builder.ite(
        id_sid,
        is_subw,
        consts.nid_instr_id(InstrId::Subw),
        result,
        None,
    );

    let f3_sr = builder.constd(sorts.sid_funct3, super::isa::funct3::SRL_SRA as u64, None);
    let is_f3_sr = builder.eq_node(bool_sid, funct3, f3_sr, None);
    let is_sraw = builder.and_node(bool_sid, is_f7_sub, is_f3_sr, None);
    result = builder.ite(
        id_sid,
        is_sraw,
        consts.nid_instr_id(InstrId::Sraw),
        result,
        None,
    );

    if config.enable_m {
        for (f3_val, instr) in [
            (super::isa::funct3::MUL, InstrId::Mulw),
            (super::isa::funct3::DIV, InstrId::Divw),
            (super::isa::funct3::DIVU, InstrId::Divuw),
            (super::isa::funct3::REM, InstrId::Remw),
            (super::isa::funct3::REMU, InstrId::Remuw),
        ] {
            let f3 = builder.constd(sorts.sid_funct3, f3_val as u64, None);
            let is_f3 = builder.eq_node(bool_sid, funct3, f3, None);
            let cond = builder.and_node(bool_sid, is_f7_mul, is_f3, None);
            result = builder.ite(id_sid, cond, consts.nid_instr_id(instr), result, None);
        }
    }

    builder.ite(
        id_sid,
        is_op32,
        result,
        nid_unknown,
        Some("OP-32 decode".to_string()),
    )
}
