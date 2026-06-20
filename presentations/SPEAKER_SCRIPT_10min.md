# Speaker script — 10-minute talk

*Pairs with `Rotor_Presentation_10min.pdf`. Two speakers; storytelling style.
Times are targets, ~10:00 total. Slide numbers match the deck. Keep eye
contact, not eyes-on-slides — the professor listens, he barely looks at the
screen.*

**Split:** Jasmin opens and carries the problem → speed → trust (slides 1–5).
Daniel takes the new capability → demo → the twist → close (slides 6–10).

---

## JASMIN

### Slide 1 — Title (0:15)
> "Imagine someone hands you a tool that checks whether software can ever
> crash — and then asks you to rebuild it from scratch, and *prove* your
> version still tells the truth. That's our project. I'm Jasmin, this is
> Daniel. Ten minutes, one story: **can you trust a verification tool you
> rewrote yourself?**"

### Slide 2 — Testing only tries some inputs (1:00)
> "The normal way to find bugs is testing: run a program on some inputs, see
> if it breaks. But a program reading just eight bytes has more possible
> inputs than there are stars in the sky — and you only ever try a handful.
> So testing can't tell you a program is *safe*; only that it didn't break
> the times you looked.
>
> There's a stronger idea: **verification**. Instead of trying inputs one by
> one, you treat the unknown input like algebra — like solving x plus three
> equals seven without guessing every number — and ask one question that
> covers *all* inputs: can this program reach a bad state, ever, within k
> steps? If yes, you even get the exact input that does it."

### Slide 3 — The pipeline (0:50)
> "Here's the assembly line. You start with a compiled program — the actual
> machine code, not the source. A tool called **rotor** translates it into a
> mathematical model of the processor running it. A solver searches that
> model for bugs.
>
> Rotor already existed — written by Professor Kirsch's group, in one
> 14-thousand-line C file. Our job: rebuild it, make it better, and add the
> last box — a way to actually *read* the answer. The two blue boxes are
> ours."

### Slide 4 — 1,000× faster + the counting (1:30)
> "So we rebuilt rotor in Rust. And it ran about a thousand times faster —
> selfie's model dropped from *139 seconds* to under a tenth of a second, and
> from 428 megabytes to twenty.
>
> When we showed Professor Kirsch, he didn't celebrate — he got *suspicious*.
> 'Maybe it's faster because it's doing less. Don't trust the number —
> count.' So we did exactly what he taught: we found the one repeated
> operation both tools share — the question 'have I already built this piece
> of the model before?' — and put a counter on it in both.
>
> The C tool answers by walking a list of everything it built so far.
> Eleven-point-seven *billion* comparisons. Our Rust version asks the same
> question with a hash table — one lookup each, 159 thousand total. Same
> question, same answer. A list versus an index. *That's* the whole
> thousand-x — nothing skipped."

### Slide 5 — Can you trust it? 36/36 (1:15, then hand off)
> "But fast is worthless if it's wrong. When we claimed our models matched,
> the professor said: 'There's no way to know that. Check them yourself.'
>
> So we made the *solver* the referee. For the same program, both models must
> reach the *same* bad state at the *exact same step number* — and that step
> count is a fingerprint of the whole execution; one wrong instruction shifts
> it. We ran all 18 of his benchmark programs, under two settings: **36 out
> of 36 identical**. And we trust that bar because every bug we made during
> development failed it *loudly* first.
>
> That's the foundation. But a verifier is only as good as what it can *see*
> and what you can *do* with its answer — and that's Daniel's half."

---

## DANIEL

### Slide 6 — A whole class of bugs was invisible (1:00)
> "Thanks. So here's a gap we found. The solver could only explore input
> typed at the keyboard — standard input. But programs also get input from
> *command-line arguments*: the words after the program's name. In the old
> models, those were frozen. So any bug that only happens for a specific
> argument — like './program CRASH' — was *structurally invisible*. The solver
> wasn't failing to find it; it wasn't allowed to look.
>
> We fixed that. We leave the argument bytes *open* — blanks the solver fills
> in. And this wasn't our idea to invent: the original rotor paper lists
> 'console arguments as symbolic input' as *future work*. We built their
> future-work item."

### Slide 7 — The X/Y bug (1:00)
> "Let me make it concrete. This program crashes only if the first letter of
> argument one is X *and* the first letter of argument two is Y — two specific
> characters, in two *different* arguments, at the same time. No amount of
> keyboard input could ever trigger it; the program never reads the keyboard.
>
> We hand the model to the solver, and in seconds it comes back with: argument
> one starts with X, argument two starts with Y. It *derived the secret* — by
> logic, not guessing. We wrote five of these traps; the solver cracked all
> five, every time with the exact bytes."

### Slide 8 — Visualizer, LIVE DEMO (1:30)
> "There's one catch: the solver's raw answer is thousands of lines of binary
> keyed to internal numbers — unreadable for anyone who didn't build the tool.
> A correct answer nobody can read convinces nobody.
>
> So we built a browser tool that turns it into a film. *(switch to demo)*
> This is the model as a graph. I load the witness for the X/Y bug, press
> play, and you watch the solver's chosen bytes — there's the X, there's the
> Y — flow through memory, into the comparison, until *this* red node, the bad
> state, lights up. The verification result stops being a wall of text and
> becomes a story you can follow. It's online, no install."

*(Demo fallback if the browser misbehaves: the deck's slide 8 box describes
the exact sequence; narrate it and move on — don't fight the laptop.)*

### Slide 9 — The twist: we gave the speed-up back, in his C (1:00)
> "And one last twist we're proud of. The professor's question — 'why is yours
> faster?' — we didn't just answer it, we *proved* it: we took the single
> idea, the hash lookup, and put it back into *his* original C tool. His own
> rotor then ran about ninety-five times faster, producing *byte-identical*
> output, and passed the same 36-out-of-36 equivalence check. So the speed was
> never about Rust versus C — it was one data structure, full stop.
>
> And that's the real lesson. The hard part of rebuilding a verification tool
> isn't writing the code — it's *proving* the new one still tells the truth.
> For us, the methodology *was* the project."

### Slide 10 — Thank you (0:15, both)
> "A thousand times faster, a whole class of bugs now reachable, every result
> checked thirty-six out of thirty-six, and an answer you can actually read.
> Thank you — we're happy to take questions."

---

## Delivery notes
- **Practice once out loud** — the professor said practice in front of a
  mirror. The script is ~1,300 words ≈ 9–10 min at a calm pace.
- **One idea per slide.** If you forget a line, say the *headline* and move on.
- **Hand-off is explicit** (end of slide 5): "...and that's Daniel's half."
- **Demo is the peak** — rehearse the browser steps so it's smooth; have the
  page already open in a tab.
- Backup/Q&A slides (deck part 2) are *only* for questions — see
  `QA_PREP_5min.md`.
