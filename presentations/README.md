# presentations/

Work-in-progress materials for course meetings and the final deliverable.
The final deliverables themselves live in the **repository root**
(`Final_Report.pdf`, `Rotor_Overview.pdf`, `Rotor_Presentation.pdf`,
`Symbolic_Arguments_Presentation.pdf`, `paper.tex`).

Everything in this folder is supporting material — drafts, iterations,
speaker scripts, and the generator scripts that produced them.

## Layout

| Folder | Contents | Tracked in git? |
|---|---|---|
| `scripts/` | Python and JS generators that produce the PDFs and decks | **Yes** — reproducibility |
| `pdfs/` | Generated PDFs (iteration drafts, speaker scripts, deep-dives) | No — regenerable from `scripts/` |
| `decks/` | PowerPoint files (.pptx) | No — regenerable from `scripts/` |

## How to regenerate

The generator scripts in `scripts/` are intended to be run from the
project root, with their `OUT_PATH` constants pointing at the final
location. For example:

```bash
python presentations/scripts/make_smooth_transcript_pdf.py
node   presentations/scripts/make_light_session_deck.js
```

Outputs are written into `presentations/pdfs/` or
`presentations/decks/` depending on the script.
