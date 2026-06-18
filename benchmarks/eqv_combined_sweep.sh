#!/usr/bin/env bash
# Combined differential-validation sweep: baseline-C vs dedup-all-C, all 18
# benchmarks x 2 exit-code configurations (36 paired verdicts), btormc at
# kmax=1500. One -P 6 pool over all 36 jobs so no slot sits idle while the
# deep recursion benchmarks grind. Run inside the btormc image with the four
# model dirs and a results dir mounted.
set -u
KMAX="${KMAX:-1500}"
OUT=/out
mkdir -p "$OUT"

verdict() {
  # echoes "P@K" on SAT, "UNSAT" if it completed with nothing reachable,
  # or "ERROR" if btormc produced no recognizable output (e.g. OOM kill).
  local full sat
  full=$(btormc -v 1 -kmax "$KMAX" "$1" 2>&1)
  sat=$(echo "$full" | grep -oE "property [0-9]+ reachable at bound k = [0-9]+" | head -1)
  if [ -n "$sat" ]; then echo "$sat"
  elif echo "$full" | grep -qE "MC|bound|reached|btor"; then echo "UNSAT"
  else echo "ERROR"; fi
}

run_one() {
  local cfg="$1" n="$2" bdir ddir
  if [ "$cfg" = "e0" ]; then bdir=/base0; ddir=/da0; else bdir=/base1; ddir=/da1; fi
  local vb vd m
  vb=$(verdict "$bdir/$n.btor2")
  vd=$(verdict "$ddir/$n.btor2")
  if [ "$vb" = "$vd" ] && [ "$vb" != "ERROR" ]; then m=MATCH; else m=DIVERGE; fi
  echo "$cfg | $n | baseline:[$vb] | dedup-all:[$vd] | $m" > "$OUT/$cfg.$n.txt"
  echo "done $cfg $n -> $m"
}
export -f run_one verdict
export KMAX OUT

# Build the 36-job list: light SAT benchmarks first, the two deep UNSAT ones
# (nested-recursion-fail, return-from-loop) last so they don't hog slots early.
{
  for cfg in e0 e1; do
    for f in /base0/*.btor2; do
      n=$(basename "$f" .btor2)
      case "$n" in nested-recursion-fail*|return-from-loop*) : ;; *) echo "$cfg $n";; esac
    done
  done
  for cfg in e0 e1; do
    echo "$cfg nested-recursion-fail-1-35"
    echo "$cfg return-from-loop-1-35"
  done
} | xargs -P 6 -n 2 bash -c 'run_one "$0" "$1"'

echo "ALL 36 DONE"
