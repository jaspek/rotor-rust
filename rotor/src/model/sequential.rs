use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::Config;
use crate::machine::core::CoreState;
use crate::machine::kernel::KernelState;
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

    // Handle ecall: write return value to a0
    let a7_val = RegisterFile::load_register_by_index(
        builder,
        sorts,
        consts,
        core.register_file_state,
        isa::regs::A7,
        Some("a7 (syscall id)".to_string()),
    );
    let a0_val = RegisterFile::load_register_by_index(
        builder,
        sorts,
        consts,
        core.register_file_state,
        isa::regs::A0,
        Some("a0 (syscall arg)".to_string()),
    );

    let syscall = KernelState::decode_syscall(builder, sorts, consts, a7_val);

    let ecall_return = KernelState::ecall_return_value(
        builder,
        sorts,
        consts,
        &syscall,
        a0_val,
        core.kernel.program_break,
        core.kernel.readable_bytes,
    );

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

    // Ecall a0 write
    let a0_addr = consts.nid_register(isa::regs::A0);
    let reg_after_ecall = RegisterFile::store_register(
        builder,
        sorts,
        core.register_file_state,
        a0_addr,
        ecall_return,
        Some("ecall: write a0".to_string()),
    );

    // Select: if writes_rd, use reg_after_rd; if ecall, use reg_after_ecall; else keep same
    let mut next_regs = core.register_file_state;
    next_regs = builder.ite(
        sorts.sid_register_state,
        comb.writes_rd,
        reg_after_rd,
        next_regs,
        Some("register update (rd)".to_string()),
    );
    next_regs = builder.ite(
        sorts.sid_register_state,
        comb.is_ecall,
        reg_after_ecall,
        next_regs,
        Some("register update (ecall)".to_string()),
    );

    builder.next(
        sorts.sid_register_state,
        core.register_file_state,
        next_regs,
        Some("next register file".to_string()),
    );

    // ===== MEMORY SEGMENTS =====
    // Store operations: write to the appropriate segment
    sequential_memory(builder, sorts, consts, config, core, comb);

    // ===== KERNEL STATE =====
    // Program break update on brk syscall
    let next_brk = KernelState::next_program_break(
        builder,
        sorts,
        consts,
        core.kernel.program_break,
        a0_val,
        syscall.is_brk,
        comb.is_ecall,
    );

    builder.next(
        sorts.sid_machine_word,
        core.kernel.program_break,
        next_brk,
        Some("next program break".to_string()),
    );

    // Readable bytes: decrement on read syscall (simplified)
    let is_read_ecall = builder.and_node(bool_sid, comb.is_ecall, syscall.is_read, None);
    let decremented = builder.sub(
        mw_sid,
        core.kernel.readable_bytes,
        consts.nid_machine_word_1,
        None,
    );
    let next_readable = builder.ite(
        mw_sid,
        is_read_ecall,
        decremented,
        core.kernel.readable_bytes,
        Some("next readable bytes".to_string()),
    );
    builder.next(
        sorts.sid_machine_word,
        core.kernel.readable_bytes,
        next_readable,
        Some("next readable bytes".to_string()),
    );

    // Read bytes counter: increment on read syscall
    let incremented = builder.add(
        mw_sid,
        core.kernel.read_bytes,
        consts.nid_machine_word_1,
        None,
    );
    let next_read = builder.ite(
        mw_sid,
        is_read_ecall,
        incremented,
        core.kernel.read_bytes,
        Some("next read bytes".to_string()),
    );
    builder.next(
        sorts.sid_machine_word,
        core.kernel.read_bytes,
        next_read,
        Some("next read bytes counter".to_string()),
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
