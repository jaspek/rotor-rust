#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum InstrId {
    Unknown = 0,
    Ecall,
    // R-type (RV32I/RV64I)
    Add,
    Sub,
    Sll,
    Slt,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
    // R-type (RV64I only)
    Addw,
    Subw,
    Sllw,
    Srlw,
    Sraw,
    // M extension (RV32M)
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
    Div,
    Divu,
    Rem,
    Remu,
    // M extension (RV64M only)
    Mulw,
    Divw,
    Divuw,
    Remw,
    Remuw,
    // I-type immediates
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
    // I-type (RV64I only)
    Addiw,
    Slliw,
    Srliw,
    Sraiw,
    // I-type loads
    Lb,
    Lh,
    Lw,
    Ld,
    Lbu,
    Lhu,
    Lwu,
    // I-type jumps
    Jalr,
    // S-type stores
    Sb,
    Sh,
    Sw,
    Sd,
    // B-type branches
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
    // U-type
    Lui,
    Auipc,
    // J-type
    Jal,
    // Compressed (RVC)
    CMv,
    CAdd,
    CJr,
    CJalr,
    CLi,
    CLui,
    CAddi,
    CAddiw,
    CAddi16sp,
    CAddi4spn,
    CSlli,
    CSrli,
    CSrai,
    CAndi,
    CSub,
    CXor,
    COr,
    CAnd,
    CAddw,
    CSubw,
    CLw,
    CLd,
    CSw,
    CSd,
    CLwsp,
    CLdsp,
    CSwsp,
    CSdsp,
    CBeqz,
    CBnez,
    CJ,
    CJal,
}

impl InstrId {
    pub fn mnemonic(self) -> &'static str {
        match self {
            Self::Unknown => "unknown",
            Self::Ecall => "ecall",
            Self::Add => "add",
            Self::Sub => "sub",
            Self::Sll => "sll",
            Self::Slt => "slt",
            Self::Sltu => "sltu",
            Self::Xor => "xor",
            Self::Srl => "srl",
            Self::Sra => "sra",
            Self::Or => "or",
            Self::And => "and",
            Self::Addw => "addw",
            Self::Subw => "subw",
            Self::Sllw => "sllw",
            Self::Srlw => "srlw",
            Self::Sraw => "sraw",
            Self::Mul => "mul",
            Self::Mulh => "mulh",
            Self::Mulhsu => "mulhsu",
            Self::Mulhu => "mulhu",
            Self::Div => "div",
            Self::Divu => "divu",
            Self::Rem => "rem",
            Self::Remu => "remu",
            Self::Mulw => "mulw",
            Self::Divw => "divw",
            Self::Divuw => "divuw",
            Self::Remw => "remw",
            Self::Remuw => "remuw",
            Self::Addi => "addi",
            Self::Slti => "slti",
            Self::Sltiu => "sltiu",
            Self::Xori => "xori",
            Self::Ori => "ori",
            Self::Andi => "andi",
            Self::Slli => "slli",
            Self::Srli => "srli",
            Self::Srai => "srai",
            Self::Addiw => "addiw",
            Self::Slliw => "slliw",
            Self::Srliw => "srliw",
            Self::Sraiw => "sraiw",
            Self::Lb => "lb",
            Self::Lh => "lh",
            Self::Lw => "lw",
            Self::Ld => "ld",
            Self::Lbu => "lbu",
            Self::Lhu => "lhu",
            Self::Lwu => "lwu",
            Self::Jalr => "jalr",
            Self::Sb => "sb",
            Self::Sh => "sh",
            Self::Sw => "sw",
            Self::Sd => "sd",
            Self::Beq => "beq",
            Self::Bne => "bne",
            Self::Blt => "blt",
            Self::Bge => "bge",
            Self::Bltu => "bltu",
            Self::Bgeu => "bgeu",
            Self::Lui => "lui",
            Self::Auipc => "auipc",
            Self::Jal => "jal",
            Self::CMv => "c.mv",
            Self::CAdd => "c.add",
            Self::CJr => "c.jr",
            Self::CJalr => "c.jalr",
            Self::CLi => "c.li",
            Self::CLui => "c.lui",
            Self::CAddi => "c.addi",
            Self::CAddiw => "c.addiw",
            Self::CAddi16sp => "c.addi16sp",
            Self::CAddi4spn => "c.addi4spn",
            Self::CSlli => "c.slli",
            Self::CSrli => "c.srli",
            Self::CSrai => "c.srai",
            Self::CAndi => "c.andi",
            Self::CSub => "c.sub",
            Self::CXor => "c.xor",
            Self::COr => "c.or",
            Self::CAnd => "c.and",
            Self::CAddw => "c.addw",
            Self::CSubw => "c.subw",
            Self::CLw => "c.lw",
            Self::CLd => "c.ld",
            Self::CSw => "c.sw",
            Self::CSd => "c.sd",
            Self::CLwsp => "c.lwsp",
            Self::CLdsp => "c.ldsp",
            Self::CSwsp => "c.swsp",
            Self::CSdsp => "c.sdsp",
            Self::CBeqz => "c.beqz",
            Self::CBnez => "c.bnez",
            Self::CJ => "c.j",
            Self::CJal => "c.jal",
        }
    }

    pub fn is_compressed(self) -> bool {
        matches!(
            self,
            Self::CMv
                | Self::CAdd
                | Self::CJr
                | Self::CJalr
                | Self::CLi
                | Self::CLui
                | Self::CAddi
                | Self::CAddiw
                | Self::CAddi16sp
                | Self::CAddi4spn
                | Self::CSlli
                | Self::CSrli
                | Self::CSrai
                | Self::CAndi
                | Self::CSub
                | Self::CXor
                | Self::COr
                | Self::CAnd
                | Self::CAddw
                | Self::CSubw
                | Self::CLw
                | Self::CLd
                | Self::CSw
                | Self::CSd
                | Self::CLwsp
                | Self::CLdsp
                | Self::CSwsp
                | Self::CSdsp
                | Self::CBeqz
                | Self::CBnez
                | Self::CJ
                | Self::CJal
        )
    }

    pub fn is_branch(self) -> bool {
        matches!(
            self,
            Self::Beq
                | Self::Bne
                | Self::Blt
                | Self::Bge
                | Self::Bltu
                | Self::Bgeu
                | Self::CBeqz
                | Self::CBnez
        )
    }

    pub fn is_load(self) -> bool {
        matches!(
            self,
            Self::Lb
                | Self::Lh
                | Self::Lw
                | Self::Ld
                | Self::Lbu
                | Self::Lhu
                | Self::Lwu
                | Self::CLw
                | Self::CLd
                | Self::CLwsp
                | Self::CLdsp
        )
    }

    pub fn is_store(self) -> bool {
        matches!(
            self,
            Self::Sb
                | Self::Sh
                | Self::Sw
                | Self::Sd
                | Self::CSw
                | Self::CSd
                | Self::CSwsp
                | Self::CSdsp
        )
    }
}

// RISC-V opcodes (bits [6:0])
pub mod opcodes {
    pub const LUI: u32 = 0b0110111;
    pub const AUIPC: u32 = 0b0010111;
    pub const JAL: u32 = 0b1101111;
    pub const JALR: u32 = 0b1100111;
    pub const BRANCH: u32 = 0b1100011;
    pub const LOAD: u32 = 0b0000011;
    pub const STORE: u32 = 0b0100011;
    pub const OP_IMM: u32 = 0b0010011;
    pub const OP: u32 = 0b0110011;
    pub const OP_IMM_32: u32 = 0b0011011;
    pub const OP_32: u32 = 0b0111011;
    pub const SYSTEM: u32 = 0b1110011;
}

// funct3 field values
pub mod funct3 {
    // Branch
    pub const BEQ: u32 = 0b000;
    pub const BNE: u32 = 0b001;
    pub const BLT: u32 = 0b100;
    pub const BGE: u32 = 0b101;
    pub const BLTU: u32 = 0b110;
    pub const BGEU: u32 = 0b111;

    // Load
    pub const LB: u32 = 0b000;
    pub const LH: u32 = 0b001;
    pub const LW: u32 = 0b010;
    pub const LD: u32 = 0b011;
    pub const LBU: u32 = 0b100;
    pub const LHU: u32 = 0b101;
    pub const LWU: u32 = 0b110;

    // Store
    pub const SB: u32 = 0b000;
    pub const SH: u32 = 0b001;
    pub const SW: u32 = 0b010;
    pub const SD: u32 = 0b011;

    // OP-IMM / OP
    pub const ADD_SUB: u32 = 0b000;
    pub const SLL: u32 = 0b001;
    pub const SLT: u32 = 0b010;
    pub const SLTU: u32 = 0b011;
    pub const XOR: u32 = 0b100;
    pub const SRL_SRA: u32 = 0b101;
    pub const OR: u32 = 0b110;
    pub const AND: u32 = 0b111;

    // M extension
    pub const MUL: u32 = 0b000;
    pub const MULH: u32 = 0b001;
    pub const MULHSU: u32 = 0b010;
    pub const MULHU: u32 = 0b011;
    pub const DIV: u32 = 0b100;
    pub const DIVU: u32 = 0b101;
    pub const REM: u32 = 0b110;
    pub const REMU: u32 = 0b111;
}

// funct7 field values
pub mod funct7 {
    pub const ZERO: u32 = 0b0000000;
    pub const SUB_SRA: u32 = 0b0100000;
    pub const MULDIV: u32 = 0b0000001;
}

// RISC-V register names
pub const REG_NAMES: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "s1", "a0", "a1", "a2", "a3", "a4",
    "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9", "s10", "s11", "t3", "t4",
    "t5", "t6",
];

// Register indices
pub mod regs {
    pub const ZR: u32 = 0;
    pub const RA: u32 = 1;
    pub const SP: u32 = 2;
    pub const GP: u32 = 3;
    pub const TP: u32 = 4;
    pub const T0: u32 = 5;
    pub const T1: u32 = 6;
    pub const T2: u32 = 7;
    pub const S0: u32 = 8;
    pub const S1: u32 = 9;
    pub const A0: u32 = 10;
    pub const A1: u32 = 11;
    pub const A2: u32 = 12;
    pub const A3: u32 = 13;
    pub const A4: u32 = 14;
    pub const A5: u32 = 15;
    pub const A6: u32 = 16;
    pub const A7: u32 = 17;
    pub const S2: u32 = 18;
    pub const S3: u32 = 19;
    pub const S4: u32 = 20;
    pub const S5: u32 = 21;
    pub const S6: u32 = 22;
    pub const S7: u32 = 23;
    pub const S8: u32 = 24;
    pub const S9: u32 = 25;
    pub const S10: u32 = 26;
    pub const S11: u32 = 27;
    pub const T3: u32 = 28;
    pub const T4: u32 = 29;
    pub const T5: u32 = 30;
    pub const T6: u32 = 31;
}

// Syscall numbers
pub mod syscalls {
    pub const EXIT: u64 = 93;
    pub const READ: u64 = 63;
    pub const WRITE: u64 = 64;
    pub const OPENAT: u64 = 56;
    pub const BRK: u64 = 214;
}

// Instruction field extraction (from a 32-bit instruction word)
pub fn get_opcode(instr: u32) -> u32 {
    instr & 0x7F
}

pub fn get_rd(instr: u32) -> u32 {
    (instr >> 7) & 0x1F
}

pub fn get_funct3(instr: u32) -> u32 {
    (instr >> 12) & 0x7
}

pub fn get_rs1(instr: u32) -> u32 {
    (instr >> 15) & 0x1F
}

pub fn get_rs2(instr: u32) -> u32 {
    (instr >> 20) & 0x1F
}

pub fn get_funct7(instr: u32) -> u32 {
    (instr >> 25) & 0x7F
}

pub fn get_funct6(instr: u32) -> u32 {
    (instr >> 26) & 0x3F
}

// Immediate extraction
pub fn get_i_imm(instr: u32) -> i32 {
    (instr as i32) >> 20
}

pub fn get_s_imm(instr: u32) -> i32 {
    let imm11_5 = (instr >> 25) & 0x7F;
    let imm4_0 = (instr >> 7) & 0x1F;
    let imm = (imm11_5 << 5) | imm4_0;
    // Sign extend from bit 11
    if imm & 0x800 != 0 {
        (imm | 0xFFFFF000) as i32
    } else {
        imm as i32
    }
}

pub fn get_sb_imm(instr: u32) -> i32 {
    let bit12 = (instr >> 31) & 1;
    let bit11 = (instr >> 7) & 1;
    let bits10_5 = (instr >> 25) & 0x3F;
    let bits4_1 = (instr >> 8) & 0xF;
    let imm = (bit12 << 12) | (bit11 << 11) | (bits10_5 << 5) | (bits4_1 << 1);
    if imm & 0x1000 != 0 {
        (imm | 0xFFFFE000) as i32
    } else {
        imm as i32
    }
}

pub fn get_u_imm(instr: u32) -> i32 {
    (instr & 0xFFFFF000) as i32
}

pub fn get_uj_imm(instr: u32) -> i32 {
    let bit20 = (instr >> 31) & 1;
    let bits19_12 = (instr >> 12) & 0xFF;
    let bit11 = (instr >> 20) & 1;
    let bits10_1 = (instr >> 21) & 0x3FF;
    let imm = (bit20 << 20) | (bits19_12 << 12) | (bit11 << 11) | (bits10_1 << 1);
    if imm & 0x100000 != 0 {
        (imm | 0xFFE00000) as i32
    } else {
        imm as i32
    }
}

// Compressed instruction field extraction
pub fn get_c_opcode(instr: u16) -> u16 {
    instr & 0x3
}

pub fn get_c_funct3(instr: u16) -> u16 {
    (instr >> 13) & 0x7
}

pub fn get_c_funct2(instr: u16) -> u16 {
    (instr >> 5) & 0x3
}

pub fn get_c_rd(instr: u16) -> u32 {
    ((instr >> 7) & 0x1F) as u32
}

pub fn get_c_rs1(instr: u16) -> u32 {
    ((instr >> 7) & 0x1F) as u32
}

pub fn get_c_rs2(instr: u16) -> u32 {
    ((instr >> 2) & 0x1F) as u32
}

// Compressed register (3-bit, maps to x8-x15)
pub fn get_c_rd_prime(instr: u16) -> u32 {
    (((instr >> 2) & 0x7) + 8) as u32
}

pub fn get_c_rs1_prime(instr: u16) -> u32 {
    (((instr >> 7) & 0x7) + 8) as u32
}

pub fn get_c_rs2_prime(instr: u16) -> u32 {
    (((instr >> 2) & 0x7) + 8) as u32
}

pub fn is_compressed_instruction(instr: u16) -> bool {
    (instr & 0x3) != 0x3
}
