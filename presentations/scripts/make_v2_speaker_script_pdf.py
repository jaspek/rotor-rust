"""
Speaker script for the single comprehensive slide (Contribution v2).
Designed to be spoken — short sentences, no abbreviations, easy on
the tongue. Total runtime ~4 minutes.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer
from reportlab.lib.colors import HexColor

OUT = r"C:\Users\jasko\Programming\Rust\Project01\Contribution_Slide_Speaker_Script.pdf"

INK    = HexColor("#1A1A1A")
SUBTLE = HexColor("#606060")
ACCENT = HexColor("#2A5DA0")
HILITE = HexColor("#FFF8D6")

doc = SimpleDocTemplate(
    OUT, pagesize=A4,
    leftMargin=2.2 * cm, rightMargin=2.2 * cm,
    topMargin=2.0 * cm, bottomMargin=2.0 * cm,
    title="Contribution slide — speaker script",
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
story.append(Paragraph("Contribution slide — speaker script", H_TITLE))
story.append(Paragraph(
    "One slide. About four minutes spoken. Walk the audience through each "
    "section in order. Pause between sections so people can find them on the slide.",
    H_SUB
))

story.extend(block("OPEN",
    "This single slide summarises everything we added to Rotor. The "
    "contribution is one feature — symbolic console arguments. I'll walk "
    "you through it.",
    cue="Pause. Let them register where to look."
))

story.extend(block("BEFORE / AFTER  (the two top bands)",
    "Top of the slide, the two coloured bands. <b>Before</b> our work: when "
    "Rotor built the model of a program, the command-line arguments were "
    "always zero. So if a program had any logic that depended on an "
    "argument — say, a branch on the first character — the model checker "
    "could not reach that branch. It was invisible. <b>After</b> our work: "
    "those argument bytes are open variables. The model checker picks "
    "values for them, and finds inputs that drive the program into a bad "
    "state.",
    cue="Point at the red band for 'before', then the green band for 'after'."
))

story.extend(block("FILES MODIFIED  (left column)",
    "Four places in the codebase. The main file got three new flags. The "
    "configuration file got three matching fields. The core file in the "
    "machine module got one new function — about 160 lines — plus four "
    "short hooks that wire it in. And we added a folder of five test "
    "programs. <b>Everything else in Rotor is unchanged.</b>"
))

story.extend(block("NEW FUNCTION — six phases  (second column)",
    "This is the heart of the feature. The function runs in six phases. "
    "<b>Phase one</b> is pure math — deciding where each byte goes. "
    "<b>Phase two</b> creates an empty stack. <b>Phase three</b> writes the "
    "program name — 'p', 'r', 'o', 'g', null. Concrete bytes. "
    "<b>Phase four</b> — and this is the moment — writes the symbolic "
    "argument bytes. One line of code creates each free byte. <b>Phase "
    "five</b> writes the pointer array. <b>Phase six</b> writes the "
    "argument counter at the stack pointer.",
    cue="Slow down on phase four. That's the line."
))

story.extend(block("USER SURFACE & WIRING  (third column)",
    "Three flags control the feature: the first turns it on; the second "
    "says how many symbolic arguments to provide; the third says how many "
    "bytes of each argument are free. Below the flags, the four hooks "
    "that integrate the new function into the rest of Rotor. The first "
    "picks the initial stack pointer. The second writes that pointer into "
    "the stack pointer register. The third writes the argument counter "
    "into the first argument register. The fourth attaches our stack "
    "value to the model's stack segment."
))

story.extend(block("STACK LAYOUT  (right column visual)",
    "This is what the stack looks like after the function runs. At the "
    "top, the actual argument bytes — yellow ones are free, the model "
    "checker picks them. Below: the program name, padding, the pointer "
    "array, and at the bottom, the argument counter. The stack pointer "
    "points right at it.",
    cue="Trace the diagram top-to-bottom with your finger. Yellow = free."
))

story.extend(block("PIPELINE  (bottom-left)",
    "End to end. A C source file becomes a RISC-V binary, via selfie. "
    "Rotor with our flag turns the binary into a BTOR2 model. The model "
    "checker — btormc — solves the model and produces a witness trace. "
    "From the trace, we read the argument bytes that crash the program. "
    "<b>Only Rotor changed.</b> Selfie and the model checker are "
    "unchanged third-party tools."
))

story.extend(block("VALIDATION  (bottom-middle)",
    "Five benchmark programs. Each one has a bug reachable only through "
    "specific argument bytes. For example, test four needs the first "
    "character of the first argument to be 'X' <i>and</i> the first "
    "character of the second argument to be 'Y' at the same time. The "
    "model checker finds the exact bytes — within seconds, for all five."
))

story.extend(block("DESIGN CHOICE  (bottom-right)",
    "One implementation choice worth knowing — in case anyone asks. We "
    "could have used BTOR2 input nodes for the free bytes. Two reasons we "
    "didn't. First — input nodes are forbidden inside initialisation "
    "expressions, and our bytes live in the stack's initialisation. "
    "Second — input nodes are re-chosen at every step, but argument bytes "
    "are fixed when the program starts. So we use uninitialised state "
    "nodes instead. They are picked once, they persist, and they are "
    "legal where we need them.",
    cue="Optional — only read this aloud if questions push. Otherwise just glance at it."
))

story.extend(block("CLOSE — the mechanism band at the bottom",
    "And that is the whole mechanism, in one line. Each free argument "
    "byte is a BTOR2 state node with no initialisation. The model checker "
    "picks any value from zero to 255, and the value never changes. The "
    "program reads it through an ordinary byte load. <b>Nothing else in "
    "Rotor knows anything is symbolic.</b> That is the entire feature.",
    cue="This is your last sentence. Land it firmly. Then stop talking. Wait for questions.",
    highlight=True
))

doc.build(story)
print(f"Wrote {OUT}")
