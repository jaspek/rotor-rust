# Symbolic argv Test Programs

Five C* programs where a bug only triggers for specific command-line argument
bytes. These validate Part 2 (symbolic argv) of the project. All are written
in the selfie C* subset (only `uint64_t`, no bitwise ops, no string library),
so byte extraction is done with division/modulo arithmetic.

## Test programs — actual bug conditions (from the sources)

| # | File | The bug (`return 1`) fires when... | Solver must find |
|---|------|------------------------------------|------------------|
| 1 | `test1_crash_string.c` | `argv[1][0] == 67` (`'C'`) | one exact byte |
| 2 | `test2_numeric_overflow.c` | `argv[1][0]*256 + argv[1][1] == 16706` (= `'A'`,`'B'`) | two exact bytes, order matters |
| 3 | `test3_length_dependent.c` | `argv[1][0] != 0 && argv[1][1] == 0` (a 1-character argument) | a length property |
| 4 | `test4_multi_arg.c` | `argv[1][0] == 88 && argv[2][0] == 89` (`'X'`,`'Y'`) | bytes in TWO different arguments simultaneously |
| 5 | `test5_checksum.c` | `argv[1][0] + argv[1][1] == 200` | an arithmetic relation between bytes |

None of these programs read stdin, so none of these bugs are reachable with
the original rotor's stdin-only symbolic input — that is the capability gap
symbolic argv closes.

Verified example (test1): btormc reports `bad-exit-code` SATISFIABLE at
k = 67 with witness byte `argv[1][0] = 01000011` (= 0x43 = `'C'`).

## Usage

```bash
# Compile with selfie (or use the committed .m binaries)
selfie -c test1_crash_string.c -o test1_crash_string.m

# Generate the BTOR2 model with symbolic argv.
# NOTE: these programs exit(1) on the bug, so target exit code 1.
rotor test1_crash_string.m --symbolic-argv --num-symbolic-args 1       --max-arglen 8 --exit-code 1 -o test1.btor2

# test4 needs two symbolic arguments:
rotor test4_multi_arg.m --symbolic-argv --num-symbolic-args 2       --max-arglen 8 --exit-code 1 -o test4.btor2

# Solve
btormc -kmax 300 test1.btor2
```

## Notes on `--max-arglen`

Keep `--max-arglen` a multiple of the machine word size (8 on x64; the
default is 8). The C* test programs read whole 64-bit words from the
argument strings; with a non-word-multiple length, a word load can span the
argument's null terminator into the neighbouring bytes of the layout.

Argument *content* bytes are fully unconstrained (0-255, including 0). An
interior zero byte corresponds to a shorter real-world string — fine for
programs that treat arguments as C strings, which all of these do.
