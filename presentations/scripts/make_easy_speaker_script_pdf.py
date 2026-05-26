"""
Listener-friendly speaker script. No abbreviations to read aloud, no
code-symbol shortcuts. Designed to sound natural when spoken.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, KeepTogether,
)
from reportlab.lib.colors import HexColor

OUT = r"C:\Users\jasko\Programming\Rust\Project01\Symbolic_Arguments_Speaker_Script_Easy.pdf"

INK    = HexColor("#1A1A1A")
SUBTLE = HexColor("#606060")
ACCENT = HexColor("#2A5DA0")
HILITE = HexColor("#FFF8D6")

doc = SimpleDocTemplate(
    OUT, pagesize=A4,
    leftMargin=2.2 * cm, rightMargin=2.2 * cm,
    topMargin=2.0 * cm, bottomMargin=2.0 * cm,
    title="Symbolic Arguments — Speaker Script (listener-friendly)",
)
base = getSampleStyleSheet()

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
                     textColor=SUBTLE, italic=True, spaceBefore=2, leftIndent=8)


def slide(num, title, body_html, cue=None, highlight=False):
    parts = [Paragraph(f"SLIDE {num}", SLIDE_NUM),
             Paragraph(title, SLIDE_TITLE)]
    body_style = BODY
    if highlight:
        body_style = ParagraphStyle("BodyHi", parent=BODY,
                                    backColor=HILITE,
                                    borderPadding=(8, 10, 8, 10),
                                    spaceAfter=10)
    parts.append(Paragraph(body_html, body_style))
    if cue:
        parts.append(Paragraph(f"<b>Cue.</b> {cue}", CUE))
    parts.append(Spacer(1, 6))
    return parts


story = []
story.append(Paragraph("Symbolic Arguments — listener-friendly script", H_TITLE))
story.append(Paragraph("Same content as before, rewritten so it sounds natural when spoken aloud. "
                       "Abbreviations expanded, code symbols replaced with words.",
                       H_SUB))

story.extend(slide(1, "Title",
    "This is part two of our work on Rotor. The topic is symbolic arguments, end to end. "
    "I'll walk through every function on the path — from the command line all the way down "
    "to a program reading the first character of its first argument. The goal is for you "
    "to see exactly what was added and where each piece lives."
))

story.extend(slide(2, "What 'symbolic arguments' actually means",
    "The user wants to ask one question — <i>can my program crash, or exit with an error, "
    "or hit some bad state for any command-line argument the user might type?</i> Symbolic "
    "arguments make argv an open question. Each character of every symbolic argument is "
    "left free, and the model checker fills it in to find a bug. Concretely: Rotor lays "
    "out a normal argv on the stack, but instead of writing concrete characters, it writes "
    "'unknown bytes' that the model checker gets to choose. The program reads its arguments "
    "exactly as it would in a real operating system — it doesn't know anything is symbolic.",
    cue="Read the italic question at audience pace. That's the framing that grounds everything else."
))

story.extend(slide(3, "Three command-line flags",
    "Three flags control the feature. The first one — <b>symbolic-argv</b> — turns it on. "
    "Without it, the program runs as if invoked with no arguments. The second — "
    "<b>symbolic-argc N</b> — says how many symbolic arguments to provide. The program name "
    "is always the literal string 'prog', so the total count the program sees is N plus one. "
    "The third — <b>max-arglen K</b> — says how many bytes of each argument are free. Each "
    "string is K plus one bytes counting the null terminator. Once these are parsed, the "
    "main function packs them into a configuration struct, and the command-line layer is "
    "done with symbolic arguments.",
    cue="Say the flag names without the dashes — clearer in speech."
))

story.extend(slide(4, "Three config fields",
    "The three flags become three fields on the configuration struct. That's it for this "
    "file. They're read once, much later, when we set up the core. Nothing else in the "
    "configuration matters for symbolic arguments.",
    cue="Short slide. About fifteen seconds."
))

story.extend(slide(5, "Four hooks in the core setup",
    "The core setup is where the feature gets wired into the machine. Four short blocks. "
    "<b>First</b>: if the symbolic-argv flag is on, we call the new function and use the "
    "stack pointer it returns; otherwise we use a default. <b>Second</b>: we write that "
    "stack pointer value into the stack pointer register. The program enters its main "
    "function with its stack pointer already pointing at the argv layout we built. "
    "<b>Third</b>: we write the argument count into the first argument register — that's "
    "the Linux startup convention. <b>Fourth</b>: we make the stack value built by the "
    "new function the stack segment's starting value. After this line, when the program "
    "reads argv, the layout we built is what it sees."
))

story.extend(slide(6, "The headline function — signature",
    "This is the headline function. It takes the builder, the type information, the "
    "configuration, the top of the stack, and the word size. It returns two things — "
    "the initial stack pointer, and the stack value. The stack pointer says where the "
    "program's stack pointer should start. The stack value says what the stack should "
    "look like at the very first moment of execution. The function has no side effects "
    "beyond producing builder nodes. It runs in six phases, which we'll walk through next."
))

story.extend(slide(7, "Phase one — compute the layout",
    "Phase one is pure arithmetic. No machine state is touched yet. We're just deciding "
    "the address of every byte we're about to write. The picture on the left is what "
    "we're building toward. At the top of the stack, the actual argument bytes — the "
    "program name first, then each symbolic argument. Below that, alignment padding. "
    "Below that, the array of pointers to those strings, terminated by a NULL. And at "
    "the bottom, the argument count itself, with the stack pointer pointing right at it. "
    "The right column shows the variables we compute: the size of each region, where "
    "each region starts, and the stack pointer value we'll return.",
    cue="Walk the diagram top-to-bottom with your finger or laser. Don't read the variable names aloud — point at them."
))

story.extend(slide(8, "Phase two — create the stack",
    "Phase two is two lines. We start with an empty stack and a variable that tracks "
    "our progress. Each subsequent write returns a new value — the stack as it would "
    "look with one additional byte set. We assign that result back to the variable. So "
    "by the end of the function, the variable holds the entire initial stack — built up "
    "from a chain of writes. The reason we start empty rather than zero-filled: bytes "
    "outside argv stay free, so the program may not assume any particular value for "
    "them. In practice the program never reads those bytes."
))

story.extend(slide(9, "Phase three — write the program name",
    "Phase three writes the program name. These are concrete bytes — the program name "
    "is always the literal string 'prog'. We loop over the four characters, write each "
    "one to its address, then write the null terminator. Programs reading the program "
    "name see exactly 'p', 'r', 'o', 'g', null. Nothing symbolic here.",
    cue="Quick slide. The point is to set up the contrast with phase four."
))

story.extend(slide(10, "Phase four — write the symbolic arguments  ★",
    "<b>This is where symbolic-ness is born.</b> Same loop as before, but the value we "
    "write is different. Instead of a constant byte, we create a fresh free eight-bit "
    "value — one line of code does it. That single line is the whole mechanism. There "
    "are exactly <i>symbolic-argc times max-arglen</i> of these in total. Every other "
    "byte on the stack is concrete; only these are free. The null terminator at the end "
    "of each string stays concrete — that way the C string convention is preserved, so "
    "functions like string-length still work.",
    cue="SLOW DOWN. Read the bold sentence carefully. Pause for two beats before moving on. This is the moment of the talk.",
    highlight=True
))

story.extend(slide(11, "Phase five — write the pointer array",
    "Phase five is all concrete again. For each argument string, we write its address "
    "into the pointer array, byte by byte, in little-endian order. Then a NULL pointer "
    "at the end — POSIX, the standard Unix convention, requires the array to be "
    "NULL-terminated. So when the program looks up the first argument, it dereferences "
    "a real address that points at the symbolic string we built in phase four."
))

story.extend(slide(12, "Phase six — write the argument count",
    "Phase six writes the argument count — the integer — at the address the stack "
    "pointer points to. Concrete value, equal to the number of symbolic arguments plus "
    "one. Then we return the stack pointer and the fully built stack value. The "
    "argument count is now in two places — on the stack here, and in the first argument "
    "register from earlier. The startup code can find it either way; both hold the same "
    "value."
))

story.extend(slide(13, "What the running program actually sees",
    "Once the model is set up, the program runs normally. There is no special path for "
    "symbolic data. Take a typical access — the program reads the first character of "
    "the first argument. <b>One</b>: it loads the argv pointer from the stack — returns "
    "the address we wrote in phase five, a concrete address. <b>Two</b>: it loads the "
    "first argument's string address — returns the address of the first symbolic "
    "string. <b>Three</b>: it loads the first character of that string — returns the "
    "free eight-bit value we created in phase four. <b>Four</b>: it branches on that "
    "value. The comparison result is itself free; both sides of the branch enter the "
    "model. <b>Five</b>: the model checker picks values for every symbolic byte that "
    "satisfy whatever chain of branches leads to a bug. <b>Crucially</b> — nothing in "
    "the memory module, the register file, the decoder, or the kernel knows about "
    "symbolic argv. They all just do their normal job.",
    cue="If anyone is going to push back, it's here. The five-step trace is your defence. Hit the last sentence firmly."
))

story.extend(slide(14, "End to end on a real benchmark",
    "Test four is intentionally minimal. It exits with code one only when the first "
    "character of the first argument equals 'X' <i>and</i> the first character of the "
    "second argument equals 'Y' at the same time. <b>Step one</b>: selfie compiles it "
    "to a RISC-V binary. <b>Step two</b>: Rotor, with the three symbolic-argv flags, "
    "produces the BTOR2 model. The core setup is called once. The new function lays "
    "out argv with two arguments of eight free bytes each. The stack pointer, the "
    "argument register, and the stack segment all get wired up. <b>Step three</b>: the "
    "model checker, given up to two hundred steps, finds the assignment — first "
    "character of the first argument equals 'X', first character of the second "
    "argument equals 'Y'. The program reaches return one; exit status is nonzero; the "
    "bad-exit property triggers. The witness trace contains those exact bytes. The "
    "visualizer can then load the model and the witness side by side and step through "
    "it.",
    cue="Concrete numbers — eighty-eight, eighty-nine — are memorable. Say them clearly. This is the proof-it-works slide."
))

story.extend(slide(15, "Recap — every function on the path",
    "Every function on the symbolic-argv path. <b>Command line</b>: the main function "
    "parses the flags and builds the configuration. <b>Pipeline</b>: the model "
    "generator loads the binary and calls the core setup. <b>Core setup</b>: branches "
    "on the flag, calls the new function, writes the stack pointer and the argument "
    "count into the register file, attaches the stack value. <b>Argv layout</b>: the "
    "new function — six phases, all in one file. <b>Runtime reads</b>: ordinary byte "
    "loads — they don't know anything is symbolic, the freedom is already in the "
    "bytes. <b>Properties</b>: defines what counts as a bug; reachability of a bad "
    "property is what the model checker proves or refutes. The whole feature is one "
    "new function plus four short hooks. Every other module is unchanged.",
    cue="Leave this slide up while you take questions. Don't say 'thank you' — let the table close."
))

doc.build(story)
print(f"Wrote {OUT}")
