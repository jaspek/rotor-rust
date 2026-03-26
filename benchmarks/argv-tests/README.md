# Symbolic argv Test Programs

Five C programs where a bug only triggers with specific command-line arguments.
These validate Part 2 (symbolic argv) of the project.

## Test Programs

| # | File | Bug condition | Expected solver finding |
|---|------|---------------|------------------------|
| 1 | `test1_crash_string.c` | `argv[1] == "CRASH"` | `argv[1] = {0x43,0x52,0x41,0x53,0x48}` |
| 2 | `test2_numeric_overflow.c` | `argv[1][0]=='A' && argv[1][1]=='B'` (div by zero) | `argv[1] = {0x41, 0x42}` |
| 3 | `test3_length_dependent.c` | `strlen(argv[1]) == 7` | Any 7 non-null bytes |
| 4 | `test4_multi_arg.c` | `argv[1][0]=='X' && argv[2][0]=='Y'` | Two-arg combination |
| 5 | `test5_checksum.c` | `s[0]+s[1]+s[2]+s[3] == 400` | e.g. `{100,100,100,100}` |

Tests 2, 4, and 5 **cannot** be found using symbolic `stdin` alone (the programs never read from stdin).

## Usage

```bash
# Compile for RISC-V
riscv64-unknown-elf-gcc -o test1.elf test1_crash_string.c

# Generate BTOR2 model with symbolic argv
rotor test1.elf --symbolic-argv --symbolic-argc 1 --max-arglen 8 -o test1.btor2

# Verify with solver
btormc test1.btor2
# or
agent-bitr test1.btor2
```
