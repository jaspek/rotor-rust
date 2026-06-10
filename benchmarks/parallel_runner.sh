#!/usr/bin/env bash
# Parallel deep-equivalence runner for the benchmarks the sequential
# harness has not reached yet. Each (benchmark, rotor) pair is one job;
# xargs -P runs them across cores. Results land in parallel_results/
# as one small file per run, merged afterwards by the caller.
set -u

ROOT="C:/Users/jasko/Programming/Rust/Project01/benchmarks"
KMAX=1500
OUT="$ROOT/parallel_results"
mkdir -p "$OUT"

run_one() {
    local name="$1" side="$2" dir file
    if [ "$side" = "C" ]; then dir="$ROOT/btor2-c-rotor"; else dir="$ROOT/btor2-rust-rotor"; fi
    file="$name.btor2"
    local res
    res=$(docker run --rm -v "$dir:/work" btormc:latest \
        -c "btormc -v 1 -kmax $KMAX /work/$file 2>&1 | grep 'SATISFIABLE' | grep -v UNSAT | head -1")
    # Distinguish a genuine UNSAT from a dead container: only record
    # UNSAT when btormc actually completed (full -v 1 log retained).
    full=$(docker run --rm -v "$dir:/work" btormc:latest         -c "btormc -v 1 -kmax $KMAX /work/$file 2>&1 | tail -5") 
    if [[ "$res" =~ bad\ state\ property\ ([0-9]+)\ reachable\ at\ bound\ k\ =\ ([0-9]+) ]]; then
        echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}" > "$OUT/$name.$side.txt"
    elif [ -n "$res$full" ]; then
        echo "UNSAT -" > "$OUT/$name.$side.txt"
    else
        echo "ERROR container-died" > "$OUT/$name.$side.txt"
    fi
    echo "done: $name [$side] -> $(cat "$OUT/$name.$side.txt")"
}
export -f run_one
export ROOT KMAX OUT

BENCHES=(
  simple-assignment-1-35
  simple-decreasing-loop-1-35
  simple-if-else-1-35
  simple-if-else-reverse-1-35
  simple-if-without-else-1-35
  simple-increasing-loop-1-35
  three-level-nested-loop-fail-1-35
  two-level-nested-loop-1-35
)

# Heavy-suspect benchmarks first so they get cores earliest
printf "%s\n" \
  "three-level-nested-loop-fail-1-35 C" \
  "three-level-nested-loop-fail-1-35 R" \
  "two-level-nested-loop-1-35 C" \
  "two-level-nested-loop-1-35 R" \
  "simple-assignment-1-35 C" \
  "simple-assignment-1-35 R" \
  "simple-decreasing-loop-1-35 C" \
  "simple-decreasing-loop-1-35 R" \
  "simple-if-else-1-35 C" \
  "simple-if-else-1-35 R" \
  "simple-if-else-reverse-1-35 C" \
  "simple-if-else-reverse-1-35 R" \
  "simple-if-without-else-1-35 C" \
  "simple-if-without-else-1-35 R" \
  "simple-increasing-loop-1-35 C" \
  "simple-increasing-loop-1-35 R" \
| xargs -P 8 -n 2 bash -c 'run_one "$0" "$1"'

echo "ALL PARALLEL RUNS COMPLETE"
