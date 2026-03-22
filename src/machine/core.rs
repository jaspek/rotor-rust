use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::Config;
use crate::machine::kernel::KernelState;
use crate::machine::segmentation::Segmentation;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::riscv::elf_loader::LoadedBinary;

/// Per-core machine state: PC, IR, register file, memory segments, kernel.
pub struct CoreState {
    // Program counter
    pub pc_state: NodeId,
    pub pc_nid: NodeId,

    // Instruction register (32-bit fetched instruction)
    pub ir: Option<NodeId>,

    // Compressed instruction register (16-bit, if C extension)
    pub c_ir: Option<NodeId>,

    // Decoded instruction ID
    pub instruction_id: Option<NodeId>,

    // Register file
    pub register_file_state: NodeId,

    // Code segment (read-only, initialized from binary)
    pub code_segment_state: NodeId,

    // Writable memory segments
    pub data_segment_state: NodeId,
    pub heap_segment_state: NodeId,
    pub stack_segment_state: NodeId,

    // Segment boundaries
    pub segmentation: Segmentation,

    // Kernel state
    pub kernel: KernelState,

    // Core identifier
    pub core_id: usize,
}

impl CoreState {
    /// Create and initialize all state for one core.
    pub fn new(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        config: &Config,
        binary: &LoadedBinary,
        core_id: usize,
    ) -> Self {
        let prefix = if config.num_cores > 1 {
            format!("core{}-", core_id)
        } else {
            String::new()
        };

        // Program counter
        let pc_state = builder.state(
            sorts.sid_machine_word,
            &format!("{}pc", prefix),
            Some("program counter".to_string()),
        );
        let entry = builder.constd(
            sorts.sid_machine_word,
            binary.entry_point,
            Some(format!("entry point 0x{:x}", binary.entry_point)),
        );
        let _pc_init = builder.init(
            sorts.sid_machine_word,
            pc_state,
            entry,
            Some("init PC to entry point".to_string()),
        );

        // Register file
        let register_file_state = builder.state(
            sorts.sid_register_state,
            &format!("{}register-file", prefix),
            Some("register file state".to_string()),
        );

        // Initialize sp (x2) to top of stack
        // We initialize the register file with x0 = 0 implicitly (array default)
        // and set sp to stack_end - 8 (typical ABI convention)
        let vaddr_top = 1u64 << 31;
        let initial_sp = vaddr_top - 8;
        let sp_val = builder.constd(
            sorts.sid_machine_word,
            initial_sp,
            Some("initial stack pointer".to_string()),
        );
        let sp_addr = consts.nid_register(crate::riscv::isa::regs::SP);
        let reg_with_sp = builder.write(
            sorts.sid_register_state,
            register_file_state,
            sp_addr,
            sp_val,
            Some("set initial SP".to_string()),
        );

        // Set x0 = 0 explicitly
        let zero_val = consts.nid_machine_word_0;
        let x0_addr = consts.nid_register(crate::riscv::isa::regs::ZR);
        let initial_regs = builder.write(
            sorts.sid_register_state,
            reg_with_sp,
            x0_addr,
            zero_val,
            Some("x0 = 0".to_string()),
        );

        let _reg_init = builder.init(
            sorts.sid_register_state,
            register_file_state,
            initial_regs,
            Some("init register file".to_string()),
        );

        // Code segment (initialized from binary)
        let code_segment_state = builder.state(
            sorts.sid_code_state,
            &format!("{}code-segment", prefix),
            Some("code segment (read-only)".to_string()),
        );
        let code_init_val = Self::initialize_code_segment(builder, sorts, consts, binary);
        let _code_init = builder.init(
            sorts.sid_code_state,
            code_segment_state,
            code_init_val,
            Some("init code segment from binary".to_string()),
        );

        // Data segment
        let data_segment_state = builder.state(
            sorts.sid_data_state,
            &format!("{}data-segment", prefix),
            Some("data segment".to_string()),
        );
        let data_init_val = Self::initialize_data_segment(builder, sorts, binary);
        let _data_init = builder.init(
            sorts.sid_data_state,
            data_segment_state,
            data_init_val,
            Some("init data segment from binary".to_string()),
        );

        // Heap segment (initially empty)
        let heap_segment_state = builder.state(
            sorts.sid_heap_state,
            &format!("{}heap-segment", prefix),
            Some("heap segment".to_string()),
        );

        // Stack segment (initially empty except for sp setup)
        let stack_segment_state = builder.state(
            sorts.sid_stack_state,
            &format!("{}stack-segment", prefix),
            Some("stack segment".to_string()),
        );

        // Segmentation
        let segmentation = Segmentation::new(
            builder,
            sorts,
            consts,
            binary,
            config.heap_allowance,
            config.stack_allowance,
        );

        // Kernel state
        let initial_brk = binary.data_start + binary.data_size;
        let kernel = KernelState::new(builder, sorts, consts, initial_brk, config.bytes_to_read);

        CoreState {
            pc_state,
            pc_nid: entry,
            ir: None,
            c_ir: None,
            instruction_id: None,
            register_file_state,
            code_segment_state,
            data_segment_state,
            heap_segment_state,
            stack_segment_state,
            segmentation,
            kernel,
            core_id,
        }
    }

    /// Initialize code segment array from binary code.
    /// Writes each 4-byte instruction word into the code segment array.
    fn initialize_code_segment(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        _consts: &MachineConstants,
        binary: &LoadedBinary,
    ) -> NodeId {
        // Start with an input (uninitialized) code segment
        let code_seg = builder.input(
            sorts.sid_code_state,
            "initial-code",
            Some("uninitialized code segment".to_string()),
        );

        let mut current = code_seg;

        // Write each instruction (4 bytes at a time) into the code segment
        let num_instrs = binary.code.len() / 4;
        for i in 0..num_instrs {
            let offset = i * 4;
            let instr_bytes = &binary.code[offset..offset + 4];
            let instr_word = u32::from_le_bytes([
                instr_bytes[0],
                instr_bytes[1],
                instr_bytes[2],
                instr_bytes[3],
            ]);

            let addr_val = binary.code_start + offset as u64;
            let addr = builder.constd(sorts.sid_code_address, addr_val, None);
            let word = builder.constd(
                sorts.sid_code_word,
                instr_word as u64,
                Some(format!("code[0x{:x}] = 0x{:08x}", addr_val, instr_word)),
            );

            current = builder.write(sorts.sid_code_state, current, addr, word, None);
        }

        current
    }

    /// Initialize data segment array from binary data.
    fn initialize_data_segment(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        binary: &LoadedBinary,
    ) -> NodeId {
        let data_seg = builder.input(
            sorts.sid_data_state,
            "initial-data",
            Some("uninitialized data segment".to_string()),
        );

        let mut current = data_seg;

        // Write each byte from the data section
        for (i, &byte_val) in binary.data.iter().enumerate() {
            if byte_val == 0 {
                continue; // Skip zero bytes (array default is unspecified, but typically models handle this)
            }
            let addr_val = binary.data_start + i as u64;
            let addr = builder.constd(sorts.sid_data_address, addr_val, None);
            let val = builder.constd(sorts.sid_byte, byte_val as u64, None);
            current = builder.write(sorts.sid_data_state, current, addr, val, None);
        }

        current
    }
}
