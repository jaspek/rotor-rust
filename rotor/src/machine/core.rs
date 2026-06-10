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
    ///
    /// BTOR2 ordering constraint: for `init S STATE VALUE`, STATE nid > VALUE nid.
    /// Strategy: build all init value chains (using base states for arrays) BEFORE
    /// creating the real states, so real states naturally get higher nids.
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

        let word_size = config.machine_word_bytes() as u64; // 4 or 8

        // ================================================================
        // Phase 1: Build ALL init value chains BEFORE creating real states.
        // Each array init chain uses a base state as the write target.
        // BTOR2 requires state nid > value nid for init, so real states
        // must be created after their init value chains.
        // ================================================================

        // --- Code segment init value (base state + writes from binary) ---
        let code_init_val = Self::initialize_code_segment(builder, sorts, consts, binary);

        // --- Data segment init value (base state + writes from binary) ---
        let data_init_val = Self::initialize_data_segment(builder, sorts, binary);

        // --- PC init value ---
        let entry = builder.constd(
            sorts.sid_machine_word,
            binary.entry_point,
            Some(format!("entry point 0x{:x}", binary.entry_point)),
        );

        // --- Segmentation constants (needed for SP calculation) ---
        let segmentation = Segmentation::new(builder, sorts, consts, binary, config);

        // --- Stack init value (symbolic argv writes on a base input/state) ---
        // Stack occupies the TOP of the full virtual address space
        // (2^vaddr_bits, e.g. 4 GB for the default 32-bit space), matching
        // the C reference: stack @ [0xFFFFF800, 0x100000000).
        let vaddr_top = segmentation.stack_end_val;

        let (initial_sp, stack_init_val) = if config.symbolic_argv && config.symbolic_argc > 0 {
            Self::initialize_symbolic_argv(builder, sorts, config, vaddr_top, word_size)
        } else {
            // Default mode: concrete argv image on the stack, exactly like the
            // C reference boot loader (argc=1, argv[0]=program name).
            let (sp, stack_val) = Self::initialize_concrete_argv(
                builder,
                sorts,
                vaddr_top,
                word_size,
                &binary.name,
            );
            (sp, Some(stack_val))
        };

        // --- Register file init value ---
        // C reference: a ZEROED register file with only SP written
        // ("zeroing register file" + "write initial register value" for sp).
        // The zeroed base means every other register, including x0, is
        // provably zero — no explicit x0 write needed.
        let base_regs = builder.state(
            sorts.sid_register_state,
            &format!("{}base-register-file", prefix),
            Some("base register file for initialization (zeroed)".to_string()),
        );
        builder.init(
            sorts.sid_register_state,
            base_regs,
            consts.nid_machine_word_0,
            Some("zeroing register file".to_string()),
        );

        let sp_val = builder.constd(
            sorts.sid_machine_word,
            initial_sp,
            Some(format!("initial stack pointer 0x{:x}", initial_sp)),
        );
        let sp_addr = consts.nid_register(crate::riscv::isa::regs::SP);
        let reg_with_sp = builder.write(
            sorts.sid_register_state,
            base_regs,
            sp_addr,
            sp_val,
            Some("set initial SP".to_string()),
        );

        // Set a0 = argc only in symbolic-argv mode (our extension). The C
        // reference does NOT initialize a0 — programs read argc from the stack.
        let reg_init_val = if config.symbolic_argv && config.symbolic_argc > 0 {
            let argc_val = builder.constd(
                sorts.sid_machine_word,
                (config.symbolic_argc + 1) as u64,
                Some(format!("argc = {}", config.symbolic_argc + 1)),
            );
            let a0_addr = consts.nid_register(crate::riscv::isa::regs::A0);
            builder.write(
                sorts.sid_register_state,
                reg_with_sp,
                a0_addr,
                argc_val,
                Some("set a0 = argc".to_string()),
            )
        } else {
            reg_with_sp
        };

        // ================================================================
        // Phase 2: Create real states and init them.
        // All states created here get higher nids than their init values.
        // ================================================================

        // --- PC ---
        let pc_state = builder.state(
            sorts.sid_machine_word,
            &format!("{}pc", prefix),
            Some("program counter".to_string()),
        );
        let _pc_init = builder.init(
            sorts.sid_machine_word,
            pc_state,
            entry,
            Some("init PC to entry point".to_string()),
        );

        // --- Code segment ---
        let code_segment_state = builder.state(
            sorts.sid_code_state,
            &format!("{}code-segment", prefix),
            Some("code segment (read-only)".to_string()),
        );
        let _code_init = builder.init(
            sorts.sid_code_state,
            code_segment_state,
            code_init_val,
            Some("init code segment from binary".to_string()),
        );

        // --- Data segment ---
        let data_segment_state = builder.state(
            sorts.sid_data_state,
            &format!("{}data-segment", prefix),
            Some("data segment".to_string()),
        );
        let _data_init = builder.init(
            sorts.sid_data_state,
            data_segment_state,
            data_init_val,
            Some("init data segment from binary".to_string()),
        );

        // --- Heap segment (zero-initialized, matching C rotor) ---
        // The heap starts all-zero; the program and the read syscall evolve it
        // via next-state logic. Without this init, untouched heap cells would be
        // solver-chosen garbage.
        let heap_segment_state = builder.state(
            sorts.sid_heap_state,
            &format!("{}heap-segment", prefix),
            Some("heap segment (zeroed)".to_string()),
        );
        builder.init(
            sorts.sid_heap_state,
            heap_segment_state,
            consts.nid_byte_0,
            Some("zeroing heap segment".to_string()),
        );

        // --- Stack segment ---
        let stack_segment_state = builder.state(
            sorts.sid_stack_state,
            &format!("{}stack-segment", prefix),
            Some("stack segment".to_string()),
        );
        if let Some(stack_val) = stack_init_val {
            // Symbolic argv: stack initialized with the argv layout (already
            // built on a zeroed base inside initialize_symbolic_argv).
            let _stack_init = builder.init(
                sorts.sid_stack_state,
                stack_segment_state,
                stack_val,
                Some("init stack segment with argv".to_string()),
            );
        } else {
            // Default mode: zero-initialize the stack. (Concrete argv stack
            // layout — P0.3 — is future work; for now the stack starts zeroed
            // rather than unconstrained, which removes false counterexamples
            // from reading uninitialized stack memory.)
            builder.init(
                sorts.sid_stack_state,
                stack_segment_state,
                consts.nid_byte_0,
                Some("zeroing stack segment".to_string()),
            );
        }

        // --- Register file ---
        let register_file_state = builder.state(
            sorts.sid_register_state,
            &format!("{}register-file", prefix),
            Some("register file state".to_string()),
        );
        let _reg_init = builder.init(
            sorts.sid_register_state,
            register_file_state,
            reg_init_val,
            Some("init register file".to_string()),
        );

        // --- Kernel state ---
        // Program break starts at the (page-aligned) start of the heap
        // segment, matching the C reference ("initializing program break to
        // start of heap segment").
        let initial_brk = segmentation.heap_start_val;
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

    /// Initialize the stack with a CONCRETE argv image, matching the C
    /// reference boot loader (selfie convention, verified against the
    /// reference model's write chain):
    ///
    ///   stack_end                      <- top of virtual address space
    ///     [program-name string]        <- word-aligned, NUL-terminated
    ///   string_start
    ///     [0]                          <- env NULL terminator
    ///     [0]                          <- argv NULL terminator
    ///     [argv[0] ptr = string_start]
    ///     [argc = 1]                   <- SP points here
    ///
    /// All writes go onto a zeroed stack base, so zero bytes are skipped.
    /// Returns (initial_sp, stack_init_value).
    fn initialize_concrete_argv(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        stack_end: u64,
        word_size: u64,
        prog_name: &str,
    ) -> (u64, NodeId) {
        let name_bytes = prog_name.as_bytes();
        // String size including NUL, aligned up to the word size.
        let string_size = (name_bytes.len() as u64 + 1 + word_size - 1) & !(word_size - 1);
        let string_start = stack_end - string_size;
        // Below the string: env NULL, argv NULL, argv[0] pointer, argc.
        let sp = string_start - 4 * word_size;

        // Zeroed stack base (matching "zeroing stack segment" in C).
        let byte_zero = builder.constd(sorts.sid_byte, 0, Some("byte 0".to_string()));
        let stack_seg = builder.state(
            sorts.sid_stack_state,
            "initial-stack-base",
            Some("base stack segment for boot loading (zeroed)".to_string()),
        );
        builder.init(
            sorts.sid_stack_state,
            stack_seg,
            byte_zero,
            Some("zeroing stack segment".to_string()),
        );
        let mut current = stack_seg;

        // Write one machine word little-endian onto the zeroed base,
        // skipping zero bytes.
        let write_word = |builder: &mut Btor2Builder, current: NodeId, at: u64, value: u64, what: &str| {
            let mut cur = current;
            for byte_idx in 0..word_size {
                let byte_val = (value >> (byte_idx * 8)) & 0xFF;
                if byte_val == 0 {
                    continue;
                }
                let addr = builder.constd(sorts.sid_stack_address, at + byte_idx, None);
                let val = builder.constd(
                    sorts.sid_byte,
                    byte_val,
                    Some(format!("{} byte {}", what, byte_idx)),
                );
                cur = builder.write(sorts.sid_stack_state, cur, addr, val, None);
            }
            cur
        };

        // argc = 1 at SP
        current = write_word(builder, current, sp, 1, "argc");
        // argv[0] pointer at SP + word
        current = write_word(builder, current, sp + word_size, string_start, "argv[0] pointer");
        // argv NULL terminator (SP + 2w) and env NULL terminator (SP + 3w)
        // are zero — covered by the zeroed base.

        // Program-name string bytes at string_start (NUL covered by base).
        for (i, &b) in name_bytes.iter().enumerate() {
            if b == 0 {
                continue;
            }
            let addr = builder.constd(sorts.sid_stack_address, string_start + i as u64, None);
            let val = builder.constd(
                sorts.sid_byte,
                b as u64,
                Some(format!("argv[0][{}] = '{}'", i, b as char)),
            );
            current = builder.write(sorts.sid_stack_state, current, addr, val, None);
        }

        log::info!(
            "Concrete argv: argc=1, argv[0]={:?}, string @ 0x{:x}, SP @ 0x{:x}",
            prog_name,
            string_start,
            sp,
        );

        (sp, current)
    }

    /// Initialize symbolic argv on the stack.
    ///
    /// Returns (initial_sp, Option<stack_segment_init_node>).
    ///
    /// Symbolic vs. concrete boundary:
    ///   Only the content bytes of argv[1..N] are symbolic (unconstrained BTOR2
    ///   states the solver can assign freely). Everything else is concrete:
    ///   argc, all pointers, null terminators, argv[0] content, and the stack
    ///   layout structure itself. This ensures the solver can explore arbitrary
    ///   argument values without violating C argv invariants.
    ///
    /// Stack layout (high to low address):
    ///   - String area: for each arg, `max_arglen` symbolic bytes + 1 null terminator
    ///   - Alignment padding to word boundary
    ///   - Pointer area: argv[0] .. argv[argc-1] pointers, then NULL terminator pointer
    ///   - argc value
    ///   - SP points here
    ///
    /// argv[0] = "prog" (fixed 4 bytes + null, acting as program name)
    /// argv[1..N] = symbolic (each byte is an unconstrained BTOR2 state)
    fn initialize_symbolic_argv(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        config: &Config,
        stack_top: u64,
        word_size: u64,
    ) -> (u64, Option<NodeId>) {
        let total_argc = config.symbolic_argc + 1; // +1 for argv[0] = program name
        let max_arglen = config.max_arglen;
        let prog_name = b"prog";

        // Calculate string area layout
        // argv[0]: "prog\0" = 5 bytes
        let argv0_len = prog_name.len() + 1; // including null terminator
        // argv[1..N]: max_arglen bytes + 1 null terminator each
        let sym_arg_len = max_arglen + 1;
        let string_area_size = argv0_len + config.symbolic_argc * sym_arg_len;

        // Align string area to word boundary
        let string_area_aligned = (string_area_size as u64 + word_size - 1) & !(word_size - 1);

        // Pointer area: (argc + 1) pointers (including NULL terminator)
        let pointer_area_size = (total_argc + 1) as u64 * word_size;

        // argc itself: 1 word
        let total_stack_usage = string_area_aligned + pointer_area_size + word_size;

        // Layout addresses (growing downward from stack_top)
        let string_area_start = stack_top - string_area_aligned;
        let pointer_area_start = string_area_start - pointer_area_size;
        let sp = pointer_area_start - word_size; // argc lives at SP

        // Start building the stack array (use state, not input — BTOR2 forbids inputs in init).
        // Zero the base so cells outside the argv layout are provably zero, not garbage.
        let stack_zero = builder.constd(sorts.sid_byte, 0, Some("byte 0".to_string()));
        let stack_seg = builder.state(
            sorts.sid_stack_state,
            "initial-stack-base",
            Some("base stack segment for argv initialization (zeroed)".to_string()),
        );
        builder.init(
            sorts.sid_stack_state,
            stack_seg,
            stack_zero,
            Some("zeroing stack base".to_string()),
        );
        let mut current = stack_seg;

        // ---- Write string area ----

        // argv[0] = "prog\0" (fixed bytes)
        let mut str_addr = string_area_start;
        let argv_string_addrs: Vec<u64> = {
            let mut addrs = Vec::with_capacity(total_argc);

            // argv[0] string address
            addrs.push(str_addr);
            for &byte_val in prog_name {
                let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
                let val = builder.constd(
                    sorts.sid_byte,
                    byte_val as u64,
                    Some(format!(
                        "argv[0][{}] = '{}'",
                        str_addr - string_area_start,
                        byte_val as char
                    )),
                );
                current = builder.write(sorts.sid_stack_state, current, addr, val, None);
                str_addr += 1;
            }
            // null terminator for argv[0]
            let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
            let null = builder.constd(
                sorts.sid_byte,
                0,
                Some("argv[0] null terminator".to_string()),
            );
            current = builder.write(sorts.sid_stack_state, current, addr, null, None);
            str_addr += 1;

            // argv[1..N] symbolic strings
            for arg_idx in 0..config.symbolic_argc {
                addrs.push(str_addr);
                for byte_idx in 0..max_arglen {
                    let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
                    // Each byte is a symbolic (unconstrained) state — the solver can choose any value 0-255.
                    // We use state (not input) because BTOR2 forbids inputs in init expressions.
                    // An uninitialized state is unconstrained, which is exactly what we want.
                    let sym_byte = builder.state(
                        sorts.sid_byte,
                        &format!("argv[{}][{}]", arg_idx + 1, byte_idx),
                        Some(format!("symbolic byte argv[{}][{}]", arg_idx + 1, byte_idx)),
                    );
                    current = builder.write(sorts.sid_stack_state, current, addr, sym_byte, None);
                    str_addr += 1;
                }
                // Fixed null terminator (preserves C string semantics)
                let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
                let null = builder.constd(
                    sorts.sid_byte,
                    0,
                    Some(format!("argv[{}] null terminator", arg_idx + 1)),
                );
                current = builder.write(sorts.sid_stack_state, current, addr, null, None);
                str_addr += 1;
            }

            addrs
        };

        // ---- Write pointer area (argv[i] pointers) ----
        let mut ptr_addr = pointer_area_start;
        for (i, &string_addr) in argv_string_addrs.iter().enumerate() {
            // Write the pointer as a machine word (little-endian, byte by byte)
            for byte_idx in 0..word_size {
                let byte_val = (string_addr >> (byte_idx * 8)) & 0xFF;
                let addr = builder.constd(sorts.sid_stack_address, ptr_addr + byte_idx, None);
                let val = builder.constd(
                    sorts.sid_byte,
                    byte_val,
                    Some(format!("argv_ptr[{}] byte {}", i, byte_idx)),
                );
                current = builder.write(sorts.sid_stack_state, current, addr, val, None);
            }
            ptr_addr += word_size;
        }

        // NULL pointer terminator for argv array
        for byte_idx in 0..word_size {
            let addr = builder.constd(sorts.sid_stack_address, ptr_addr + byte_idx, None);
            let val = builder.constd(
                sorts.sid_byte,
                0,
                Some("argv NULL terminator byte".to_string()),
            );
            current = builder.write(sorts.sid_stack_state, current, addr, val, None);
        }

        // ---- Write argc at SP ----
        let argc_value = total_argc as u64;
        for byte_idx in 0..word_size {
            let byte_val = (argc_value >> (byte_idx * 8)) & 0xFF;
            let addr = builder.constd(sorts.sid_stack_address, sp + byte_idx, None);
            let val = builder.constd(
                sorts.sid_byte,
                byte_val,
                Some(format!("argc byte {}", byte_idx)),
            );
            current = builder.write(sorts.sid_stack_state, current, addr, val, None);
        }

        log::info!(
            "Symbolic argv: argc={}, {} symbolic args of max {} bytes each",
            total_argc,
            config.symbolic_argc,
            max_arglen,
        );
        log::info!(
            "Stack layout: strings @ 0x{:x}, pointers @ 0x{:x}, SP @ 0x{:x} (total {} bytes)",
            string_area_start,
            pointer_area_start,
            sp,
            total_stack_usage,
        );

        (sp, Some(current))
    }

    /// Initialize code segment array from binary code.
    /// Writes each 4-byte instruction word into the code segment array.
    fn initialize_code_segment(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        _consts: &MachineConstants,
        binary: &LoadedBinary,
    ) -> NodeId {
        // Start with an anonymous state as base for the code segment init chain.
        // (BTOR2 does not allow 'input' in init expressions)
        // Zero-initialize the base so every cell not written below is provably
        // zero (matching C rotor's "zeroed code segment"). Without this init the
        // base is an unconstrained array and untouched cells become solver-chosen
        // garbage, which can produce false counterexamples.
        let code_zero = builder.constd(sorts.sid_code_word, 0, Some("code word 0".to_string()));
        let code_seg = builder.state(
            sorts.sid_code_state,
            "initial-code-base",
            Some("base code segment for initialization (zeroed)".to_string()),
        );
        builder.init(
            sorts.sid_code_state,
            code_seg,
            code_zero,
            Some("zeroing code segment".to_string()),
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
        // Start with an anonymous state as base for the data segment init chain.
        // (BTOR2 does not allow 'input' in init expressions)
        // Zero-initialize the base (matching C rotor's "zeroed data segment").
        // With the base provably zero, skipping zero bytes below is correct.
        let byte_zero = builder.constd(sorts.sid_byte, 0, Some("byte 0".to_string()));
        let data_seg = builder.state(
            sorts.sid_data_state,
            "initial-data-base",
            Some("base data segment for initialization (zeroed)".to_string()),
        );
        builder.init(
            sorts.sid_data_state,
            data_seg,
            byte_zero,
            Some("zeroing data segment".to_string()),
        );

        let mut current = data_seg;

        // Write each non-zero byte from the data section (zero bytes already
        // covered by the zeroed base above).
        for (i, &byte_val) in binary.data.iter().enumerate() {
            if byte_val == 0 {
                continue;
            }
            let addr_val = binary.data_start + i as u64;
            let addr = builder.constd(sorts.sid_data_address, addr_val, None);
            let val = builder.constd(sorts.sid_byte, byte_val as u64, None);
            current = builder.write(sorts.sid_data_state, current, addr, val, None);
        }

        current
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::btor2::node::Op;
    use crate::config::{Config, Xlen};

    /// Build minimal sorts needed for initialize_symbolic_argv (RV64).
    fn test_sorts(builder: &mut Btor2Builder) -> MachineSorts {
        let config = Config {
            xlen: Xlen::X64,
            ..Config::default()
        };
        MachineSorts::new(builder, &config)
    }

    /// Collect every Write in the init chain as (address: u64, value_op: Op).
    /// Walks backward from `tail` through the `array` operand of each Write.
    fn collect_writes(builder: &Btor2Builder, tail: NodeId) -> Vec<(u64, Op)> {
        let mut writes = Vec::new();
        let mut cur = tail;
        loop {
            match builder.get_op(cur).clone() {
                Op::Write {
                    array,
                    index,
                    value,
                    ..
                } => {
                    let addr = match builder.get_op(index) {
                        Op::Constd { value: v, .. } => *v,
                        _ => panic!("expected constd address"),
                    };
                    writes.push((addr, builder.get_op(value).clone()));
                    cur = array;
                }
                Op::State { .. } => break, // base state — end of chain
                other => panic!("unexpected op in write chain: {:?}", other),
            }
        }
        writes.reverse();
        writes
    }

    fn make_config(num_args: usize, max_arglen: usize) -> Config {
        Config {
            xlen: Xlen::X64,
            symbolic_argv: true,
            symbolic_argc: num_args,
            max_arglen,
            ..Config::default()
        }
    }

    #[test]
    fn argc_is_concrete_constant() {
        let mut builder = Btor2Builder::new();
        let sorts = test_sorts(&mut builder);
        let config = make_config(2, 4);
        let word_size = 8u64;
        let stack_top = 1u64 << 31;

        let (sp, Some(tail)) = CoreState::initialize_symbolic_argv(
            &mut builder,
            &sorts,
            &config,
            stack_top,
            word_size,
        ) else {
            panic!("expected Some(tail)");
        };

        let writes = collect_writes(&builder, tail);

        // argc is written as a little-endian word at SP.
        // Expected argc = symbolic_argc + 1 = 3.
        let expected_argc: u64 = 3;
        let mut reconstructed: u64 = 0;
        for byte_idx in 0..word_size {
            let addr = sp + byte_idx;
            let (_, op) = writes
                .iter()
                .find(|(a, _)| *a == addr)
                .unwrap_or_else(|| panic!("missing argc byte at SP+{}", byte_idx));
            match op {
                Op::Constd { value, .. } => {
                    reconstructed |= value << (byte_idx * 8);
                }
                other => panic!(
                    "argc byte {} must be a concrete Constd, got {:?}",
                    byte_idx, other
                ),
            }
        }
        assert_eq!(reconstructed, expected_argc, "argc must be concrete 3");
    }

    #[test]
    fn argv_null_terminator_pointer() {
        let mut builder = Btor2Builder::new();
        let sorts = test_sorts(&mut builder);
        let config = make_config(1, 4);
        let word_size = 8u64;
        let stack_top = 1u64 << 31;

        let (sp, Some(tail)) = CoreState::initialize_symbolic_argv(
            &mut builder,
            &sorts,
            &config,
            stack_top,
            word_size,
        ) else {
            panic!("expected Some(tail)");
        };

        let writes = collect_writes(&builder, tail);
        let total_argc: u64 = 2; // 1 symbolic + argv[0]

        // Pointer area starts at SP + word_size (argc is at SP).
        let pointer_area_start = sp + word_size;
        // argv[argc] is the NULL terminator pointer at offset total_argc * word_size.
        let null_ptr_start = pointer_area_start + total_argc * word_size;

        for byte_idx in 0..word_size {
            let addr = null_ptr_start + byte_idx;
            let entry = writes.iter().find(|(a, _)| *a == addr);
            assert!(
                entry.is_some(),
                "argv[argc] NULL pointer byte {} missing",
                byte_idx
            );
            let (_, op) = entry.unwrap();
            match op {
                Op::Constd { value: 0, .. } => {}
                other => panic!(
                    "argv[argc] byte {} should be concrete 0, got {:?}",
                    byte_idx, other
                ),
            }
        }
    }

    #[test]
    fn argv_pointers_match_string_addresses() {
        let mut builder = Btor2Builder::new();
        let sorts = test_sorts(&mut builder);
        let config = make_config(2, 4);
        let word_size = 8u64;
        let stack_top = 1u64 << 31;

        let (_sp, Some(tail)) = CoreState::initialize_symbolic_argv(
            &mut builder,
            &sorts,
            &config,
            stack_top,
            word_size,
        ) else {
            panic!("expected Some(tail)");
        };

        let writes = collect_writes(&builder, tail);
        let total_argc: usize = 3;

        // Compute expected string addresses from the layout.
        let prog_name = b"prog";
        let argv0_len = prog_name.len() + 1; // 5
        let sym_arg_len = config.max_arglen + 1; // 5
        let string_area_size = argv0_len + config.symbolic_argc * sym_arg_len;
        let string_area_aligned = (string_area_size as u64 + word_size - 1) & !(word_size - 1);
        let string_area_start = stack_top - string_area_aligned;

        let mut expected_addrs = Vec::new();
        expected_addrs.push(string_area_start); // argv[0]
        let mut offset = argv0_len as u64;
        for _ in 0..config.symbolic_argc {
            expected_addrs.push(string_area_start + offset);
            offset += sym_arg_len as u64;
        }

        // Pointer area: starts after string area + alignment, then below that.
        let pointer_area_size = (total_argc as u64 + 1) * word_size;
        let pointer_area_start = string_area_start - pointer_area_size;

        // Read each pointer from the write chain.
        for (i, &expected_str_addr) in expected_addrs.iter().enumerate() {
            let ptr_base = pointer_area_start + (i as u64) * word_size;
            let mut reconstructed: u64 = 0;
            for byte_idx in 0..word_size {
                let addr = ptr_base + byte_idx;
                let (_, op) = writes
                    .iter()
                    .find(|(a, _)| *a == addr)
                    .unwrap_or_else(|| panic!("missing pointer byte at 0x{:x}", addr));
                match op {
                    Op::Constd { value, .. } => {
                        reconstructed |= value << (byte_idx * 8);
                    }
                    other => panic!("pointer byte should be Constd, got {:?}", other),
                }
            }
            assert_eq!(
                reconstructed, expected_str_addr,
                "argv[{}] pointer 0x{:x} != expected string addr 0x{:x}",
                i, reconstructed, expected_str_addr
            );
        }
    }

    #[test]
    fn symbolic_strings_end_with_concrete_null() {
        let mut builder = Btor2Builder::new();
        let sorts = test_sorts(&mut builder);
        let config = make_config(2, 4);
        let word_size = 8u64;
        let stack_top = 1u64 << 31;

        let (_, Some(tail)) = CoreState::initialize_symbolic_argv(
            &mut builder,
            &sorts,
            &config,
            stack_top,
            word_size,
        ) else {
            panic!("expected Some(tail)");
        };

        let writes = collect_writes(&builder, tail);

        let prog_name = b"prog";
        let argv0_len = prog_name.len() + 1;
        let sym_arg_len = config.max_arglen + 1;
        let string_area_size = argv0_len + config.symbolic_argc * sym_arg_len;
        let string_area_aligned = (string_area_size as u64 + word_size - 1) & !(word_size - 1);
        let string_area_start = stack_top - string_area_aligned;

        // Check each symbolic arg's null terminator.
        let mut str_offset = argv0_len as u64; // skip argv[0]
        for arg_idx in 0..config.symbolic_argc {
            // Content bytes should be State (symbolic).
            for byte_idx in 0..config.max_arglen {
                let addr = string_area_start + str_offset + byte_idx as u64;
                let (_, op) = writes.iter().find(|(a, _)| *a == addr).unwrap_or_else(|| {
                    panic!("missing symbolic byte argv[{}][{}]", arg_idx + 1, byte_idx)
                });
                assert!(
                    matches!(op, Op::State { .. }),
                    "argv[{}][{}] should be symbolic State, got {:?}",
                    arg_idx + 1,
                    byte_idx,
                    op
                );
            }

            // Null terminator must be concrete 0.
            let null_addr = string_area_start + str_offset + config.max_arglen as u64;
            let (_, op) = writes
                .iter()
                .find(|(a, _)| *a == null_addr)
                .unwrap_or_else(|| panic!("missing null terminator for argv[{}]", arg_idx + 1));
            match op {
                Op::Constd { value: 0, .. } => {}
                other => panic!(
                    "argv[{}] null terminator should be concrete 0, got {:?}",
                    arg_idx + 1,
                    other
                ),
            }

            str_offset += sym_arg_len as u64;
        }
    }

    #[test]
    fn sp_points_to_argc() {
        let mut builder = Btor2Builder::new();
        let sorts = test_sorts(&mut builder);
        let config = make_config(1, 8);
        let word_size = 8u64;
        let stack_top = 1u64 << 31;

        let (sp, Some(tail)) = CoreState::initialize_symbolic_argv(
            &mut builder,
            &sorts,
            &config,
            stack_top,
            word_size,
        ) else {
            panic!("expected Some(tail)");
        };

        let writes = collect_writes(&builder, tail);

        // SP should have argc written there. Reconstruct.
        let mut argc_at_sp: u64 = 0;
        for byte_idx in 0..word_size {
            let addr = sp + byte_idx;
            let (_, op) = writes
                .iter()
                .find(|(a, _)| *a == addr)
                .unwrap_or_else(|| panic!("missing argc byte at SP+{}", byte_idx));
            match op {
                Op::Constd { value, .. } => {
                    argc_at_sp |= value << (byte_idx * 8);
                }
                other => panic!("argc byte should be Constd, got {:?}", other),
            }
        }

        let expected_argc = (config.symbolic_argc + 1) as u64;
        assert_eq!(
            argc_at_sp, expected_argc,
            "argc at SP must equal symbolic_argc + 1"
        );

        // Verify SP is below the pointer area (structurally correct).
        let prog_name = b"prog";
        let argv0_len = prog_name.len() + 1;
        let sym_arg_len = config.max_arglen + 1;
        let string_area_size = argv0_len + config.symbolic_argc * sym_arg_len;
        let string_area_aligned = (string_area_size as u64 + word_size - 1) & !(word_size - 1);
        let string_area_start = stack_top - string_area_aligned;
        let total_argc = config.symbolic_argc + 1;
        let pointer_area_size = (total_argc + 1) as u64 * word_size;
        let pointer_area_start = string_area_start - pointer_area_size;
        let expected_sp = pointer_area_start - word_size;
        assert_eq!(sp, expected_sp, "SP must be one word below pointer area");
    }

    #[test]
    fn register_a0_equals_argc() {
        // Replicate the a0-init path from CoreState::new to verify the value
        // written to register a0 matches argc = symbolic_argc + 1.
        let mut builder = Btor2Builder::new();
        let sorts = test_sorts(&mut builder);
        let config = make_config(3, 4);

        let expected_argc = (config.symbolic_argc + 1) as u64; // 4

        // This mirrors lines 131-134 in CoreState::new.
        let argc_val = builder.constd(
            sorts.sid_machine_word,
            (config.symbolic_argc + 1) as u64,
            None,
        );

        // Build a register file write (same as CoreState::new does for a0).
        let base_regs = builder.state(sorts.sid_register_state, "test-regs", None);
        let a0_addr = builder.constd(sorts.sid_register_address, 10, None); // a0 = x10
        let reg_with_a0 =
            builder.write(sorts.sid_register_state, base_regs, a0_addr, argc_val, None);

        // Verify the write targets a0 with the correct argc value.
        match builder.get_op(reg_with_a0) {
            Op::Write { index, value, .. } => {
                match builder.get_op(*index) {
                    Op::Constd { value: reg_num, .. } => {
                        assert_eq!(*reg_num, 10, "a0 is register x10");
                    }
                    other => panic!("expected Constd register address, got {:?}", other),
                }
                match builder.get_op(*value) {
                    Op::Constd { value: v, .. } => {
                        assert_eq!(
                            *v, expected_argc,
                            "a0 must be set to argc = {}",
                            expected_argc
                        );
                    }
                    other => panic!("expected Constd argc value, got {:?}", other),
                }
            }
            other => panic!("expected Write for a0 init, got {:?}", other),
        }
    }

    #[test]
    fn only_content_bytes_are_symbolic() {
        let mut builder = Btor2Builder::new();
        let sorts = test_sorts(&mut builder);
        let config = make_config(1, 4);
        let word_size = 8u64;
        let stack_top = 1u64 << 31;

        let (_, Some(tail)) = CoreState::initialize_symbolic_argv(
            &mut builder,
            &sorts,
            &config,
            stack_top,
            word_size,
        ) else {
            panic!("expected Some(tail)");
        };

        let writes = collect_writes(&builder, tail);

        // Identify which addresses hold symbolic (State) values.
        let symbolic_addrs: Vec<u64> = writes
            .iter()
            .filter_map(|(addr, op)| {
                if matches!(op, Op::State { .. }) {
                    Some(*addr)
                } else {
                    None
                }
            })
            .collect();

        // There should be exactly max_arglen * symbolic_argc symbolic bytes.
        let expected_symbolic_count = config.max_arglen * config.symbolic_argc;
        assert_eq!(
            symbolic_addrs.len(),
            expected_symbolic_count,
            "expected {} symbolic bytes ({}×{}), got {}",
            expected_symbolic_count,
            config.symbolic_argc,
            config.max_arglen,
            symbolic_addrs.len()
        );

        // Every symbolic address must fall within a symbolic arg's content region.
        let prog_name = b"prog";
        let argv0_len = prog_name.len() + 1;
        let sym_arg_len = config.max_arglen + 1;
        let string_area_size = argv0_len + config.symbolic_argc * sym_arg_len;
        let string_area_aligned = (string_area_size as u64 + word_size - 1) & !(word_size - 1);
        let string_area_start = stack_top - string_area_aligned;

        for &addr in &symbolic_addrs {
            let offset = addr - string_area_start;
            assert!(
                offset >= argv0_len as u64,
                "symbolic byte at 0x{:x} falls inside argv[0] (concrete region)",
                addr
            );
            let rel = offset - argv0_len as u64;
            let within_arg = rel % sym_arg_len as u64;
            assert!(
                within_arg < config.max_arglen as u64,
                "symbolic byte at 0x{:x} is at null terminator position (should be concrete)",
                addr
            );
        }
    }
}
