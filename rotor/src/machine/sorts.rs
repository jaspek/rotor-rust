use std::collections::HashMap;

use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::{Config, Xlen};
use crate::riscv::isa::InstrId;

/// All well-known BTOR2 sorts used in the machine model.
pub struct MachineSorts {
    // Basic sorts
    pub sid_boolean: NodeId,
    pub sid_byte: NodeId,
    pub sid_half_word: NodeId,
    pub sid_single_word: NodeId,
    pub sid_double_word: NodeId,
    pub sid_machine_word: NodeId,
    pub sid_double_machine_word: NodeId,

    // Bit-width sorts for immediate extraction
    pub sid_2bit: NodeId,
    pub sid_3bit: NodeId,
    pub sid_4bit: NodeId,
    pub sid_5bit: NodeId,
    pub sid_6bit: NodeId,
    pub sid_7bit: NodeId,
    pub sid_8bit: NodeId,
    pub sid_9bit: NodeId,
    pub sid_10bit: NodeId,
    pub sid_12bit: NodeId,
    pub sid_13bit: NodeId,
    pub sid_16bit: NodeId,
    pub sid_20bit: NodeId,
    pub sid_21bit: NodeId,

    // Instruction decoding sorts
    pub sid_opcode: NodeId,
    pub sid_funct3: NodeId,
    pub sid_funct6: NodeId,
    pub sid_funct7: NodeId,
    pub sid_instruction_id: NodeId,

    // Register file
    pub sid_register_address: NodeId,
    pub sid_register_state: NodeId,

    // Code segment
    pub sid_code_word: NodeId,
    pub sid_code_address: NodeId,
    pub sid_code_state: NodeId,

    // Memory segments
    pub sid_memory_word: NodeId,
    pub sid_data_address: NodeId,
    pub sid_data_state: NodeId,
    pub sid_heap_address: NodeId,
    pub sid_heap_state: NodeId,
    pub sid_stack_address: NodeId,
    pub sid_stack_state: NodeId,

    // Input buffer
    pub sid_input_address: NodeId,
    pub sid_input_buffer: NodeId,
}

impl MachineSorts {
    pub fn new(builder: &mut Btor2Builder, config: &Config) -> Self {
        let word_bits = config.machine_word_bits();

        // Basic sorts
        let sid_boolean = builder.bitvec(1, Some("Boolean".to_string()));
        let sid_byte = builder.bitvec(8, Some("8-bit byte".to_string()));
        let sid_half_word = builder.bitvec(16, Some("16-bit half word".to_string()));
        let sid_single_word = builder.bitvec(32, Some("32-bit single word".to_string()));
        let sid_double_word = builder.bitvec(64, Some("64-bit double word".to_string()));
        let sid_machine_word = if word_bits == 64 {
            sid_double_word
        } else {
            sid_single_word
        };
        let sid_double_machine_word = if word_bits == 64 {
            builder.bitvec(128, Some("128-bit double machine word".to_string()))
        } else {
            sid_double_word
        };

        // Bit-width sorts
        let sid_2bit = builder.bitvec(2, Some("2-bit".to_string()));
        let sid_3bit = builder.bitvec(3, Some("3-bit".to_string()));
        let sid_4bit = builder.bitvec(4, Some("4-bit".to_string()));
        let sid_5bit = builder.bitvec(5, Some("5-bit".to_string()));
        let sid_6bit = builder.bitvec(6, Some("6-bit".to_string()));
        let sid_7bit = builder.bitvec(7, Some("7-bit".to_string()));
        let sid_8bit = sid_byte;
        let sid_9bit = builder.bitvec(9, Some("9-bit".to_string()));
        let sid_10bit = builder.bitvec(10, Some("10-bit".to_string()));
        let sid_12bit = builder.bitvec(12, Some("12-bit".to_string()));
        let sid_13bit = builder.bitvec(13, Some("13-bit".to_string()));
        let sid_16bit = sid_half_word;
        let sid_20bit = builder.bitvec(20, Some("20-bit".to_string()));
        let sid_21bit = builder.bitvec(21, Some("21-bit".to_string()));

        // Instruction decode sorts
        let sid_opcode = builder.bitvec(7, Some("opcode".to_string()));
        let sid_funct3 = builder.bitvec(3, Some("funct3".to_string()));
        let sid_funct6 = builder.bitvec(6, Some("funct6".to_string()));
        let sid_funct7 = builder.bitvec(7, Some("funct7".to_string()));
        let sid_instruction_id = builder.bitvec(8, Some("instruction ID".to_string()));

        // Register file
        let sid_register_address = builder.bitvec(5, Some("5-bit register address".to_string()));
        let sid_register_state = builder.array(
            sid_register_address,
            sid_machine_word,
            Some("register file [5-bit -> machine word]".to_string()),
        );

        // Code segment — indexed by machine word (same width as PC)
        let sid_code_word = sid_single_word; // Instructions are always 32-bit
        let sid_code_address = sid_machine_word; // Use machine word for addressing
        let sid_code_state = builder.array(
            sid_code_address,
            sid_code_word,
            Some("code segment [addr -> 32-bit instruction]".to_string()),
        );

        // Memory segments (byte-addressable, indexed by machine word)
        let sid_memory_word = sid_byte;
        let sid_data_address = sid_machine_word;
        let sid_data_state = builder.array(
            sid_data_address,
            sid_memory_word,
            Some("data segment [addr -> byte]".to_string()),
        );
        let sid_heap_address = sid_machine_word;
        let sid_heap_state = builder.array(
            sid_heap_address,
            sid_memory_word,
            Some("heap segment [addr -> byte]".to_string()),
        );
        let sid_stack_address = sid_machine_word;
        let sid_stack_state = builder.array(
            sid_stack_address,
            sid_memory_word,
            Some("stack segment [addr -> byte]".to_string()),
        );

        // Input buffer
        let sid_input_address = sid_machine_word;
        let sid_input_buffer = builder.array(
            sid_input_address,
            sid_byte,
            Some("input buffer [addr -> byte]".to_string()),
        );

        Self {
            sid_boolean,
            sid_byte,
            sid_half_word,
            sid_single_word,
            sid_double_word,
            sid_machine_word,
            sid_double_machine_word,
            sid_2bit,
            sid_3bit,
            sid_4bit,
            sid_5bit,
            sid_6bit,
            sid_7bit,
            sid_8bit,
            sid_9bit,
            sid_10bit,
            sid_12bit,
            sid_13bit,
            sid_16bit,
            sid_20bit,
            sid_21bit,
            sid_opcode,
            sid_funct3,
            sid_funct6,
            sid_funct7,
            sid_instruction_id,
            sid_register_address,
            sid_register_state,
            sid_code_word,
            sid_code_address,
            sid_code_state,
            sid_memory_word,
            sid_data_address,
            sid_data_state,
            sid_heap_address,
            sid_heap_state,
            sid_stack_address,
            sid_stack_state,
            sid_input_address,
            sid_input_buffer,
        }
    }
}

/// Well-known constants used throughout the model.
pub struct MachineConstants {
    // Boolean
    pub nid_false: NodeId,
    pub nid_true: NodeId,

    // Machine word constants
    pub nid_machine_word_0: NodeId,
    pub nid_machine_word_1: NodeId,
    pub nid_machine_word_2: NodeId,
    pub nid_machine_word_3: NodeId,
    pub nid_machine_word_4: NodeId,
    pub nid_machine_word_5: NodeId,
    pub nid_machine_word_6: NodeId,
    pub nid_machine_word_7: NodeId,
    pub nid_machine_word_8: NodeId,
    pub nid_machine_word_minus_1: NodeId,

    // Byte constants
    pub nid_byte_0: NodeId,

    // Single word constants
    pub nid_single_word_0: NodeId,

    // Instruction size
    pub nid_instruction_size: NodeId,
    pub nid_compressed_instruction_size: NodeId,

    // Register address constants
    pub nid_register_addresses: [NodeId; 32],

    // Instruction ID constants
    pub nid_instruction_ids: HashMap<InstrId, NodeId>,

    // Syscall ID constants
    pub nid_exit_syscall: NodeId,
    pub nid_read_syscall: NodeId,
    pub nid_write_syscall: NodeId,
    pub nid_openat_syscall: NodeId,
    pub nid_brk_syscall: NodeId,
}

impl MachineConstants {
    pub fn new(builder: &mut Btor2Builder, sorts: &MachineSorts, config: &Config) -> Self {
        let sid_bool = sorts.sid_boolean;
        let sid_mw = sorts.sid_machine_word;
        let sid_byte = sorts.sid_byte;
        let sid_sw = sorts.sid_single_word;
        let sid_ra = sorts.sid_register_address;
        let sid_id = sorts.sid_instruction_id;

        // Boolean
        let nid_false = builder.constd(sid_bool, 0, Some("false".to_string()));
        let nid_true = builder.constd(sid_bool, 1, Some("true".to_string()));

        // Machine word
        let nid_machine_word_0 = builder.constd(sid_mw, 0, Some("machine word 0".to_string()));
        let nid_machine_word_1 = builder.constd(sid_mw, 1, Some("machine word 1".to_string()));
        let nid_machine_word_2 = builder.constd(sid_mw, 2, Some("machine word 2".to_string()));
        let nid_machine_word_3 = builder.constd(sid_mw, 3, Some("machine word 3".to_string()));
        let nid_machine_word_4 = builder.constd(sid_mw, 4, Some("machine word 4".to_string()));
        let nid_machine_word_5 = builder.constd(sid_mw, 5, Some("machine word 5".to_string()));
        let nid_machine_word_6 = builder.constd(sid_mw, 6, Some("machine word 6".to_string()));
        let nid_machine_word_7 = builder.constd(sid_mw, 7, Some("machine word 7".to_string()));
        let nid_machine_word_8 = builder.constd(sid_mw, 8, Some("machine word 8".to_string()));

        let minus_1 = if config.xlen == Xlen::X64 {
            u64::MAX
        } else {
            0xFFFFFFFF
        };
        let nid_machine_word_minus_1 = builder.constd(sid_mw, minus_1, Some("machine word -1".to_string()));

        // Byte
        let nid_byte_0 = builder.constd(sid_byte, 0, Some("byte 0".to_string()));

        // Single word
        let nid_single_word_0 = builder.constd(sid_sw, 0, Some("single word 0".to_string()));

        // Instruction sizes
        let nid_instruction_size = builder.constd(sid_mw, 4, Some("instruction size (4 bytes)".to_string()));
        let nid_compressed_instruction_size = builder.constd(
            sid_mw,
            2,
            Some("compressed instruction size (2 bytes)".to_string()),
        );

        // Register addresses
        let mut nid_register_addresses = [nid_false; 32]; // placeholder
        for i in 0..32u32 {
            nid_register_addresses[i as usize] = builder.constd(
                sid_ra,
                i as u64,
                Some(format!("register {}", crate::riscv::isa::REG_NAMES[i as usize])),
            );
        }

        // Instruction IDs
        let mut nid_instruction_ids = HashMap::new();
        let all_instrs = [
            InstrId::Unknown,
            InstrId::Ecall,
            InstrId::Add, InstrId::Sub, InstrId::Sll, InstrId::Slt, InstrId::Sltu,
            InstrId::Xor, InstrId::Srl, InstrId::Sra, InstrId::Or, InstrId::And,
            InstrId::Addw, InstrId::Subw, InstrId::Sllw, InstrId::Srlw, InstrId::Sraw,
            InstrId::Mul, InstrId::Mulh, InstrId::Mulhsu, InstrId::Mulhu,
            InstrId::Div, InstrId::Divu, InstrId::Rem, InstrId::Remu,
            InstrId::Mulw, InstrId::Divw, InstrId::Divuw, InstrId::Remw, InstrId::Remuw,
            InstrId::Addi, InstrId::Slti, InstrId::Sltiu, InstrId::Xori, InstrId::Ori, InstrId::Andi,
            InstrId::Slli, InstrId::Srli, InstrId::Srai,
            InstrId::Addiw, InstrId::Slliw, InstrId::Srliw, InstrId::Sraiw,
            InstrId::Lb, InstrId::Lh, InstrId::Lw, InstrId::Ld, InstrId::Lbu, InstrId::Lhu, InstrId::Lwu,
            InstrId::Jalr,
            InstrId::Sb, InstrId::Sh, InstrId::Sw, InstrId::Sd,
            InstrId::Beq, InstrId::Bne, InstrId::Blt, InstrId::Bge, InstrId::Bltu, InstrId::Bgeu,
            InstrId::Lui, InstrId::Auipc, InstrId::Jal,
            InstrId::CMv, InstrId::CAdd, InstrId::CJr, InstrId::CJalr,
            InstrId::CLi, InstrId::CLui, InstrId::CAddi, InstrId::CAddiw,
            InstrId::CAddi16sp, InstrId::CAddi4spn,
            InstrId::CSlli, InstrId::CSrli, InstrId::CSrai, InstrId::CAndi,
            InstrId::CSub, InstrId::CXor, InstrId::COr, InstrId::CAnd,
            InstrId::CAddw, InstrId::CSubw,
            InstrId::CLw, InstrId::CLd, InstrId::CSw, InstrId::CSd,
            InstrId::CLwsp, InstrId::CLdsp, InstrId::CSwsp, InstrId::CSdsp,
            InstrId::CBeqz, InstrId::CBnez, InstrId::CJ, InstrId::CJal,
        ];

        for (idx, instr) in all_instrs.iter().enumerate() {
            let nid = builder.constd(
                sid_id,
                idx as u64,
                Some(format!("ID_{}", instr.mnemonic())),
            );
            nid_instruction_ids.insert(*instr, nid);
        }

        // Syscall IDs (stored in a7 register)
        let nid_exit_syscall = builder.constd(sid_mw, crate::riscv::isa::syscalls::EXIT, Some("exit syscall id".to_string()));
        let nid_read_syscall = builder.constd(sid_mw, crate::riscv::isa::syscalls::READ, Some("read syscall id".to_string()));
        let nid_write_syscall = builder.constd(sid_mw, crate::riscv::isa::syscalls::WRITE, Some("write syscall id".to_string()));
        let nid_openat_syscall = builder.constd(sid_mw, crate::riscv::isa::syscalls::OPENAT, Some("openat syscall id".to_string()));
        let nid_brk_syscall = builder.constd(sid_mw, crate::riscv::isa::syscalls::BRK, Some("brk syscall id".to_string()));

        Self {
            nid_false,
            nid_true,
            nid_machine_word_0,
            nid_machine_word_1,
            nid_machine_word_2,
            nid_machine_word_3,
            nid_machine_word_4,
            nid_machine_word_5,
            nid_machine_word_6,
            nid_machine_word_7,
            nid_machine_word_8,
            nid_machine_word_minus_1,
            nid_byte_0,
            nid_single_word_0,
            nid_instruction_size,
            nid_compressed_instruction_size,
            nid_register_addresses,
            nid_instruction_ids,
            nid_exit_syscall,
            nid_read_syscall,
            nid_write_syscall,
            nid_openat_syscall,
            nid_brk_syscall,
        }
    }

    pub fn nid_instr_id(&self, instr: InstrId) -> NodeId {
        self.nid_instruction_ids[&instr]
    }

    pub fn nid_register(&self, reg: u32) -> NodeId {
        self.nid_register_addresses[reg as usize]
    }
}
