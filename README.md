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
| 1 | Rust rewrite of the translator | **Complete** — same 24 bad-state properties as the C reference; 18/18 standard benchmarks give the same btormc verdict; selfie self-model takes 47 ms in 20 MB (vs 106 s / 431 MB for the C version). |
| 2 | Symbolic argv support | **Complete** — 5 benchmark programs, each with a bug reachable *only* via argv, are discovered by btormc within seconds. |
| 3 | Witness-trace visualizer | **Complete** — browser tool with manifest-driven example picker (12 examples), full witness playback including array-valued symbolic inputs; available [live online](https://jaspek.github.io/rotor-rust/). |

Code for each part lives in its own subdirectory: `rotor/`, `benchmarks/argv-tests/`, and `visualizer/`.

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

# RV32 mode
rotor <binary.elf> --xlen 32

# Enable compressed instructions
rotor <binary.elf> --enable-c

# Disable M extension
rotor <binary.elf> --no-m

# Code synthesis mode (symbolic code, no binary)
rotor --synthesis -o model.btor2

# With debug comments in output
rotor <binary.elf> --comments
```

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
# Compile with selfie, then generate BTOR2 with symbolic argv
rotor <binary.elf> --symbolic-argv -o model-argv.btor2
```

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
- **Witness trace animation**: Step-by-step playback of btormc counterexample traces
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
  btor2-rust-rotor/     Rust Rotor output (with and without argv)
  btor2-c-rotor/        C Rotor reference output
  binaries/             Compiled RISC-V binaries (.m format)
  Dockerfile            Docker setup for selfie compilation
  Dockerfile.btormc     Docker setup for btormc model checker
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

| Metric | C Rotor (reference) | Rust Rotor | Ratio |
|---|---:|---:|---:|
| Wall-clock model generation | 106 s | **47 ms** | ~2,250× faster |
| Peak memory | 431 MB | **20 MB** | ~21× less |
| Output BTOR2 size | 10.6 MB | **3.1 MB** | 3.4× smaller |
| btormc validation (`catbtor` + `-kmax 0`) | — | **PASS** | — |

#### Property-level equivalence

| Aspect | C Rotor | Rust Rotor |
|---|---|---|
| **Bad-state properties** | 24 (by name) | **24** — same set, same names |
| **18-benchmark btormc verdict at depth 35** | reference | **18 / 18 match** |
| **Deduplication algorithm** | linear scan, O(N²) total (turned off for binary-loading section because of cost) | HashMap lookup, O(N) total (left on everywhere) |
| **Initialization encoding** | Unfolded over time steps (`zeroed-*` → `loaded-*`) | Direct `init` statements from binary data |

The Rust output's smaller size comes from two places: leaving CSE turned on for the binary-loading section (which the C version had to switch off for performance) and the more compact `init`-based initialization encoding. Both rotors emit the same set of `bad` nodes by name — `bad-exit-code`, `division-by-zero`, `illegal-instruction`, `fetch-invalid-address`, `load-seg-fault`, `stack-pointer-invalid-address`, `unknown-syscall-ID`, etc. — and produce the same verdict on the 18 standard benchmarks at depth 35.

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
