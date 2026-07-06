use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::Config;
use crate::machine::core::CoreState;
use crate::machine::memory::Memory;
use crate::machine::registers::RegisterFile;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::model::combinational::CombinationalResult;
use crate::riscv::isa::{self, InstrId};

/// Generate sequential (next-state) logic for one core.
/// This creates `next` lines for PC, register file, and memory segments.
pub fn rotor_sequential(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    core: &CoreState,
    comb: &CombinationalResult,
) {
    let bool_sid = sorts.sid_boolean;
    let mw_sid = sorts.sid_machine_word;

    // ===== PROGRAM COUNTER =====
    // Next PC: use combinational result, but ecall doesn't advance PC to a new
    // instruction — it stays at PC+4 (the syscall handler is modeled inline).
    // For ecall, PC advances normally to PC+4.
    let next_pc = comb.next_pc;

    builder.next(
        sorts.sid_machine_word,
        core.pc_state,
        next_pc,
        Some("next PC".to_string()),
    );

    // ===== REGISTER FILE =====
    // Compute the new register file state after this instruction.
    // All kernel/syscall signals come precomputed from the combinational
    // phase (C rotor's kernel_combinational), via comb.kernel.
    let kernel = &comb.kernel;

    // Normal rd write
    let reg_after_rd = RegisterFile::conditional_store(
        builder,
        sorts,
        consts,
        core.register_file_state,
        comb.rd_addr,
        comb.rd_value,
        Some("write rd".to_string()),
    );

    // Kernel register data flow — C rotor's exact nested ITE
    // (rotor.c:11077-11119):
    //   ecall ? (brk    ? regs[a0 := eval_program_break]
    //          : openat ? regs[a0 := fd + 1]
    //          : (read AND returning) ? regs[a0 := read_return_value]
    //          : write  ? regs[a0 := a2]
    //          : regs)
    //   : instruction register data flow
    let a0_addr = consts.nid_register(isa::regs::A0);
    let reg_sid = sorts.sid_register_state;

    let regs_brk = RegisterFile::store_register(
        builder,
        sorts,
        core.register_file_state,
        a0_addr,
        kernel.eval_program_break,
        Some("store new program break in a0".to_string()),
    );
    let regs_openat = RegisterFile::store_register(
        builder,
        sorts,
        core.register_file_state,
        a0_addr,
        kernel.eval_file_descriptor,
        Some("store new file descriptor in a0".to_string()),
    );
    let regs_read = RegisterFile::store_register(
        builder,
        sorts,
        core.register_file_state,
        a0_addr,
        kernel.read_return_value,
        Some("store read return value in a0".to_string()),
    );
    let regs_write = RegisterFile::store_register(
        builder,
        sorts,
        core.register_file_state,
        a0_addr,
        kernel.a2,
        Some("store write return value in a0".to_string()),
    );

    // read returns only when there is at most one more byte to read
    let read_returning = {
        let not_more = builder.not(
            bool_sid,
            kernel.more_than_one_readable_byte_to_read,
            Some("read system call returns if at most one more byte to read".to_string()),
        );
        builder.and_node(
            bool_sid,
            kernel.is_read,
            not_more,
            Some("update a0 when read system call returns".to_string()),
        )
    };

    let mut kernel_regs = core.register_file_state;
    kernel_regs = builder.ite(
        reg_sid,
        kernel.is_write,
        regs_write,
        kernel_regs,
        Some("write system call register data flow".to_string()),
    );
    kernel_regs = builder.ite(
        reg_sid,
        read_returning,
        regs_read,
        kernel_regs,
        Some("read system call register data flow".to_string()),
    );
    kernel_regs = builder.ite(
        reg_sid,
        kernel.is_openat,
        regs_openat,
        kernel_regs,
        Some("openat system call register data flow".to_string()),
    );
    kernel_regs = builder.ite(
        reg_sid,
        kernel.is_brk,
        regs_brk,
        kernel_regs,
        Some("brk system call register data flow".to_string()),
    );

    // Instruction register data flow (rd write), then ecall branch wins.
    let mut next_regs = core.register_file_state;
    next_regs = builder.ite(
        reg_sid,
        comb.writes_rd,
        reg_after_rd,
        next_regs,
        Some("register update (rd)".to_string()),
    );
    next_regs = builder.ite(
        reg_sid,
        comb.is_ecall,
        kernel_regs,
        next_regs,
        Some("register data flow".to_string()),
    );

    builder.next(
        reg_sid,
        core.register_file_state,
        next_regs,
        Some("next register file".to_string()),
    );

    // ===== MEMORY SEGMENTS =====
    // Store operations: write to the appropriate segment
    sequential_memory(builder, sorts, consts, config, core, comb);

    // ===== KERNEL STATE ===== (C rotor kernel_sequential, rotor.c:11153-11253)

    // Program break: updated only by an active brk syscall, to the validated
    // new break (eval_program_break already checks [brk, heap_end]).
    let next_brk = builder.ite(
        mw_sid,
        kernel.active_brk,
        kernel.eval_program_break,
        core.kernel.program_break,
        Some("new program break".to_string()),
    );
    builder.next(
        mw_sid,
        core.kernel.program_break,
        next_brk,
        Some("new program break".to_string()),
    );

    // File descriptor: incremented by an active openat syscall.
    let next_fd = builder.ite(
        mw_sid,
        kernel.active_openat,
        kernel.eval_file_descriptor,
        core.kernel.file_descriptor,
        Some("new file descriptor".to_string()),
    );
    builder.next(
        mw_sid,
        core.kernel.file_descriptor,
        next_fd,
        Some("new file descriptor".to_string()),
    );

    // Readable bytes: decrement WHILE the read syscall is still reading
    // (one byte per transition).
    let decremented = builder.sub(
        mw_sid,
        core.kernel.readable_bytes,
        consts.nid_machine_word_1,
        Some("decrement readable bytes".to_string()),
    );
    let next_readable = builder.ite(
        mw_sid,
        kernel.still_reading_active_read,
        decremented,
        core.kernel.readable_bytes,
        Some("decrement readable bytes if system call is still reading".to_string()),
    );
    builder.next(
        mw_sid,
        core.kernel.readable_bytes,
        next_readable,
        Some("readable bytes".to_string()),
    );

    // Read-bytes counter: increments while the active read continues, and
    // RESETS TO ZERO otherwise (including when the read completes) — exactly
    // like C rotor (rotor.c:11237-11252).
    let read_continuing = builder.and_node(
        bool_sid,
        kernel.active_read,
        kernel.more_than_one_readable_byte_to_read,
        Some("more than one byte to read by active read system call".to_string()),
    );
    let incremented = builder.add(
        mw_sid,
        core.kernel.read_bytes,
        consts.nid_machine_word_1,
        Some("increment bytes already read by read system call".to_string()),
    );
    let next_read = builder.ite(
        mw_sid,
        read_continuing,
        incremented,
        consts.nid_machine_word_0,
        Some("increment bytes already read if read system call is active".to_string()),
    );
    builder.next(
        mw_sid,
        core.kernel.read_bytes,
        next_read,
        Some("bytes already read in active read system call".to_string()),
    );

    // Input buffer: frozen (read-only). Without an explicit self-loop next,
    // BTOR2 would let the solver re-choose the buffer at every step.
    builder.next(
        sorts.sid_input_buffer,
        core.kernel.input_buffer,
        core.kernel.input_buffer,
        Some("read-only uninitialized input buffer".to_string()),
    );
}

/// Generate next-state logic for memory segments (store instructions).
fn sequential_memory(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    core: &CoreState,
    comb: &CombinationalResult,
) {
    let bool_sid = sorts.sid_boolean;

    // For each store instruction, update the appropriate segment.
    // Store byte
    let sb_cond = builder.eq_node(
        bool_sid,
        comb.instruction_id,
        consts.nid_instr_id(InstrId::Sb),
        None,
    );
    // Store half-word
    let sh_cond = builder.eq_node(
        bool_sid,
        comb.instruction_id,
        consts.nid_instr_id(InstrId::Sh),
        None,
    );
    // Store word
    let sw_cond = builder.eq_node(
        bool_sid,
        comb.instruction_id,
        consts.nid_instr_id(InstrId::Sw),
        None,
    );
    // Store double-word (RV64)
    let sd_cond = if config.xlen == crate::config::Xlen::X64 {
        builder.eq_node(
            bool_sid,
            comb.instruction_id,
            consts.nid_instr_id(InstrId::Sd),
            None,
        )
    } else {
        consts.nid_false
    };

    // Determine which segment the store address falls into
    let in_data = core
        .segmentation
        .is_in_data_segment(builder, sorts, comb.store_addr);
    let in_heap = core
        .segmentation
        .is_in_heap_segment(builder, sorts, comb.store_addr);
    let in_stack = core
        .segmentation
        .is_in_stack_segment(builder, sorts, comb.store_addr);

    // Helper: build store for each width for a given segment
    let build_segment_store = |builder: &mut Btor2Builder,
                               seg_state: NodeId,
                               seg_sid: NodeId,
                               in_seg: NodeId|
     -> NodeId {
        let sb_mem = Memory::store_byte(
            builder,
            sorts,
            seg_state,
            comb.store_addr,
            comb.store_value,
            seg_sid,
        );
        let sh_mem = Memory::store_half_word(
            builder,
            sorts,
            consts,
            seg_state,
            comb.store_addr,
            comb.store_value,
            seg_sid,
        );
        let sw_mem = Memory::store_word(
            builder,
            sorts,
            consts,
            seg_state,
            comb.store_addr,
            comb.store_value,
            seg_sid,
        );

        let mut result = seg_state;
        result = builder.ite(seg_sid, sb_cond, sb_mem, result, None);
        result = builder.ite(seg_sid, sh_cond, sh_mem, result, None);
        result = builder.ite(seg_sid, sw_cond, sw_mem, result, None);

        if config.xlen == crate::config::Xlen::X64 {
            let sd_mem = Memory::store_double_word(
                builder,
                sorts,
                consts,
                seg_state,
                comb.store_addr,
                comb.store_value,
                seg_sid,
            );
            result = builder.ite(seg_sid, sd_cond, sd_mem, result, None);
        }

        // Only apply if address is in this segment AND instruction is a store
        builder.ite(seg_sid, in_seg, result, seg_state, None)
    };

    // Data segment next state
    let next_data = build_segment_store(
        builder,
        core.data_segment_state,
        sorts.sid_data_state,
        in_data,
    );
    let next_data = builder.ite(
        sorts.sid_data_state,
        comb.writes_memory,
        next_data,
        core.data_segment_state,
        Some("next data segment".to_string()),
    );
    builder.next(
        sorts.sid_data_state,
        core.data_segment_state,
        next_data,
        Some("next data segment".to_string()),
    );

    // Heap segment next state
    let next_heap = build_segment_store(
        builder,
        core.heap_segment_state,
        sorts.sid_heap_state,
        in_heap,
    );
    let next_heap = builder.ite(
        sorts.sid_heap_state,
        comb.writes_memory,
        next_heap,
        core.heap_segment_state,
        Some("next heap segment".to_string()),
    );

    // Kernel read data flow (C rotor rotor.c:11131-11148): while a read
    // syscall is still reading, store ONE input byte per transition into the
    // heap at a1 + read_bytes. The byte comes from
    // input_buffer[bytes_to_read - readable_bytes].
    let next_heap = {
        let kernel = &comb.kernel;
        let bytes_to_read = builder.constd(
            sorts.sid_machine_word,
            config.bytes_to_read,
            Some(format!("bytes to read {}", config.bytes_to_read)),
        );
        let input_index = builder.sub(
            sorts.sid_machine_word,
            bytes_to_read,
            core.kernel.readable_bytes,
            Some("input address".to_string()),
        );
        let input_byte = builder.read(
            sorts.sid_byte,
            core.kernel.input_buffer,
            input_index,
            Some("read input byte".to_string()),
        );
        let dest_addr = builder.add(
            sorts.sid_machine_word,
            kernel.a1,
            core.kernel.read_bytes,
            Some("a1 + number of already read bytes".to_string()),
        );
        let heap_after_read = builder.write(
            sorts.sid_heap_state,
            core.heap_segment_state,
            dest_addr,
            input_byte,
            Some("store input byte in heap segment".to_string()),
        );
        builder.ite(
            sorts.sid_heap_state,
            kernel.still_reading_active_read,
            heap_after_read,
            next_heap,
            Some("heap segment data flow".to_string()),
        )
    };

    builder.next(
        sorts.sid_heap_state,
        core.heap_segment_state,
        next_heap,
        Some("next heap segment".to_string()),
    );

    // Stack segment next state
    let next_stack = build_segment_store(
        builder,
        core.stack_segment_state,
        sorts.sid_stack_state,
        in_stack,
    );
    let next_stack = builder.ite(
        sorts.sid_stack_state,
        comb.writes_memory,
        next_stack,
        core.stack_segment_state,
        Some("next stack segment".to_string()),
    );
    builder.next(
        sorts.sid_stack_state,
        core.stack_segment_state,
        next_stack,
        Some("next stack segment".to_string()),
    );

    // Code segment: read-only, no next needed (but BTOR2 requires it)
    builder.next(
        sorts.sid_code_state,
        core.code_segment_state,
        core.code_segment_state,
        Some("code segment unchanged".to_string()),
    );
}
