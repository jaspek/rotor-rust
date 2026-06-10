# Semantic-equivalence plan: Rust rotor → C rotor

Goal: on every benchmark, btormc fires the **same bad property at the same
least-k** from both rotors' models. This is the only honest, checkable
definition of "equivalent" (full formal equivalence is undecidable; the
rotor paper itself lists formal verification as future work).

Status legend: ✅ done+verified · 🔧 in progress · ❌ not started

---

## Proven non-equivalence (current state)

On `division-by-zero-3-35` at kmax=200:

| Rotor | Fires | Correct? |
|---|---|---|
| C reference | `b7` = division-by-zero | ✅ the benchmark's actual bug |
| Rust (after P0.1) | `b9` = load-invalid-address | ❌ spurious, wrong property, shallower step |

The earlier "18/18 match at k=35" was meaningless — k=35 is below the depth
at which *any* property fires (paper: bugs fire at k=66…1438).

---

## P0.1 — Zero-initialize segment bases ✅ DONE (commit cce50c8)

Code/data/heap/stack bases now `init` to zero element (code word-0, others
byte-0). Verified: catbtor + btormc -kmax 0 pass. Necessary, not sufficient.

---

## P-FOUNDATION — Segment boundaries (must come before P1) ✅ DONE

Verified against the C reference (division-by-zero benchmark):
heap start 0x12000 (page-aligned) ✓ · heap end 0x12800 ✓ · stack
[0xFFFFF800, 2^32) ✓ · brk init = heap start ✓ · SP = 0xFFFFFFF8 ✓ ·
wrap-aware checks implemented · catbtor + btormc load PASS · symbolic-argv
regression PASS. Deep run still fires the spurious load (expected — root
causes are P0.3 + P1, below).

### Original spec (for reference)

Our segments do not match C. Until they do, no address property can match.

**Evidence (C reference btor2, division-by-zero):**
```
130 consth 4 00010000 ; code start 0x10000
131 consth 4 00010250 ; code end (rounded up to instruction boundary)
132 consth 4 00011000 ; data start 0x11000
133 consth 4 00011008 ; data end 0x11008
134 consth 4 00012000 ; heap start 0x12000   ← PAGE-ALIGNED (not data_end!)
135 consth 4 00012800 ; heap end  0x12800     (= heap_start + 2048 allowance)
136 consth 4 FFFFF800 ; stack start 0xFFFFF800 (= 2^32 - 2048 allowance)
137 consth 4 00000000 ; stack end 0x00000000   (= 2^32, WRAPPED to 0)
```

**Fixes in `machine/segmentation.rs`:**
1. heap_start = page-align-up(data_start + data_size) to 4096. Currently
   `data_start + data_size` with no alignment.
2. heap_end = heap_start + heap_allowance.
3. Address space = full 32-bit (4 GB). vaddr_top = 2^32, not 2^31.
4. stack_start = 2^32 - stack_allowance (= 0xFFFFF800 for 2048).
5. stack_end = 2^32, represented wrapped to 0 in 32-bit vaddr space.
6. initial_sp (core.rs) = stack_end - word_size accordingly.

**Wrap-aware membership — port `is_block_in_segment` (rotor.c:6519):**
```c
start_comparison = start >= segment_start;
if (eval(segment_end) == 0)      // end wrapped to zero
    return start_comparison;      // skip the upper-bound check
else
    return start_comparison AND (end < segment_end);
```
Our `is_in_*_segment` always does both comparisons — wrong for the stack
whose end is 0.

Also port `cast_machine_word_to_virtual_address` and
`does_machine_word_work_as_virtual_address` (rotor.c:7706) — for a 32-bit
vaddr space on a 64-bit word, validity requires `machine_word <= highest
virtual address` AND the inner property.

---

## P0.3 — Concrete argv on the stack ✅ DONE

Implemented `initialize_concrete_argv` (selfie boot-loader layout: argc=1 at
SP, argv[0] pointer, argv/env NULL words, program-name string at the top,
all on a zeroed base). Also completed P0.1 for the REGISTER FILE — the base
register file is now zero-initialized like C ("zeroing register file"), only
SP written; a0=argc is set only in symbolic-argv mode (C doesn't set it).

Verified (division-by-zero): argv image present in model (string @
0xFFFFFFE8, SP @ 0xFFFFFFC8, argc/pointer writes confirmed), catbtor +
btormc PASS, symbolic-argv regression PASS. Deep run: spurious
load-invalid-address GONE; now fires read-seg-fault (b22) instead of
division-by-zero — attributable to remaining gaps P0.2 (read flow) and P1
(read-seg-fault semantics: single pointer vs C's range-in-heap).

### Original spec (for reference)

C boot-loader writes argc + argv pointers + the program-path string onto the
zeroed stack (see reference btor2 lines 10900-10926: word writes of the path
`>examples/symbolic/divis...zero-3-35.c`, argc=1, pointer 0xFFFFFFC8).

**Fix:** in default mode (no `--symbolic-argv`), build the same concrete argv
image on the zeroed stack base, set SP, write argc to the stack and a0 per
RISC-V calling convention. Without this, the program's startup reads argv,
gets zeros, and the (currently mis-defined) load property fires spuriously —
this is the direct cause of the division benchmark firing load-invalid.

---

## P1 — Faithful property semantics ✅ DONE (includes P0.4)

Full rewrite of properties.rs in the C reference's exact emission order
(b0..b23 by name and index). All bodies ported from rotor.c, including the
three that differed from initial inference (verified by reading the source):
brk-seg-fault has NO lower bound (brk(0) queries valid), openat checks
range [a1, a1+127] in heap, read-seg-fault fires only at read START
(read_bytes == 0). Legacy open (1024) added to the openat decode. Compressed
load/store addresses extracted from the real RVC encodings. Target exit
code CLI flag added (P0.4): bad-exit-code = exit(target), default 0.

**INDEX-EXACT EQUIVALENCE on division-by-zero-3-35:**
```
C reference:  bad state property 7 reachable at bound k = 76 SATISFIABLE
Rust rotor:   bad state property 7 reachable at bound k = 76 SATISFIABLE
```
Same b-index, same name, same least-k. catbtor + btormc load PASS;
symbolic-argv (--exit-code 1) regression PASS.

NOTE: symbolic-argv users must now pass --exit-code 1 for "bug = exit(1)"
benchmarks (old behavior was hardcoded a0 != 0); visualizer examples to be
regenerated in P2.

### Original spec (for reference)

Our property bodies are wrong (I wrote approximations). Exact C definitions:

| Property | C definition (rotor.c) | Our current (WRONG) |
|---|---|---|
| illegal-instruction | `is_illegal_shamt(ir)` — bad shift amount (1518) | "decoder returned Unknown" |
| known-instructions | constraint: `is_enabled(instruction_ID)` → bad = NOT known | duplicate of illegal-instruction |
| fetch-invalid-address | `is_machine_word_virtual_address(next_pc)` on **control_flow** (11806) | current PC, wrong predicate |
| fetch-unaligned | `(next_pc AND mask) == 0` on **control_flow** (11812) | current PC |
| fetch-seg-fault | NOT `is_address_in_machine_word_in_segment(next_pc, code)` (11824) | "PC in writable segment" |
| load-invalid-address | NOT `load_valid_address` = NOT(`maddr <= highest_vaddr`) (11849) | conflated with seg-fault |
| load-seg-fault | NOT `load_no_seg_faults` = NOT(sized block in data∪heap∪stack) (11886) | "addr in code segment" |
| store-* | symmetric with S-immediate address | same errors |
| compressed-* | same, via c_ir + compressed address | approximations |
| stack-pointer-invalid | NOT `is_machine_word_virtual_address(sp)` (11914) | approximation |
| stack-pointer-seg-fault | NOT `is_address_in_machine_word_in_segment(sp, stack)` (11917) | approximation |
| read/write-seg-fault | range `[a1, a1+a2)` in **heap** (11380-11403) | single pointer a1 |

Key helpers to port (exact bodies extracted):
- `is_machine_word_virtual_address` (7552): `machine_word <= highest_vaddr`
- `is_sized_block_in_main_memory` (7773): block fits in data∪heap∪stack, with
  per-access size (double/single/half/byte minus 1) via `decode_load`/`decode_store`
- `is_range_in_machine_word_in_segment` (7766): `[addr, addr+range-1]` in segment
- `is_block_in_segment` (6519): wrap-aware (above)

**Property ORDER must match C** so b-indices line up:
illegal-instruction, illegal-compressed, known-instructions,
fetch-invalid, fetch-unaligned, fetch-seg-fault, unknown-syscall-ID,
division-by-zero, signed-division-overflow, load-invalid, store-invalid,
compressed-load-invalid, compressed-store-invalid, stack-pointer-invalid,
load-seg-fault, store-seg-fault, compressed-load-seg-fault,
compressed-store-seg-fault, stack-pointer-seg-fault, brk-seg-fault,
openat-seg-fault, read-seg-fault, write-seg-fault, bad-exit-code.
(C order — see reference b0..b23 mapping in this repo's notes.)

---

## P0.2 — read-syscall data flow + PC stall ✅ DONE (includes P0.5)

Full port of C's kernel_combinational + kernel_sequential: per-step syscall
decode, brk validity, fd increment (P0.5 file-descriptor state included),
read-progress helpers, exact nested-ITE read return value, PC stall on exit
and ongoing reads, one-input-byte-per-transition heap delivery, read_bytes
reset-to-zero, input-buffer freeze (self-loop next).

**VERIFIED EQUIVALENCE DATA POINT (division-by-zero-3-35):**
- C reference: division-by-zero SATISFIABLE at bound k = 76
- Rust rotor:  division-by-zero SATISFIABLE at bound k = 76
Same property, same least-k. catbtor + btormc load PASS, symbolic-argv
regression PASS. Property INDEX differs (b1 vs b7) until P1 fixes emission
order.

### Original spec (for reference)

C (`kernel_combinational` 11131, `kernel_sequential` 11225):
- While `still_reading_active_read`: PC stalls (control_flow = pc, not next).
- Each transition stores `input_buffer[bytes_to_read - readable_bytes]`
  into heap at `a1 + read_bytes`.
- `readable_bytes` decrements, `read_bytes` increments while reading.
- a0 gets the read return value (bytes read / -1 / 0) when the read finishes.

Our model has the input_buffer state but never reads it and never stalls PC.
Needs wiring into combinational (heap data flow + control flow) and sequential.

---

## P0.4 — Target exit-code parameter ❌

C: bad-exit = `active_exit AND a0 == target_exit_code`; good-exit =
`a0 != target`. Target from CLI (`rotor … - N`). Ours hardcodes `a0 != 0`.
Add a `--exit-code N` config field; default target 0 (matching `- 0`).

## P0.5 — file-descriptor kernel state ❌

C: state `file-descriptor`, init 0, openat returns then increments it.
Add to `KernelState`.

---

## P2 — Deep equivalence harness + CSE-off experiment ❌

1. For each of the 18 benchmarks, run btormc on **both** rotors at a kmax
   large enough (≥1500 per paper) and record (fired property, least-k).
   Equivalent ⇔ identical (property, least-k) for all 18.
2. Professor's experiment: add `--no-cse` flag (builder already has
   `set_cse`; just expose it), regenerate, and compare model size + solver
   behavior with C rotor's reuse disabled. Confirms the speed story is the
   data-structure change and nothing semantic.

---

## Order of execution (each step verified before the next)

1. P-FOUNDATION (segments + wrap-aware checks) — nothing matches without it.
2. P0.3 (argv) — stops the spurious load on startup.
3. P1 (property semantics + order) — make the right property fire.
4. Re-test division-by-zero: must fire division-by-zero in BOTH at same k.
5. P0.2 (read flow), P0.4 (exit target), P0.5 (fd).
6. P2 harness across all 18 + CSE-off experiment.

Each step: rebuild, regenerate, `catbtor` + `btormc -kmax 0` must pass, then
deep-run the affected benchmark and compare fired property + least-k with C.
Only claim equivalence where btormc actually shows it.
