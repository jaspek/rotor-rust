"""
Speaker script for the direction-feedback slide.
Online, light session, ~90 seconds. Conversational tone.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer
from reportlab.lib.colors import HexColor

OUT = r"C:\Users\jasko\Programming\Rust\Project01\Direction_Feedback_Speaker_Script.pdf"

INK    = HexColor("#1A1A1A")
SUBTLE = HexColor("#606060")
ACCENT = HexColor("#2A5DA0")
HILITE = HexColor("#FFF8D6")

doc = SimpleDocTemplate(
    OUT, pagesize=A4,
    leftMargin=2.2 * cm, rightMargin=2.2 * cm,
    topMargin=2.0 * cm, bottomMargin=2.0 * cm,
    title="Direction-feedback slide — speaker script",
)
base = getSampleStyleSheet()

H_TITLE = ParagraphStyle("HT", parent=base["Heading1"],
                         fontName="Helvetica-Bold", fontSize=22, leading=26,
                         textColor=INK, spaceBefore=0, spaceAfter=4)
H_SUB = ParagraphStyle("HS", parent=base["BodyText"],
                       fontName="Helvetica", fontSize=12, leading=15,
                       textColor=SUBTLE, spaceAfter=14, italic=True)
SECTION = ParagraphStyle("Sec", parent=base["BodyText"],
                         fontName="Helvetica-Bold", fontSize=12, leading=15,
                         textColor=ACCENT, spaceBefore=12, spaceAfter=3,
                         keepWithNext=True)
BODY = ParagraphStyle("Body", parent=base["BodyText"],
                      fontName="Helvetica", fontSize=12.5, leading=18,
                      textColor=INK, spaceAfter=10, alignment=TA_LEFT)
CUE = ParagraphStyle("Cue", parent=BODY, fontSize=10, leading=14,
                     textColor=SUBTLE, italic=True, spaceBefore=2, leftIndent=8)


def block(label, text, cue=None, highlight=False):
    parts = [Paragraph(label, SECTION)]
    body_style = BODY
    if highlight:
        body_style = ParagraphStyle("BH", parent=BODY,
                                    backColor=HILITE,
                                    borderPadding=(8, 10, 8, 10),
                                    spaceAfter=10)
    parts.append(Paragraph(text, body_style))
    if cue:
        parts.append(Paragraph(f"<b>Cue.</b> {cue}", CUE))
    return parts


story = []
story.append(Paragraph("Direction-feedback slide — speaker script", H_TITLE))
story.append(Paragraph(
    "Online light session. ~90 seconds spoken. Conversational. You are asking "
    "him a question, not giving a presentation.",
    H_SUB
))

story.extend(block("OPEN",
    "Quick one — we'd like your input on a direction question. Symbolic "
    "arguments is done. Five benchmarks, each bug found by the model "
    "checker. Symbolic stdin already exists in Rotor, and the heap is free "
    "until written. So the basic symbolic surface is covered.",
    cue="Calm, friendly. Don't open by apologising or hedging — just state where things are."
))

story.extend(block("THE PROBLEM",
    "We have time for one more extension before the final deliverable, and "
    "we don't know which direction to commit to. Three candidates on the "
    "slide. The question is which of them would be most useful for your "
    "SMT-solver work.",
    cue="State the problem plainly. This is the whole frame of the slide."
))

story.extend(block("OPTION 1 — combined argv + stdin",
    "<b>First option</b> — combined argument values and stdin. Let "
    "programs use both at the same time. Use case: a program that takes a "
    "configuration string from its arguments and a data stream from "
    "standard input. Smallest lift, since both pieces already exist."
))

story.extend(block("OPTION 2 — symbolic environment variables",
    "<b>Second option</b> — symbolic environment variables. Let the "
    "getenv function return free bytes. Use case: programs that read "
    "configuration from environment variables, like PATH or custom keys. "
    "Same idea as the arguments, just one layer up the stack. Medium "
    "lift."
))

story.extend(block("OPTION 3 — symbolic file contents",
    "<b>Third option</b> — symbolic file contents. Let read calls from a "
    "file return free bytes. Use case: programs that parse a file and "
    "crash on malformed input. This is the biggest lift — we'd need to "
    "properly model file descriptors and the openat syscall."
))

story.extend(block("THE ASK",
    "Which of these is most useful for your SMT-solver back-end? Or is "
    "there a fourth direction we're missing — something you'd want from "
    "Rotor that we haven't thought of?",
    cue="Stop. Wait. Let him think and answer. Don't fill the silence.",
    highlight=True
))

story.extend(block("IF HE ASKS FOLLOW-UPS",
    "Two things to have ready in case he asks:<br/><br/>"
    "<b>'How quickly can you do each?'</b> — Option 1 is days. Option 2 "
    "is maybe a week. Option 3 is two-plus weeks.<br/><br/>"
    "<b>'What about argc being symbolic?'</b> — We thought about it. It "
    "would mean a dynamic stack layout, which BTOR2 doesn't naturally "
    "support without exploding the model. We'd rather extend the surface "
    "than make argc itself a variable.",
    cue="These are only if he asks. Don't volunteer them."
))

doc.build(story)
print(f"Wrote {OUT}")
