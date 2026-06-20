# Speaker script — 10-minute talk

*Pairs with `Rotor_Presentation_10min.pdf`. Two speakers, storytelling style.
The slides are visual; the words are here. Talk to the room, not the screen.
Times are targets (~10:00 total). Slide numbers match the deck.*

**Split:** Jasmin — slides 1–6 (problem → speed → trust). Daniel — slides
7–11 (the new capability → demo → the insight → close).

---

## JASMIN

### Slide 1 — Title (0:15)
> "We were handed a tool that checks whether software can ever crash — and
> asked to rebuild it, and prove our version still tells the truth. I'm
> Jasmin, this is Daniel. Ten minutes, one question: can you trust a
> verification tool you rewrote yourself?"

### Slide 2 — You cannot test every input (0:55)
> "Start with the problem. The usual way to find bugs is testing: run a
> program on some inputs, see if it breaks. But a program reading just eight
> bytes has more possible inputs than there are stars in the sky. You try a
> handful — the blue squares. The one input that crashes can sit anywhere in
> the grey, and you would never know. So testing can tell you a program broke;
> it can never tell you it is safe."

### Slide 3 — Verification asks one question (0:50)
> "There is a stronger idea. Instead of trying inputs one at a time, you ask a
> single question that covers all of them at once: can this program reach a
> bad state within k steps, for any input? You treat the unknown input like
> algebra, not like a guess. If the answer is yes, you get the exact input
> that does it. If it is no, you get a guarantee. That is verification — and
> it is what this whole pipeline is for."

### Slide 4 — The pipeline (0:45)
> "Here is the pipeline. You start with a compiled program — the real machine
> code, not the source. A tool called rotor translates it into a mathematical
> model of the processor running it. A solver searches that model. Rotor
> already existed — one fourteen-thousand-line C file. Our job was to rebuild
> it, and add the final step: a way to actually read the answer. The two blue
> boxes are ours."

### Slide 5 — Three orders of magnitude faster (1:25)
> "So we rebuilt it in Rust, and it ran about a thousand times faster — a
> model that took a hundred and thirty-nine seconds now takes under a tenth of
> a second, in a twentieth of the memory. A jump that big deserves suspicion:
> is it faster because it is doing less? So we measured. Both tools repeat one
> operation millions of times — the question, 'have I already built this piece
> of the model before?' We counted it. The old tool answers by walking a list:
> nearly twelve billion comparisons. Ours answers with a hash table: a hundred
> and fifty-nine thousand lookups. Same question, same answer — a list versus
> an index. That single data structure is the entire speed-up. Nothing was
> skipped."

### Slide 6 — Faster is worthless if it is wrong (1:10, then hand off)
> "And that matters, because speed is worthless if the answer is wrong. So we
> tested it the only honest way: we let the solver be the referee. For the
> same program, both models must reach the same bad state at the exact same
> step number — and that step number is a fingerprint of the entire run; one
> wrong instruction shifts it. Across all eighteen benchmark programs, under
> two settings — thirty-six out of thirty-six identical. And we trust that bar
> because every mistake we made while building failed it loudly first.
>
> That is the foundation. But a verifier is only as good as what it can see,
> and what you can do with its answer — and that is Daniel's half."

---

## DANIEL

### Slide 7 — A whole class of bugs was unreachable (1:00)
> "Thanks. Here is something the original tool could not see. The solver could
> only explore input typed at the keyboard. But programs also take input from
> command-line arguments — the words after the program's name. Those were
> frozen in the model. So any bug that needs a specific argument was
> structurally invisible: the solver was not failing to find it, it was not
> allowed to look. We changed that — we leave the argument bytes open, blanks
> the solver fills in. And this was the original paper's own listed future
> work. We built it."

### Slide 8 — The solver discovers the secret (1:00)
> "Concretely: this program crashes only if the first letter of argument one
> is X and the first letter of argument two is Y — two specific characters, in
> two different arguments, at the same time. Nothing typed at the keyboard
> could ever trigger it. We hand the model to the solver, and in seconds it
> comes back with exactly: argument one starts with X, argument two starts
> with Y. It derived the secret by logic. We wrote five of these traps; the
> solver cracked all five."

### Slide 9 — The answer, made watchable (1:20)
> "But there is a catch: the solver's raw answer is thousands of lines of
> binary keyed to internal numbers. A correct answer nobody can read convinces
> nobody. So we built a browser tool that turns it into a graph you can read.
>
> This is the actual model. Each node is a piece of the machine — the diamonds
> are constants, the green box is the register state. We replay the witness on
> top of it, and at the final step this red node — the bad state — is reported
> VIOLATED, fed by exactly the logic that read the argument bytes the solver
> chose. The verification result stops being a wall of text and becomes
> something you can point at. The tool is online with twelve examples."
>
> *(This is a screenshot — no live demo needed. If you want to show it live
> as well, the page is at jaspek.github.io/rotor-rust; but the slide stands on
> its own.)*

### Slide 10 — One idea, proved in C too (1:00)
> "One last result we are proud of. To show the speed was the idea and not
> the language, we took that single change — the hash lookup — and put it back
> into the original C tool. The original then ran about ninety-five times
> faster, produced byte-identical output, and passed the same thirty-six out
> of thirty-six check. So the speed was never Rust versus C — it was one data
> structure. And that is the real lesson: the hard part of rebuilding a
> verification tool is not writing the code, it is proving the new one still
> tells the truth. The methodology was the project."

### Slide 11 — Close (0:15, both)
> "A thousand times faster, a whole class of bugs now reachable, every result
> checked thirty-six out of thirty-six, and an answer you can actually read.
> Thank you — we are happy to take questions."

---

## Delivery notes
- ~1,250 words ≈ 9–10 minutes at a calm pace. Practice once out loud.
- One idea per slide — if a line slips, say the headline and move on.
- The hand-off is explicit at the end of slide 6.
- Rehearse the demo with the page already open in a tab; it is the peak.
- Backup slides (deck, after "Thank you") are for questions only — see
  `QA_PREP_5min.md`.
