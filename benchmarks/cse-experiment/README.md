# The CSE-off experiment ("disable the duplicate check and see if the universe ends")

Proposed by the supervisor (2026-05-26 meeting): disable the duplicate
check in BOTH rotors and compare what comes out. This isolates whether the
Rust rotor's ~1,000x faster generation comes from doing the same
deduplication work faster, or from doing less/different work.

## Result summary (verified 2026-06-10/11)

| | C rotor (`reuse_lines = 0`) | Rust rotor (`--no-cse`) |
|---|---|---|
| Outcome | **CRASHES**: `ite then sort mismatch error`, exit 14, 0.08 s in | works |
| Model growth | n/a | 1,693 -> 2,433 lines on this benchmark (1.44x); selfie 110,904 -> 159,018 (1.43x) |
| catbtor | n/a | PASS |
| btormc verdict | n/a | **division-by-zero @ k = 76 — IDENTICAL to the CSE-on model** |

Conclusion: dedup in the Rust rotor is semantically neutral (file size
changes, meaning never does), while the C generator *relies* on line reuse
for node identity — its duplicate check is load-bearing, not merely an
optimization. The speed difference is therefore the lookup data structure
(hash map O(1) vs linear list scan O(N), i.e. O(N) vs O(N^2) total), not a
difference in what gets deduplicated.

## Files

- `division-by-zero-cse-on.btor2` — normal output (dedup on, default)
- `division-by-zero-cse-off.btor2` — same program, `--no-cse`; contains the
  duplicated constants/subexpressions (e.g. the address-validity check
  `ulte ... ; is machine word virtual address?` appears 11x instead of 1x)
- `Dockerfile.cse` — builds the C rotor twice from unmodified upstream
  selfie (once as-is, once with `reuse_lines = 0` via sed), runs both on
  selfie.c, and records time/memory/output. The CSE-off build crashes.

## Reproduce

```bash
# Rust side
rotor division-by-zero-3-35.m --xlen x64 --bytes-to-read 1 \
      --heap 2048 --stack 2048 -o cse-on.btor2
rotor division-by-zero-3-35.m --xlen x64 --bytes-to-read 1 \
      --heap 2048 --stack 2048 --no-cse -o cse-off.btor2
btormc -v 1 -kmax 100 cse-off.btor2   # -> property 7 @ k=76, same as cse-on

# C side (crashes in the CSE-off stage, by design of the experiment)
docker build -t selfie-cse -f Dockerfile.cse .
docker run --rm selfie-cse
```
