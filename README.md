# Rotor (Rust)

[![CI](https://github.com/jaspek/rotor-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/jaspek/rotor-rust/actions/workflows/ci.yml)

> **University of Salzburg** — Advanced Systems Engineering
> Jasmin Begic & Daniel Wassie, supervised by Prof. Christoph Kirsch

---

## Problem

Formal verification of low-level software relies on tools that translate a compiled program into a mathematical model, which a solver can then check against safety properties up to a bounded execution depth. In practice, three obstacles make this workflow hard to use.

First, the existing translator for RISC-V binaries is a large, monolithic C program — difficult to read, extend, or reason about, and awkward to integrate with modern toolchains. Second, the generated models only let a solver explore inputs the program reads from standard input; bugs that depend on **command-line arguments** are structurally unreachable, even though a real operating system would expose those bytes to the program. Third, when the solver does find a counterexample, its output is a flat textual trace that is effectively unreadable without intimate knowledge of the model format — the result, however correct, is inaccessible to anyone who did not build the tool.

Taken together, these three obstacles limit who can use bounded model checking on real binaries, what bugs it can find, and what a user can do with the answer once they have it.

## Approach

We address the three obstacles in three parts:

1. We **re-implement the translator in Rust**, replacing the monolithic C codebase with a modular crate that is easier to maintain, extend, and audit.
2. We **extend the generated model so that command-line arguments can be left symbolic**, letting the solver search over them instead of over stdin alone.
3. We **build a browser-based visualizer** that takes the solver's counterexample and shows, step by step, which instruction fires and which memory or register state changes — so the verification result becomes something a non-expert can actually read.

## Status

| Part | Scope | Status |
|------|-------|:------:|
| 1 | Rust rewrite of the translator | **Complete — verified equivalent on all 18 standard benchmarks (18/18)**: same 24 bad-state properties as the C reference by name, index, and ported predicate; btormc fires the **same property at the same least bound k** from both rotors' models on every benchmark at kmax=1500. Selfie self-model generates in ~0.1 s / 20 MB (vs 139 s / 428 MB for C, binary-only, re-measured). Evidence: `P2_RESULTS.md`, `benchmarks/deep_equivalence_results.csv`. |
| 2 | Symbolic argv support | **Complete** — 5 benchmark programs, each with a bug reachable *only* via argv, are discovered by btormc within seconds. |
| 3 | Witness-trace visualizer | **Complete** — redesigned browser tool: example picker (12 examples), witness playback with timeline scrubber, drag & drop loading, keyboard shortcuts, full symbolic-input display; [live online](https://jaspek.github.io/rotor-rust/). |

Code for each part lives in its own subdirectory: `rotor/`, `benchmarks/argv-tests/`, and `visualizer/`.

**Semantic-equivalence campaign** (in response to supervisor feedback — *"check
them yourself"*): the machine model was made faithful to the C reference piece
by piece — zero-initialized memory and registers, page-aligned heap and full
4 GB stack at `[0xFFFFF800, 2^32)`, concrete argc/argv boot image, the real
read-syscall semantics (one input byte per transition with the PC stalled),
file-descriptor state, and all 24 safety properties re-ported from `rotor.c`
in the C output's exact emission order. Each step was verified with catbtor +
btormc before the next. The deep harness (`benchmarks/run_deep_equivalence.ps1`,
parallelized across cores by `benchmarks/parallel_runner.sh`) compares the
fired property and least-k on both rotors at kmax=1500. **Final result:
18/18 benchmarks equivalent** (`benchmarks/deep_equivalence_results.csv`).

Deliverables (slides, reports, and the full course paper) are published on the [GitHub releases page](../../releases) so the repository stays free of large binary artefacts. Generator scripts for the slide decks live under `presentations/scripts/`.

---

## Technical details

The sections below describe each part in more depth. Readers who only want the high-level picture can stop here.

### Part 1: Rotor in Rust

#### Features

- **RISC-V support**: RV32I/RV64I base integer ISA, M extension (multiply/divide), C extension (compressed instructions)
- **Multi-core**: Configurable number of cores
- **Segmented memory model**: Code (read-only), data, heap, and stack segments
- **Kernel syscall modeling**: `exit`, `read`, `write`, `openat`, `brk`
- **Safety properties**: 24 bad-state properties matching the C reference — bad/good/any exit, division by zero, signed-division overflow, illegal-instruction (full + compressed + known-instructions), fetch (invalid-address, unaligned, seg-fault), load/store (invalid-address + seg-fault, plus compressed variants), stack-pointer (invalid-address + seg-fault), unknown-syscall-ID, syscall-arg seg-faults (brk/openat/read/write)
- **HashMap-based CSE**: O(1) common subexpression elimination on every node (vs O(n) per node in the C original, which had to be turned off for the binary-loading section to stay tractable). On selfie's self-compiled binary (~43k RISC-U instructions, ~110k BTOR2 nodes), model generation takes 47 ms in 20 MB peak memory — vs 106 s in 431 MB for the C reference on the same input
- **Arena allocation**: Cache-friendly node storage with stable indices

#### Building

```bash
cd rotor
cargo build --release
```

#### Usage

```bash
# Generate BTOR2 model from a RISC-V ELF binary
rotor <binary.elf> -o model.btor2

# Match the C reference settings (used by the equivalence harness)
rotor <binary.elf> --bytes-to-read 1 --heap 2048 --stack 2048 --exit-code 0

# RV32 mode
rotor <binary.elf> --xlen x32

# Target exit code: bad-exit-code fires on exit(N) (C rotor's "rotor ... - N")
rotor <binary.elf> --exit-code 1

# Symbolic command-line arguments (see Part 2)
rotor <binary.elf> --symbolic-argv --num-symbolic-args 2 --max-arglen 8 --exit-code 1

# Disable common-subexpression elimination (duplicate-check experiment)
rotor <binary.elf> --no-cse

# Code synthesis mode (symbolic code, no binary)
rotor --synthesis -o model.btor2
```

Note: `bad-exit-code` follows the C reference semantics — it fires when the
program exits **with** the target exit code (`--exit-code N`, default 0), not
on any non-zero exit. For the argv benchmarks (which exit(1) on the bug) pass
`--exit-code 1`.

#### Architecture

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

#### Pipeline

1. **Load** ELF binary (code + data segments)
2. **Initialize** BTOR2 sorts and machine constants
3. **Create** per-core state (PC, registers, memory, kernel)
4. **Generate** combinational logic (fetch, decode, ALU, control flow)
5. **Generate** sequential logic (next-state for PC, registers, memory)
6. **Generate** safety properties (bad states)
7. **Print** BTOR2 model

### Part 2: Symbolic argv

Extends Rotor to support verification of programs with symbolic command-line arguments. Instead of concrete input values, `argv` entries are modeled as unconstrained symbolic bitvectors, allowing the model checker to explore all possible inputs.

#### Test Programs

Five C test programs in `benchmarks/argv-tests/` exercise different input-dependent behaviors:

| Program | What it tests |
|---------|---------------|
| `test1_crash_string.c` | String comparison triggering a crash |
| `test2_numeric_overflow.c` | Integer overflow from parsed input |
| `test3_length_dependent.c` | Behavior dependent on argument length |
| `test4_multi_arg.c` | Multiple argument interaction |
| `test5_checksum.c` | Checksum computation over input bytes |

#### Generating argv Models

```bash
# Compile with selfie, then generate BTOR2 with symbolic argv.
# The argv test programs exit(1) when the bug input is found, so the
# target exit code is 1 (see the bad-exit-code note in Part 1).
rotor <binary.elf> --symbolic-argv --num-symbolic-args 2 --max-arglen 8 \
      --exit-code 1 -o model-argv.btor2
```

Without `--symbolic-argv` the stack is booted with a CONCRETE argv image
(argc=1, argv[0]=program name) exactly like the C reference boot loader —
so default-mode models match the C rotor's machine.

### Part 3: BTOR2 Visualizer

An interactive web-based graph viewer for BTOR2 hardware models with witness trace animation.

#### Features

- **Graph visualization**: Renders BTOR2 models as interactive node graphs using Cytoscape.js
- **Dual layout modes**: Hierarchical (dagre) and force-directed (cose) layouts
- **Subgraph views**: View the cone of influence for any bad property or state node
- **Depth-limited exploration**: Slider to control how deep into the dependency tree to display
- **Node collapse/expand**: Double-click nodes to collapse their subtrees
- **Category clumping**: Group logic, state, memory, or constant nodes into single meta-nodes
- **Longest path highlighting**: Visualize the critical path through the model
- **Witness trace animation**: Step-by-step playback of btormc counterexample traces, with a **timeline scrubber** for jumping to any step
- **Drag & drop**: Drop a `.btor2` model or `.wit` witness anywhere in the window
- **Keyboard shortcuts**: Space (play/pause), arrows (step), Home/End (jump), F (fit), +/− (zoom), / (search), ? (help overlay)
- **Toasts & empty-state**: Load notifications, and a hero screen with one-click examples for first-time users
- **Export**: PNG and SVG graph export
- **Search**: Find nodes by ID, operation, or name
- **Node shapes by category**: Octagon (bad), diamond (constant), barrel (input), pentagon (memory), hexagon (constraint)

#### Live Demo

**[Try the visualizer online](https://jaspek.github.io/rotor-rust/)** — no installation needed.

#### Running Locally

```bash
# Serve the visualizer directory with any HTTP server
cd visualizer
python -m http.server 8080

# Then open http://localhost:8080 in your browser
```

#### Loading Models

- **Upload**: Click "Upload" to load a `.btor2` file from disk
- **Paste**: Click "Paste" to paste BTOR2 text directly
- **Example dropdown**: Pick one of 12 bundled examples (5 symbolic-argv tests with witnesses, 4 standard selfie benchmarks, 3 tiny exploration examples). Picking an entry loads both the model and its witness in one click.

#### Witness Trace Playback

The visualizer can animate counterexample witness traces produced by [btormc](https://github.com/Boolector/btor2tools):

```bash
# Generate a witness trace with btormc
btormc --trace-gen-full -kmax 100 model.btor2 > trace.wit

# Or via Docker
docker run --rm --entrypoint /bin/bash \
  -v "$(pwd):/work" btormc \
  -c "btormc --trace-gen-full -kmax 100 /work/model.btor2"
```

Then load the `.wit` file in the visualizer using "Load Trace" or click "Example" in the Witness Trace section for a demo.

**Playback controls**: Play/pause, step forward/back, jump to start/end, adjustable speed. Keyboard shortcuts: Arrow keys (step), Space (play/pause), Home/End (jump).

#### Bundled examples

Listed in `visualizer/examples/manifest.json`. The dropdown groups them into three categories:

| Category | Examples | Notes |
|---|---|---|
| **Symbolic argv (Rust Rotor)** | `argv_test1_crash_string` · `argv_test2_numeric_overflow` · `argv_test3_length_dependent` · `argv_test4_multi_arg` · `argv_test5_checksum` | All 5 have SAT witnesses — btormc finds the specific argv bytes that drive the program into a bad state. |
| **Standard selfie benchmarks** | `bench_division_by_zero` · `bench_simple_if_else` · `bench_recursive_fibonacci` · `bench_memory_access_fail` | Models only (no symbolic input). Useful for inspecting the graph structure. |
| **Tiny exploration** | `simple-assignment-1-35` · `counter-with-input` · `tiny-counter` | Small models for quickly trying the layout, witness playback, and view options. |

### Benchmarks

Pre-generated BTOR2 models for 17+ selfie test programs:

```
benchmarks/
  btor2-c-rotor/               C Rotor reference output (committed)
  binaries/                    Compiled RISC-V binaries (.m format)
  Dockerfile                   Docker setup for selfie compilation
  Dockerfile.btormc            Docker setup for btormc model checker
  run_deep_equivalence.ps1     THE equivalence harness: btormc at kmax=1500
                               on both rotors, compares fired property + least-k
  deep_equivalence_results.csv harness results (regenerated by the script)
  run_equivalence_check.ps1    older shallow harness (kept for reference)
```

#### Running Benchmarks

```bash
cd benchmarks

# Build Docker images
docker build -t selfie .
docker build -t btormc -f Dockerfile.btormc .

# Compile a C program with selfie
docker run --rm -v "$(pwd):/work" selfie \
  /opt/selfie/selfie -c /work/test.c -o /work/test.m

# Generate BTOR2 with Rust Rotor
cargo run --release -- /work/test.m -o model.btor2

# Verify with btormc
docker run --rm --entrypoint /bin/bash \
  -v "$(pwd):/work" btormc \
  -c "btormc -kmax 100 /work/model.btor2"
```

### Rust vs C Rotor: Model Comparison

Both implementations of Rotor generate valid BTOR2 models that btormc can verify. The Rust rewrite emits the **same 24 bad-state properties** as the C reference and produces semantically equivalent models — but generates them about three orders of magnitude faster and with twenty times less memory.

#### Selfie self-model (selfie compiled into a RISC-U binary of itself, ~43k instructions)

Re-measured 2026-06-10, strictly apples-to-apples: both rotors consume the
**pre-compiled selfie.m binary** (C: `rotor -m64 -l selfie.m - 0` under
`/usr/bin/time -v` in the container; Rust: 3 runs, wall clock + polled peak
working set):

| Metric | C Rotor (reference) | Rust Rotor | Ratio |
|---|---:|---:|---:|
| Wall-clock model generation | 139 s | **0.06–0.14 s** | ~1,000–2,000× faster |
| Peak memory | 428 MB | **20 MB** | ~21× less |
| Internal formula lines created | 3,165,611 | ~111k (duplicates never created) | 28× |
| Output BTOR2 size | 10.6 MB | **3.1 MB** | 3.4× smaller |
| btormc validation (`catbtor` + `-kmax 0`) | PASS | **PASS** | — |

**Why is it so much faster?** Both rotors deduplicate identical formula
lines. C checks each new line by walking a linear list of all previous
lines — O(N) per line, O(N²) overall; its own log shows **3,165,611 lines
generated**, so that is billions of comparisons. The Rust rotor performs
the *same syntactic check* with a hash map: one constant-time lookup per
node. Same work, different data structure — confirmed by the `--no-cse`
experiment above (dedup off changes size, never meaning).

**Why so much less memory?** C *creates* all 3.17 M line records in memory
and deduplicates afterwards — at ~135 bytes per record that is ≈428 MB.
The Rust rotor's hash lookup happens *before* allocation, so duplicates
never exist: only the ~111k unique nodes are ever stored (compact arena +
hash index ≈ 20 MB). It is create-then-filter vs filter-then-create.

**What is the btormc validation row?** Two smoke tests that the output is
a *legal, usable* model: `catbtor` (the official BTOR2 checker) parses and
sort-checks every line, and `btormc -kmax 0` actually loads the model and
evaluates the initial state. This proves well-formedness only — the
*behavioural* proof is the 18/18 same-property-same-k table below.

#### Property-level equivalence (deep check: same property, same least-k)

The strong test (see `EQUIVALENCE_PLAN.md`): run `btormc -kmax 1500` on both
rotors' models of the same binary and require the **same bad-state property
index** to fire at the **same least bound k**. **FINAL RESULT: 18/18
benchmarks equivalent** (`benchmarks/deep_equivalence_results.csv`; full
table and methodology in `P2_RESULTS.md`). Highlights:

| Benchmark | C reference | Rust rotor | Match |
|---|---|---|:---:|
| division-by-zero-3-35 | division-by-zero @ k=76 | division-by-zero @ k=76 | YES |
| invalid-memory-access-fail-2-35 | store-invalid-address @ k=79 | store-invalid-address @ k=79 | YES |
| memory-access-fail-1-35 | load-seg-fault @ k=66 | load-seg-fault @ k=66 | YES |
| nested-if-else-1-35 | bad-exit-code @ k=100 | bad-exit-code @ k=100 | YES |
| nested-if-else-reverse-1-35 | bad-exit-code @ k=103 | bad-exit-code @ k=103 | YES |
| nested-recursion-fail-1-35 | UNSAT @ kmax=1500 | UNSAT @ kmax=1500 | YES |
| recursive-ackermann-1-35 | bad-exit-code @ k=152 | bad-exit-code @ k=152 | YES |
| recursive-factorial-fail-1-35 | bad-exit-code @ k=119 | bad-exit-code @ k=119 | YES |
| recursive-fibonacci-1-10 | bad-exit-code @ k=118 | bad-exit-code @ k=118 | YES |
| return-from-loop-1-35 | UNSAT @ kmax=1500 | UNSAT @ kmax=1500 | YES |
| simple-assignment-1-35 | bad-exit-code @ k=96 | bad-exit-code @ k=96 | YES |
| simple-decreasing-loop-1-35 | bad-exit-code @ k=99 | bad-exit-code @ k=99 | YES |
| simple-if-else-1-35 | bad-exit-code @ k=108 | bad-exit-code @ k=108 | YES |
| simple-if-else-reverse-1-35 | bad-exit-code @ k=108 | bad-exit-code @ k=108 | YES |
| simple-if-without-else-1-35 | bad-exit-code @ k=101 | bad-exit-code @ k=101 | YES |
| simple-increasing-loop-1-35 | bad-exit-code @ k=93 | bad-exit-code @ k=93 | YES |
| three-level-nested-loop-fail-1-35 | bad-exit-code @ k=103 | bad-exit-code @ k=103 | YES |
| two-level-nested-loop-1-35 | bad-exit-code @ k=99 | bad-exit-code @ k=99 | YES |

This required making the machine model faithful to the C reference:
zero-initialized segments and registers, page-aligned heap, full 4 GB stack
(`[0xFFFFF800, 2^32)`), the concrete argc/argv boot image, the real
read-syscall data flow (one input byte per transition, PC stalled while
reading), file-descriptor state, and all 24 properties ported predicate-by-
predicate from `rotor.c` in the C output's exact emission order.

| Aspect | C Rotor | Rust Rotor |
|---|---|---|
| **Bad-state properties** | 24 | **24** — same names, same order, same predicates |
| **Deduplication algorithm** | linear list scan, O(N²) total | HashMap lookup, O(N) total |
| **Dedup disabled** (supervisor's experiment) | **crashes** (`ite then sort mismatch`, exit 14) — the C generator relies on line reuse for node identity | works (`--no-cse`): models grow ~1.43× and stay catbtor-valid |
| **Initialization encoding** | Unfolded over time steps (`zeroed-*` → `loaded-*`) | Direct `init` statements from binary data |

The dedup experiment confirms the speed difference is purely the data
structure (constant-time lookups vs linear scans over millions of checks),
not a difference in what gets deduplicated — and that dedup is semantically
neutral in the Rust implementation.

#### Why do the file sizes differ if the models contain the same things?

Because BTOR2 file size is a *syntactic* artifact — the same content can be
spelled differently. Measured decomposition of the selfie self-models
(C: 138,820 lines / 10.6 MB; Rust: 110,904 lines / 3.1 MB):

| Factor | C rotor | Rust rotor |
|---|---|---|
| comment bytes | **5.24 MB** (49.5% of the file) | 0.15 MB |
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
counts (138.6k vs 110.9k, only 1.25x apart) show the *content* is nearly
identical; the byte ratio (3.4x) is comments + binary-string constants +
longer nids. Think pretty-printed JSON with comments vs the same JSON
minified — and the P2 equivalence results are the proof that the meaning is
preserved: btormc returns the same verdict at the same bound from both.

### Verification

The generated BTOR2 models can be verified with:
- [btormc](https://github.com/Boolector/btor2tools) — BTOR2 bounded model checker
- [Bitwuzla](https://bitwuzla.github.io/) — SMT solver with BTOR2 support

### Dependencies

#### Rust (rotor)

| Crate | Purpose |
|-------|---------|
| `goblin` | ELF binary parsing |
| `clap` | CLI argument parsing |
| `thiserror` | Error types |
| `log` + `env_logger` | Debug logging |

#### Visualizer (CDN)

| Library | Purpose |
|---------|---------|
| [Cytoscape.js](https://js.cytoscape.org/) | Graph rendering and interaction |
| [dagre](https://github.com/dagrejs/dagre) | Hierarchical graph layout |
| [cytoscape-dagre](https://github.com/cytoscape/cytoscape.js-dagre) | Cytoscape-dagre integration |
| [cytoscape-svg](https://github.com/kinimesi/cytoscape-svg) | SVG graph export |

### References

- [Selfie project](https://github.com/cksystemsteaching/selfie) — Original C implementation
- [BTOR2 format](https://link.springer.com/chapter/10.1007/978-3-319-96145-3_32) — BTOR2, BtorMC and Boolector 3.0
- [Agent-BiTR](https://github.com/cksystemsgroup/agent-bitr) — Related work on agent-based bounded model checking
- Diller (2022) — *Visualizing BTOR2 Models* (thesis, inspiration for visualizer features)

### License

See the [selfie project](https://github.com/cksystemsteaching/selfie) for licensing details.
