use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::riscv::isa::InstrId;

/// Kernel state for one core, modeling syscall handling.
pub struct KernelState {
    /// Current program break (heap limit)
    pub program_break: NodeId,
    pub program_break_init: NodeId,
    /// Input buffer state (array of bytes)
    pub input_buffer: NodeId,
    /// Number of readable bytes remaining
    pub readable_bytes: NodeId,
    /// Number of bytes already read
    pub read_bytes: NodeId,
}

impl KernelState {
    /// Create initial kernel state.
    pub fn new(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        initial_brk: u64,
        bytes_to_read: u64,
    ) -> Self {
        // Create init values first (BTOR2 requires state nid > value nid)
        let initial_brk_nid = builder.constd(
            sorts.sid_machine_word,
            initial_brk,
            Some("initial program break".to_string()),
        );
        let init_readable = builder.constd(
            sorts.sid_machine_word,
            bytes_to_read,
            Some(format!("{} bytes to read", bytes_to_read)),
        );
        // read_bytes uses consts.nid_machine_word_0 which is already created

        // Now create states
        let program_break = builder.state(
            sorts.sid_machine_word,
            "program-break",
            Some("program break state".to_string()),
        );
        let _program_break_init = builder.init(
            sorts.sid_machine_word,
            program_break,
            initial_brk_nid,
            Some("init program break".to_string()),
        );

        let input_buffer = builder.state(
            sorts.sid_input_buffer,
            "input-buffer",
            Some("symbolic input buffer".to_string()),
        );

        let readable_bytes = builder.state(
            sorts.sid_machine_word,
            "readable-bytes",
            Some("readable bytes remaining".to_string()),
        );
        let _readable_init = builder.init(
            sorts.sid_machine_word,
            readable_bytes,
            init_readable,
            Some("init readable bytes".to_string()),
        );

        let read_bytes = builder.state(
            sorts.sid_machine_word,
            "read-bytes",
            Some("bytes read so far".to_string()),
        );
        let _read_init = builder.init(
            sorts.sid_machine_word,
            read_bytes,
            consts.nid_machine_word_0,
            Some("init read bytes counter".to_string()),
        );

        Self {
            program_break,
            program_break_init: initial_brk_nid,
            input_buffer,
            readable_bytes,
            read_bytes,
        }
    }

    /// Detect if the current instruction is an ecall.
    pub fn is_ecall(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        instruction_id: NodeId,
    ) -> NodeId {
        let ecall_id = consts.nid_instr_id(InstrId::Ecall);
        builder.eq_node(
            sorts.sid_boolean,
            instruction_id,
            ecall_id,
            Some("is ecall?".to_string()),
        )
    }

    /// Determine which syscall is being invoked (value in a7).
    pub fn decode_syscall(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        a7_value: NodeId,
    ) -> SyscallDecode {
        let bool_sid = sorts.sid_boolean;

        let is_exit = builder.eq_node(
            bool_sid,
            a7_value,
            consts.nid_exit_syscall,
            Some("syscall == exit?".to_string()),
        );
        let is_read = builder.eq_node(
            bool_sid,
            a7_value,
            consts.nid_read_syscall,
            Some("syscall == read?".to_string()),
        );
        let is_write = builder.eq_node(
            bool_sid,
            a7_value,
            consts.nid_write_syscall,
            Some("syscall == write?".to_string()),
        );
        let is_openat = builder.eq_node(
            bool_sid,
            a7_value,
            consts.nid_openat_syscall,
            Some("syscall == openat?".to_string()),
        );
        let is_brk = builder.eq_node(
            bool_sid,
            a7_value,
            consts.nid_brk_syscall,
            Some("syscall == brk?".to_string()),
        );

        SyscallDecode {
            is_exit,
            is_read,
            is_write,
            is_openat,
            is_brk,
        }
    }

    /// Compute next program break after a brk syscall.
    /// brk(addr): if addr == 0, return current brk; else set brk = addr and return addr.
    pub fn next_program_break(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        current_brk: NodeId,
        a0_value: NodeId,
        is_brk_syscall: NodeId,
        is_ecall: NodeId,
    ) -> NodeId {
        let bool_sid = sorts.sid_boolean;
        let mw_sid = sorts.sid_machine_word;

        // a0 == 0 means query current break
        let a0_is_zero = builder.eq_node(bool_sid, a0_value, consts.nid_machine_word_0, None);
        let new_brk = builder.ite(
            mw_sid,
            a0_is_zero,
            current_brk,
            a0_value,
            Some("brk: query or set".to_string()),
        );

        // Only update if this is actually a brk ecall
        let is_brk_ecall = builder.and_node(bool_sid, is_ecall, is_brk_syscall, None);
        builder.ite(
            mw_sid,
            is_brk_ecall,
            new_brk,
            current_brk,
            Some("next program break".to_string()),
        )
    }

    /// Compute the return value (written to a0) for ecall.
    pub fn ecall_return_value(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        syscall: &SyscallDecode,
        a0_value: NodeId,
        current_brk: NodeId,
        readable_bytes: NodeId,
    ) -> NodeId {
        let mw_sid = sorts.sid_machine_word;

        // exit: no return (but model keeps a0)
        // brk: return new break value
        let a0_is_zero =
            builder.eq_node(sorts.sid_boolean, a0_value, consts.nid_machine_word_0, None);
        let brk_return = builder.ite(mw_sid, a0_is_zero, current_brk, a0_value, None);

        // read: return number of bytes actually read (min of requested and available)
        let read_return = readable_bytes; // simplified: return available bytes

        // write: return number of bytes written (= a2, the count argument, simplified)
        let write_return = a0_value; // simplified

        // openat: return file descriptor (simplified as 0)
        let openat_return = consts.nid_machine_word_0;

        // Build ITE chain for return value
        let mut result = a0_value; // default: keep a0 unchanged
        result = builder.ite(mw_sid, syscall.is_openat, openat_return, result, None);
        result = builder.ite(mw_sid, syscall.is_write, write_return, result, None);
        result = builder.ite(mw_sid, syscall.is_read, read_return, result, None);
        result = builder.ite(
            mw_sid,
            syscall.is_brk,
            brk_return,
            result,
            Some("ecall return value".to_string()),
        );

        result
    }
}

/// Result of decoding which syscall is being invoked.
pub struct SyscallDecode {
    pub is_exit: NodeId,
    pub is_read: NodeId,
    pub is_write: NodeId,
    pub is_openat: NodeId,
    pub is_brk: NodeId,
}
