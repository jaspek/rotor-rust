"""
One-page smooth speaker script for the single-slide light session deck.
Plain English, no abbreviations, conversational rhythm.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import SimpleDocTemplate, Paragraph, Spacer
from reportlab.lib.colors import HexColor

OUT = r"C:\Users\jasko\Programming\Rust\Project01\Light_Session_Speaker_Script_v5.pdf"

INK    = HexColor("#1A1A1A")
SUBTLE = HexColor("#606060")
ACCENT = HexColor("#2A5DA0")
HILITE = HexColor("#FFF8D6")

doc = SimpleDocTemplate(
    OUT, pagesize=A4,
    leftMargin=2.2 * cm, rightMargin=2.2 * cm,
    topMargin=1.8 * cm, bottomMargin=1.8 * cm,
    title="Light session — speaker script",
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
                         textColor=ACCENT, spaceBefore=10, spaceAfter=3,
                         keepWithNext=True)
BODY = ParagraphStyle("Body", parent=base["BodyText"],
                      fontName="Helvetica", fontSize=12, leading=18,
                      textColor=INK, spaceAfter=10, alignment=TA_LEFT)
CUE = ParagraphStyle("Cue", parent=BODY, fontSize=10, leading=13,
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
story.append(Paragraph("Speaker script — one slide, about 2.5 minutes", H_TITLE))
story.append(Paragraph(
    "Plain English throughout. Conversational rhythm — sentences flow into each "
    "other rather than 'first… second… third…' Aim for an engineer briefing "
    "a senior colleague.",
    H_SUB
))

story.extend(block("OPEN",
    "So this slide pulls the project together and asks for your read. We are "
    "not proposing new work — we want to know whether what we built lines up "
    "with what you had in mind when we first talked about this.",
    cue="Calm and direct. Eye contact through the camera. Then move to pillar one."
))

story.extend(block("PILLAR ONE — the Rust rewrite",
    "You wanted Rotor rewritten in Rust — easier to read, easier to extend, "
    "easier to use with modern tools. That part is done. The new code is "
    "split into separate files, each handling one job. It produces the same "
    "models as the original on every test program we have, and the models "
    "come out about three-and-a-half times smaller — because the new builder "
    "spots when the same expression appears more than once and writes it only "
    "once."
))

story.extend(block("PILLAR TWO — symbolic command-line arguments",
    "You also wanted the bounded model checker to be able to reason about "
    "command-line input — right now it would have been blind to anything "
    "coming from the command line. That is also done. When the program reads "
    "its command-line arguments, those bytes in memory are unknown — the "
    "bounded model checker picks them. The program reads them through a "
    "normal load, no special handling. We wrote five small test programs "
    "that each have a bug only reachable through a specific input, and the "
    "bounded model checker finds the right characters within seconds on "
    "every one of them.",
    cue="If anyone wants the mechanism in more detail, we have the deep-dive PDF on standby. Don't open it here."
))

story.extend(block("PILLAR THREE — the witness viewer",
    "The third piece is the witness viewer. A witness is the bounded model "
    "checker's answer — it gives you the input that breaks the program, plus "
    "a step-by-step replay of how the program reaches the bad state. The "
    "viewer opens a model and a witness, can show only a slice of the model "
    "instead of thousands of nodes at once, highlights only the parts that "
    "mattered for the bug, and lets you walk the trace one step at a time. "
    "This one is mostly there. What's still being finished is the view for "
    "memory arrays — when the program accesses chunks of memory, we don't "
    "yet display those nicely at each step."
))

story.extend(block("THE ASK",
    "So two questions for you. Does this match what you had in mind? Are we "
    "hitting the bar you wanted, or is there something we missed? And for "
    "the witness viewer specifically — what is the most important thing for "
    "us to finish well? The memory-array view we just mentioned, or something "
    "else? We have time to focus on one thing, and we'd rather focus on the "
    "thing you'd actually use.",
    cue="Stop. Wait. Online silences feel longer than they are — don't fill them.",
    highlight=True
))

story.extend(block("IF HE PROBES — answers ready, do not volunteer",
    "<b>'Show me the viewer.'</b> Open the browser tool, load one of the "
    "argv-test witnesses, walk a few steps.<br/><br/>"
    "<b>'How long until the memory-array piece is done?'</b> A week of "
    "focused work; less if we know which views he cares about.<br/><br/>"
    "<b>'Anything broken you didn't mention?'</b> A couple of edge cases in "
    "the witness parser, and the step-by-step inspector still shows memory "
    "arrays as a placeholder. That's it.",
    cue="Reference material. Skim before the call. Do not read aloud unless asked."
))

doc.build(story)
print(f"Wrote {OUT}")
