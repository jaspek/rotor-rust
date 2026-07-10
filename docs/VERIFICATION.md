# Verification evidence

This document collects the machine-checked evidence behind the
repository's claims, in three parts:

- **[A. Deep equivalence](#a-deep-equivalence-across-the-18-standard-benchmarks)** — the differential campaign against the C reference
  (36/36 paired verdicts), including the CSE-off experiment.
- **[B. Profiling](#b-profiling-the-speed-difference-by-counting)** — the counter-based analysis attributing the entire
  performance difference, plus the output-size decomposition.
- **[C. Crash report](#c-crash-report-rotor-aborts-with-ite-then-sort-mismatch-error-when-reuse_lines--0)** — the reference rotor's abort under
  `reuse_lines = 0`; self-contained and sendable as-is.

All paths are relative to the repository root unless stated otherwise.

---

## A. Deep equivalence across the 18 standard benchmarks

Method: for each benchmark, generate the Rust model with C-matching
parameters (`--bytes-to-read 1 --heap 2048 --stack 2048 --exit-code 0`),
then run `btormc -kmax 1500` (covering the rotor paper's reported 66..1438
bug range) on BOTH the C reference model and the Rust model, and compare
the **fired property index** and the **least bound k**.

Equivalence criterion: same property at the same k (or both UNSAT@1500).

Results (from benchmarks/deep_equivalence_results.csv) — **FINAL: 18/18**:

| Benchmark | C rotor | Rust rotor | Match |
|---|---|---|---|
| division-by-zero-3-35 | division-by-zero @ 76 | division-by-zero @ 76 | YES |
| invalid-memory-access-fail-2-35 | store-invalid-address @ 79 | store-invalid-address @ 79 | YES |
| memory-access-fail-1-35 | load-seg-fault @ 66 | load-seg-fault @ 66 | YES |
| nested-if-else-1-35 | bad-exit-code @ 100 | bad-exit-code @ 100 | YES |
| nested-if-else-reverse-1-35 | bad-exit-code @ 103 | bad-exit-code @ 103 | YES |
| nested-recursion-fail-1-35 | UNSAT @ 1500 | UNSAT @ 1500 | YES |
| recursive-ackermann-1-35 | bad-exit-code @ 152 | bad-exit-code @ 152 | YES |
| recursive-factorial-fail-1-35 | bad-exit-code @ 119 | bad-exit-code @ 119 | YES |
| recursive-fibonacci-1-10 | bad-exit-code @ 118 | bad-exit-code @ 118 | YES |
| return-from-loop-1-35 | UNSAT @ 1500 | UNSAT @ 1500 | YES |
| simple-assignment-1-35 | bad-exit-code @ 96 | bad-exit-code @ 96 | YES |
| simple-decreasing-loop-1-35 | bad-exit-code @ 99 | bad-exit-code @ 99 | YES |
| simple-if-else-1-35 | bad-exit-code @ 108 | bad-exit-code @ 108 | YES |
| simple-if-else-reverse-1-35 | bad-exit-code @ 108 | bad-exit-code @ 108 | YES |
| simple-if-without-else-1-35 | bad-exit-code @ 101 | bad-exit-code @ 101 | YES |
| simple-increasing-loop-1-35 | bad-exit-code @ 93 | bad-exit-code @ 93 | YES |
| three-level-nested-loop-fail-1-35 | bad-exit-code @ 103 | bad-exit-code @ 103 | YES |
| two-level-nested-loop-1-35 | bad-exit-code @ 99 | bad-exit-code @ 99 | YES |

Five distinct property types fire (division, store-invalid, load-seg-fault,
bad-exit-code, plus two agreed-UNSAT rows); the recursion benchmarks match
step-exactly through hundreds of instructions of nested call frames.

### Second configuration: target exit code 1

The rotor paper's experiments hunt "non-zero exit code" bad states. The
entire suite was re-run with both rotors at target exit code 1 (C:
`rotor -m64 -c <prog>.c - 1`; Rust: `--exit-code 1`), all other parameters
unchanged. Result: **18/18 again** (deep_equivalence_results_exit1.csv).

| Group | Benchmarks | Both rotors report |
|---|---|---|
| exit-independent properties | division-by-zero, invalid-memory-access, memory-access-fail | identical to the exit-0 run (b7@76, b10@79, b14@66) — these fire before any exit |
| planted exit(1) bugs reachable | return-from-loop @ 95, simple-assignment @ 91, simple-if-else @ 107, simple-if-else-reverse @ 105, simple-if-without-else @ 106 | same property, same k |
| not reachable at kmax=1500 | the remaining 10 | both UNSAT@1500 |

Notably return-from-loop FLIPS between configurations (UNSAT under target 0,
SAT@95 under target 1) and both rotors flip identically — the equivalence is
not an artifact of one parameter choice. Combined evidence: 36 paired
verdicts across two configurations, zero divergences.

### Methodology note: the harness caught itself once

During the parallel run, three-level-nested-loop-fail briefly looked like a
divergence (C "UNSAT", Rust SAT@103). Investigation showed the C-side btormc
container had died under 9-way memory pressure and the runner recorded the
empty output as UNSAT — a harness bug, not a rotor bug. Re-run alone, C
fires bad-exit-code @ 103, identical to Rust. The runner now distinguishes
"btormc completed, nothing reachable" from "no output" (recorded as ERROR),
and all UNSAT rows were re-verified with completion evidence.

### The CSE-off experiment (duplicate check disabled in both rotors)

Experiment: the duplicate check is disabled in both rotors, the models are
regenerated, and the outputs are compared for well-formedness and
model-checking behaviour.

**C rotor (reuse_lines = 0, set globally, rebuilt from source).** With the
duplicate check disabled globally, the C rotor **crashes** while modeling
selfie:

```
/rotor-cse-off: ite then sort mismatch error
Exit status: 14   (0.08 s into the run)
```

The C implementation depends on line reuse for node identity: with reuse
off, sort lines get duplicated, an ITE's branches end up referencing
different (but equal) sort instances, and rotor's internal sort check
aborts. The duplicate check in C is not only a size optimization — parts of
the generator are correct only because identical lines are reused. The
complete reproduction and root-cause analysis is Part C below.

Baseline C run for reference (reuse ON, selfie.c, same container):
wall 2:02.8, peak memory 432 MB, model 138,820 lines (10.6 MB),
"3,165,611 lines of model formulae generated" before reuse/pruning.
(This run includes selfie compiling itself via `-c`; the apples-to-apples
binary-only figures are 139 s / 428 MB — see Part B below.)

**Rust rotor (`--no-cse` flag).** Models grow but stay correct, because
every node carries its sort as an explicit parameter — branch
sort-consistency does not depend on deduplication:

| Input | CSE on | CSE off | Growth | Valid? |
|---|---|---|---|---|
| division-by-zero-3-35.m | 1,693 lines (52 KB) | 2,433 lines (74 KB) | 1.44x | catbtor PASS |
| selfie.m (43k instructions) | 110,904 lines (3.1 MB) | 159,018 lines (5.5 MB) | 1.43x | catbtor PASS |

The modest 1.43x growth confirms the C-side observation that the bulk of a
binary-initialized model (the init chains) has little reuse to exploit; the
dedup matters most in the instruction-semantics logic.

**Conclusion.** Dedup is semantically neutral in the Rust implementation
(verified: same btormc verdict with and without). The speed difference
between the rotors is the data structure (HashMap O(1) vs list walk O(N)
per lookup), not the amount of dedup performed: C rotor cannot even run
with reuse globally off, while Rust rotor runs fine either way.

### Reproducers

```powershell
# the deep harness
cd benchmarks; .\run_deep_equivalence.ps1 -Kmax 1500

# Rust CSE off
.\target\release\rotor.exe <bin.m> --xlen x64 --no-cse -o model.btor2

# C CSE off (crashes)
docker build -t selfie-cse -f benchmarks/cse-experiment/Dockerfile.cse benchmarks/cse-experiment
docker run --rm selfie-cse
```

---

## B. Profiling the speed difference by counting

Method (counter-based profiling): the basic operation — the question
*"is this subexpression already in the system?"* — is defined, the code
location answering it is identified in both tools, invocations and hits are
counted with plain integer counters, the ballparks are compared, and the
timing is then reasoned about from the counts.

### Instrumentation points

| | C rotor | Rust rotor |
|---|---|---|
| creation entry | `new_line` (tools/rotor.c:3712) | `Btor2Builder::intern` (rotor/src/btor2/builder.rs) |
| the question | `find_equal_line` (tools/rotor.c:3697), linear walk calling `are_lines_equal` per node | `HashMap::get` — one probe |
| counters added | `profile_new_line_calls`, `profile_find_calls`, `profile_comparisons`, `profile_hits` | `profile_lookups`, `profile_hits` |

Both instrumented builds run on the same input: selfie.m (43,406
instructions), 64-bit, 1 input byte, heap/stack 2048.

### The counters (measured 2026-06-11)

| Counter | C rotor | Rust rotor |
|---|---:|---:|
| node creations (question opportunities) | 3,171,632 | 159,018 |
| dedup question actually asked | 9,976 | 159,018 (always) |
| hits — answer "yes, already exists" | 6,021 | 48,114 |
| basic operations spent answering | **11,695,232,963** list comparisons | **159,018** hash probes |
| unique lines/nodes in the output | 138,820 (written; unreachable pruned at print) | 110,904 |

Self-consistency check (each counter is checked against a value the tool
already reports):
C: 3,171,632 calls − 6,021 reuses = 3,165,611 = exactly the "lines of model
formulae generated" the tool itself reports. Rust: 159,018 lookups − 48,114
hits = 110,904 = exactly the unique node count. Both tools' counters are
internally consistent.

### Findings

**1. The ballparks do NOT match — the sharing IS different.**
Differing ballparks alone already establish that the two tools share
subexpressions differently. The C rotor asks the dedup question only 9,976
times out of 3.17M creations — reuse is deliberately switched off in the
hot (loading) regions precisely because each question is so expensive. The
Rust rotor asks it on every single creation, because each question costs
one hash probe.

**2. The cost per question explains the entire wall-time difference.**
C spends 11.7 *billion* list comparisons to answer ~10k questions
(≈ 1.17 million comparisons per question — the line list is millions of
entries long by then). At a few ns per comparison this is on the order of
100+ seconds, which matches the measured 139 s wall time. The Rust rotor's
159,018 hash probes are microseconds in total; its measured wall time is
0.06–0.14 s, dominated by I/O.

**3. A second architectural difference shows up in the creation counts.**
C creates 3.17M line records and prunes unreachable ones when printing
(create-then-filter); the Rust generator never creates duplicates or dead
nodes in the first place (filter-at-creation), attempting only 159k
creations. This is also the memory story: 3.17M records ≈ 428 MB peak vs
~111k nodes ≈ 20 MB peak.

**4. Despite the different sharing strategies, the models are equivalent**
— same property at the same least-k on all 18 benchmarks under two
configurations (Part A above). The dedup strategy affects file size and
speed, never meaning (confirmed independently by the --no-cse runs).

### Comment-stripped size comparison

Generated with `-nocomments` (C, placed after `- 0`) and `--no-comments`
(Rust, flag added for this experiment):

| | with comments | without comments | comments share |
|---|---:|---:|---:|
| C rotor | 10,588,507 B / 138,820 lines | 5,210,809 B / 138,820 lines | **50.8%** |
| Rust rotor | 3,122,826 B / 110,904 lines | 2,966,913 B / 110,904 lines | 5.0% |

Raw-vs-raw ratio: 1.76x (was 3.4x with comments). The remaining gap is the
constant encoding (32/64-digit binary `const` strings vs decimal `constd`),
7-digit vs 5-digit nids, and the ~28k extra lines. Both stripped files pass
catbtor.

### Why the file sizes differ while the models contain the same things

BTOR2 file size is a *syntactic* artifact — the same content can be spelled
differently. Measured decomposition of the selfie self-models
(C: 138,820 lines / 10.6 MB; Rust: 110,904 lines / 3.1 MB):

| Factor | C rotor | Rust rotor |
|---|---|---|
| comment bytes | **5.38 MB** (50.8% of the file) | 0.16 MB (5.0%) |
| constant encoding | 32/64-digit **binary strings** (`const`, avg 65–83 chars/line) | decimal (`constd`, avg 24 chars/line) |
| nid magnitude | avg **7 digits** (structured namespaces) | avg 5 digits (sequential) |
| constant lines | 90,257 (dedup off in the loading section → ~33k duplicates) | 57,478 (dedup on globally) |
| avg line length | 75.3 chars | 27.2 chars |

~95% of both files is the same thing — the init chain writing selfie's
binary image into memory. One loaded instruction in C:

```
1000004 const 4 10000101100000101000001010010011 ; code 0x85828293
```

The Rust file writes the same word as a ~24-character decimal `constd` with
a short comment. Same meaning, a third of the characters. The statement
counts (138.8k vs 110.9k, only 1.25x apart) show the *content* is nearly
identical; the byte ratio (3.4x) is comments + binary-string constants +
longer nids. The equivalence results (Part A) are the proof that the
meaning is preserved: btormc returns the same verdict at the same bound
from both.

### Reproduce

- Rust counters are built in: run with `RUST_LOG=info`, the line
  `PROFILE: ... lookups, ... hits, ... unique nodes` prints at the end.
- C counters: the patch script is `benchmarks/cse-experiment/patch_c_counters.py`,
  applied to tools/rotor.c inside the cse-experiment container (adds 4 counters +
  prints next to the existing statistics line).

---

## C. Crash report: rotor aborts with "ite then sort mismatch error" when reuse_lines = 0

A complete, self-contained reproduction of a crash in the reference rotor
(upstream selfie, `tools/rotor.c`) that occurs when line reuse is disabled.

### Summary

Disabling line reuse globally (`reuse_lines = 0`) makes rotor abort while
generating any model. The sort-compatibility checks compare sort lines by
POINTER equality, which is only sound while line reuse maintains the
invariant "pointer equivalence iff structural equivalence". With reuse off,
two structurally identical sort lines are distinct pointers and the first
ITE construction fails its then-branch sort check.

### Reproduction (3 steps)

Environment: Ubuntu 24.04, gcc 13, current selfie main
(github.com/cksystemsteaching/selfie). A ready-made Dockerfile reproducing
this end-to-end is in this repository:
`benchmarks/cse-experiment/Dockerfile.cse`.

```bash
# 1. one-line change in tools/rotor.c (line ~215):
-  uint64_t reuse_lines = 1; // flag for reusing lines
+  uint64_t reuse_lines = 0; // flag for reusing lines

# 2. build as usual
make rotor

# 3. run on anything
./rotor -m64 -c selfie.c - 0
```

### Observed output

```
./rotor: selfie compiling selfie.c to 64-bit RISC-V using 64-bit starc
...
./rotor: ********************************************************************************
./rotor: ite then sort mismatch error
(exit status 14, ~0.08 s into model generation)
```

Expected (by analogy with the Rust rotor's --no-cse mode): a larger but
valid model.

### Root cause (file/line references, current main)

1. `new_line` documents the invariant the generator relies on
   (tools/rotor.c:3716): `// invariant: pointer equivalence iff structural
   equivalence`. The invariant is established by `find_equal_line` /
   `reuse_lines` — and silently broken when `reuse_lines = 0`, because
   every `new_bitvec`/`new_array` call then allocates a fresh sort line
   even for structurally identical sorts.

2. `match_sorts` (tools/rotor.c:4618) checks sort compatibility by pointer
   comparison:
   ```c
   void match_sorts(uint64_t* sid1, uint64_t* sid2, char* comment) {
     if (sid1 == sid2) ...
   ```

3. First failure site: `new_ternary` for ITE
   (tools/rotor.c:5089):
   ```c
   match_sorts(get_sid(line), get_sid(then_nid), "ite then");
   ```
   `get_sid(line)` and `get_sid(then_nid)` are two distinct
   `new_bitvec(64, ...)` lines — structurally equal, different pointers —
   so the check reports a sort mismatch that is not semantically there.

### Possible fixes (pick one)

- Make `match_sorts` structural: fall back to `are_lines_equal(sid1, sid2)`
  when the pointers differ (cost is negligible; sorts are tiny lines).
- Or: always reuse SORT lines regardless of `reuse_lines` (there are only a
  handful of distinct sorts; a dedicated small registry keeps the pointer
  invariant for sorts while letting everything else duplicate).

### Context

The crash was encountered during a comparison experiment in which the
duplicate check is disabled in both rotors and the outputs compared. The
Rust rotor with `--no-cse` produces a 1.43x larger model that catbtor
accepts and btormc gives identical verdicts on; the C rotor cannot complete
the same experiment because of this crash.
