# Speaker script — the final talk

*Pairs with `Rotor_Presentation_Final.pdf` — the 15-plate hand-coloured
engraving deck. The slides are images; the words live here. Talk to the
room, not the screen.*

**Voice.** Warm, plain, unhurried — but precise with the numbers. You are
letting the room in on something you are proud of, and treating the tool
that came before you with respect. One quiet spine runs through the whole
talk: *the hard part was never the speed — it was proving the new tool still
tells the truth.*

**Split.** **Jasmin** — plates 1–10 (the problem → the craft → the proof in
C). **Daniel** — plates 11–15 (the new capability → the validation → the
plate → the close). The hand-off is at the end of plate 10.

**Pacing.** ~13–15 minutes. *(Italics are stage directions.)* `→` = next plate.

---

## JASMIN

### 1 — Frontispiece *(0:20)*
*→ Title. Let the landscape sit for a moment.*

> "We were handed a tool that can look at a piece of compiled software and
> tell you — not guess, *tell* you — whether it can ever crash. And we were
> asked to rebuild it, and to do the harder thing: prove our version still
> tells the truth. I'm Jasmin, this is Daniel. This is *Rotor in Rust*."

### 2 — The Haystack *(0:55)*
*→ The stippled field, one bare ring.*

> "Start with the problem. The usual way to find a bug is to test — run the
> program on some inputs, see if it breaks. But a program reading just eight
> bytes of input has more than eighteen *billion billion* possible inputs.
> Two to the sixty-four. You try a handful. The one input that crashes can
> sit anywhere in the rest, and you would never know. So testing can tell you
> a program *broke* — it can never tell you it's *safe*."

### 3 — One Question, Asked of All *(0:50)*
*→ The sun shining over the field.*

> "There's a stronger idea: *bounded model checking*. Instead of trying inputs
> one at a time, you ask a single question that covers all of them at once —
> can this program reach a bad state, for *any* input, within k steps? You
> treat the unknown input like algebra, not like a guess. If the answer is
> yes, you get the exact input that does it. If it's no, you get a guarantee
> up to k. That one question is what this whole pipeline is for."

### 4 — The Apparatus *(0:45)*
*→ The four vessels: binary → model → solver → picture.*

> "Here's the pipeline. You start with a *compiled* binary — the real machine
> code, not the source, so what's verified is what actually runs. A tool
> called *rotor* translates it into a bit-precise mathematical model. A solver
> called btormc searches that model. Rotor already existed — one fourteen-
> thousand-line C file. Our job was to rebuild it, and add the last step: a
> way to actually read the answer. The two stages marked *this work* are ours."

### 5 — The Mechanism *(0:45)*
*→ The 24-tooth wheel.*

> "What does rotor build? Think of a board game: a *position*, the *legal
> moves*, and the *forbidden squares*. The position is the machine's state —
> program counter, registers, memory. The moves are one transition for every
> instruction. And the forbidden squares are twenty-four *bad-state
> properties* — division-by-zero, segmentation faults, a wrong exit code. One
> tooth of this wheel is division-by-zero. The whole project is about getting
> all twenty-four to mean exactly the same thing in both tools."

### 6 — Order from Tangle *(0:40)*
*→ The scribble ball → the chest of four drawers.*

> "So we rebuilt it in Rust. The original was one enormous file with hundreds
> of globals. Ours is a clean, modular crate — four parts: the node graph, the
> instruction decoder, the machine model, and the property generator. Every
> piece of the model is a typed node, and we re-derived the twenty-four
> properties from the original, in the same order — because the solver
> identifies them by index."

### 7 — Swiftness *(0:40)*
*→ The two hourglasses, "≈1000×".*

> "And it ran about a *thousand* times faster. A model that took the old tool
> a hundred and thirty-nine seconds, ours builds in under a tenth of a second
> — in a twentieth of the memory, four hundred megabytes down to twenty.
> *(a beat)* Now. When something is suddenly a thousand times faster, a
> sensible person doesn't celebrate — they get suspicious. Is it faster
> because it's better, or because it's quietly doing *less*? So we didn't
> trust the number. We counted."

### 8 — Do Not Trust — Count *(0:55)*
*→ The ledger vs. the index card.*

> "Both tools repeat one question millions of times: *have I already built
> this piece of the model before?* The old tool answers it by walking a list,
> top to bottom, every time. We added a counter — nearly *twelve billion*
> comparisons. Ours answers the same question with a hash index — a hundred
> and fifty-nine thousand look-ups. Same question, same answer. A list,
> walked, against an index. *That* single data structure is the entire
> speed-up. Nothing was skipped — and we know, because each tool's counters
> add up exactly to its own reported output."

### 9 — Three Castings of One Engine *(0:55)*
*→ The three orbs: Original C, C + the trick, Rust.*

> "And here I want to be fair to the tool that came before us. The easy story
> is 'the old thing was slow, we made a fast new thing.' That story is
> ungrateful, and it's false. There are really *three* rotors. On the left,
> the **original** C — a master with one old habit: a hundred and thirty-nine
> seconds, three million pieces, eleven billion comparisons. On the right,
> **Rust** — a tenth of a second, because it never builds the duplicates in
> the first place. And in the middle — the one I'm proudest of — the original
> C, taught the *single new trick*. We'll come back to it."

### 10 — Teaching the Old Engine *(0:50, then hand off)*
*→ The two meshing cogs, "95×, byte for byte".*

> "So we proved the point where it's hardest to argue. We took that one trick
> — the index instead of the list — and put it back into the *original C
> itself*. Same language, same compiler. It ran about *ninety-five times*
> faster, and its output came out *byte for byte identical* to before. Not
> similar — identical. So the speed was never Rust versus C. It was one small
> thing, done well.
>
> But speed is worthless if the answer is wrong — and a faster tool can see
> things the old one couldn't. That's Daniel's half."

---

## DANIEL

### 11 — The Door Unsealed *(1:00)*
*→ The opened padlock; two small frozen locks.*

> "Thanks. Here's something the original tool could not see. It could only
> explore input typed at the keyboard — standard input. But programs take
> input another way too: the words *after* the program's name, the
> command-line arguments. In the old model those were *frozen* — fixed before
> the question was ever asked. So any bug that needed a particular argument
> was structurally invisible. The solver wasn't failing to find it; it wasn't
> *allowed to look*.
>
> We opened that door. We leave the argument bytes *open* — blanks the solver
> fills in — using the paper's own device for input: an uninitialized state.
> Everything else, the count and the pointers, stays concrete. And this was
> listed as *future work* in the original rotor paper. We built it."

### 12 — The Secret, Found by Reason *(1:00)*
*→ The two dials landing on X and Y.*

> "Concretely. We wrote a little program that behaves perfectly — *unless* the
> first letter of its first argument is X, *and* the first letter of its
> second argument is Y. Both, at once. Two particular letters, in two
> different places, that have to meet. You could sit at a keyboard for a
> thousand years and never stumble on it.
>
> We handed the model to the solver, and in seconds it came back with exactly:
> argument one starts with X, argument two starts with Y. It had never been
> told — it worked it out, by reason. We wrote five of these traps; the solver
> cracked all five."

### 13 — The Impartial Balance *(1:15)*
*→ The level scale, "36 / 36".*

> "Now — the question that hangs over the whole project. How do we know any of
> our models are *faithful*? Comparing the files is meaningless — same meaning,
> different spelling. And proving two generators equal is undecidable. So we
> let the solver be the *referee*.
>
> For the same program, both rotors must make the *same* bad state reachable
> at the *same* step number — or both find nothing. And that step number is a
> fingerprint of the entire run; one wrong instruction shifts it. Across all
> eighteen benchmark programs, in two settings — *thirty-six out of
> thirty-six*, identical. And we trust that bar because every mistake we made
> while building failed it *loudly* first. Once, three results disagreed — and
> it turned out to be a bug in our own measuring harness, not the rotor. We
> caught it because one row disagreed with the other seventeen. The method
> polices itself.
>
> *(briefly, honestly)* We don't claim a formal proof — the evidence is
> bounded, eighteen programs, depth fifteen hundred. But it's *falsifiable*,
> and it was falsified, again and again, until it wasn't."

### 14 — From Cipher to Picture *(1:00)*
*→ The wall of digits → the engraved graph with the red bad node.*

> "Last piece — and maybe the one that matters most. When the solver finds a
> fault, it's perfectly right. And it tells you so like *this*: thousands of
> lines of binary, every one tied to a number that means something only to the
> machine. A correct answer nobody can read convinces nobody.
>
> So we built a browser tool that turns it into a graph you can read. The model
> becomes a picture; we replay the solver's answer across it, step by step,
> until — there — the bad state lights up *violated*, fed by exactly the path
> that read the bytes the solver chose. The proof stops being a wall of text
> and becomes something you can point a finger at. It's online, with twelve
> examples."

### 15 — The Summit *(0:30, both)*
*→ The painted peak, the flag, the cartouche.*

> "So — a thousand times faster, a whole class of bugs now reachable, every
> result checked thirty-six out of thirty-six, the same trick proven in the
> original C, and an answer you can finally read.
>
> *(slower)* We were asked to make a verification tool faster. We did. But the
> fast part was the easy part. The hard part — the whole climb — was proving
> the new one still tells the truth. That was the work. Thank you."

---

## Q&A — keep these ready

**"How exactly did you measure the speed-up?"**
We instrumented the de-duplication question in both tools with plain integer
counters. The C asks it ~10,000 times, but each answer scans a million-entry
list — **11.7 billion** comparisons. Rust asks it every time, but each is one
hash probe — **159,018**. The counters are self-consistent: creations − reuses
= each tool's own reported line count, to the integer.

**"If the hash is the whole story, why is Rust still faster than your hashed C?"**
Two optimisations, not one. The hash fixes the *lookup* cost. But the C still
*creates* 3.17 million records and prunes later — create-then-filter. Rust
checks *before* allocating, so duplicates never exist — filter-at-creation.
That second difference is the memory story (20 MB vs 485 MB).

**"Disabling the duplicate check?"**
The original C **crashes** (a sort-pointer invariant) — reuse there is
load-bearing. Rust just runs: 1.43× bigger, **identical** verdict. A full
crash reproduction and a fix are in the repo.

**"Shouldn't equal models be the same size?"**
Size is *spelling*. Half the C file is comments; the rest is encoding — it
writes constants as 64-digit binary, we write decimals. Same meaning,
fewer characters. The proof of sameness isn't size; it's the solver's 36/36.

**"How does symbolic argv actually work, and its limits?"**
The argument *content* bytes are uninitialized model states (the paper's own
device); count, pointers and terminators stay concrete. Honest limits: we
**fix the number** of arguments and **bound their length**, and explore
content — sound for any program treating arguments as C strings.

**"Are the two rotors *provably* equivalent?"**
No — and we say so. That's undecidable, and the paper's own future work. What
we have is strong *checkable* evidence: 18 programs, depth 1500, two settings,
all matching — and one apparent mismatch that turned out to be our *own*
harness.

**"Which of you did what?"**
Jasmin: the Rust rewrite, the faithful machine model, the equivalence campaign,
the C back-port. Daniel: the symbolic-argv programs and analysis, the
comment/size investigation, the visualizer examples. Both: the validation
methodology and the write-up.

**"Could this scale to real (gcc) binaries?"**
Rotor already supports gcc binaries and compressed instructions — we generate
those models. We just haven't run the full *differential* validation on them
yet; the harness already exists. That's the obvious next step.

---

### If you lose your place
Three anchors carry the whole talk:
**one question, not a billion guesses · one small thing, done well · an answer
you can actually read.** Say the one you're nearest to, and the story finds
its feet.
