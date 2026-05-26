"""
Smooth transcript for the single-slide Light Session deck.
For the presenter to read from. No abbreviations, natural rhythm,
fixed-vs-symbolic framing in pillar 2 (slide stays as-is).
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer
from reportlab.lib.colors import HexColor

OUT = r"C:\Users\jasko\Programming\Rust\Project01\Light_Session_Smooth_Transcript.pdf"

INK    = HexColor("#1A1A1A")
SUBTLE = HexColor("#606060")
ACCENT = HexColor("#2A5DA0")
HILITE = HexColor("#FFF8D6")
SOFT   = HexColor("#F2F4F7")

doc = SimpleDocTemplate(
    OUT, pagesize=A4,
    leftMargin=2.2 * cm, rightMargin=2.2 * cm,
    topMargin=1.8 * cm, bottomMargin=1.8 * cm,
    title="Light session — smooth transcript",
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
                      fontName="Helvetica", fontSize=12.5, leading=18,
                      textColor=INK, spaceAfter=10, alignment=TA_LEFT)
NOTE = ParagraphStyle("Note", parent=BODY, fontSize=10, leading=13,
                      textColor=SUBTLE, italic=True, spaceBefore=4,
                      leftIndent=10, rightIndent=10,
                      backColor=SOFT, borderPadding=(6, 8, 6, 8))
CUE = ParagraphStyle("Cue", parent=BODY, fontSize=10, leading=13,
                     textColor=SUBTLE, italic=True, spaceBefore=2, leftIndent=8)


def block(label, text, cue=None, highlight=False, note=None):
    parts = [Paragraph(label, SECTION)]
    body_style = BODY
    if highlight:
        body_style = ParagraphStyle("BH", parent=BODY,
                                    backColor=HILITE,
                                    borderPadding=(8, 10, 8, 10),
                                    spaceAfter=10)
    parts.append(Paragraph(text, body_style))
    if note:
        parts.append(Paragraph(note, NOTE))
    if cue:
        parts.append(Paragraph(f"<b>Cue.</b> {cue}", CUE))
    return parts


story = []
story.append(Paragraph("Smooth transcript — one slide, about 2.5 minutes", H_TITLE))
story.append(Paragraph(
    "For the presenter. Flows in continuous speech, not bullet points. "
    "Slide stays as-is — pillar two uses a slightly tighter framing aloud "
    "than the slide displays.",
    H_SUB
))

story.extend(block("OPENING",
    "So this slide pulls the project together. Three pieces we added to "
    "your Rotor project — and for each one, what we wanted, and what we "
    "built. We're not asking for new work; we want your read on whether "
    "this matches what you had in mind.",
    cue="Eye contact through the camera. Calm and direct. Then go to pillar one."
))

story.extend(block("PILLAR ONE — the Rust rewrite",
    "The first piece is the Rust rewrite. You wanted Rotor easier to "
    "read, easier to extend, easier to use with modern tools. That part "
    "is done. The new code is split into separate files, each handling "
    "one part of the work — one for building the model, one for "
    "simulating the machine, one for decoding the instructions, one for "
    "the safety checks. It produces the same models as the original on "
    "every test program we have. And the models come out about "
    "three-and-a-half times smaller, because the new builder doesn't "
    "write the same expression twice."
))

story.extend(block("PILLAR TWO — symbolic command-line arguments",
    "The second piece is symbolic command-line arguments. Before this, "
    "the command-line input in the model was fixed — always empty. So "
    "the bounded model checker had nothing to vary, and any branch that "
    "depended on what the user typed simply couldn't fire. What we did "
    "is leave those bytes open. When the program reads its command-line "
    "arguments, the bounded model checker picks the values. The program "
    "reads them like any other byte in memory — no special handling "
    "anywhere else in Rotor. We wrote five small test programs, each "
    "with a bug only reachable through a specific input, and the model "
    "checker finds the right characters within seconds on every one of "
    "them.",
    note="This softens the slide's 'blind' wording without changing the slide — uses the fixed-vs-open framing.",
    cue="Slow down here. This is the piece the audience needs the most words to follow."
))

story.extend(block("PILLAR THREE — the witness viewer",
    "The third piece is the witness viewer. A witness is the bounded "
    "model checker's answer — the input that breaks the program, plus "
    "a step-by-step replay of how the program gets to the bad state. We "
    "built a small browser tool that opens a model and a witness file. "
    "It can show only a slice of the model, instead of thousands of "
    "nodes at once. It highlights only the parts that mattered for the "
    "bug. And it walks through the trace one step at a time. This one's "
    "mostly there — what's still being finished is the view for memory "
    "contents. When the program reads or writes a chunk of memory, we "
    "don't yet display that nicely at each step."
))

story.extend(block("THE ASK",
    "So two questions for you. Does this match what you had in mind? "
    "Are we hitting the bar you wanted, or is there a gap we missed? "
    "And for the witness viewer specifically — what is the most "
    "important thing for us to finish well? The memory view we just "
    "mentioned, or something else entirely? We have time to focus on "
    "one thing, and we'd rather focus on the thing you'd actually use.",
    cue="Stop. Wait. Online silences feel longer than they are — don't fill them.",
    highlight=True
))

story.extend(block("DELIVERY NOTES",
    "<b>Slow down on Pillar 2.</b> The fixed-vs-open framing carries the slide.<br/><br/>"
    "<b>Pause between the two questions in the ask.</b> Otherwise they blur and he answers only the second.<br/><br/>"
    "<b>Do not skip the 'no new work' line in the opening.</b> Without it, the slide reads like a pitch instead of a sanity check.<br/><br/>"
    "<b>If he interrupts during a pillar, let him.</b> The transcript flows fine if you have to restart at any pillar boundary.",
    cue="Read once before the call. Don't say any of this aloud during the session."
))

doc.build(story)
print(f"Wrote {OUT}")
