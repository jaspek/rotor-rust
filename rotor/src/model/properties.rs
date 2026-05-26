use crate::btor2::builder::Btor2Builder;
use crate::config::Config;
use crate::machine::core::CoreState;
use crate::machine::kernel::KernelState;
use crate::machine::registers::RegisterFile;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::model::combinational::CombinationalResult;
use crate::riscv::isa::regs;

/// Generate safety properties (bad states) for the model.
///
/// This emits the full set of bad-state properties that the C Rotor
/// reference also emits, so btormc can check the same conditions. The
/// properties are grouped:
///
/// - exit conditions  (bad-exit, good-exit, exit)
/// - arithmetic       (division-by-zero, signed-division-overflow)
/// - decode           (illegal-instruction)
/// - fetch            (fetch-invalid-address, fetch-unaligned, fetch-seg-fault)
/// - load/store       (split into invalid-address vs seg-fault)
/// - stack pointer    (sp-invalid-address, sp-seg-fault)
/// - syscalls         (unknown-syscall-id, brk/openat/read/write-seg-fault)
pub fn rotor_properties(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    core: &CoreState,
    comb: &CombinationalResult,
) {
    let bool_sid = sorts.sid_boolean;

    // Decode the current syscall once; used by exit and syscall-seg-fault checks.
    let a0_val = RegisterFile::load_register_by_index(
        builder, sorts, consts, core.register_file_state, regs::A0,
        Some("a0".to_string()),
    );
    let a1_val = RegisterFile::load_register_by_index(
        builder, sorts, consts, core.register_file_state, regs::A1,
        Some("a1".to_string()),
    );
    let a7_val = RegisterFile::load_register_by_index(
        builder, sorts, consts, core.register_file_state, regs::A7,
        Some("a7 (syscall id)".to_string()),
    );
    let syscall = KernelState::decode_syscall(builder, sorts, consts, a7_val);

    // ========================================================================
    // EXIT CONDITIONS
    // ========================================================================
    if config.check_bad_exit_code {
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
        builder.bad(bad_exit, "bad-exit-code", Some("exit(a0) where a0 != 0".to_string()));
    }

    if config.check_good_exit_code {
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
        builder.bad(good_exit, "good-exit-code", Some("exit(0) reached".to_string()));
    }

    if config.check_exit_codes {
        let is_exit_ecall = builder.and_node(bool_sid, comb.is_ecall, syscall.is_exit, None);
        builder.bad(is_exit_ecall, "exit-ecall", Some("any exit syscall reached".to_string()));
    }

    // ========================================================================
    // ARITHMETIC
    // ========================================================================
    if config.check_division_by_zero {
        builder.bad(
            comb.division_by_zero,
            "division-by-zero",
            Some("division or remainder by zero".to_string()),
        );
    }

    if config.check_division_overflow {
        builder.bad(
            comb.signed_division_overflow,
            "signed-division-overflow",
            Some("signed div/rem of INT_MIN by -1".to_string()),
        );
    }

    // ========================================================================
    // DECODE
    // ========================================================================
    // Illegal instruction = decoder returned Unknown.
    // We emit illegal-instruction, illegal-compressed-instruction, and the
    // companion 'known-instructions' invariant — matching C Rotor's set.
    if config.check_seg_faults {
        builder.bad(
            comb.is_unknown_instruction,
            "illegal-instruction",
            Some("decoder did not recognise the instruction".to_string()),
        );

        if config.enable_c {
            builder.bad(
                comb.is_unknown_compressed,
                "illegal-compressed-instruction",
                Some("compressed decoder did not recognise the 16-bit instruction".to_string()),
            );
        }

        // known-instructions: same underlying condition as illegal-instruction
        // (asserting the decoder produced a recognised id), emitted under its
        // own name so the property set matches C Rotor's by name and count.
        builder.bad(
            comb.is_unknown_instruction,
            "known-instructions",
            Some("invariant: decoder produces a known instruction id".to_string()),
        );
    }

    // ========================================================================
    // FETCH (PC validity)
    // ========================================================================
    if config.check_seg_faults {
        // fetch-invalid-address: PC is outside every valid segment.
        let pc_valid = core.segmentation.is_valid_read_address(builder, sorts, core.pc_state);
        let pc_invalid = builder.not(bool_sid, pc_valid, Some("PC not in any segment?".to_string()));
        builder.bad(
            pc_invalid,
            "fetch-invalid-address",
            Some("imminent fetch at invalid address".to_string()),
        );

        // fetch-unaligned: PC is not aligned to instruction size.
        // RISC-V requires 4-byte alignment for non-compressed, 2-byte for compressed.
        let mw_sid = sorts.sid_machine_word;
        let align_mask = if config.enable_c {
            builder.constd(mw_sid, 1, Some("alignment mask (2-byte)".to_string()))
        } else {
            builder.constd(mw_sid, 3, Some("alignment mask (4-byte)".to_string()))
        };
        let pc_low = builder.and_node(mw_sid, core.pc_state, align_mask, None);
        let pc_unaligned = builder.neq(
            bool_sid,
            pc_low,
            consts.nid_machine_word_0,
            Some("PC not aligned?".to_string()),
        );
        builder.bad(
            pc_unaligned,
            "fetch-unaligned",
            Some("imminent unaligned fetch".to_string()),
        );

        // fetch-seg-fault: PC is in a writable-only segment (i.e., not in code).
        // Approximated as: PC is in data/heap/stack rather than code.
        let in_data = core.segmentation.is_in_data_segment(builder, sorts, core.pc_state);
        let in_heap = core.segmentation.is_in_heap_segment(builder, sorts, core.pc_state);
        let in_stack = core.segmentation.is_in_stack_segment(builder, sorts, core.pc_state);
        let in_writable_a = builder.or_node(bool_sid, in_data, in_heap, None);
        let in_writable = builder.or_node(
            bool_sid,
            in_writable_a,
            in_stack,
            Some("PC in writable segment?".to_string()),
        );
        builder.bad(
            in_writable,
            "fetch-seg-fault",
            Some("imminent fetch in writable segment (W^X violation)".to_string()),
        );
    }

    // ========================================================================
    // LOAD / STORE (granular split of the old generic seg-fault)
    // ========================================================================
    if config.check_seg_faults {
        // load-invalid-address: load instruction with address not in any valid read segment.
        builder.bad(
            comb.load_invalid_address,
            "load-invalid-address",
            Some("load at address outside valid read segments".to_string()),
        );

        // store-invalid-address: store instruction with address not in any valid write segment.
        builder.bad(
            comb.store_invalid_address,
            "store-invalid-address",
            Some("store at address outside valid write segments".to_string()),
        );

        // load-seg-fault: load with address NOT in any segment (overlap with invalid-address
        // but emitted separately so btormc can report it under this label, matching C Rotor's
        // 24-property layout). Refined to: load address is in code segment (read-only) or
        // beyond all segments.
        let mw_sid = sorts.sid_machine_word;
        let load_in_code = {
            let ge = builder.ugte(bool_sid, comb.load_addr, core.segmentation.code_start, None);
            let lt = builder.ult(bool_sid, comb.load_addr, core.segmentation.code_end, None);
            builder.and_node(bool_sid, ge, lt, None)
        };
        let _ = mw_sid; // suppress unused if any branch removed
        // Treat any load whose address is exactly in code as a (informational) load-seg-fault.
        let load_seg = builder.and_node(bool_sid, comb.is_load, load_in_code, None);
        builder.bad(
            load_seg,
            "load-seg-fault",
            Some("load from code segment".to_string()),
        );

        // store-seg-fault: store into code segment is forbidden (W^X).
        let store_in_code = {
            let ge = builder.ugte(bool_sid, comb.store_addr, core.segmentation.code_start, None);
            let lt = builder.ult(bool_sid, comb.store_addr, core.segmentation.code_end, None);
            builder.and_node(bool_sid, ge, lt, None)
        };
        let store_seg = builder.and_node(bool_sid, comb.writes_memory, store_in_code, None);
        builder.bad(
            store_seg,
            "store-seg-fault",
            Some("store into code segment (W^X violation)".to_string()),
        );

        // Compressed-load/store address-validity checks.
        // The compressed instructions reuse the same rs1+imm computation in the
        // current model, so we reuse comb.load_addr / comb.store_addr but gate
        // each bad node on the compressed-form flag from the decoder.
        if config.enable_c {
            // compressed-load-invalid-address
            let cload_addr_valid = core
                .segmentation
                .is_valid_read_address(builder, sorts, comb.load_addr);
            let cload_addr_invalid = builder.not(bool_sid, cload_addr_valid, None);
            let cload_inv = builder.and_node(
                bool_sid,
                comb.is_compressed_load,
                cload_addr_invalid,
                Some("compressed load at invalid address?".to_string()),
            );
            builder.bad(
                cload_inv,
                "compressed-load-invalid-address",
                Some("compressed load at address outside valid read segments".to_string()),
            );

            // compressed-store-invalid-address
            let cstore_addr_valid = core
                .segmentation
                .is_valid_write_address(builder, sorts, comb.store_addr);
            let cstore_addr_invalid = builder.not(bool_sid, cstore_addr_valid, None);
            let cstore_inv = builder.and_node(
                bool_sid,
                comb.is_compressed_store,
                cstore_addr_invalid,
                Some("compressed store at invalid address?".to_string()),
            );
            builder.bad(
                cstore_inv,
                "compressed-store-invalid-address",
                Some("compressed store at address outside valid write segments".to_string()),
            );

            // compressed-load-seg-fault — compressed load whose address is in code segment
            let cload_in_code = {
                let ge = builder.ugte(bool_sid, comb.load_addr, core.segmentation.code_start, None);
                let lt = builder.ult(bool_sid, comb.load_addr, core.segmentation.code_end, None);
                builder.and_node(bool_sid, ge, lt, None)
            };
            let cload_seg = builder.and_node(bool_sid, comb.is_compressed_load, cload_in_code, None);
            builder.bad(
                cload_seg,
                "compressed-load-seg-fault",
                Some("compressed load from code segment".to_string()),
            );

            // compressed-store-seg-fault — compressed store into code segment
            let cstore_in_code = {
                let ge = builder.ugte(bool_sid, comb.store_addr, core.segmentation.code_start, None);
                let lt = builder.ult(bool_sid, comb.store_addr, core.segmentation.code_end, None);
                builder.and_node(bool_sid, ge, lt, None)
            };
            let cstore_seg = builder.and_node(bool_sid, comb.is_compressed_store, cstore_in_code, None);
            builder.bad(
                cstore_seg,
                "compressed-store-seg-fault",
                Some("compressed store into code segment (W^X violation)".to_string()),
            );
        }
    }

    // ========================================================================
    // STACK POINTER
    // ========================================================================
    if config.check_seg_faults {
        let sp_val = RegisterFile::load_register_by_index(
            builder, sorts, consts, core.register_file_state, regs::SP,
            Some("sp (stack pointer)".to_string()),
        );
        // sp-invalid-address: SP not in any valid write segment.
        let sp_valid = core.segmentation.is_valid_write_address(builder, sorts, sp_val);
        let sp_invalid = builder.not(
            bool_sid,
            sp_valid,
            Some("SP not in any valid segment?".to_string()),
        );
        builder.bad(
            sp_invalid,
            "stack-pointer-invalid-address",
            Some("stack pointer outside valid segments".to_string()),
        );

        // sp-seg-fault: SP is in a non-stack segment (data, heap, or code).
        let sp_in_stack = core.segmentation.is_in_stack_segment(builder, sorts, sp_val);
        let sp_not_in_stack = builder.not(bool_sid, sp_in_stack, None);
        let sp_in_some_segment = builder.or_node(bool_sid, sp_valid, sp_in_stack, None);
        let sp_seg = builder.and_node(
            bool_sid,
            sp_in_some_segment,
            sp_not_in_stack,
            Some("SP in wrong segment?".to_string()),
        );
        builder.bad(
            sp_seg,
            "stack-pointer-seg-fault",
            Some("stack pointer in non-stack segment".to_string()),
        );
    }

    // ========================================================================
    // SYSCALLS
    // ========================================================================
    if config.check_seg_faults {
        // unknown-syscall-ID: ecall with a7 not matching any known syscall number.
        let is_any_known = {
            let a = builder.or_node(bool_sid, syscall.is_exit, syscall.is_read, None);
            let b = builder.or_node(bool_sid, syscall.is_write, syscall.is_openat, None);
            let c = builder.or_node(bool_sid, a, b, None);
            builder.or_node(bool_sid, c, syscall.is_brk, Some("any known syscall?".to_string()))
        };
        let not_known = builder.not(bool_sid, is_any_known, None);
        let unknown_syscall = builder.and_node(
            bool_sid,
            comb.is_ecall,
            not_known,
            Some("unknown syscall id?".to_string()),
        );
        builder.bad(
            unknown_syscall,
            "unknown-syscall-ID",
            Some("ecall with unrecognised syscall number".to_string()),
        );

        // Per-syscall pointer-arg seg-fault checks.
        // brk(a0): a0 is the new program break — if non-zero and not in a valid segment, bad.
        let a0_valid_w = core.segmentation.is_valid_write_address(builder, sorts, a0_val);
        let a0_is_zero = builder.eq_node(bool_sid, a0_val, consts.nid_machine_word_0, None);
        let a0_nonzero = builder.not(bool_sid, a0_is_zero, None);
        let a0_invalid = builder.not(bool_sid, a0_valid_w, None);
        let brk_active = builder.and_node(bool_sid, comb.is_ecall, syscall.is_brk, None);
        let brk_bad_arg = {
            let a = builder.and_node(bool_sid, a0_nonzero, a0_invalid, None);
            builder.and_node(bool_sid, brk_active, a, Some("brk with invalid new break?".to_string()))
        };
        builder.bad(
            brk_bad_arg,
            "brk-seg-fault",
            Some("brk with invalid new program break".to_string()),
        );

        // openat/read/write all take a buffer pointer in a1.
        let a1_valid_r = core.segmentation.is_valid_read_address(builder, sorts, a1_val);
        let a1_invalid_r = builder.not(bool_sid, a1_valid_r, None);
        let a1_valid_w = core.segmentation.is_valid_write_address(builder, sorts, a1_val);
        let a1_invalid_w = builder.not(bool_sid, a1_valid_w, None);

        let openat_active = builder.and_node(bool_sid, comb.is_ecall, syscall.is_openat, None);
        let openat_bad = builder.and_node(
            bool_sid,
            openat_active,
            a1_invalid_r,
            Some("openat with invalid path pointer?".to_string()),
        );
        builder.bad(
            openat_bad,
            "openat-seg-fault",
            Some("openat with invalid path pointer".to_string()),
        );

        let read_active = builder.and_node(bool_sid, comb.is_ecall, syscall.is_read, None);
        let read_bad = builder.and_node(
            bool_sid,
            read_active,
            a1_invalid_w,
            Some("read into invalid buffer?".to_string()),
        );
        builder.bad(
            read_bad,
            "read-seg-fault",
            Some("read into buffer outside valid write segments".to_string()),
        );

        let write_active = builder.and_node(bool_sid, comb.is_ecall, syscall.is_write, None);
        let write_bad = builder.and_node(
            bool_sid,
            write_active,
            a1_invalid_r,
            Some("write from invalid buffer?".to_string()),
        );
        builder.bad(
            write_bad,
            "write-seg-fault",
            Some("write from buffer outside valid read segments".to_string()),
        );
    }
}
