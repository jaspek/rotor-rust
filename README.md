# Rotor (Rust)

[![CI](https://github.com/jaspek/rotor-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/jaspek/rotor-rust/actions/workflows/ci.yml)

> **University of Salzburg** — Advanced Systems Engineering
> Jasmin Begic & Daniel Wassie, supervised by Prof. Christoph Kirsch

**rotor-rust** translates RISC-V binaries into bit-precise
[BTOR2](https://link.springer.com/chapter/10.1007/978-3-319-96145-3_32)
models for bounded model checking. Point it at a compiled program and it
emits a transition system — every register, memory segment, and instruction
of the machine, plus 24 safety properties (division by zero, segmentation
faults, illegal instructions, a target exit code, …) — which a model
checker such as [btormc](https://github.com/Boolector/btor2tools) then
searches exhaustively, for **every possible input**, up to a chosen depth.

It is a Rust reimplementation of the `rotor` tool from the
[selfie project](https://github.com/cksystemsteaching/selfie), and adds
three things on top of the C reference:

- **Speed** — the model of selfie's own 43k-instruction self-compilation
  generates in ~0.1 s and 20 MB of memory, where the reference needs
  139 s and 428 MB; verified equivalent on the full benchmark suite, not
  approximated.
- **Symbolic command-line arguments** (`--symbolic-argv`) — argv bytes
  become part of the solver's search space, so bugs that depend on what a
  user types on the command line become findable.
- **A browser-based witness visualizer**
  ([try it online](https://jaspek.github.io/rotor-rust/)) — replays the
  solver's counterexample step by step, from input byte to fired property.

---

## Problem

Formal verification of low-level software relies on tools that translate a compiled program into a mathematical model, which a solver can then check against safety properties up to a bounded execution depth. In practice, three obstacles make this workflow hard to use.

First, the existing translator for RISC-V binaries is a large, monolithic C program — difficult to read, extend, or reason about, and awkward to integrate with modern toolchains. Second, the generated models only let a solver explore inputs the program reads from standard input; bugs that depend on **command-line arguments** are structurally unreachable, even though a real operating system would expose those bytes to the program. Third, when the solver does find a counterexample, its output is a flat textual trace that is effectively unreadable without intimate knowledge of the model format — the result, however correct, is inaccessible to anyone who did not build the tool.

Taken together, these three obstacles limit who can use bounded model checking on real binaries, what bugs it can find, and what a user can do with the answer once they have it.

The three obstacles are addressed in three parts:

1. **The translator is re-implemented in Rust**, replacing the monolithic C codebase with a modular crate that is easier to maintain, extend, and audit.
2. **The generated model is extended so that command-line arguments can be left symbolic**, letting the solver search over them instead of over stdin alone.
3. **A browser-based visualizer** takes the solver's counterexample and shows, step by step, which instruction fires and which memory or register state changes — so the verification result becomes something a non-expert can actually read.

Code for each part lives in its own subdirectory: `rotor/`,
`benchmarks/argv-tests/`, and `visualizer/`.

---

## Install

Prerequisites:

- [Rust](https://rustup.rs/) (stable) — builds the generator; nothing else
  is needed for model generation itself.
- [Docker](https://www.docker.com/) — optional; only needed to *check* the
  generated models (btormc/catbtor run in a container) and to compile your
  own test programs with selfie.

```bash
git clone https://github.com/jaspek/rotor-rust.git
cd rotor-rust

# Build the generator (takes seconds)
cargo build --release
```

The binary lands at `target/release/rotor` (`rotor.exe` on Windows).

To also run the model checker, build the checker image once — btormc and
catbtor compiled from the official Boolector sources:

```bash
docker build -t btormc -f benchmarks/Dockerfile.btormc .
```

---

## Usage

### Generate a model

The repository ships pre-compiled RISC-V binaries, so a first model needs
nothing but the build above:

```bash
./target/release/rotor benchmarks/binaries/division-by-zero-3-35.m \
    --xlen x64 --bytes-to-read 1 --heap 2048 --stack 2048 -o model.btor2
```

### Find the bug

```bash
# Well-formedness check (catbtor is the official BTOR2 validator)
docker run --rm -v "$PWD:/w" btormc -c "catbtor /w/model.btor2"

# Bounded model checking
docker run --rm -v "$PWD:/w" btormc -c "btormc -v 1 -kmax 100 /w/model.btor2"
#   -> "bad state property 7 reachable at bound k = 76 SATISFIABLE"
#      some input byte reaches the division by zero after exactly
#      76 machine instructions; the witness shows which byte.
```

### Watch the counterexample

```bash
docker run --rm -v "$PWD:/w" btormc \
    -c "btormc --trace-gen-full -kmax 100 /w/model.btor2" > model.wit
```

Open the [visualizer](https://jaspek.github.io/rotor-rust/) (or serve
`visualizer/` locally) and drag `model.btor2` + `model.wit` into the
window — the trace plays back step by step.

### Symbolic command-line arguments

The five programs in `benchmarks/argv-tests/` contain bugs reachable
**only** through argv — no exploration of standard input can trigger them:

```bash
# test1 exits with code 1 iff argv[1][0] == 'C'
./target/release/rotor benchmarks/argv-tests/test1_crash_string.m \
    --xlen x64 --symbolic-argv --num-symbolic-args 1 --max-arglen 8 \
    --exit-code 1 -o argv.btor2
docker run --rm -v "$PWD:/w" btormc -c "btormc -kmax 100 /w/argv.btor2" | head -5
#   -> witness byte argv[1][0] = 01000011 = 'C'
```

| Program | Bug depends on |
|---------|----------------|
| `test1_crash_string.c` | an exact argument byte (`'C'`) |
| `test2_numeric_overflow.c` | an ordered two-byte pair parsed as a number |
| `test3_length_dependent.c` | the argument's *length* |
| `test4_multi_arg.c` | two different arguments simultaneously |
| `test5_checksum.c` | an arithmetic relation over the input bytes |

Without `--symbolic-argv` the stack is booted with a CONCRETE argv image
(argc=1, argv[0]=program name) exactly like the C reference boot loader —
so default-mode models match the C rotor's machine. Details and expected
witnesses per test: [benchmarks/argv-tests/README.md](benchmarks/argv-tests/README.md).

### All options

```bash
# Match the C reference settings (used by the equivalence harness)
rotor <binary.elf> --bytes-to-read 1 --heap 2048 --stack 2048 --exit-code 0

# RV32 mode
rotor <binary.elf> --xlen x32

# Target exit code: bad-exit-code fires on exit(N) (C rotor's "rotor ... - N")
rotor <binary.elf> --exit-code 1

# Symbolic command-line arguments
rotor <binary.elf> --symbolic-argv --num-symbolic-args 2 --max-arglen 8 --exit-code 1

# Disable common-subexpression elimination (duplicate-check experiment)
rotor <binary.elf> --no-cse

# Strip all comments from the output (C rotor's -nocomments)
rotor <binary.elf> --no-comments

# Code synthesis mode (EXPERIMENTAL stub — code segment is not yet symbolic)
rotor --synthesis -o model.btor2
```

Note: `bad-exit-code` follows the C reference semantics — it fires when the
program exits **with** the target exit code (`--exit-code N`, default 0), not
on any non-zero exit. For the argv benchmarks (which exit(1) on the bug) pass
`--exit-code 1`.

### Model your own program

The committed `benchmarks/binaries/*.m` cover the standard suite. To model
your **own** C\* program, compile it with selfie inside the container (the
image's batch entrypoint must be bypassed):

```bash
cd benchmarks
docker build -t selfie .

# Compile a C program with selfie (builds selfie on first use)
docker run --rm --entrypoint /bin/sh -v "$(pwd):/work" selfie \
  -c "make -C /selfie selfie && /selfie/selfie -c /work/test.c -o /work/test.m"

# Generate BTOR2 with Rust Rotor (host side — note the HOST path)
cd .. && cargo run --release -- benchmarks/test.m -o model.btor2
```

Then validate and check exactly as above.

---

## Visualizer

An interactive web-based graph viewer for BTOR2 models with witness trace
animation. **[Try it online](https://jaspek.github.io/rotor-rust/)** — no
installation needed.

- **Graph visualization** with Cytoscape.js: nodes shaped by role — octagon
  (bad state), diamond (constant), barrel (input), pentagon (memory),
  hexagon (constraint) — in hierarchical (dagre) or force-directed layouts
- **Focus tools** for large models: cone-of-influence subgraph per property,
  depth-limited exploration, double-click subtree collapse, category
  clumping, longest-path highlighting, search by ID/operation/name
- **Witness playback**: step-by-step animation of btormc counterexamples
  with a timeline scrubber; play/pause, step, jump, adjustable speed
- **Comfort**: drag & drop any `.btor2`/`.wit` file, keyboard shortcuts
  (Space play/pause, arrows step, Home/End jump, F fit, +/− zoom, / search,
  ? help), PNG/SVG export, one-click examples for first-time users

To run it locally:

```bash
cd visualizer
python -m http.server 8080     # then open http://localhost:8080
```

Note: the graph libraries are loaded from unpkg.com, so an internet
connection is required even when serving locally.

**Bundled examples** (see `visualizer/examples/manifest.json`) — the
dropdown groups 12 examples into three categories:

| Category | Examples | Notes |
|---|---|---|
| **Symbolic argv (Rust Rotor)** | `argv_test1_crash_string` · `argv_test2_numeric_overflow` · `argv_test3_length_dependent` · `argv_test4_multi_arg` · `argv_test5_checksum` | All 5 have SAT witnesses — btormc finds the specific argv bytes that drive the program into a bad state. |
| **Standard selfie benchmarks** | `bench_division_by_zero` · `bench_simple_if_else` · `bench_recursive_fibonacci` · `bench_memory_access_fail` | Models only (no symbolic input). Useful for inspecting the graph structure. |
| **Tiny exploration** | `simple-assignment-1-35` · `counter-with-input` · `tiny-counter` | Small models for quickly trying the layout, witness playback, and view options. |

A witness for any model can be produced with
`btormc --trace-gen-full -kmax 100 model.btor2 > trace.wit` and loaded via
"Load Trace" or drag & drop.

---

## Features

- **RISC-V support**: RV32I/RV64I base integer ISA, M extension
  (multiply/divide), C extension (compressed instructions)
- **Multi-core**: configurable number of cores
- **Segmented memory model**: code (read-only), data, heap, and stack segments
- **Kernel syscall modeling**: `exit`, `read`, `write`, `openat`, `brk`
- **Safety properties**: 24 bad-state properties matching the C reference —
  target exit code, division by zero, signed-division overflow,
  illegal-instruction (full + compressed + known-instructions), fetch
  (invalid-address, unaligned, seg-fault), load/store (invalid-address +
  seg-fault, plus compressed variants), stack-pointer (invalid-address +
  seg-fault), unknown-syscall-ID, syscall-arg seg-faults
  (brk/openat/read/write) — the full list in emission order is Appendix A
  of the paper
- **HashMap-based CSE**: O(1) common-subexpression elimination on every
  node; duplicates are never even allocated
- **Arena allocation**: cache-friendly node storage with stable indices
- **Deterministic output**: same binary + same flags = byte-identical model,
  across runs and across platforms (enforced by CI)

---

## Results

| Part | Scope | Status |
|------|-------|:------:|
| 1 | Rust rewrite of the translator | **Complete — verified equivalent on all 18 standard benchmarks under TWO configurations (36/36 paired verdicts)**: same 24 bad-state properties as the C reference by name, index, and ported predicate; btormc fires the **same property at the same least bound k** from both rotors' models on every benchmark at kmax=1500, for target exit code 0 and 1 alike. Selfie self-model generates in ~0.1 s / 20 MB (vs 139 s / 428 MB for C, binary-only). Evidence: [docs/VERIFICATION.md](docs/VERIFICATION.md), `benchmarks/deep_equivalence_results*.csv`. |
| 2 | Symbolic argv support | **Complete** — 5 benchmark programs, each with a bug reachable *only* via argv, are discovered by btormc within seconds. |
| 3 | Witness-trace visualizer | **Complete** — example picker (12 examples), witness playback with timeline scrubber, drag & drop loading, keyboard shortcuts, full symbolic-input display; [live online](https://jaspek.github.io/rotor-rust/). |

Deliverables (slides, reports, and the full course paper) are published on
the [GitHub releases page](../../releases) so the repository stays free of
large binary artefacts.

### Performance

Re-measured 2026-06-10, strictly apples-to-apples: both rotors consume the
**pre-compiled selfie.m binary** (selfie compiled into a RISC-U binary of
itself, ~43k instructions; C: `rotor -m64 -l selfie.m - 0` under
`/usr/bin/time -v` in the container; Rust: 3 runs, wall clock + polled peak
working set):

| Metric | C Rotor (reference) | Rust Rotor | Ratio |
|---|---:|---:|---:|
| Wall-clock model generation | 139 s | **0.06–0.14 s** | ~1,000–2,000× faster |
| Peak memory | 428 MB | **20 MB** | ~21× less |
| Internal formula lines created | 3,165,611 | ~111k (duplicates never created) | 28× |
| Output BTOR2 size | 10.6 MB | **3.1 MB** | 3.4× smaller |
| btormc validation (`catbtor` + `-kmax 0`) | PASS | **PASS** | — |

**Why is it so much faster? (measured, not estimated)** The dedup question
— *"is this subexpression already in the system?"* — was instrumented with
plain counters in BOTH tools (full data:
[docs/VERIFICATION.md](docs/VERIFICATION.md)). C answers it by
walking a linear list: **11,695,232,963 comparisons** for just 9,976
questions (~1.17M comparisons each; reuse is deliberately disabled in its
hot loading path because the question is so expensive). The Rust rotor asks
the question on every creation — 159,018 times — at **one hash probe each**.
11.7 billion comparisons at nanosecond scale ≈ 100+ s, matching the
measured 139 s wall time. Counters are self-consistent on both sides
(calls − hits = lines generated, exactly).

**Why so much less memory?** The creation counters tell the story: C calls
`new_line` **3,171,632 times** and keeps every record in memory, pruning
unreachable lines only when printing (create-then-filter) — at ~135 bytes
per record that is ≈428 MB. The Rust rotor attempts only **159,018**
creations because the hash lookup happens *before* allocation; duplicates
and dead nodes never exist (filter-at-creation), leaving ~111k unique
nodes ≈ 20 MB.

The output *files* differ in size while encoding the same content — the
difference is comments (50.8% of the C file vs 5.0%), binary-string vs
decimal constants, and node-id width; the decomposition is in
[docs/VERIFICATION.md](docs/VERIFICATION.md). The hash-consing
optimisation was also ported back into the reference C with byte-identical
output (139 s → 1.5 s, ~93×): see
[rotor-c-hashcons](https://github.com/jaspek/rotor-c-hashcons).

### Equivalence

A rewrite of a verification tool must itself be validated. The criterion is
behavioural and uses the model checker as an impartial referee: run
`btormc -kmax 1500` on both rotors' models of the same binary and require
the **same bad-state property index** to fire at the **same least bound k**
(or both UNSAT through the bound limit).

**Final result: 18/18 benchmarks equivalent under BOTH tested
configurations (target exit code 0 and 1) — 36/36 paired verdicts, zero
divergences.** Under target 1, five planted bugs fire at k = 91..107 with
identical k in both rotors, and return-from-loop FLIPS from UNSAT to SAT@95
— identically in both. Full tables, methodology, and the raw CSVs:
[docs/VERIFICATION.md](docs/VERIFICATION.md),
`benchmarks/deep_equivalence_results*.csv`.

Reaching this required making the machine model faithful to the C reference
piece by piece — zero-initialized memory and registers, page-aligned heap,
full 4 GB stack at `[0xFFFFF800, 2^32)`, the concrete argc/argv boot image,
the real read-syscall semantics (one input byte per transition with the PC
stalled), file-descriptor state, and all 24 safety properties ported
predicate-by-predicate from `rotor.c` in the C output's exact emission
order. Each step was verified with catbtor + btormc before the next.

| Aspect | C Rotor | Rust Rotor |
|---|---|---|
| **Bad-state properties** | 24 | **24** — same names, same order, same predicates |
| **Deduplication algorithm** | linear list scan, O(N²) total | HashMap lookup, O(N) total |
| **Dedup disabled** (dedup-off experiment) | **crashes** (`ite then sort mismatch`, exit 14) — the C generator relies on line reuse for node identity; see the crash report in [docs/VERIFICATION.md](docs/VERIFICATION.md) | works (`--no-cse`): models grow ~1.43× and stay catbtor-valid |
| **Initialization encoding** | Unfolded over time steps (`zeroed-*` → `loaded-*`) | Direct `init` statements from binary data |

The dedup experiment confirms the speed difference is purely the data
structure (constant-time lookups vs linear scans over millions of checks),
not a difference in what gets deduplicated — and that dedup is semantically
neutral in the Rust implementation.

### Reproducing the results

```bash
# The full equivalence table (PowerShell harness; hours — on Linux/macOS
# install pwsh; results are committed as the two
# deep_equivalence_results*.csv files)
cd benchmarks && ./run_deep_equivalence.ps1 -Kmax 1500

# The test suite (includes a golden test: the generated division model
# must be byte-identical to the campaign-verified artifact)
cargo test --release
```

CI repeats the pipeline on every push to master: build, generate all 18
models from the committed binaries, catbtor each, replay a witness with
btorsim, and enforce the byte-identity regression gate.

---

## Architecture

```
rotor/src/
  main.rs              CLI entry point (clap)
  lib.rs               Public API re-exports
  config.rs            Config: RV32/64, M/C extensions, property checks
  btor2/
    builder.rs         BTOR2 IR builder with HashMap CSE
    node.rs            NodeId, Op enum, BinaryOp, Node
    sort.rs            Sort enum (Bitvec, Array)
    printer.rs         BTOR2 text output (topological order)
  riscv/
    isa.rs             InstrId enum, opcode/funct constants
    elf_loader.rs      ELF loading via goblin
    decode.rs          RV64I/RV32I + M instruction decode
    compressed.rs      RVC compressed instruction decode
  machine/
    sorts.rs           MachineSorts + MachineConstants
    registers.rs       Register file model (32 regs)
    memory.rs          Segmented memory (code/data/heap/stack)
    segmentation.rs    Segment bounds, address translation
    kernel.rs          Kernel state (syscalls, brk, I/O)
    core.rs            Per-core state (PC, IR)
  model/
    combinational.rs   Instruction semantics (data flow + control flow)
    sequential.rs      Next-state logic (PC, regs, memory)
    properties.rs      Bad states (exit!=0, div-by-0, seg faults)
    generator.rs       Top-level model generation pipeline
```

The generation pipeline: **load** the ELF binary (code + data segments) →
**initialize** BTOR2 sorts and machine constants → **create** per-core
state (PC, registers, memory, kernel) → **generate** combinational logic
(fetch, decode, ALU, control flow) → **generate** sequential logic
(next-state for PC, registers, memory) → **generate** safety properties
(bad states) → **print** the BTOR2 model.

---

## Benchmarks directory

Pre-generated BTOR2 models for the 18 selfie test programs, the
verification harnesses, and the committed campaign results:

```
benchmarks/
  btor2-c-rotor/               C Rotor reference output (committed)
  binaries/                    Compiled RISC-V binaries (.m format)
  Dockerfile                   Docker setup for selfie compilation
  Dockerfile.btormc            Docker setup for btormc model checker
  run_deep_equivalence.ps1     THE equivalence harness: btormc at kmax=1500
                               on both rotors, compares fired property + least-k
  parallel_runner.sh           same check parallelized across cores
  parallel_runner_exit1.sh     second configuration (target exit code 1)
  deep_equivalence_results.csv         final 18/18 table (exit code 0)
  deep_equivalence_results_exit1.csv   final 18/18 table (exit code 1)
  btor2-c-rotor-exit1/         C reference models regenerated with "- 1"
  cse-experiment/              dedup-off artifacts: model pair, Dockerfile
                               reproducing the C crash, counter patch script
  argv-tests/                  the five symbolic-argv test programs (+ README)
  btor2-rust-rotor-exit1/      Rust models under target exit code 1
  compile.sh / compile-argv.sh helper scripts used inside the selfie image
  eqv_combined_sweep.sh        combined equivalence sweep helper
  parallel_results/            raw per-run outputs (exit-code-0 campaign)
  parallel_results_exit1/      raw per-run outputs (exit-code-1 campaign)
  run_equivalence_check.ps1    older shallow harness (kept for reference)
```

---

## Dependencies

Rust crates (`rotor/`):

| Crate | Purpose |
|-------|---------|
| `goblin` | ELF binary parsing |
| `clap` | CLI argument parsing |
| `thiserror` | Error types |
| `log` + `env_logger` | Debug logging |

Visualizer (loaded from CDN):

| Library | Purpose |
|---------|---------|
| [Cytoscape.js](https://js.cytoscape.org/) | Graph rendering and interaction |
| [dagre](https://github.com/dagrejs/dagre) | Hierarchical graph layout |
| [cytoscape-dagre](https://github.com/cytoscape/cytoscape.js-dagre) | Cytoscape-dagre integration |
| [cytoscape-svg](https://github.com/kinimesi/cytoscape-svg) | SVG graph export |
| [Google Fonts](https://fonts.google.com/) (Inter, JetBrains Mono) | Typography |

---

## References

- [Selfie project](https://github.com/cksystemsteaching/selfie) — Original C implementation
- [BTOR2 format](https://link.springer.com/chapter/10.1007/978-3-319-96145-3_32) — BTOR2, BtorMC and Boolector 3.0
- [BiTR](https://github.com/cksystemsgroup/bitr) — Related work on agent-based bounded model checking (formerly agent-bitr)
- Diller (2022) — *Visualizing BTOR2 Models* (thesis, inspiration for visualizer features)

## License

This repository is licensed under the [BSD 2-Clause License](LICENSE).
The modeled machine semantics derive from the
[selfie project](https://github.com/cksystemsteaching/selfie), itself
BSD-2-Clause licensed.
