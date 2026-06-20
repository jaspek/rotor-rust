# Q&A preparation — ~5 minutes

*Backup slides live at the end of `Rotor_Presentation_10min.pdf` ("Backup"
section). When a question maps to one, flip to it and use the answer below.
Answer concretely, give the number, and state the limits honestly — that is
more convincing than a confident guess.*

---

### Q1. "How exactly did you measure the speed-up?" → backup slide *the counting, in full*
We instrumented the dedup question in both tools with plain integer counters.
C asks it about 10k times, but each answer scans a million-entry list —
**11.7 billion** comparisons total. Rust
asks it every time, but each is one hash probe — **159k**. The counters are
self-consistent: *creations − reuses = each tool's own reported line count, to
the integer.* That's how we know we counted the right thing.

### Q2. "If the hash is the whole story, why is Rust still faster than your hashed C?" → backup slide *Rust vs the optimized C*
Two optimizations, not one. The hash fixed the **lookup cost**. But the C
still *creates* 3.17 million line records and prunes later — create-then-
filter. Rust checks *before* allocating, so duplicates never exist — filter-
at-creation. That second difference is the memory story (20 MB vs 485 MB) and
the last factor of speed. We built that into the C too; it stays equivalent.

### Q3. "What happened when you disabled the duplicate check in both?" → backup slide *disable the duplicate check*
The original C rotor **crashes** — an internal sort-pointer invariant breaks
(`ite then sort mismatch`) — so line reuse there is load-bearing, not just an
optimization. Ours just runs: the model gets 1.4× bigger and the solver gives
the **identical** verdict. That proves dedup only affects size and speed,
never meaning. A complete crash reproduction and a fix are in the repository.

### Q4. "Shouldn't the models be the same size if they're equal?" → backup slide *file sizes*
Size is *spelling*, not meaning. Half the C file is **comments** — strip them
and 10.6 MB becomes 5.2. The rest is encoding: C writes constants as 64-digit
binary strings, we write decimals; its node numbers are longer. Same model,
fewer characters — like minified vs pretty-printed JSON. The proof they mean
the same isn't size; it's the solver's verdict, 36/36.

### Q5. "How is symbolic argv actually implemented, and what are its limits?" → backup slide *how symbolic argv works*
We build the real process-start stack — count, pointers, terminators all
concrete — but the argument **content** bytes are left *uninitialized states*,
exactly the device the original models already use for stdin. It reads them
through ordinary memory loads; we changed nothing in the decoder or the safety
checks. Honest limits: we **fix the number** of arguments and **bound their
length**, and explore content — sound for any program treating arguments as C
strings.

### Q6. "So are they *provably* equivalent?" → backup slide *what we do not claim*
No — and we're careful to say so. A formal proof that two generators are
equivalent is **undecidable**; the paper itself lists it as future work. What
we have is strong *checkable* evidence: 18 programs, depth 1500, two settings,
all matching. Two of those agree only by *both* finding nothing (UNSAT), which
is weaker than a step-exact hit. And once an apparent mismatch turned out to
be **our own measurement harness**, not the rotor — we caught it because one
result disagreed with the other seventeen. The methodology policing itself.

---

## Questions we may get without a dedicated slide (have these ready)

**"Which of you did what?"**
Jasmin: the Rust rewrite, the faithful machine model, the equivalence
campaign, and the C back-port. Daniel: the symbolic-argv test programs and
analysis, the comment/size investigation, and the visualizer examples. Both:
the validation methodology and the write-up.

**"Why Rust and not just fix the C?"**
We didn't have to choose — we did both. Rust gave us a clean, modular,
type-safe base to *extend* (argv, the properties), and the rewrite is where
the speed came from naturally. Then we proved the idea transfers by fixing the
C too. The rewrite was the vehicle; the data structure was the cause.

**"Could this scale to real (gcc) binaries?"**
Rotor already supports gcc binaries and compressed instructions — we generate
those models. We just haven't run the full *differential* validation on them
yet; that's the obvious next step, and the harness already exists.

**"What was the hardest bug?"**
A division benchmark that fired the *wrong* property. We traced it through
three layers — uninitialized memory, wrong segment geometry, a missing argv
boot image — each fix moving the failure, until the model matched the
reference exactly. The equivalence test is what made every one of those
visible.

**"Is the AI-generated part a problem academically?"**
We used AI as a tool and we can defend every line and every number — we
re-measured everything ourselves, and the artifacts are
all reproducible from the repo. The understanding is ours; the typing was
faster.

---

## If you blank
Fall back to the three numbers that anchor everything:
**1,000× faster · 36/36 equivalent · 5 argv-only bugs found.**
Then say "happy to go deeper on any of those."
