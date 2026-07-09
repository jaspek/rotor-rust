# The CSE-off experiment

The duplicate check is to be disabled in BOTH rotors and the outputs compared. This is intended to isolate whether the Rust rotor's ~1,000× faster generation is produced by the same deduplication work being done faster, or by less/different work being done.

## Result summary

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
- `patch_c_counters.py` - not part of this experiment, it patches the
  profiling counters into `/selfie/tools/rotor.c` (and restores `reuse_lines = 1`,
   which the CSE-off build above leaves at 0). Used by the profiling
  run written up in `../../PROFILING_RESULTS.md`
  ''
  

## Reproduce

All paths are relative to the repository root. Build the generator ('cargo build --release') and the checker image
('docker build -t btormc -f benchmarks/Dockerfile.btormc .') first - 'rotor' and 'btormc' are not on the 'PATH';
the former is a build artifact and the latter only exists inside the image.


```bash
# Rust side — regenerate both committed models
./target/release/rotor benchmarks/binaries/division-by-zero-3-35.m \
  --xlen x64 --bytes-to-read 1 --heap 2048 --stack 2048 \
  -o benchmarks/cse-experiment/division-by-zero-cse-on.btor2
./target/release/rotor benchmarks/binaries/division-by-zero-3-35.m \
  --xlen x64 --bytes-to-read 1 --heap 2048 --stack 2048 --no-cse \
  -o benchmarks/cse-experiment/division-by-zero-cse-off.btor2

# Both must yield the SAME verdict — that is the point of the experiment
docker run --rm -v "$PWD:/w" btormc \
  -c "btormc -v 1 -kmax 100 /w/benchmarks/cse-experiment/division-by-zero-cse-on.btor2"
docker run --rm -v "$PWD:/w" btormc \
  -c "btormc -v 1 -kmax 100 /w/benchmarks/cse-experiment/division-by-zero-cse-off.btor2"
#   -> both: bad state property 7 reachable at bound k = 76

# C side (crashes in the CSE-off stage, by design of the experiment)
docker build -t selfie-cse -f benchmarks/cse-experiment/Dockerfile.cse \
  benchmarks/cse-experiment
docker run --rm selfie-cse
```
