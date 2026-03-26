use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::riscv::isa::InstrId;

/// Decode compressed instruction into an InstrId
pub fn decode_compressed(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    c_ir: NodeId,
) -> NodeId {
    let c_opcode = builder.slice(sorts.sid_2bit, c_ir, 1, 0, Some("c-opcode".to_string()));
    let c_funct3 = builder.slice(sorts.sid_funct3, c_ir, 15, 13, Some("c-funct3".to_string()));

    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let c_op_0 = builder.constd(sorts.sid_2bit, 0, None);
    let c_op_1 = builder.constd(sorts.sid_2bit, 1, None);
    let c_op_2 = builder.constd(sorts.sid_2bit, 2, None);

    let is_q0 = builder.eq_node(bool_sid, c_opcode, c_op_0, None);
    let is_q1 = builder.eq_node(bool_sid, c_opcode, c_op_1, None);
    let is_q2 = builder.eq_node(bool_sid, c_opcode, c_op_2, None);

    let q0_result = decode_quadrant_0(builder, sorts, consts, c_funct3);
    let q1_result = decode_quadrant_1(builder, sorts, consts, c_ir, c_funct3);
    let q2_result = decode_quadrant_2(builder, sorts, consts, c_ir, c_funct3);

    let mut result = nid_unknown;
    result = builder.ite(id_sid, is_q2, q2_result, result, Some("C quadrant 2?".to_string()));
    result = builder.ite(id_sid, is_q1, q1_result, result, Some("C quadrant 1?".to_string()));
    result = builder.ite(id_sid, is_q0, q0_result, result, Some("C quadrant 0?".to_string()));
    result
}

fn decode_funct3_ite(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    c_funct3: NodeId,
    mappings: &[(u32, InstrId)],
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);
    let mut result = nid_unknown;
    for (f3_val, instr) in mappings.iter().rev() {
        let f3 = builder.constd(sorts.sid_funct3, *f3_val as u64, None);
        let m = builder.eq_node(bool_sid, c_funct3, f3, None);
        let nid = consts.nid_instr_id(*instr);
        result = builder.ite(id_sid, m, nid, result, None);
    }
    result
}

fn decode_quadrant_0(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    c_funct3: NodeId,
) -> NodeId {
    decode_funct3_ite(builder, sorts, consts, c_funct3, &[
        (0b000, InstrId::CAddi4spn),
        (0b010, InstrId::CLw),
        (0b011, InstrId::CLd),
        (0b110, InstrId::CSw),
        (0b111, InstrId::CSd),
    ])
}

fn decode_quadrant_1(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    c_ir: NodeId,
    c_funct3: NodeId,
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let mut result = nid_unknown;

    // Simple funct3 mappings
    let f3_000 = builder.constd(sorts.sid_funct3, 0, None);
    let is_000 = builder.eq_node(bool_sid, c_funct3, f3_000, None);
    result = builder.ite(id_sid, is_000, consts.nid_instr_id(InstrId::CAddi), result, None);

    let f3_001 = builder.constd(sorts.sid_funct3, 0b001, None);
    let is_001 = builder.eq_node(bool_sid, c_funct3, f3_001, None);
    result = builder.ite(id_sid, is_001, consts.nid_instr_id(InstrId::CAddiw), result, None);

    let f3_010 = builder.constd(sorts.sid_funct3, 0b010, None);
    let is_010 = builder.eq_node(bool_sid, c_funct3, f3_010, None);
    result = builder.ite(id_sid, is_010, consts.nid_instr_id(InstrId::CLi), result, None);

    // LUI / ADDI16SP: funct3=011, distinguished by rd
    let f3_011 = builder.constd(sorts.sid_funct3, 0b011, None);
    let is_011 = builder.eq_node(bool_sid, c_funct3, f3_011, None);
    let rd = builder.slice(sorts.sid_register_address, c_ir, 11, 7, None);
    let sp_reg = builder.constd(sorts.sid_register_address, 2, None);
    let is_sp = builder.eq_node(bool_sid, rd, sp_reg, None);
    let lui_or_sp = builder.ite(id_sid, is_sp, consts.nid_instr_id(InstrId::CAddi16sp), consts.nid_instr_id(InstrId::CLui), None);
    result = builder.ite(id_sid, is_011, lui_or_sp, result, None);

    // Arithmetic group: funct3=100
    let f3_100 = builder.constd(sorts.sid_funct3, 0b100, None);
    let is_100 = builder.eq_node(bool_sid, c_funct3, f3_100, None);
    let arith = decode_q1_arith(builder, sorts, consts, c_ir);
    result = builder.ite(id_sid, is_100, arith, result, None);

    let f3_101 = builder.constd(sorts.sid_funct3, 0b101, None);
    let is_101 = builder.eq_node(bool_sid, c_funct3, f3_101, None);
    result = builder.ite(id_sid, is_101, consts.nid_instr_id(InstrId::CJ), result, None);

    let f3_110 = builder.constd(sorts.sid_funct3, 0b110, None);
    let is_110 = builder.eq_node(bool_sid, c_funct3, f3_110, None);
    result = builder.ite(id_sid, is_110, consts.nid_instr_id(InstrId::CBeqz), result, None);

    let f3_111 = builder.constd(sorts.sid_funct3, 0b111, None);
    let is_111 = builder.eq_node(bool_sid, c_funct3, f3_111, None);
    result = builder.ite(id_sid, is_111, consts.nid_instr_id(InstrId::CBnez), result, None);

    result
}

fn decode_q1_arith(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    c_ir: NodeId,
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let bits_11_10 = builder.slice(sorts.sid_2bit, c_ir, 11, 10, None);
    let v00 = builder.constd(sorts.sid_2bit, 0b00, None);
    let v01 = builder.constd(sorts.sid_2bit, 0b01, None);
    let v10 = builder.constd(sorts.sid_2bit, 0b10, None);
    let v11 = builder.constd(sorts.sid_2bit, 0b11, None);

    let is_00 = builder.eq_node(bool_sid, bits_11_10, v00, None);
    let is_01 = builder.eq_node(bool_sid, bits_11_10, v01, None);
    let is_10 = builder.eq_node(bool_sid, bits_11_10, v10, None);
    let is_11 = builder.eq_node(bool_sid, bits_11_10, v11, None);

    // For 11: decode further using bits [6:5] and bit [12]
    let funct2 = builder.slice(sorts.sid_2bit, c_ir, 6, 5, None);
    let bit12 = builder.slice(sorts.sid_boolean, c_ir, 12, 12, None);
    let is_bit12_0 = builder.not(bool_sid, bit12, None);

    let f2_00 = builder.constd(sorts.sid_2bit, 0b00, None);
    let f2_01 = builder.constd(sorts.sid_2bit, 0b01, None);
    let f2_10 = builder.constd(sorts.sid_2bit, 0b10, None);
    let f2_11 = builder.constd(sorts.sid_2bit, 0b11, None);

    let is_f2_00 = builder.eq_node(bool_sid, funct2, f2_00, None);
    let is_f2_01 = builder.eq_node(bool_sid, funct2, f2_01, None);
    let is_f2_10 = builder.eq_node(bool_sid, funct2, f2_10, None);
    let is_f2_11 = builder.eq_node(bool_sid, funct2, f2_11, None);

    let mut sub_result = nid_unknown;
    // bit12=0
    let c = builder.and_node(bool_sid, is_bit12_0, is_f2_00, None);
    sub_result = builder.ite(id_sid, c, consts.nid_instr_id(InstrId::CSub), sub_result, None);
    let c = builder.and_node(bool_sid, is_bit12_0, is_f2_01, None);
    sub_result = builder.ite(id_sid, c, consts.nid_instr_id(InstrId::CXor), sub_result, None);
    let c = builder.and_node(bool_sid, is_bit12_0, is_f2_10, None);
    sub_result = builder.ite(id_sid, c, consts.nid_instr_id(InstrId::COr), sub_result, None);
    let c = builder.and_node(bool_sid, is_bit12_0, is_f2_11, None);
    sub_result = builder.ite(id_sid, c, consts.nid_instr_id(InstrId::CAnd), sub_result, None);
    // bit12=1 (RV64C)
    let c = builder.and_node(bool_sid, bit12, is_f2_00, None);
    sub_result = builder.ite(id_sid, c, consts.nid_instr_id(InstrId::CSubw), sub_result, None);
    let c = builder.and_node(bool_sid, bit12, is_f2_01, None);
    sub_result = builder.ite(id_sid, c, consts.nid_instr_id(InstrId::CAddw), sub_result, None);

    let mut result = nid_unknown;
    result = builder.ite(id_sid, is_11, sub_result, result, None);
    result = builder.ite(id_sid, is_10, consts.nid_instr_id(InstrId::CAndi), result, None);
    result = builder.ite(id_sid, is_01, consts.nid_instr_id(InstrId::CSrai), result, None);
    result = builder.ite(id_sid, is_00, consts.nid_instr_id(InstrId::CSrli), result, None);
    result
}

fn decode_quadrant_2(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    c_ir: NodeId,
    c_funct3: NodeId,
) -> NodeId {
    let bool_sid = sorts.sid_boolean;
    let id_sid = sorts.sid_instruction_id;
    let nid_unknown = consts.nid_instr_id(InstrId::Unknown);

    let mut result = nid_unknown;

    let f3_000 = builder.constd(sorts.sid_funct3, 0, None);
    let is_000 = builder.eq_node(bool_sid, c_funct3, f3_000, None);
    result = builder.ite(id_sid, is_000, consts.nid_instr_id(InstrId::CSlli), result, None);

    let f3_010 = builder.constd(sorts.sid_funct3, 0b010, None);
    let is_010 = builder.eq_node(bool_sid, c_funct3, f3_010, None);
    result = builder.ite(id_sid, is_010, consts.nid_instr_id(InstrId::CLwsp), result, None);

    let f3_011 = builder.constd(sorts.sid_funct3, 0b011, None);
    let is_011 = builder.eq_node(bool_sid, c_funct3, f3_011, None);
    result = builder.ite(id_sid, is_011, consts.nid_instr_id(InstrId::CLdsp), result, None);

    // funct3=100: JR/MV/JALR/ADD
    let f3_100 = builder.constd(sorts.sid_funct3, 0b100, None);
    let is_100 = builder.eq_node(bool_sid, c_funct3, f3_100, None);
    let bit12 = builder.slice(sorts.sid_boolean, c_ir, 12, 12, None);
    let rs2 = builder.slice(sorts.sid_register_address, c_ir, 6, 2, None);
    let zero_reg = builder.constd(sorts.sid_register_address, 0, None);
    let rs2_is_zero = builder.eq_node(bool_sid, rs2, zero_reg, None);
    let bit12_0 = builder.ite(id_sid, rs2_is_zero, consts.nid_instr_id(InstrId::CJr), consts.nid_instr_id(InstrId::CMv), None);
    let bit12_1 = builder.ite(id_sid, rs2_is_zero, consts.nid_instr_id(InstrId::CJalr), consts.nid_instr_id(InstrId::CAdd), None);
    let q2_100 = builder.ite(id_sid, bit12, bit12_1, bit12_0, None);
    result = builder.ite(id_sid, is_100, q2_100, result, None);

    let f3_110 = builder.constd(sorts.sid_funct3, 0b110, None);
    let is_110 = builder.eq_node(bool_sid, c_funct3, f3_110, None);
    result = builder.ite(id_sid, is_110, consts.nid_instr_id(InstrId::CSwsp), result, None);

    let f3_111 = builder.constd(sorts.sid_funct3, 0b111, None);
    let is_111 = builder.eq_node(bool_sid, c_funct3, f3_111, None);
    result = builder.ite(id_sid, is_111, consts.nid_instr_id(InstrId::CSdsp), result, None);

    result
}
