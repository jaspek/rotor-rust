# Crash report: rotor aborts with "ite then sort mismatch error" when reuse_lines = 0

A complete, self-contained reproduction of a crash in the reference rotor
(upstream selfie, `tools/rotor.c`) that occurs when line reuse is disabled.

## Summary

Disabling line reuse globally (`reuse_lines = 0`) makes rotor abort while
generating any model. The sort-compatibility checks compare sort lines by
POINTER equality, which is only sound while line reuse maintains the
invariant "pointer equivalence iff structural equivalence". With reuse off,
two structurally identical sort lines are distinct pointers and the first
ITE construction fails its then-branch sort check.

## Reproduction (3 steps)

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

## Observed output

```
./rotor: selfie compiling selfie.c to 64-bit RISC-V using 64-bit starc
...
./rotor: ********************************************************************************
./rotor: ite then sort mismatch error
(exit status 14, ~0.08 s into model generation)
```

Expected (by analogy with the Rust rotor's --no-cse mode): a larger but
valid model.

## Root cause (file/line references, current main)

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

## Possible fixes (pick one)

- Make `match_sorts` structural: fall back to `are_lines_equal(sid1, sid2)`
  when the pointers differ (cost is negligible; sorts are tiny lines).
- Or: always reuse SORT lines regardless of `reuse_lines` (there are only a
  handful of distinct sorts; a dedicated small registry keeps the pointer
  invariant for sorts while letting everything else duplicate).

## Context

The crash was encountered during a comparison experiment in which the
duplicate check is disabled in both rotors and the outputs compared. The
Rust rotor with `--no-cse` produces a 1.43x larger model that catbtor
accepts and btormc gives identical verdicts on; the C rotor cannot complete
the same experiment because of this crash.
