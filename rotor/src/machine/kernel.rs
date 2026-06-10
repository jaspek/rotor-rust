use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::riscv::isa::InstrId;

/// Kernel state for one core, modeling syscall handling.
pub struct KernelState {
    /// Current program break (heap limit)
    pub program_break: NodeId,
    pub program_break_init: NodeId,
    /// Next file descriptor returned by openat (C: "file descriptor", init 0)
    pub file_descriptor: NodeId,
    /// Input buffer state (array of bytes)
    pub input_buffer: NodeId,
    /// Number of readable bytes remaining
    pub readable_bytes: NodeId,
    /// Number of bytes already read
    pub read_bytes: NodeId,
}

/// All combinational kernel signals for one step, mirroring C rotor's
/// kernel_combinational (rotor.c:10930-11151). Everything here is a pure
/// function of the current state.
pub struct KernelFlows {
    pub active_ecall: NodeId,
    pub is_exit: NodeId,
    pub is_brk: NodeId,
    pub is_openat: NodeId,
    pub is_read: NodeId,
    pub is_write: NodeId,
    pub active_exit: NodeId,
    pub active_brk: NodeId,
    pub active_openat: NodeId,
    pub active_read: NodeId,
    pub active_write: NodeId,
    pub a0: NodeId,
    pub a1: NodeId,
    pub a2: NodeId,
    pub a7: NodeId,
    /// brk: new break if in [current brk, heap end], else unchanged
    pub eval_program_break: NodeId,
    /// openat: fd + 1
    pub eval_file_descriptor: NodeId,
    /// readable_bytes > 0
    pub more_readable_bytes: NodeId,
    /// active_read AND read_bytes < a2 AND more_readable_bytes
    pub still_reading_active_read: NodeId,
    /// (read_bytes+1 < a2) AND (readable_bytes > 1)
    pub more_than_one_readable_byte_to_read: NodeId,
    /// the value read() returns in a0 when it completes
    pub read_return_value: NodeId,
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

        // File descriptor counter, initialized to zero like the C reference
        // ("initializing file descriptor to zero"). openat returns fd+1 and
        // increments this state.
        let file_descriptor = builder.state(
            sorts.sid_machine_word,
            "file-descriptor",
            Some("file descriptor".to_string()),
        );
        let _fd_init = builder.init(
            sorts.sid_machine_word,
            file_descriptor,
            consts.nid_machine_word_0,
            Some("initializing file descriptor to zero".to_string()),
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
            file_descriptor,
            input_buffer,
            readable_bytes,
            read_bytes,
        }
    }

    /// Compute all combinational kernel signals for this step, mirroring C
    /// rotor's kernel_combinational (rotor.c:10930-11151).
    ///
    /// `is_ecall` is the decoded "current instruction is ECALL" condition;
    /// `heap_end` is the static end of the heap segment (brk validity bound).
    #[allow(clippy::too_many_arguments)]
    pub fn kernel_combinational(
        builder: &mut Btor2Builder,
        sorts: &MachineSorts,
        consts: &MachineConstants,
        config: &crate::config::Config,
        kernel: &KernelState,
        register_file: NodeId,
        heap_end: NodeId,
        is_ecall: NodeId,
    ) -> KernelFlows {
        use crate::machine::registers::RegisterFile;
        use crate::riscv::isa::regs;

        let bool_sid = sorts.sid_boolean;
        let mw_sid = sorts.sid_machine_word;

        let a0 = RegisterFile::load_register_by_index(
            builder, sorts, consts, register_file, regs::A0, Some("a0 value".to_string()),
        );
        let a1 = RegisterFile::load_register_by_index(
            builder, sorts, consts, register_file, regs::A1, Some("a1 value".to_string()),
        );
        let a2 = RegisterFile::load_register_by_index(
            builder, sorts, consts, register_file, regs::A2, Some("a2 value".to_string()),
        );
        let a7 = RegisterFile::load_register_by_index(
            builder, sorts, consts, register_file, regs::A7, Some("a7 value".to_string()),
        );

        // syscall id decode (a7 == ID)
        let is_exit = builder.eq_node(bool_sid, a7, consts.nid_exit_syscall,
            Some("a7 == exit syscall ID?".to_string()));
        let is_brk = builder.eq_node(bool_sid, a7, consts.nid_brk_syscall,
            Some("a7 == brk syscall ID?".to_string()));
        let is_openat = builder.eq_node(bool_sid, a7, consts.nid_openat_syscall,
            Some("a7 == openat syscall ID?".to_string()));
        let is_read = builder.eq_node(bool_sid, a7, consts.nid_read_syscall,
            Some("a7 == read syscall ID?".to_string()));
        let is_write = builder.eq_node(bool_sid, a7, consts.nid_write_syscall,
            Some("a7 == write syscall ID?".to_string()));

        let active_exit = builder.and_node(bool_sid, is_ecall, is_exit,
            Some("active exit system call".to_string()));
        let active_brk = builder.and_node(bool_sid, is_ecall, is_brk,
            Some("active brk system call".to_string()));
        let active_openat = builder.and_node(bool_sid, is_ecall, is_openat,
            Some("active openat system call".to_string()));
        let active_read = builder.and_node(bool_sid, is_ecall, is_read,
            Some("active read system call".to_string()));
        let active_write = builder.and_node(bool_sid, is_ecall, is_write,
            Some("active write system call".to_string()));

        // brk: new break valid iff current brk <= a0 <= heap end
        let a0_ge_brk = builder.ugte(bool_sid, a0, kernel.program_break,
            Some("new program break >= current program break?".to_string()));
        let a0_le_heap_end = builder.ulte(bool_sid, a0, heap_end,
            Some("new program break <= end of heap segment?".to_string()));
        let new_brk_valid = builder.and_node(bool_sid, a0_ge_brk, a0_le_heap_end,
            Some("is new program break in heap segment?".to_string()));
        let eval_program_break = builder.ite(mw_sid, new_brk_valid, a0, kernel.program_break,
            Some("update brk if new program break is in heap segment".to_string()));

        // openat: fd + 1
        let eval_file_descriptor = builder.add(mw_sid, kernel.file_descriptor,
            consts.nid_machine_word_1, Some("increment file descriptor".to_string()));

        // read helpers (C rotor names preserved)
        let more_readable_bytes = builder.ugt(bool_sid, kernel.readable_bytes,
            consts.nid_machine_word_0, Some("more readable bytes".to_string()));

        let read_lt_a2 = builder.ult(bool_sid, kernel.read_bytes, a2,
            Some("more bytes to read as requested in a2".to_string()));
        let can_and_would = builder.and_node(bool_sid, read_lt_a2, more_readable_bytes,
            Some("can and still would like to read more bytes".to_string()));
        let still_reading_active_read = builder.and_node(bool_sid, active_read, can_and_would,
            Some("still reading active read system call".to_string()));

        let incremented_read_bytes = builder.add(mw_sid, kernel.read_bytes,
            consts.nid_machine_word_1,
            Some("increment bytes already read by read system call".to_string()));
        let more_than_one_byte_to_read = builder.ult(bool_sid, incremented_read_bytes, a2,
            Some("more than one byte to read as requested in a2".to_string()));
        let more_than_one_readable_byte = builder.ugt(bool_sid, kernel.readable_bytes,
            consts.nid_machine_word_1, Some("more than one readable byte".to_string()));
        let more_than_one_readable_byte_to_read = builder.and_node(
            bool_sid, more_than_one_byte_to_read, more_than_one_readable_byte,
            Some("can and still would like to read more than one byte".to_string()));

        // read return value (C rotor's exact nested ITE, rotor.c:11055-11075):
        //   a2 > 0 ? (more_readable ? (more_to_read ? (more_readable>1 ? a0
        //             : read_bytes+1) : a2) : -1) : 0
        let minus_one_val: u64 = if config.machine_word_bits() == 64 {
            u64::MAX
        } else {
            0xFFFF_FFFF
        };
        let minus_one = builder.constd(mw_sid, minus_one_val,
            Some("machine word -1".to_string()));

        let inner1 = builder.ite(mw_sid, more_than_one_readable_byte, a0,
            incremented_read_bytes,
            Some("return bytes read so far + 1 if only one more readable byte".to_string()));
        let inner2 = builder.ite(mw_sid, more_than_one_byte_to_read, inner1, a2,
            Some("return a2 if read_bytes == a2 - 1 and still readable".to_string()));
        let inner3 = builder.ite(mw_sid, more_readable_bytes, inner2, minus_one,
            Some("return -1 if a2 > 0 and nothing readable at start".to_string()));
        let a2_gt_0 = builder.ugt(bool_sid, a2, consts.nid_machine_word_0,
            Some("more than 0 bytes to read".to_string()));
        let read_return_value = builder.ite(mw_sid, a2_gt_0, inner3,
            consts.nid_machine_word_0, Some("return 0 if a2 == 0".to_string()));

        KernelFlows {
            active_ecall: is_ecall,
            is_exit,
            is_brk,
            is_openat,
            is_read,
            is_write,
            active_exit,
            active_brk,
            active_openat,
            active_read,
            active_write,
            a0,
            a1,
            a2,
            a7,
            eval_program_break,
            eval_file_descriptor,
            more_readable_bytes,
            still_reading_active_read,
            more_than_one_readable_byte_to_read,
            read_return_value,
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
