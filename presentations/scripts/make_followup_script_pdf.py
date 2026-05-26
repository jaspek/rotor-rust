"""
Speaker script for Meeting_Followup_Deck.pptx (two slides).
Plain English. Conversational rhythm. About 5 minutes total.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer, PageBreak
from reportlab.lib.colors import HexColor

OUT = r"C:\Users\jasko\Programming\Rust\Project01\Meeting_Followup_Script.pdf"

INK    = HexColor("#1A1A1A")
SUBTLE = HexColor("#606060")
ACCENT = HexColor("#2A5DA0")
HILITE = HexColor("#FFF8D6")
WARN   = HexColor("#FCEDED")
WARN_B = HexColor("#8B2A2A")

doc = SimpleDocTemplate(
    OUT, pagesize=A4,
    leftMargin=2.2 * cm, rightMargin=2.2 * cm,
    topMargin=1.8 * cm, bottomMargin=1.8 * cm,
    title="Meeting follow-up — speaker script",
)
base = getSampleStyleSheet()

H_TITLE = ParagraphStyle("HT", parent=base["Heading1"],
                         fontName="Helvetica-Bold", fontSize=20, leading=24,
                         textColor=INK, spaceBefore=0, spaceAfter=4)
H_SUB = ParagraphStyle("HS", parent=base["BodyText"],
                       fontName="Helvetica", fontSize=11, leading=14,
                       textColor=SUBTLE, spaceAfter=14, italic=True)
SECTION = ParagraphStyle("Sec", parent=base["BodyText"],
                         fontName="Helvetica-Bold", fontSize=12, leading=15,
                         textColor=ACCENT, spaceBefore=12, spaceAfter=4,
                         keepWithNext=True)
BODY = ParagraphStyle("Body", parent=base["BodyText"],
                      fontName="Helvetica", fontSize=12, leading=17.5,
                      textColor=INK, spaceAfter=10, alignment=TA_LEFT)
CUE = ParagraphStyle("Cue", parent=BODY, fontSize=10, leading=13,
                     textColor=SUBTLE, italic=True, spaceBefore=2, leftIndent=8)


def block(label, text, cue=None, highlight=False, warn=False):
    parts = [Paragraph(label, SECTION)]
    body_style = BODY
    if highlight:
        body_style = ParagraphStyle("BH", parent=BODY,
                                    backColor=HILITE,
                                    borderPadding=(8, 10, 8, 10),
                                    spaceAfter=10)
    if warn:
        body_style = ParagraphStyle("BW", parent=BODY,
                                    backColor=WARN, borderColor=WARN_B, borderWidth=0.6,
                                    borderPadding=(8, 10, 8, 10),
                                    spaceAfter=10)
    parts.append(Paragraph(text, body_style))
    if cue:
        parts.append(Paragraph(f"<b>Cue.</b> {cue}", CUE))
    return parts


story = []

# ============ Slide 1 ===============================================
story.append(Paragraph("Slide 1 — Project status (3 pillars)", H_TITLE))
story.append(Paragraph(
    "About 2 minutes. Same shape as last time — three pieces, what we wanted, "
    "what we built. Use it to set context before the selfie numbers on slide 2.",
    H_SUB
))

story.extend(block("OPENING",
    "Quick recap of where the three pieces stand. Two are done, one is "
    "mostly done. Then I'll walk through your selfie challenge from last "
    "meeting on slide two."
))

story.extend(block("PILLAR ONE — the Rust rewrite",
    "First piece, the Rust rewrite. The new code is split into separate "
    "files, each handling one job. Same models as the original on every "
    "test program we have — and about 2,250 times faster to generate. The "
    "speed number comes from your selfie challenge; we'll go through it on "
    "the next slide.",
    cue="That 2,250× number is your hook for slide 2 — don't elaborate yet, just plant it."
))

story.extend(block("PILLAR TWO — symbolic command-line arguments",
    "Second piece, symbolic command-line arguments. The argument bytes are "
    "open in the model — the bounded model checker picks them. The program "
    "reads them like any other byte in memory. Five test programs, each "
    "with a bug only reachable through a specific input; the model checker "
    "finds the right characters within seconds on every one."
))

story.extend(block("PILLAR THREE — the witness viewer",
    "Third piece, the witness viewer. A witness is the bounded model "
    "checker's answer — the input that breaks the program plus a "
    "step-by-step replay. We built a browser tool that opens a model and a "
    "witness, shows a slice of the model, highlights what mattered for the "
    "bug, and walks the trace one step at a time. Still being finished: "
    "showing the contents of memory nicely at each step."
))

story.extend(block("BRIDGE",
    "Now to your challenge — modelling all of selfie."
))

story.append(PageBreak())

# ============ Slide 2 ===============================================
story.append(Paragraph("Slide 2 — Response to the selfie challenge", H_TITLE))
story.append(Paragraph(
    "About 3 minutes. Numbers first, complexity second, the honest finding "
    "third. Lead with measurements, not arguments.",
    H_SUB
))

story.extend(block("OPENING THE NUMBERS",
    "Same input — selfie self-compiled into a forty-three-thousand-instruction "
    "RISC-U binary — handed to both rotors. C Rotor took 106 seconds and "
    "used 431 megabytes of memory. Rust Rotor took 47 milliseconds and used "
    "20 megabytes. About 2,250 times faster, about 21 times less memory.",
    cue="Pause after this. Let those numbers sit before moving to the explanation."
))

story.extend(block("WHY — the complexity story",
    "The reason is on the left half of the slide. Both rotors use the same "
    "primitive structural deduplication check — they spot when the same "
    "expression appears more than once and reuse it. The C version does "
    "that check with a linear scan, so the total cost is quadratic in the "
    "number of nodes. That's why you turned the check off in the "
    "binary-loading part of C Rotor. The Rust version uses a HashMap, so "
    "the per-node cost is constant and the total cost is linear. We can "
    "leave the check on everywhere — including the binary-loading part. "
    "That's where the speedup comes from."
))

story.extend(block("★ THE HONEST FINDING",
    "Now the part on the right of the slide. Before claiming the smaller "
    "output as a clean win, we ran an equivalence check. On all 18 "
    "standard benchmarks, the bounded model checker returns the same "
    "verdict from both rotors' outputs at depth 35. So functionally they "
    "agree. <b>But</b> — and this is the honest part — C Rotor emits 24 "
    "bad-state properties per benchmark; our Rust Rotor emits only 3. So "
    "part of the 3.4× size reduction comes from CSE doing its job, and "
    "part comes from our model checking fewer things. The speed and "
    "memory wins are real and stand. The size-reduction claim needs this "
    "caveat.",
    cue="This is the slide's emotional centre. Slow down. Say it confidently — finding it before he did is a win, not a weakness.",
    warn=True
))

story.extend(block("NEXT STEPS",
    "Three things from here. First, investigate the 21 missing property "
    "checks and add them, so the models become semantically equivalent in "
    "what they check. Second, re-measure size and equivalence once we "
    "reach parity. Third, finish the witness viewer's memory-contents "
    "view."
))

story.extend(block("CLOSE",
    "That's where we are. Speed and memory wins are solid. We found a "
    "limitation in our own work before bringing it to you, and we have a "
    "plan to close it.",
    cue="Stop. Don't fill the silence. Let him respond."
))

doc.build(story)
print(f"Wrote {OUT}")
