use crate::btor2::builder::Btor2Builder;
use crate::btor2::node::NodeId;
use crate::config::{Config, Xlen};
use crate::machine::core::CoreState;
use crate::machine::registers::RegisterFile;
use crate::machine::sorts::{MachineConstants, MachineSorts};
use crate::model::combinational::CombinationalResult;
use crate::riscv::isa::{InstrId, regs};

/// Generate safety properties (bad states), ported 1:1 from the C reference
/// (rotor.c rotor_properties at 11782 and kernel_properties at 11255).
///
/// EMISSION ORDER MATTERS: btormc identifies properties by their position
/// (b0, b1, ...). This function emits in the exact order of the C reference
/// output so property indices line up between the two rotors:
///
///   b0  illegal-instruction            b12 compressed-store-invalid-address
///   b1  illegal-compressed-instruction b13 stack-pointer-invalid-address
///   b2  known-instructions             b14 load-seg-fault
///   b3  fetch-invalid-address          b15 store-seg-fault
///   b4  fetch-unaligned                b16 compressed-load-seg-fault
///   b5  fetch-seg-fault                b17 compressed-store-seg-fault
///   b6  unknown-syscall-ID             b18 stack-pointer-seg-fault
///   b7  division-by-zero               b19 brk-seg-fault
///   b8  signed-division-overflow       b20 openat-seg-fault
///   b9  load-invalid-address           b21 read-seg-fault
///   b10 store-invalid-address          b22 write-seg-fault
///   b11 compressed-load-invalid-addr   b23 bad-exit-code
pub fn rotor_properties(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    core: &CoreState,
    comb: &CombinationalResult,
) {
    let bool_sid = sorts.sid_boolean;
    let mw_sid = sorts.sid_machine_word;
    let ir = comb.ir;
    let kernel = &comb.kernel;
    let seg = &core.segmentation;

    // ---- shared helpers -----------------------------------------------

    // sp value (used by stack-pointer checks)
    let sp_val = RegisterFile::load_register_by_index(
        builder,
        sorts,
        consts,
        core.register_file_state,
        regs::SP,
        Some("sp value".to_string()),
    );

    // access-size-minus-one constants for sized-block checks
    let size_1 = consts.nid_machine_word_1; // half word
    let size_3 = builder.constd(mw_sid, 3, Some("single word size - 1".to_string()));
    let size_7 = builder.constd(mw_sid, 7, Some("double word size - 1".to_string()));

    // instruction-class conditions by decoded id
    let is_id = |builder: &mut Btor2Builder, id: InstrId| -> NodeId {
        builder.eq_node(bool_sid, comb.instruction_id, consts.nid_instr_id(id), None)
    };

    // good-condition -> bad property: bad = NOT good (C's state_property).
    macro_rules! bad_from_good {
        ($good:expr, $name:expr, $comment:expr) => {{
            let not_good = builder.not(bool_sid, $good, Some("targeting".to_string()));
            builder.bad(not_good, $name, Some($comment.to_string()));
        }};
    }

    // ====================================================================
    // b0: illegal-instruction — illegal shift amounts (C is_illegal_shamt).
    // RV64: slliw/srliw/sraiw (OP-IMM-32, funct3 001/101) with ir[25] set.
    // RV32: slli/srli/srai (OP-IMM, funct3 001/101) with ir[25] set.
    // ====================================================================
    {
        let opcode = builder.slice(sorts.sid_7bit, ir, 6, 0, Some("opcode".to_string()));
        let funct3 = builder.slice(sorts.sid_3bit, ir, 14, 12, Some("funct3".to_string()));
        let bit25 = builder.slice(
            sorts.sid_boolean,
            ir,
            25,
            25,
            Some("ir[25] (shamt[5])".to_string()),
        );

        let shift_funct = {
            let f001 = builder.constd(sorts.sid_3bit, 0b001, None);
            let f101 = builder.constd(sorts.sid_3bit, 0b101, None);
            let is_sll = builder.eq_node(bool_sid, funct3, f001, None);
            let is_srx = builder.eq_node(bool_sid, funct3, f101, None);
            builder.or_node(bool_sid, is_sll, is_srx, Some("shift funct3?".to_string()))
        };

        let shamt_opcode_val: u64 = if config.xlen == Xlen::X64 {
            0b0011011 // OP-IMM-32: 32-bit shifts on RV64 have 5-bit shamt
        } else {
            0b0010011 // OP-IMM: shifts on RV32 have 5-bit shamt
        };
        let shamt_opcode = builder.constd(sorts.sid_7bit, shamt_opcode_val, None);
        let is_shamt_op = builder.eq_node(bool_sid, opcode, shamt_opcode, None);

        let funct_and_op = builder.and_node(bool_sid, is_shamt_op, shift_funct, None);
        let illegal_shamt = builder.and_node(
            bool_sid,
            funct_and_op,
            bit25,
            Some("illegal shift amount?".to_string()),
        );
        builder.bad(
            illegal_shamt,
            "illegal-instruction",
            Some("illegal instruction".to_string()),
        );
    }

    // ====================================================================
    // b1: illegal-compressed-instruction — illegal compressed imm/shamt
    // (C is_illegal_compressed_instruction_imm_shamt). Self-gating on the
    // compressed quadrants (c_ir[1:0] != 11), so it cannot fire on
    // uncompressed code. Cases: c.addi4spn with nzuimm == 0 (including the
    // all-zero illegal instruction); on RV32 c.slli/c.srli/c.srai with
    // shamt[5] (c_ir[12]) set.
    // ====================================================================
    {
        let c_ir = builder.slice(
            sorts.sid_half_word,
            ir,
            15,
            0,
            Some("compressed IR".to_string()),
        );
        let c_op = builder.slice(
            sorts.sid_2bit,
            c_ir,
            1,
            0,
            Some("compressed opcode".to_string()),
        );
        let c_funct3 = builder.slice(
            sorts.sid_3bit,
            c_ir,
            15,
            13,
            Some("compressed funct3".to_string()),
        );

        let c0 = builder.constd(sorts.sid_2bit, 0b00, None);
        let is_c0 = builder.eq_node(bool_sid, c_op, c0, None);
        let f000 = builder.constd(sorts.sid_3bit, 0b000, None);
        let is_f000 = builder.eq_node(bool_sid, c_funct3, f000, None);

        // c.addi4spn nzuimm = c_ir[12:5]; zero means illegal (covers 0x0000)
        let nzuimm = builder.slice(
            sorts.sid_8bit,
            c_ir,
            12,
            5,
            Some("c.addi4spn nzuimm".to_string()),
        );
        let zero8 = builder.constd(sorts.sid_8bit, 0, None);
        let nzuimm_zero = builder.eq_node(bool_sid, nzuimm, zero8, None);

        let addi4spn = builder.and_node(bool_sid, is_c0, is_f000, None);
        let mut illegal_c = builder.and_node(
            bool_sid,
            addi4spn,
            nzuimm_zero,
            Some("c.addi4spn with zero immediate (illegal)?".to_string()),
        );

        if config.xlen == Xlen::X32 {
            // RV32: compressed shifts with shamt[5] (c_ir[12]) are illegal
            let c1 = builder.constd(sorts.sid_2bit, 0b01, None);
            let c2 = builder.constd(sorts.sid_2bit, 0b10, None);
            let is_c1 = builder.eq_node(bool_sid, c_op, c1, None);
            let is_c2 = builder.eq_node(bool_sid, c_op, c2, None);
            let bit12 = builder.slice(sorts.sid_boolean, c_ir, 12, 12, None);

            // c.srli/c.srai: C1, funct3 100
            let f100 = builder.constd(sorts.sid_3bit, 0b100, None);
            let is_f100 = builder.eq_node(bool_sid, c_funct3, f100, None);
            let sr_c1 = builder.and_node(bool_sid, is_c1, is_f100, None);
            // c.slli: C2, funct3 000
            let slli_c2 = builder.and_node(bool_sid, is_c2, is_f000, None);
            let shift_c = builder.or_node(bool_sid, sr_c1, slli_c2, None);
            let illegal_shift = builder.and_node(
                bool_sid,
                shift_c,
                bit12,
                Some("compressed shift with shamt[5] on RV32 (illegal)?".to_string()),
            );
            illegal_c = builder.or_node(bool_sid, illegal_c, illegal_shift, None);
        }

        builder.bad(
            illegal_c,
            "illegal-compressed-instruction",
            Some("illegal compressed instruction".to_string()),
        );
    }

    // ====================================================================
    // b2: known-instructions — good = decoder produced a known instruction
    // (C is_enabled(instruction_ID)); bad = decoded Unknown.
    // ====================================================================
    builder.bad(
        comb.is_unknown_instruction,
        "known-instructions",
        Some("known instructions".to_string()),
    );

    // ====================================================================
    // b3-b5: fetch checks on the NEXT pc ("imminent fetch", C uses
    // control_flow_nid = the kernel-adjusted next pc).
    // ====================================================================
    {
        let next_pc = comb.next_pc;

        // b3: fetch-invalid-address — next pc fits the virtual address space
        let good = seg
            .is_machine_word_virtual_address(builder, sorts, next_pc)
            .unwrap_or(consts.nid_true);
        bad_from_good!(
            good,
            "fetch-invalid-address",
            "imminent fetch at invalid address"
        );

        // b4: fetch-unaligned — next pc aligned to the instruction grid
        // (2-byte with RVC, else 4-byte)
        let mask_val: u64 = if config.enable_c { 1 } else { 3 };
        let mask = builder.constd(
            mw_sid,
            mask_val,
            Some("instruction word size mask".to_string()),
        );
        let low = builder.and_node(mw_sid, next_pc, mask, Some("next pc alignment".to_string()));
        let aligned = builder.eq_node(
            bool_sid,
            low,
            consts.nid_machine_word_0,
            Some("next pc unaligned".to_string()),
        );
        bad_from_good!(aligned, "fetch-unaligned", "imminent unaligned fetch");

        // b5: fetch-seg-fault — next pc within the code segment
        let in_code = seg.is_address_in_code_segment(builder, sorts, next_pc);
        bad_from_good!(
            in_code,
            "fetch-seg-fault",
            "imminent fetch segmentation fault"
        );
    }

    // ====================================================================
    // b6: unknown-syscall-ID — active ecall with unrecognized a7
    // (exit, brk, openat/open, read, write are known).
    // ====================================================================
    {
        let known1 = builder.or_node(bool_sid, kernel.is_exit, kernel.is_brk, None);
        let known2 = builder.or_node(bool_sid, kernel.is_openat, kernel.is_read, None);
        let known12 = builder.or_node(bool_sid, known1, known2, None);
        let known = builder.or_node(
            bool_sid,
            known12,
            kernel.is_write,
            Some("known syscall ID?".to_string()),
        );
        let unknown = builder.not(bool_sid, known, None);
        let bad = builder.and_node(
            bool_sid,
            kernel.active_ecall,
            unknown,
            Some("unknown syscall ID".to_string()),
        );
        builder.bad(
            bad,
            "unknown-syscall-ID",
            Some("unknown syscall ID".to_string()),
        );
    }

    // ====================================================================
    // b7: division-by-zero
    // ====================================================================
    if config.check_division_by_zero {
        builder.bad(
            comb.division_by_zero,
            "division-by-zero",
            Some("division by zero".to_string()),
        );
    }

    // ====================================================================
    // b8: signed-division-overflow
    // ====================================================================
    if config.check_division_overflow {
        builder.bad(
            comb.signed_division_overflow,
            "signed-division-overflow",
            Some("signed division overflow".to_string()),
        );
    }

    // ====================================================================
    // b9-b13: invalid-address checks (addresses fit the virtual address
    // space; C load/store_valid_address + stack pointer).
    // ====================================================================
    if config.check_invalid_addresses {
        // b9: load-invalid-address
        let load_addr_ok = seg
            .is_machine_word_virtual_address(builder, sorts, comb.load_addr)
            .unwrap_or(consts.nid_true);
        let not_load = builder.not(bool_sid, comb.is_load, None);
        let good = builder.or_node(
            bool_sid,
            not_load,
            load_addr_ok,
            Some("load at valid address?".to_string()),
        );
        bad_from_good!(good, "load-invalid-address", "load at invalid address");

        // b10: store-invalid-address
        let store_addr_ok = seg
            .is_machine_word_virtual_address(builder, sorts, comb.store_addr)
            .unwrap_or(consts.nid_true);
        let not_store = builder.not(bool_sid, comb.writes_memory, None);
        let good = builder.or_node(
            bool_sid,
            not_store,
            store_addr_ok,
            Some("store at valid address?".to_string()),
        );
        bad_from_good!(good, "store-invalid-address", "store at invalid address");

        // b11/b12: compressed load/store invalid address. The compressed
        // memory address (rs1'/sp + uimm) — gated on the instruction being
        // a compressed load/store, so they cannot fire on uncompressed code.
        let (c_load_addr, c_store_addr) =
            compressed_mem_addresses(builder, sorts, consts, config, core, comb);

        let ok = seg
            .is_machine_word_virtual_address(builder, sorts, c_load_addr)
            .unwrap_or(consts.nid_true);
        let not_cl = builder.not(bool_sid, comb.is_compressed_load, None);
        let good = builder.or_node(bool_sid, not_cl, ok, None);
        bad_from_good!(
            good,
            "compressed-load-invalid-address",
            "compressed load at invalid address"
        );

        let ok = seg
            .is_machine_word_virtual_address(builder, sorts, c_store_addr)
            .unwrap_or(consts.nid_true);
        let not_cs = builder.not(bool_sid, comb.is_compressed_store, None);
        let good = builder.or_node(bool_sid, not_cs, ok, None);
        bad_from_good!(
            good,
            "compressed-store-invalid-address",
            "compressed store at invalid address"
        );

        // b13: stack-pointer-invalid-address
        let sp_ok = seg
            .is_machine_word_virtual_address(builder, sorts, sp_val)
            .unwrap_or(consts.nid_true);
        bad_from_good!(
            sp_ok,
            "stack-pointer-invalid-address",
            "stack pointer invalid address"
        );
    }

    // ====================================================================
    // b14-b18: segmentation-fault checks (sized blocks fully inside
    // data ∪ heap ∪ stack; C load/store_no_seg_faults).
    // ====================================================================
    if config.check_seg_faults {
        // per-width good conditions for loads at comb.load_addr
        let block_d = seg.is_sized_block_in_main_memory(builder, sorts, comb.load_addr, size_7);
        let block_w = seg.is_sized_block_in_main_memory(builder, sorts, comb.load_addr, size_3);
        let block_h = seg.is_sized_block_in_main_memory(builder, sorts, comb.load_addr, size_1);
        let addr_in = seg.is_address_in_main_memory(builder, sorts, comb.load_addr);

        // good = for each load class: block fits; non-loads: TRUE
        let mut good = consts.nid_true;
        let apply = |builder: &mut Btor2Builder, good_acc: NodeId, cond: NodeId, ok: NodeId| {
            let not_cond = builder.not(bool_sid, cond, None);
            let implied = builder.or_node(bool_sid, not_cond, ok, None);
            builder.and_node(bool_sid, good_acc, implied, None)
        };

        if config.xlen == Xlen::X64 {
            let is_ld = is_id(builder, InstrId::Ld);
            good = apply(builder, good, is_ld, block_d);
            let is_lwu = is_id(builder, InstrId::Lwu);
            good = apply(builder, good, is_lwu, block_w);
        }
        let is_lw = is_id(builder, InstrId::Lw);
        good = apply(builder, good, is_lw, block_w);
        let is_lh = is_id(builder, InstrId::Lh);
        good = apply(builder, good, is_lh, block_h);
        let is_lhu = is_id(builder, InstrId::Lhu);
        good = apply(builder, good, is_lhu, block_h);
        let is_lb = is_id(builder, InstrId::Lb);
        good = apply(builder, good, is_lb, addr_in);
        let is_lbu = is_id(builder, InstrId::Lbu);
        good = apply(builder, good, is_lbu, addr_in);
        bad_from_good!(good, "load-seg-fault", "load segmentation fault");

        // stores at comb.store_addr
        let block_d = seg.is_sized_block_in_main_memory(builder, sorts, comb.store_addr, size_7);
        let block_w = seg.is_sized_block_in_main_memory(builder, sorts, comb.store_addr, size_3);
        let block_h = seg.is_sized_block_in_main_memory(builder, sorts, comb.store_addr, size_1);
        let addr_in = seg.is_address_in_main_memory(builder, sorts, comb.store_addr);

        let mut good = consts.nid_true;
        if config.xlen == Xlen::X64 {
            let is_sd = is_id(builder, InstrId::Sd);
            good = apply(builder, good, is_sd, block_d);
        }
        let is_sw = is_id(builder, InstrId::Sw);
        good = apply(builder, good, is_sw, block_w);
        let is_sh = is_id(builder, InstrId::Sh);
        good = apply(builder, good, is_sh, block_h);
        let is_sb = is_id(builder, InstrId::Sb);
        good = apply(builder, good, is_sb, addr_in);
        bad_from_good!(good, "store-seg-fault", "store segmentation fault");

        // b16/b17: compressed load/store seg faults (word/double widths)
        let (c_load_addr, c_store_addr) =
            compressed_mem_addresses(builder, sorts, consts, config, core, comb);

        let cl_w = seg.is_sized_block_in_main_memory(builder, sorts, c_load_addr, size_3);
        let cl_d = seg.is_sized_block_in_main_memory(builder, sorts, c_load_addr, size_7);
        let mut good = consts.nid_true;
        let is_clw = {
            let a = is_id(builder, InstrId::CLw);
            let b = is_id(builder, InstrId::CLwsp);
            builder.or_node(bool_sid, a, b, None)
        };
        good = apply(builder, good, is_clw, cl_w);
        if config.xlen == Xlen::X64 {
            let is_cld = {
                let a = is_id(builder, InstrId::CLd);
                let b = is_id(builder, InstrId::CLdsp);
                builder.or_node(bool_sid, a, b, None)
            };
            good = apply(builder, good, is_cld, cl_d);
        }
        bad_from_good!(
            good,
            "compressed-load-seg-fault",
            "compressed load segmentation fault"
        );

        let cs_w = seg.is_sized_block_in_main_memory(builder, sorts, c_store_addr, size_3);
        let cs_d = seg.is_sized_block_in_main_memory(builder, sorts, c_store_addr, size_7);
        let mut good = consts.nid_true;
        let is_csw = {
            let a = is_id(builder, InstrId::CSw);
            let b = is_id(builder, InstrId::CSwsp);
            builder.or_node(bool_sid, a, b, None)
        };
        good = apply(builder, good, is_csw, cs_w);
        if config.xlen == Xlen::X64 {
            let is_csd = {
                let a = is_id(builder, InstrId::CSd);
                let b = is_id(builder, InstrId::CSdsp);
                builder.or_node(bool_sid, a, b, None)
            };
            good = apply(builder, good, is_csd, cs_d);
        }
        bad_from_good!(
            good,
            "compressed-store-seg-fault",
            "compressed store segmentation fault"
        );

        // b18: stack-pointer-seg-fault — sp within the stack segment
        let sp_in_stack = seg.is_address_in_stack_segment(builder, sorts, sp_val);
        bad_from_good!(
            sp_in_stack,
            "stack-pointer-seg-fault",
            "stack pointer segmentation fault"
        );

        // ================================================================
        // b19-b22: kernel seg-fault checks (C kernel_properties)
        // ================================================================

        // b19: brk-seg-fault — active brk with INVALID new program break.
        // C (rotor.c:11336-11349): invalid = NOT(vaddr-ok(a0) AND
        // a0 <= heap end). NOTE: no lower bound against the current brk in
        // the PROPERTY (the data flow handles that); brk(0) queries are valid.
        {
            let a0_le_end = builder.ulte(
                bool_sid,
                kernel.a0,
                seg.heap_end,
                Some("new program break <= end of heap segment?".to_string()),
            );
            let valid = match seg.is_machine_word_virtual_address(builder, sorts, kernel.a0) {
                Some(vaddr_ok) => builder.and_node(
                    bool_sid,
                    vaddr_ok,
                    a0_le_end,
                    Some("does machine word work as virtual address?".to_string()),
                ),
                None => a0_le_end,
            };
            let invalid = builder.not(
                bool_sid,
                valid,
                Some("is new program break invalid?".to_string()),
            );
            let bad = builder.and_node(
                bool_sid,
                kernel.active_brk,
                invalid,
                Some("invalid new program break with active brk system call".to_string()),
            );
            builder.bad(
                bad,
                "brk-seg-fault",
                Some("possible brk segmentation fault".to_string()),
            );
        }

        // b20: openat-seg-fault — filename access range
        // [a1, a1 + MAX_STRING_LENGTH - 1] not fully inside the HEAP segment
        // (C rotor.c:11353-11364, MAX_STRING_LENGTH = 128).
        {
            let max_string_length =
                builder.constd(mw_sid, 128, Some("maximum string length".to_string()));
            let range_ok = seg.is_range_in_heap_segment(
                builder,
                sorts,
                kernel.a1,
                max_string_length,
                consts.nid_machine_word_1,
            );
            let range_bad = builder.not(
                bool_sid,
                range_ok,
                Some("is filename access not in heap segment?".to_string()),
            );
            let bad = builder.and_node(
                bool_sid,
                kernel.active_openat,
                range_bad,
                Some("openat system call filename access may cause segmentation fault".to_string()),
            );
            builder.bad(
                bad,
                "openat-seg-fault",
                Some("possible openat segmentation fault".to_string()),
            );
        }

        // b21: read-seg-fault — checked only at the START of a read
        // (read_bytes == 0): buffer range [a1, a1+a2) not fully in the HEAP
        // segment while reading more than 0 bytes (C rotor.c:11366-11386).
        {
            let read_starting = {
                let no_bytes_yet = builder.eq_node(
                    bool_sid,
                    core.kernel.read_bytes,
                    consts.nid_machine_word_0,
                    Some("have bytes been read yet?".to_string()),
                );
                builder.and_node(
                    bool_sid,
                    kernel.active_read,
                    no_bytes_yet,
                    Some("no bytes read yet by active read system call".to_string()),
                )
            };
            let a2_gt_0 = builder.ugt(
                bool_sid,
                kernel.a2,
                consts.nid_machine_word_0,
                Some("bytes to be read > 0?".to_string()),
            );
            let range_ok = seg.is_range_in_heap_segment(
                builder,
                sorts,
                kernel.a1,
                kernel.a2,
                consts.nid_machine_word_1,
            );
            let range_bad = builder.not(
                bool_sid,
                range_ok,
                Some("is read system call access not in heap segment?".to_string()),
            );
            let cond = builder.and_node(
                bool_sid,
                a2_gt_0,
                range_bad,
                Some("may bytes to be read not be stored in heap segment?".to_string()),
            );
            let bad = builder.and_node(
                bool_sid,
                read_starting,
                cond,
                Some("storing bytes to be read may cause segmentation fault".to_string()),
            );
            builder.bad(
                bad,
                "read-seg-fault",
                Some("possible read segmentation fault".to_string()),
            );
        }

        // b22: write-seg-fault — symmetric for writes
        {
            let a2_gt_0 = builder.ugt(
                bool_sid,
                kernel.a2,
                consts.nid_machine_word_0,
                Some("bytes to be written > 0?".to_string()),
            );
            let range_ok = seg.is_range_in_heap_segment(
                builder,
                sorts,
                kernel.a1,
                kernel.a2,
                consts.nid_machine_word_1,
            );
            let range_bad = builder.not(
                bool_sid,
                range_ok,
                Some("is write system call access not in heap segment?".to_string()),
            );
            let cond = builder.and_node(bool_sid, a2_gt_0, range_bad, None);
            let bad = builder.and_node(
                bool_sid,
                kernel.active_write,
                cond,
                Some("loading bytes to be written may cause segmentation fault".to_string()),
            );
            builder.bad(
                bad,
                "write-seg-fault",
                Some("possible write segmentation fault".to_string()),
            );
        }
    }

    // ====================================================================
    // b23: bad-exit-code — active exit AND a0 == target exit code
    // (C: "rotor ... - N"; the reference benchmarks use N = 0).
    // ====================================================================
    if config.check_bad_exit_code {
        let target = builder.constd(
            mw_sid,
            config.target_exit_code,
            Some(format!("bad exit code {}", config.target_exit_code)),
        );
        let code_match = builder.eq_node(
            bool_sid,
            kernel.a0,
            target,
            Some("actual exit code == bad exit code?".to_string()),
        );
        let bad = builder.and_node(
            bool_sid,
            kernel.active_exit,
            code_match,
            Some("active exit system call with bad exit code".to_string()),
        );
        builder.bad(
            bad,
            "bad-exit-code",
            Some(format!("exit({})", config.target_exit_code)),
        );
    }

    // good-exit-code (optional, not in the reference run): exit with any
    // OTHER code than the target.
    if config.check_good_exit_code {
        let target = builder.constd(
            mw_sid,
            config.target_exit_code,
            Some(format!("good exit code {}", config.target_exit_code)),
        );
        let code_differs = builder.neq(
            bool_sid,
            kernel.a0,
            target,
            Some("actual exit code != good exit code?".to_string()),
        );
        let bad = builder.and_node(
            bool_sid,
            kernel.active_exit,
            code_differs,
            Some("active exit system call with good exit code".to_string()),
        );
        builder.bad(
            bad,
            "good-exit-code",
            Some(format!("exit({})", config.target_exit_code)),
        );
    }
}

/// Compute the memory addresses of compressed loads and stores from the
/// compressed instruction encoding (c_ir = low 16 bits of the fetched word).
///
///   c.lw/c.sw   (C0): rs1' + uimm[6:2]   where uimm[5:3]=c_ir[12:10],
///                     uimm[2]=c_ir[6], uimm[6]=c_ir[5]
///   c.ld/c.sd   (C0): rs1' + uimm[7:3]   where uimm[5:3]=c_ir[12:10],
///                     uimm[7:6]=c_ir[6:5]
///   c.lwsp      (C2): sp + uimm          uimm[5]=c_ir[12], uimm[4:2]=c_ir[6:4],
///                     uimm[7:6]=c_ir[3:2]
///   c.ldsp      (C2): sp + uimm          uimm[5]=c_ir[12], uimm[4:3]=c_ir[6:5],
///                     uimm[8:6]=c_ir[4:2]
///   c.swsp      (C2): sp + uimm          uimm[5:2]=c_ir[12:9], uimm[7:6]=c_ir[8:7]
///   c.sdsp      (C2): sp + uimm          uimm[5:3]=c_ir[12:10], uimm[8:6]=c_ir[9:7]
///
/// Returns (load_address, store_address) selected per decoded instruction id;
/// for non-compressed-memory instructions the value is unused (the properties
/// gate on is_compressed_load/store).
fn compressed_mem_addresses(
    builder: &mut Btor2Builder,
    sorts: &MachineSorts,
    consts: &MachineConstants,
    config: &Config,
    core: &CoreState,
    comb: &CombinationalResult,
) -> (NodeId, NodeId) {
    let bool_sid = sorts.sid_boolean;
    let mw_sid = sorts.sid_machine_word;
    let ir = comb.ir;

    // helper: zero-extend a slice of c_ir into a machine word shifted left
    let field = |builder: &mut Btor2Builder, hi: u32, lo: u32, shift: u32| -> NodeId {
        let bits = hi - lo + 1;
        let word_bits = config.machine_word_bits();
        let sid = match bits {
            1 => sorts.sid_boolean,
            2 => sorts.sid_2bit,
            3 => sorts.sid_3bit,
            4 => sorts.sid_4bit,
            _ => sorts.sid_5bit,
        };
        let s = builder.slice(sid, ir, hi, lo, None);
        let ext = builder.uext(mw_sid, s, word_bits - bits, None);
        if shift > 0 {
            let sh = builder.constd(mw_sid, shift as u64, None);
            builder.sll(mw_sid, ext, sh, None)
        } else {
            ext
        }
    };

    // register operands
    let rs1_prime = {
        // 8 + c_ir[9:7] as a 5-bit register index
        let r = builder.slice(sorts.sid_3bit, ir, 9, 7, None);
        let r5 = builder.uext(sorts.sid_register_address, r, 2, None);
        let eight = builder.constd(sorts.sid_register_address, 8, None);
        builder.add(
            sorts.sid_register_address,
            r5,
            eight,
            Some("rs1' = 8 + c_ir[9:7]".to_string()),
        )
    };
    let rs1_prime_val = builder.read(
        mw_sid,
        core.register_file_state,
        rs1_prime,
        Some("rs1' value".to_string()),
    );
    let sp_addr = consts.nid_register(regs::SP);
    let sp_val = builder.read(
        mw_sid,
        core.register_file_state,
        sp_addr,
        Some("sp value".to_string()),
    );

    // C0 word offsets: uimm[5:3]=c_ir[12:10]<<3, uimm[2]=c_ir[6]<<2, uimm[6]=c_ir[5]<<6
    let w_53 = field(builder, 12, 10, 3);
    let w_2 = field(builder, 6, 6, 2);
    let w_6 = field(builder, 5, 5, 6);
    let c0w_off = {
        let t = builder.or_node(mw_sid, w_53, w_2, None);
        builder.or_node(mw_sid, t, w_6, Some("c.lw/c.sw offset".to_string()))
    };
    // C0 double offsets: uimm[5:3]=c_ir[12:10]<<3, uimm[7:6]=c_ir[6:5]<<6
    let d_53 = field(builder, 12, 10, 3);
    let d_76 = field(builder, 6, 5, 6);
    let c0d_off = builder.or_node(mw_sid, d_53, d_76, Some("c.ld/c.sd offset".to_string()));

    // C2 lwsp: uimm[5]=c_ir[12]<<5, uimm[4:2]=c_ir[6:4]<<2, uimm[7:6]=c_ir[3:2]<<6
    let lwsp_5 = field(builder, 12, 12, 5);
    let lwsp_42 = field(builder, 6, 4, 2);
    let lwsp_76 = field(builder, 3, 2, 6);
    let lwsp_off = {
        let t = builder.or_node(mw_sid, lwsp_5, lwsp_42, None);
        builder.or_node(mw_sid, t, lwsp_76, Some("c.lwsp offset".to_string()))
    };
    // C2 ldsp: uimm[5]=c_ir[12]<<5, uimm[4:3]=c_ir[6:5]<<3, uimm[8:6]=c_ir[4:2]<<6
    let ldsp_5 = field(builder, 12, 12, 5);
    let ldsp_43 = field(builder, 6, 5, 3);
    let ldsp_86 = field(builder, 4, 2, 6);
    let ldsp_off = {
        let t = builder.or_node(mw_sid, ldsp_5, ldsp_43, None);
        builder.or_node(mw_sid, t, ldsp_86, Some("c.ldsp offset".to_string()))
    };
    // C2 swsp: uimm[5:2]=c_ir[12:9]<<2, uimm[7:6]=c_ir[8:7]<<6
    let swsp_52 = field(builder, 12, 9, 2);
    let swsp_76 = field(builder, 8, 7, 6);
    let swsp_off = builder.or_node(mw_sid, swsp_52, swsp_76, Some("c.swsp offset".to_string()));
    // C2 sdsp: uimm[5:3]=c_ir[12:10]<<3, uimm[8:6]=c_ir[9:7]<<6
    let sdsp_53 = field(builder, 12, 10, 3);
    let sdsp_86 = field(builder, 9, 7, 6);
    let sdsp_off = builder.or_node(mw_sid, sdsp_53, sdsp_86, Some("c.sdsp offset".to_string()));

    // addresses per form
    let c0w_addr = builder.add(mw_sid, rs1_prime_val, c0w_off, None);
    let c0d_addr = builder.add(mw_sid, rs1_prime_val, c0d_off, None);
    let lwsp_addr = builder.add(mw_sid, sp_val, lwsp_off, None);
    let ldsp_addr = builder.add(mw_sid, sp_val, ldsp_off, None);
    let swsp_addr = builder.add(mw_sid, sp_val, swsp_off, None);
    let sdsp_addr = builder.add(mw_sid, sp_val, sdsp_off, None);

    let is_id = |builder: &mut Btor2Builder, id: InstrId| -> NodeId {
        builder.eq_node(bool_sid, comb.instruction_id, consts.nid_instr_id(id), None)
    };

    // select load address by decoded id (default: c.lw form)
    let mut load_addr = c0w_addr;
    let is_clwsp = is_id(builder, InstrId::CLwsp);
    load_addr = builder.ite(mw_sid, is_clwsp, lwsp_addr, load_addr, None);
    if config.xlen == Xlen::X64 {
        let is_cld = is_id(builder, InstrId::CLd);
        load_addr = builder.ite(mw_sid, is_cld, c0d_addr, load_addr, None);
        let is_cldsp = is_id(builder, InstrId::CLdsp);
        load_addr = builder.ite(mw_sid, is_cldsp, ldsp_addr, load_addr, None);
    }

    // select store address by decoded id (default: c.sw form)
    let mut store_addr = c0w_addr;
    let is_cswsp = is_id(builder, InstrId::CSwsp);
    store_addr = builder.ite(mw_sid, is_cswsp, swsp_addr, store_addr, None);
    if config.xlen == Xlen::X64 {
        let is_csd = is_id(builder, InstrId::CSd);
        store_addr = builder.ite(mw_sid, is_csd, c0d_addr, store_addr, None);
        let is_csdsp = is_id(builder, InstrId::CSdsp);
        store_addr = builder.ite(mw_sid, is_csdsp, sdsp_addr, store_addr, None);
    }

    (load_addr, store_addr)
}
