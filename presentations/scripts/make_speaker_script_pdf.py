"""
Speaker script for the Symbolic Arguments presentation.
A single document, formatted for reading aloud at a podium.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, PageBreak, KeepTogether,
)
from reportlab.lib.colors import HexColor

OUT = r"C:\Users\jasko\Programming\Rust\Project01\Symbolic_Arguments_Speaker_Script.pdf"

# --- palette ---------------------------------------------------------------
INK    = HexColor("#1A1A1A")
SUBTLE = HexColor("#606060")
ACCENT = HexColor("#2A5DA0")
RULE   = HexColor("#C8C8C8")
HILITE = HexColor("#FFF8D6")  # subtle yellow for the headline-moment slide

# --- doc -------------------------------------------------------------------
doc = SimpleDocTemplate(
    OUT, pagesize=A4,
    leftMargin=2.2 * cm, rightMargin=2.2 * cm,
    topMargin=2.0 * cm, bottomMargin=2.0 * cm,
    title="Symbolic Arguments — Speaker Script",
)
base = getSampleStyleSheet()

# Larger-than-usual body text, generous leading. Designed to be read from
# 50-70cm away at a podium.
H_TITLE = ParagraphStyle("HT", parent=base["Heading1"],
                         fontName="Helvetica-Bold", fontSize=22, leading=26,
                         textColor=INK, spaceBefore=0, spaceAfter=4)
H_SUB = ParagraphStyle("HS", parent=base["BodyText"],
                       fontName="Helvetica", fontSize=12, leading=15,
                       textColor=SUBTLE, spaceAfter=14, italic=True)

SLIDE_NUM = ParagraphStyle("SN", parent=base["BodyText"],
                           fontName="Helvetica-Bold", fontSize=11, leading=14,
                           textColor=ACCENT, spaceBefore=10, spaceAfter=2,
                           keepWithNext=True)
SLIDE_TITLE = ParagraphStyle("ST", parent=base["Heading2"],
                             fontName="Helvetica-Bold", fontSize=15, leading=19,
                             textColor=INK, spaceBefore=0, spaceAfter=6,
                             keepWithNext=True)

BODY = ParagraphStyle("Body", parent=base["BodyText"],
                      fontName="Helvetica", fontSize=12.5, leading=18,
                      textColor=INK, spaceAfter=10, alignment=TA_LEFT)

CUE = ParagraphStyle("Cue", parent=BODY, fontSize=10, leading=14,
                     textColor=SUBTLE, italic=True, spaceBefore=2,
                     leftIndent=8, borderPadding=0)

# --- helpers ---------------------------------------------------------------
def slide(num, title, body_html, cue=None, highlight=False):
    """One slide entry — number, title, the spoken text, optional cue."""
    parts = []
    parts.append(Paragraph(f"SLIDE {num}", SLIDE_NUM))
    parts.append(Paragraph(title, SLIDE_TITLE))
    body_style = BODY
    if highlight:
        body_style = ParagraphStyle("BodyHi", parent=BODY,
                                    backColor=HILITE,
                                    borderPadding=(8, 10, 8, 10),
                                    spaceAfter=10)
    parts.append(Paragraph(body_html, body_style))
    if cue:
        parts.append(Paragraph(f"<b>Delivery cue.</b> {cue}", CUE))
    parts.append(Spacer(1, 6))
    return parts


# --- story -----------------------------------------------------------------
story = []

story.append(Paragraph("Symbolic Arguments", H_TITLE))
story.append(Paragraph("Speaker script — Part 2 of the deep dive (15 slides, ~10 minutes)",
                       H_SUB))

# ---------------------------------------------------------------------------
story.extend(slide(1, "Title",
    "This is Part 2 of our work on Rotor — symbolic arguments, end to end. "
    "I'll walk through every function on the path from the command line down "
    "to a program reading <b>argv[1][0]</b> in its main. The goal is for you to "
    "see exactly what was added and where each piece lives.",
    cue="Open calmly. Don't rush — set the frame for the next ten minutes."
))

story.extend(slide(2, "What 'symbolic arguments' actually means",
    "The user wants to ask one question — <i>can my program crash, or exit "
    "nonzero, or hit some bad state for any command-line argument the user "
    "might type?</i> Symbolic arguments make argv an open question. Each "
    "character of <b>argv[1..N]</b> is left free; the solver fills it in to "
    "find a bug. Concretely: Rotor lays out a normal argv on the stack, but "
    "instead of writing concrete characters, it writes \"unknown bytes\" that "
    "the solver gets to choose. The program reads argv exactly as it would in "
    "a real OS — it doesn't know anything is symbolic.",
    cue="Read the italicised question at the user's pace; that's the framing the prof asked you to lead with."
))

story.extend(slide(3, "2.1  CLI flags — main.rs",
    "Three flags control the feature. <b>--symbolic-argv</b> turns it on. "
    "Without it, argv is empty and the program runs as if invoked with no "
    "arguments. <b>--symbolic-argc N</b> says how many symbolic arguments to "
    "provide; argv[0] is always the literal string \"prog\", so the total argc "
    "the program sees is N plus one. <b>--max-arglen K</b> says how many bytes "
    "of each argument are free — each string is K plus one bytes counting the "
    "null terminator. Once these are parsed, main packs them into a Config and "
    "the CLI never touches symbolic argv again."
))

story.extend(slide(4, "2.2  Config fields — config.rs",
    "The three flags become three fields on Config. That's it for this file. "
    "They're read once, much later, by <b>CoreState::new</b>. Nothing else in "
    "config.rs matters for symbolic arguments.",
    cue="Short slide. Move quickly — about 15 seconds."
))

story.extend(slide(5, "2.3  CoreState::new — four hooks",
    "CoreState::new is where the feature gets wired into the machine. Four "
    "short blocks. <b>(a)</b> If --symbolic-argv is on, call the new function "
    "and use the SP it returns; otherwise default SP to one word below stack "
    "top. <b>(b)</b> Write that SP value into register x2 — RISC-V calling "
    "convention. The program enters main with its stack pointer already "
    "pointing at the argv layout we built. <b>(c)</b> Write argc into "
    "register a0; that's the Linux startup convention. <b>(d)</b> Bind the "
    "stack value built by the new function to be the stack segment's starting "
    "value. After this line, dereferencing **(sp+8) finds our argv."
))

story.extend(slide(6, "2.4  initialize_symbolic_argv — signature",
    "This is the headline function. Inputs: the builder, the sorts, the "
    "config, the stack top, and the word size. Output: a tuple — the initial "
    "stack pointer and the stack value. The stack pointer is where SP should "
    "start; the stack value is what the stack array should look like at step "
    "0. The function has no side effects beyond producing builder nodes. It "
    "runs in six phases, which we'll walk through next."
))

story.extend(slide(7, "Phase 1 — compute the layout",
    "Phase 1 is pure arithmetic. No machine state is touched yet. We're just "
    "deciding the address of every argv byte, every pointer, and argc. The "
    "picture on the left is what we're building toward: at the top of the "
    "stack the actual argv bytes — argv[0] first, then argv[1] through "
    "argv[N]. Below that, alignment padding. Below that, the pointer array, "
    "terminated by NULL — that's POSIX. And at the bottom, argc itself, with "
    "SP pointing right at it. The right column shows the variables we "
    "compute: the size of each region, where each region starts, and the SP "
    "value we'll return.",
    cue="Use your laser pointer or finger to walk the stack diagram top-to-bottom while you speak."
))

story.extend(slide(8, "Phase 2 — create the stack array",
    "Phase 2 is two lines. We start with an empty state and a variable "
    "<b>current</b> that tracks the stack value as we go. Each subsequent "
    "<b>builder.write</b> returns a new value with one byte updated; we "
    "assign that result back to current. So by the end of the function, "
    "current holds the entire initial stack — built up from a chain of "
    "writes. The reason we start empty rather than zero-filled: the bytes we "
    "never touch stay free, so the program may not assume any particular "
    "value for them. In practice the program never reads those bytes."
))

story.extend(slide(9, 'Phase 3 — argv[0] = "prog\\0"',
    "Phase 3 writes argv[0]. These are concrete bytes — the program name is "
    "always the literal string \"prog\\0\". We loop over the four characters, "
    "write each one to its address, then write the null terminator. Programs "
    "reading argv[0] see exactly the characters 'p', 'r', 'o', 'g', null. "
    "Nothing symbolic here.",
    cue="Quick slide. The point is to set up the contrast with Phase 4."
))

story.extend(slide(10, "Phase 4 — argv[1..N]   (the symbolic part)  ★",
    "<b>This is where symbolic-ness is born.</b> Same loop shape as Phase 3, "
    "but the value we write is different: instead of a constant byte, we "
    "create a fresh free 8-bit state with no init expression — "
    "<b>let sym_byte = builder.state(...)</b>. That single line is the whole "
    "mechanism. There are <b>symbolic_argc × max_arglen</b> of these in "
    "total. Every other byte on the stack is concrete; only these are free. "
    "The null terminator at the end of each string stays concrete so C "
    "string semantics are preserved — strlen still works.",
    cue="SLOW DOWN here. This is the moment of the talk. Read the bold line aloud and pause for two beats before moving on.",
    highlight=True
))

story.extend(slide(11, "Phase 5 — write the pointer area",
    "Phase 5 is all concrete again. For each argv string, we write its "
    "address into the pointer array, byte by byte, little-endian. Then a "
    "NULL pointer at the end — POSIX requires the array to be "
    "NULL-terminated. So when the program does argv[1], it dereferences a "
    "real address that points at the symbolic string we built in Phase 4."
))

story.extend(slide(12, "Phase 6 — write argc at SP",
    "Phase 6 writes argc itself, byte by byte, at the address SP points to. "
    "Concrete integer, equal to symbolic_argc + 1. Then we return SP and the "
    "fully built stack value. argc is now in two places: on the stack here, "
    "and in register a0 from hook (c). RISC-V startup code can find it "
    "either way; both hold the same value."
))

story.extend(slide(13, "2.5  What the running program actually sees",
    "Once the model is set up, the program runs normally. There is no "
    "special path for symbolic data. Take a typical access — the program "
    "reads argv[1][0]. <b>One</b>: it loads argv from the stack, returns "
    "the pointer-area value — a concrete address. <b>Two</b>: it loads "
    "argv[1], returns the address of the first symbolic string. <b>Three</b>: "
    "it loads argv[1][0] with lb — that's Memory::load_byte on the stack, "
    "returning the free 8-bit value we created in Phase 4. <b>Four</b>: it "
    "branches on that value; the comparison result is itself free, both "
    "sides are part of the model. <b>Five</b>: the solver picks values for "
    "every symbolic byte that satisfy whatever chain of branches leads to a "
    "bug. <b>Crucially</b>, nothing in Memory, RegisterFile, the decoder, "
    "or the kernel knows about symbolic argv. They all just do their normal "
    "job.",
    cue="If the prof is going to push back anywhere, it's here. The five-step trace is your defence. Hit the last sentence firmly."
))

story.extend(slide(14, "2.6  End-to-end on a real benchmark",
    "<b>test4_multi_arg.c</b> is intentionally minimal — it exits 1 only "
    "when argv[1][0] equals 'X' <i>and</i> argv[2][0] equals 'Y' at the "
    "same time. <b>Step one</b>, selfie compiles it to a RISC-V binary. "
    "<b>Step two</b>, rotor with the three symbolic-argv flags produces the "
    "BTOR2 model — CoreState::new gets called once, initialize_symbolic_argv "
    "lays out argv with two arguments of 8 free bytes each, and SP, a0, and "
    "the stack segment all get wired up. <b>Step three</b>, btormc with a "
    "kmax of 200 finds the assignment — argv[1][0] = 88 for 'X', "
    "argv[2][0] = 89 for 'Y'. The program reaches return 1, exit status is "
    "nonzero, the bad-exit property triggers. The witness trace contains "
    "those exact bytes. The visualizer can then load the model and the "
    "witness side by side and step through it.",
    cue="Concrete numbers (88, 89) are memorable — say them clearly. This is the proof-it-works slide."
))

story.extend(slide(15, "2.7  Recap",
    "Every function on the symbolic-argv path. <b>CLI</b>: main parses the "
    "flags and builds the Config. <b>Pipeline</b>: model_rotor loads the "
    "binary and calls CoreState::new. <b>Core setup</b>: CoreState::new "
    "branches on the flag, calls the new function, writes SP and argc into "
    "the register file, attaches the stack value. <b>Argv layout</b>: "
    "initialize_symbolic_argv — six phases, all in this file. <b>Runtime "
    "reads</b>: ordinary byte loads — they don't know anything is symbolic, "
    "the freedom is already in the bytes. <b>Properties</b>: rotor_properties "
    "defines what counts as a bug; reachability of a bad property is what "
    "btormc proves or refutes. The whole feature is one new function plus "
    "four short hooks. Every other module is unchanged.",
    cue="Leave this slide up while you take questions. Don't say \"thank you\" — let the table do the closing."
))

# ---------------------------------------------------------------------------
doc.build(story)
print(f"Wrote {OUT}")
