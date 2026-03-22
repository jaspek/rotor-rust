# Rotor (Rust)

A BTOR2 model generator for RISC-V machines, rewritten in Rust from the [selfie project](https://github.com/cksystemsteaching/selfie).

Rotor translates RISC-V ELF binaries into [BTOR2](https://link.springer.com/chapter/10.1007/978-3-319-96145-3_32) format for formal verification via bounded model checking.

## Features

- **RISC-V support**: RV32I/RV64I base integer ISA, M extension (multiply/divide), C extension (compressed instructions)
- **Multi-core**: Configurable number of cores
- **Segmented memory model**: Code (read-only), data, heap, and stack segments
- **Kernel syscall modeling**: `exit`, `read`, `write`, `openat`, `brk`
- **Safety properties**: Bad exit codes, division by zero, segmentation faults
- **HashMap-based CSE**: O(1) common subexpression elimination (vs O(n) in the C original)
- **Arena allocation**: Cache-friendly node storage with stable indices

## Building

```bash
cargo build --release
```

## Usage

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

## Architecture

```
src/
  btor2/          BTOR2 IR builder, node types, printer
  riscv/          ISA definitions, ELF loader, instruction decode
  machine/        Sorts, registers, memory, kernel, per-core state
  model/          Combinational logic, sequential logic, properties
```

### Pipeline

1. **Load** ELF binary (code + data segments)
2. **Initialize** BTOR2 sorts and machine constants
3. **Create** per-core state (PC, registers, memory, kernel)
4. **Generate** combinational logic (fetch, decode, ALU, control flow)
5. **Generate** sequential logic (next-state for PC, registers, memory)
6. **Generate** safety properties (bad states)
7. **Print** BTOR2 model

## Verification

The generated BTOR2 models can be verified with:
- [btormc](https://github.com/Boolector/btor2tools) — BTOR2 bounded model checker
- [Bitwuzla](https://bitwuzla.github.io/) — SMT solver with BTOR2 support

## References

- [Selfie project](https://github.com/cksystemsteaching/selfie) — Original C implementation
- [BTOR2 format](https://link.springer.com/chapter/10.1007/978-3-319-96145-3_32) — BTOR2, BtorMC and Boolector 3.0
- [Agent-BiTR](https://github.com/cksystemsgroup/agent-bitr) — Related work on agent-based bounded model checking

## License

See the [selfie project](https://github.com/cksystemsteaching/selfie) for licensing details.
