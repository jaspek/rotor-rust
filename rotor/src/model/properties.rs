use crate::btor2::builder::Btor2Builder;
use crate::config::Config;
use crate::machine::core::CoreState;
use crate::machine::kernel::KernelState;
use crate::machine::registers::RegisterFile;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::model::combinational::CombinationalResult;

/// Generate safety properties (bad states) for the model.
pub fn rotor_properties(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    core: &CoreState,
    comb: &CombinationalResult,
) {
    let bool_sid = sorts.sid_boolean;

    // ===== BAD EXIT CODE =====
    if config.check_bad_exit_code {
        // exit(a0) where a0 != 0 is a bad state
        let a0_val = RegisterFile::load_register_by_index(
            builder,
            sorts,
            consts,
            core.register_file_state,
            crate::riscv::isa::regs::A0,
            Some("a0 (exit code)".to_string()),
        );
        let a7_val = RegisterFile::load_register_by_index(
            builder,
            sorts,
            consts,
            core.register_file_state,
            crate::riscv::isa::regs::A7,
            None,
        );

        let syscall = KernelState::decode_syscall(builder, sorts, consts, a7_val);

        let is_exit_ecall = builder.and_node(bool_sid, comb.is_ecall, syscall.is_exit, None);
        let a0_nonzero = builder.neq(
            bool_sid,
            a0_val,
            consts.nid_machine_word_0,
            Some("exit code != 0".to_string()),
        );
        let bad_exit = builder.and_node(
            bool_sid,
            is_exit_ecall,
            a0_nonzero,
            Some("exit with non-zero code".to_string()),
        );

        builder.bad(
            bad_exit,
            "bad-exit-code",
            Some("exit(a0) where a0 != 0".to_string()),
        );
    }

    // ===== GOOD EXIT CODE =====
    if config.check_good_exit_code {
        let a0_val = RegisterFile::load_register_by_index(
            builder,
            sorts,
            consts,
            core.register_file_state,
            crate::riscv::isa::regs::A0,
            None,
        );
        let a7_val = RegisterFile::load_register_by_index(
            builder,
            sorts,
            consts,
            core.register_file_state,
            crate::riscv::isa::regs::A7,
            None,
        );

        let syscall = KernelState::decode_syscall(builder, sorts, consts, a7_val);

        let is_exit_ecall = builder.and_node(bool_sid, comb.is_ecall, syscall.is_exit, None);
        let a0_zero = builder.eq_node(
            bool_sid,
            a0_val,
            consts.nid_machine_word_0,
            Some("exit code == 0".to_string()),
        );
        let good_exit = builder.and_node(
            bool_sid,
            is_exit_ecall,
            a0_zero,
            Some("exit with zero code".to_string()),
        );

        builder.bad(
            good_exit,
            "good-exit-code",
            Some("exit(0) reached".to_string()),
        );
    }

    // ===== EXIT CODES (any exit) =====
    if config.check_exit_codes {
        let a7_val = RegisterFile::load_register_by_index(
            builder,
            sorts,
            consts,
            core.register_file_state,
            crate::riscv::isa::regs::A7,
            None,
        );
        let syscall = KernelState::decode_syscall(builder, sorts, consts, a7_val);

        let is_exit_ecall = builder.and_node(bool_sid, comb.is_ecall, syscall.is_exit, None);
        builder.bad(
            is_exit_ecall,
            "exit-ecall",
            Some("any exit syscall reached".to_string()),
        );
    }

    // ===== DIVISION BY ZERO =====
    if config.check_division_by_zero {
        builder.bad(
            comb.division_by_zero,
            "division-by-zero",
            Some("division or remainder by zero".to_string()),
        );
    }

    // ===== SEGMENTATION FAULTS =====
    if config.check_seg_faults {
        builder.bad(
            comb.invalid_address,
            "segmentation-fault",
            Some("memory access outside valid segments".to_string()),
        );
    }
}
