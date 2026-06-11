# Profiling the speed difference by counting (the prescribed methodology)

Method from the 2026-06-11 meeting: define the basic operation — the
question *"is this subexpression already in the system?"* — find the code
location answering it in both tools, count invocations and hits with plain
integer counters, check the ballparks, then reason about timing.

## Instrumentation points

| | C rotor | Rust rotor |
|---|---|---|
| creation entry | `new_line` (tools/rotor.c:3712) | `Btor2Builder::intern` (rotor/src/btor2/builder.rs) |
| the question | `find_equal_line` (tools/rotor.c:3697), linear walk calling `are_lines_equal` per node | `HashMap::get` — one probe |
| counters added | `profile_new_line_calls`, `profile_find_calls`, `profile_comparisons`, `profile_hits` | `profile_lookups`, `profile_hits` |

Both instrumented builds run on the same input: selfie.m (43,406
instructions), 64-bit, 1 input byte, heap/stack 2048.

## The counters (measured 2026-06-11)

| Counter | C rotor | Rust rotor |
|---|---:|---:|
| node creations (question opportunities) | 3,171,632 | 159,018 |
| dedup question actually asked | 9,976 | 159,018 (always) |
| hits — answer "yes, already exists" | 6,021 | 48,114 |
| basic operations spent answering | **11,695,232,963** list comparisons | **159,018** hash probes |
| unique lines/nodes in the output | 138,820 (written; unreachable pruned at print) | 110,904 |

Self-consistency check (the "conjecture on what the counter should be"):
C: 3,171,632 calls − 6,021 reuses = 3,165,611 = exactly the "lines of model
formulae generated" the tool itself reports. Rust: 159,018 lookups − 48,114
hits = 110,904 = exactly the unique node count. Both tools' counters are
internally consistent.

## Findings

**1. The ballparks do NOT match — the sharing IS different.**
Exactly the branch the methodology anticipates ("if they are not in the
same ballpark then you already know the sharing is different"). The C rotor
asks the dedup question only 9,976 times out of 3.17M creations — reuse is
deliberately switched off in the hot (loading) regions precisely because
each question is so expensive. The Rust rotor asks it on every single
creation, because each question costs one hash probe.

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
configurations (see P2_RESULTS.md). The dedup strategy affects file size
and speed, never meaning (confirmed independently by the --no-cse runs).

## Comment-stripped size comparison (also requested)

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

## Reproduce

- Rust counters are now built in: run with `RUST_LOG=info`, the line
  `PROFILE: ... lookups, ... hits, ... unique nodes` prints at the end.
- C counters: the patch script is `benchmarks/cse-experiment/patch_c_counters.py`,
  applied to tools/rotor.c inside the cse-experiment container (adds 4 counters +
  prints next to the existing statistics line).
