#!/usr/bin/env bash
# Parallel deep run for the EXIT-CODE-1 comparison (paper's "non-zero exit
# code" planted bugs). Both rotors' models generated with target exit 1.
set -u

# Session-specific helper: C side of the 18 benchmarks under target
# exit code 1 (the Rust side of that campaign ran separately; the
# merged output = the committed exit1 CSV).
# Prerequisites: btormc image built; btor2-c-rotor-exit1/ present.
ROOT="$(cd "$(dirname "$0")" && pwd)"
KMAX=1500
OUT="$ROOT/parallel_results_exit1"
mkdir -p "$OUT"

run_one() {
    local name="$1" side="$2" dir file
    if [ "$side" = "C" ]; then dir="$ROOT/btor2-c-rotor-exit1"; else dir="$ROOT/btor2-rust-rotor-exit1"; fi
    file="$name.btor2"
    if [ ! -f "$dir/$file" ]; then
        echo "ERROR missing-model" > "$OUT/$name.$side.txt"
        echo "done: $name [$side] -> ERROR missing-model"
        return
    fi
    local full res
    full=$(docker run --rm -v "$dir:/work" btormc:latest \
        -c "btormc -v 1 -kmax $KMAX /work/$file 2>&1")
    res=$(echo "$full" | grep 'SATISFIABLE' | grep -v UNSAT | head -1)
    if [[ "$res" =~ bad\ state\ property\ ([0-9]+)\ reachable\ at\ bound\ k\ =\ ([0-9]+) ]]; then
        echo "${BASH_REMATCH[1]} ${BASH_REMATCH[2]}" > "$OUT/$name.$side.txt"
    elif echo "$full" | grep -qE "btor|MC|reached|unreach|bound"; then
        echo "UNSAT -" > "$OUT/$name.$side.txt"
    else
        echo "ERROR container-died" > "$OUT/$name.$side.txt"
    fi
    echo "done: $name [$side] -> $(cat "$OUT/$name.$side.txt")"
}
export -f run_one
export ROOT KMAX OUT

# C side only (Rust side already completed correctly in the previous run).
# Deep-suspect benchmarks first so they get cores earliest.
printf '%s\n' \
  "nested-recursion-fail-1-35 C" \
  "recursive-ackermann-1-35 C" \
  "three-level-nested-loop-fail-1-35 C" \
  "two-level-nested-loop-1-35 C" \
  "nested-if-else-1-35 C" \
  "nested-if-else-reverse-1-35 C" \
  "recursive-fibonacci-1-10 C" \
  "recursive-factorial-fail-1-35 C" \
  "simple-decreasing-loop-1-35 C" \
  "simple-increasing-loop-1-35 C" \
  "division-by-zero-3-35 C" \
  "invalid-memory-access-fail-2-35 C" \
  "memory-access-fail-1-35 C" \
  "return-from-loop-1-35 C" \
  "simple-assignment-1-35 C" \
  "simple-if-else-1-35 C" \
  "simple-if-else-reverse-1-35 C" \
  "simple-if-without-else-1-35 C" \
| xargs -P 8 -n 2 bash -c 'run_one "$0" "$1"'

echo "ALL EXIT1 C-SIDE RUNS COMPLETE"
