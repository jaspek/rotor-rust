# Speaker script — the storytelling cut

*Pairs with `Rotor_Presentation_Art03.pdf` (the three-act deck). The slides
are pure image; the words live here.*

**The voice.** Tell it the way Marco Pierre White tells a story at the Oxford
Union — slowly, warmly, plainly. Short sentences. Let the silences breathe.
You are not selling; you are letting people in on something you love. Treat
the tool that came before you with respect, the way a young cook talks about
the chef who taught him. The whole talk has one quiet spine, and it is his
line: *"perfection is lots of little things done well."* Say it like you mean
it, because in this work it happens to be literally true.

**Suggested split.** Jasmin takes **Act I** and **Act II**; Daniel takes
**Act III** and shares the close. Or take an act each. It is a story either
way — hand it over like you are passing a plate.

**Pacing.** Roughly 10–12 minutes. *(Italics are stage directions, not
spoken.)* The `→` marks a slide change.

---

## Opening — slide 1

*→ Title. Let it sit for a second before you speak.*

> "We were handed something rather beautiful, and rather frightening. A tool
> that can look at a piece of software and tell you — not guess, *tell* you —
> whether it can ever crash. And we were asked to rebuild it. And then to do
> the harder thing: to prove that our version still tells the truth.
>
> I'm Jasmin, this is Daniel. I'd like to tell you the story in three parts.
> The question. The craft. And the plate."

---

## I. The Question — *symbolic execution*

### Act card — slide 2

*→ "I. The Question." Don't rush off it.*

> "The first part is about a question. A single question, asked the right way."

### You cannot taste every dish — slide 3

*→ The field of inputs, the one bright point hiding among them.*

> "You see, the ordinary way to find a fault in a program is to test it. You
> give it an input, you watch what it does. And that is fine, as far as it
> goes. But it does not go very far.
>
> *(slower)* A program that reads just eight bytes has more possible inputs
> than there are stars you can see tonight. You try a handful of them. And the
> one input that breaks it — the one — can sit anywhere in all that darkness,
> and you would walk straight past it and never know.
>
> Testing can tell you a program broke. It can never tell you it is safe.
> That always troubled me. There had to be a better way to ask."

### One question of all of them — slide 4

*→ The single beam falling over every input at once.*

> "And there is. Instead of trying inputs one at a time, for the rest of your
> life — you ask one question that covers all of them at once. *Can this
> program reach a bad state, for any input, within so many steps?*
>
> You stop treating the unknown input as a guess. You treat it as the unknown.
> Like algebra. You let it be every value at the same time, and you let the
> mathematics carry it. If the answer comes back yes, it hands you the exact
> input that does the damage. If it comes back no, you have something testing
> can never give you — a guarantee.
>
> That is the whole idea. Everything after this is just craftsmanship in
> service of that one question."

### The frozen command line opens — slide 5

*→ The padlocks: two shut, the rest springing open into light.*

> "Now. The tool we inherited could only ask that question about one kind of
> input — what you type at the keyboard while the program runs. And that left
> a door shut.
>
> Because programs take input another way, too. The words you put after the
> program's name when you start it — the arguments. *(a small gesture)* In the
> old model those words were frozen. Fixed. Set in stone before the question
> was ever asked. So any fault that needed a particular argument — the solver
> wasn't failing to find it. It was *not allowed to look*. The door was shut.
>
> So we opened it. We leave those bytes open — blanks, for the solver to fill.
> And I'll be honest with you: this was the original authors' own wish. They
> wrote it down as the work still to be done. We were lucky enough to be the
> ones who got to do it."

### The secret revealed — slide 6

*→ The two dials, X and Y, the detonation of light between them.*

> "Let me show you what that buys you. We wrote a little program that hides a
> secret. It behaves perfectly — unless the first letter of its first argument
> is X, *and* the first letter of its second argument is Y. Both. At the same
> time.
>
> Think about that. Two particular letters, in two different places, that have
> to meet. You could sit at a keyboard for a thousand years and never stumble
> on it.
>
> *(quietly)* We handed the model to the solver. And in a few seconds it came
> back and said: first argument starts with X. Second argument starts with Y.
> It had never been told. It worked it out. By reason. That is the question,
> doing what only it can do."

---

## II. The Craft — *the three rotors*

### Act card — slide 7

*→ "II. The Craft."*

> "The second part is about craft. And about humility, actually — about how
> the fast thing was never the clever thing."

### The speed — slide 8

*→ The comet. "1000×."*

> "We rebuilt the tool in Rust. And it ran about a thousand times faster. A
> model that took the old one a hundred and thirty-nine seconds, ours built in
> under a tenth of a second. In a twentieth of the memory.
>
> *(a beat — almost wary)* Now. When something is suddenly a thousand times
> faster, a sensible person doesn't celebrate. A sensible person gets
> suspicious. Is it faster because it's better — or faster because it's
> quietly doing less? That suspicion is the most important thing I'll say
> today. So we didn't trust the number. We went and counted."

### Don't trust the number — count — slide 9

*→ The tower of comparisons beside the single bright probe.*

> "Both tools, the old one and ours, do one thing over and over, millions of
> times. They ask: *have I already built this little piece of the model
> before?* It's the bookkeeping question. Nothing glamorous.
>
> The old tool answers it by walking a list. Top to bottom. Every time. We
> added a counter, and we let it run, and it came back with a number I still
> find hard to say out loud — *(slow)* nearly twelve billion comparisons.
> Ours answers the same question with an index — a hundred and fifty-nine
> thousand lookups. *(let it land)*
>
> Same question. Same answer. A list, walked — against an index. That is the
> entire speed-up. Nothing was skipped. Nothing was cheapened."

### The three rotors — slide 10  *(the heart of this part)*

*→ The three glowing pedestals: Original C, Hash-consed C, Rust.*

> "And here is where I want to be very fair to the tool that came before us.
> Because the easy story is 'the old thing was slow, we made a fast new
> thing.' That story is a lie, and it's an ungrateful one. Let me give you the
> true one. There are really three rotors here.
>
> *(gesture left)* On the left, the **original**. Written in C, years ago, by
> people who knew exactly what they were doing. A hundred and thirty-nine
> seconds. It builds three million little pieces and walks that list — eleven
> billion comparisons. It is not crude. It is a master with one old habit.
>
> *(gesture right)* On the right, **ours**, in Rust. A tenth of a second.
> Twenty megabytes. It builds a hundred and fifty-nine thousand pieces,
> because it checks before it builds, and it never makes the duplicates in the
> first place. The apprentice who took the lesson all the way.
>
> *(gesture centre — slow down here)* But the one in the middle is the one I'm
> proudest of. That is the **original C** — the master himself — taught the
> single new habit. The index instead of the list. One small change. And he
> went from a hundred and thirty-nine seconds to one and a half. Ninety-five
> times faster. And — this is the part — his output came out *byte for byte
> identical*. Not similar. Identical.
>
> *(quietly, the spine of the talk)* So it was never Rust against C. It was
> never the language at all. It was one small thing, done well. Perfection, a
> wise man said, is lots of little things done well. This is one of them, and
> we found it by respecting the thing we were given enough to measure it
> honestly."

### Proved in C — slide 11

*→ Twin suns, the plasma arc, "95×, byte-identical."*

> "And we didn't just say that. We proved it where it's hardest to argue. We
> took our one change and we put it back into the original — into its own
> kitchen, in its own language. Same code. Same compiler. Ninety-five times
> faster, and the very same model, character for character. *(a breath)* When
> the idea survives in the other man's hands, you know the idea was real, and
> not just your own cleverness."

---

## III. The Plate — *the visualizer*

### Act card — slide 12

*→ "III. The Plate."*

> "The last part is the smallest, and it might be the one that matters most.
> Because a great answer that no one can receive isn't worth much."

### An answer nobody can read — slide 13

*→ The wall of binary.*

> "When the solver finds a fault, it is completely, perfectly right. And it
> tells you so like this. *(let them look at the wall of digits)* Thousands of
> lines of binary, every one tied to a number that means something only to the
> machine. It is correct. It is true. And it is no use to a single human being
> in this room.
>
> *(gently)* A correct answer that nobody can read convinces nobody. I think
> that's true of cooking as well — it doesn't matter how right you are in the
> kitchen if you can't put it on a plate the guest actually wants to eat."

### Made watchable — slide 14

*→ The model as a glowing graph; the red VIOLATED star at the top.*

> "So we plated it. We built a little tool, in the browser, that turns that
> wall of numbers into something you can simply look at. The model becomes a
> graph. Each light is a piece of the machine. And we replay the solver's
> answer across it, step by step, until — there — that red star at the top
> lights up: *violated.* Fed by exactly the path that read the bytes the
> solver chose.
>
> The proof stops being a wall of text and becomes something you can point a
> finger at and say: *there. That's how it breaks.* It's online, with twelve
> examples, for anyone who's curious. That's the plate."

---

## Close — slide 15

*→ The aurora, the constellation, "proving it still tells the truth."*

> "So — three parts. A question, asked of every input at once. A craft, whose
> speed turned out to be one small thing done well, and done with respect for
> what came before. And a plate, so the answer can actually be received.
>
> *(slow, no hurry to finish)* We were asked to make a verification tool
> faster. We did. But the fast part was the easy part. The hard part — the
> whole craft, really — was proving that the new one still tells the truth.
> *(a beat)* That was the work. Thank you."

---

### If you lose your place

Three anchors carry the whole talk:
**one question, not a billion guesses · one small thing done well · an answer
you can actually read.** Say the one you're nearest to, and the story finds
its feet again.
