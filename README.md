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
| 1 | Rust rewrite of the translator | **Complete** — generates BTOR2 models semantically equivalent to the C reference on 18 test programs. |
| 2 | Symbolic argv support | **Complete** — 5 benchmark programs, each with a bug reachable *only* via argv, are discovered by btormc. |
| 3 | Witness-trace visualizer | **In progress** — loads real btormc witness traces, supports depth-limited subgraphs and cone-of-influence views; per-step inspector is being polished. |

Code for each part lives in its own subdirectory: `rotor/`, `benchmarks/argv-tests/`, and `visualizer/`.

Deliverables (slides, reports) are in the repository root: `Final_Report.pdf`, `Rotor_Overview.pdf`, `Rotor_Presentation.pdf`, `Symbolic_Arguments_Presentation.pdf`, `paper.tex`. The thesis-direction write-up is distributed via the [GitHub releases page](../../releases) (it is unrelated to this course project).

---

## Technical details

The sections below describe each part in more depth. Readers who only want the high-level picture can stop here.

### Part 1: Rotor in Rust

#### Features

- **RISC-V support**: RV32I/RV64I base integer ISA, M extension (multiply/divide), C extension (compressed instructions)
- **Multi-core**: Configurable number of cores
- **Segmented memory model**: Code (read-only), data, heap, and stack segments
- **Kernel syscall modeling**: `exit`, `read`, `write`, `openat`, `brk`
- **Safety properties**: Bad exit codes, division by zero, segmentation faults
- **HashMap-based CSE**: O(1) common subexpression elimination (vs O(n) in the C original)
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
- **Example**: Click "Example" to load a bundled example model

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

#### Example Files

| File | Description |
|------|-------------|
| `examples/simple-assignment-1-35.btor2` | Rotor output for a simple C program (~1142 nodes) |
| `examples/counter-with-input.btor2` | Small counter model with state + input (19 nodes) |
| `examples/counter-with-input.wit` | Real btormc witness trace (6 steps, counter overflow) |
| `examples/simple-assignment-1-35.wit` | Synthetic witness trace (34 steps) |
| `examples/division-by-zero-c.wit` | Real btormc witness trace (77 steps, C rotor division-by-zero) |
| `examples/division-by-zero-rust.wit` | Real btormc witness trace (111 steps, Rust rotor seg-fault) |

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

Both the Rust and C implementations of Rotor generate valid BTOR2 models that btormc can verify. The Rust rewrite produces more compact models with a different initialization encoding:

| Aspect | C Rotor | Rust Rotor |
|--------|---------|------------|
| **Model size** (division-by-zero) | 4,163 lines | 1,176 lines (3.5x smaller) |
| **Bad properties** | 24 (granular, per-instruction type) | 3 (abstract: exit code, div-by-zero, seg-fault) |
| **States** | 14 (with phased init/zeroed/loaded) | 13 |
| **Initialization** | Unfolded over time steps (`zeroed-*` → `loaded-*`) | Direct init from binary data |
| **btormc counterexample** | Step 77 (kmax=100) | Step 111 (kmax=200) |

The C rotor uses **phased initialization** — separate `zeroed-code-segment` and `loaded-code-segment` states that unfold memory loading over clock cycles. The Rust rotor encodes initialization **directly** via `init` statements, producing a more compact model.

Both produce semantically equivalent results: btormc finds counterexamples in both. The Rust model requires a higher `kmax` bound because its compact encoding results in more unrolling steps for the solver, but the generated models are ~3.5x smaller.

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
