# P2 results: deep equivalence + the CSE-off experiment

## Part 1 — The professor's CSE-off experiment ("see if the universe ends")

Asked in the 2026-05-26 meeting: *"In C rotor you disable the duplicate
check, and in your tool as well. And then you see what comes up. Just
disable it and see if the universe ends or not."*

### C rotor (reuse_lines = 0, set globally, rebuilt from source)

**The universe ends.** C rotor with the duplicate check disabled globally
**crashes** while modeling selfie:

```
/rotor-cse-off: ite then sort mismatch error
Exit status: 14   (0.08 s into the run)
```

The C implementation depends on line reuse for node identity: with reuse
off, sort lines get duplicated, an ITE's branches end up referencing
different (but equal) sort instances, and rotor's internal sort check
aborts. The duplicate check in C is not only a size optimization — parts of
the generator are correct only because identical lines are reused.

Baseline C run for reference (reuse ON, selfie.c, same container):
wall 2:02.8, peak memory 432 MB, model 138,820 lines (10.6 MB),
"3,165,611 lines of model formulae generated" before reuse/pruning.

### Rust rotor (--no-cse flag)

**The universe survives.** Models grow but stay correct, because every node
carries its sort as an explicit parameter — branch sort-consistency does not
depend on deduplication:

| Input | CSE on | CSE off | Growth | Valid? |
|---|---|---|---|---|
| division-by-zero-3-35.m | 1,693 lines (52 KB) | 2,433 lines (74 KB) | 1.44x | catbtor PASS |
| selfie.m (43k instructions) | 110,904 lines (3.1 MB) | 159,018 lines (5.5 MB) | 1.43x | catbtor PASS |

The modest 1.43x growth confirms the C-side observation that the bulk of a
binary-initialized model (the init chains) has little reuse to exploit; the
dedup matters most in the instruction-semantics logic.

### Conclusion for the meeting

Dedup is semantically neutral in the Rust implementation (verified: same
btormc verdict with and without). The speed difference between the rotors is
the data structure (HashMap O(1) vs list walk O(N) per lookup), not the
amount of dedup performed: C rotor cannot even run with reuse globally off,
while Rust rotor runs fine either way.

## Part 2 — Deep equivalence across the 18 standard benchmarks

Method: for each benchmark, generate the Rust model with C-matching
parameters (`--bytes-to-read 1 --heap 2048 --stack 2048 --exit-code 0`),
then run `btormc -kmax 1500` (covering the paper's reported 66..1438 bug
range) on BOTH the C reference model and the Rust model, and compare the
**fired property index** and the **least bound k**.

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

### Methodology note: the harness caught itself once

During the parallel run, three-level-nested-loop-fail briefly looked like a
divergence (C "UNSAT", Rust SAT@103). Investigation showed the C-side btormc
container had died under 9-way memory pressure and the runner recorded the
empty output as UNSAT — a harness bug, not a rotor bug. Re-run alone, C
fires bad-exit-code @ 103, identical to Rust. The runner now distinguishes
"btormc completed, nothing reachable" from "no output" (recorded as ERROR),
and all UNSAT rows were re-verified with completion evidence.

## Part 3 — Second configuration: target exit code 1 (paper's planted bugs)

The rotor paper's experiments hunt "non-zero exit code" bad states. We
re-ran the entire suite with both rotors at target exit code 1 (C:
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

## Reproducers

```powershell
# the deep harness
cd benchmarks; .\run_deep_equivalence.ps1 -Kmax 1500

# Rust CSE off
.\target\release\rotor.exe <bin.m> --xlen x64 --no-cse -o model.btor2

# C CSE off (crashes)
docker build -t selfie-cse -f C:\Users\jasko\Programming\Rust\Selfie\Dockerfile.cse .
docker run --rm selfie-cse
```
