"""
Short, concrete document: what was added to the original Rotor codebase
to support symbolic console arguments. Written in the professor's
vocabulary. Does not explain how the machine works.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, PageBreak,
    Table, TableStyle, Preformatted, KeepTogether,
)
from reportlab.lib.colors import HexColor

OUT_PATH = r"C:\Users\jasko\Programming\Rust\Project01\Rotor_Contribution_Summary.pdf"

# ---- palette -------------------------------------------------------------
INK         = HexColor("#1d232b")
SUBTLE      = HexColor("#5a6470")
ACCENT      = HexColor("#1f5fa1")
RULE        = HexColor("#cfd6dd")
CODE_BG     = HexColor("#f3f4f6")
CODE_BORDER = HexColor("#dde1e6")
TABLE_HEAD  = HexColor("#e8edf2")
KEY_BG      = HexColor("#fffbe6")
KEY_BORDER  = HexColor("#e6dca0")

# ---- doc -----------------------------------------------------------------
doc = SimpleDocTemplate(
    OUT_PATH, pagesize=A4,
    leftMargin=2.0 * cm, rightMargin=2.0 * cm,
    topMargin=1.8 * cm, bottomMargin=1.8 * cm,
    title="Rotor — what was added (symbolic console arguments)",
    author="Project notes",
)
base = getSampleStyleSheet()

H1 = ParagraphStyle("H1", parent=base["Heading1"],
                    fontName="Helvetica-Bold", fontSize=20, leading=24,
                    textColor=INK, spaceBefore=4, spaceAfter=8, keepWithNext=True)
H2 = ParagraphStyle("H2", parent=base["Heading2"],
                    fontName="Helvetica-Bold", fontSize=13.5, leading=17,
                    textColor=ACCENT, spaceBefore=14, spaceAfter=5, keepWithNext=True)
H3 = ParagraphStyle("H3", parent=base["Heading3"],
                    fontName="Helvetica-Bold", fontSize=11, leading=14,
                    textColor=INK, spaceBefore=8, spaceAfter=3, keepWithNext=True)

BODY = ParagraphStyle("Body", parent=base["BodyText"],
                      fontName="Helvetica", fontSize=10, leading=14,
                      textColor=INK, spaceAfter=6, alignment=TA_LEFT)
BULLET = ParagraphStyle("Bullet", parent=BODY,
                        leftIndent=14, bulletIndent=2, spaceAfter=2)
KEY = ParagraphStyle("Key", parent=BODY, fontSize=10, leading=14,
                     leftIndent=10, rightIndent=10,
                     backColor=KEY_BG, borderColor=KEY_BORDER, borderWidth=0.6,
                     borderPadding=(8, 9, 8, 9), spaceBefore=8, spaceAfter=10)
CODE = ParagraphStyle("Code", parent=base["Code"],
                      fontName="Courier", fontSize=8.4, leading=10.8,
                      textColor=INK, leftIndent=0, rightIndent=0,
                      backColor=CODE_BG, borderColor=CODE_BORDER, borderWidth=0.5,
                      borderPadding=(6, 7, 6, 7), spaceBefore=4, spaceAfter=8)


def code(text):
    return KeepTogether([Spacer(1, 2), Preformatted(text.rstrip("\n"), CODE)])


def p(text, style=BODY):
    return Paragraph(text, style)


def bullets(items):
    return [Paragraph(f"&bull;&nbsp;&nbsp;{t}", BULLET) for t in items]


def table(rows, col_widths=None, header=True):
    tbl = Table(rows, colWidths=col_widths, hAlign="LEFT")
    style = [
        ("FONT", (0, 0), (-1, -1), "Helvetica", 9),
        ("TEXTCOLOR", (0, 0), (-1, -1), INK),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LINEABOVE", (0, 0), (-1, 0), 0.6, RULE),
        ("LINEBELOW", (0, -1), (-1, -1), 0.6, RULE),
        ("LINEBELOW", (0, 0), (-1, 0), 0.4, RULE),
        ("LEFTPADDING", (0, 0), (-1, -1), 6),
        ("RIGHTPADDING", (0, 0), (-1, -1), 6),
        ("TOPPADDING", (0, 0), (-1, -1), 4),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 4),
    ]
    if header:
        style += [
            ("FONT", (0, 0), (-1, 0), "Helvetica-Bold", 9),
            ("BACKGROUND", (0, 0), (-1, 0), TABLE_HEAD),
        ]
    tbl.setStyle(TableStyle(style))
    return KeepTogether([Spacer(1, 2), tbl, Spacer(1, 6)])


# ---------------------------------------------------------------------------
story = []

# --- Title ---
story.append(Paragraph("Rotor: what was added", H1))
story.append(Paragraph(
    "A short summary of the contribution to Rotor, framed as a delta against "
    "the original codebase. Vocabulary follows the project framing: Rotor as a "
    "front end, BTOR2 as the model format, witness traces as the output of the "
    "bounded model checker.",
    ParagraphStyle("Sub", parent=BODY, fontSize=10, textColor=SUBTLE, spaceAfter=12)
))

# --- The headline ---
story.append(Paragraph(
    "<b>One feature was added: symbolic console arguments.</b> "
    "In the original Rotor, the BTOR2 model exposed only stdin to the bounded "
    "model checker — argv was always zero, so any program behavior that depends "
    "on command-line arguments was unreachable in the witness trace. "
    "After the change, the user can mark argv as symbolic and the bounded model "
    "checker will return concrete byte values for each argument character that "
    "lead the program into a bad state.",
    KEY,
))

# --- Section 1: what changed (file-level) ---
story.append(Paragraph("1. What changed in the codebase", H2))
story.append(p(
    "The change is small and localized. Below: the files added or modified "
    "for the symbolic-console-arguments feature."
))

story.append(table([
    ["File", "Change", "What it adds"],
    ["main.rs", "modified",
     "Three new CLI flags: --symbolic-argv, --symbolic-argc N, --max-arglen K."],
    ["config.rs", "modified",
     "Three new Config fields that mirror the flags."],
    ["machine/core.rs", "modified",
     "One new function: initialize_symbolic_argv (~160 lines). Three short "
     "additions inside CoreState::new that consume its result."],
    ["benchmarks/argv-tests/", "added",
     "Five small C programs, each containing a bug reachable only through "
     "specific argv bytes."],
], col_widths=[4.2*cm, 2.5*cm, 9.5*cm]))

story.append(Paragraph("Files <i>not</i> touched", H3))
story.append(p(
    "The decoder, the RISC-V instruction handlers, the kernel module that "
    "models syscalls, the property checks, the BTOR2 builder, the printer — "
    "none of these were changed for this feature. The contribution is contained "
    "to the entry-point setup of the model. That is the design intent: "
    "symbolic argv is a property of the initial state, not a special path "
    "through the rest of Rotor."
))

# --- Section 2: how the user invokes it ---
story.append(Paragraph("2. How the feature is used", H2))
story.append(p("Three new flags on the rotor command line:"))
story.append(code(
"""rotor program.elf \\
    --symbolic-argv \\
    --symbolic-argc 2 \\
    --max-arglen 8 \\
    -o program.btor2"""
))
story.append(p(
    "Meaning: the bounded model checker should treat argv[1] and argv[2] as "
    "symbolic (argv[0] remains the literal program name 'prog'), each up to "
    "8 bytes long. Without --symbolic-argv, Rotor behaves exactly as before."
))

# --- Section 3: what the model contains, conceptually ---
story.append(Paragraph("3. What appears in the BTOR2 model", H2))
story.append(p(
    "The BTOR2 model emitted by the new code differs from the original in one "
    "place: the initial value of the stack segment."
))
story.append(Paragraph("Before", H3))
story.append(p(
    "The stack started zero-initialized. argc was zero. argv was a NULL pointer. "
    "Any branch in the program that depended on argv was therefore unreachable "
    "in the bounded model checker."
))
story.append(Paragraph("After", H3))
story.append(p(
    "When --symbolic-argv is set, the stack is initialized with a real argv "
    "layout: argc at the stack pointer, the argv pointer array above it, and "
    "the argument strings above that. Most of these bytes are concrete "
    "(argc, the pointers, argv[0] = 'prog\\0', the null terminators). The bytes "
    "of argv[1..N] are emitted as <b>unconstrained</b> — the bounded model "
    "checker picks them. Register a0 is also set to argc, and register sp is "
    "set to the bottom of this layout, so the program enters main() with the "
    "values it expects from a real OS."
))

story.append(Paragraph(
    "The single mechanism: the bytes of argv[i&gt;0] are emitted as BTOR2 state "
    "nodes with no init expression. State nodes without an init are "
    "unconstrained — the bounded model checker is free to choose any value 0..255 "
    "for each of them. The rest of the model reads them through ordinary "
    "byte loads, so the freedom propagates into whichever instructions consume "
    "the data.",
    KEY,
))

# --- Section 4: the design choice ---
story.append(Paragraph("4. The one non-obvious design choice", H2))
story.append(p(
    "An alternative would have been to use BTOR2 input nodes for the argv "
    "bytes instead of state nodes. Two reasons that was not done:"
))
for b in [
    "BTOR2 forbids input nodes inside init expressions, and the argv bytes have "
    "to live inside the stack segment's init expression so they are present at "
    "step 0.",
    "An input node is re-chosen at every step of the bounded model checker. "
    "argv is set once, when the program starts, and does not change. State "
    "nodes match that semantics; inputs do not.",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))
story.append(p(
    "Uninitialized state nodes are the natural fit: they are picked once at "
    "step 0, they stay that value for the whole run, and they are legal inside "
    "an init expression. The cost in BTOR2 lines is roughly "
    "<font face='Courier'>8 × symbolic_argc × max_arglen</font> — about 64 "
    "extra lines per model at default settings."
))

# --- Section 5: validation ---
story.append(Paragraph("5. How to verify it works", H2))
story.append(p(
    "Five benchmark programs in <font face='Courier'>benchmarks/argv-tests/</font>. "
    "Each one contains a bug reachable only through specific argv bytes:"
))
story.append(code(
"""// test4_multi_arg.c — bug reachable only when argv[1][0]='X' AND argv[2][0]='Y'
uint64_t main(uint64_t argc, uint64_t* argv) {
    if (argc > 2) {
        if (((uint64_t*) *(argv + 1))[0] == 'X')
            if (((uint64_t*) *(argv + 2))[0] == 'Y')
                return 1;
    }
    return 0;
}"""
))
story.append(p("End-to-end run:"))
story.append(code(
"""$ selfie -c test4_multi_arg.c -o test4.m
$ rotor   test4.m  --symbolic-argv --symbolic-argc 2 --max-arglen 8 \\
                   -o test4.btor2
$ btormc  -kmax 200 test4.btor2  >  test4.wit"""
))
story.append(p(
    "The bounded model checker returns a witness trace assigning "
    "<font face='Courier'>argv[1][0] = 'X'</font> and "
    "<font face='Courier'>argv[2][0] = 'Y'</font>, "
    "the only assignment that drives the program into the bad-exit branch. "
    "All five benchmarks behave equivalently."
))

# --- Section 6: pointer to the code ---
story.append(Paragraph("6. Pointers into the code", H2))
story.append(table([
    ["Where", "What to look at"],
    ["main.rs",
     "The three #[arg(long)] attributes at the bottom of the Cli struct. "
     "Look for symbolic_argv, symbolic_argc, max_arglen."],
    ["config.rs",
     "The three matching pub fields on Config."],
    ["machine/core.rs:272-431",
     "fn initialize_symbolic_argv. The whole feature lives here. The phases "
     "are: (1) compute the layout, (2) create the stack, (3) write argv[0]='prog', "
     "(4) write argv[1..N] as free state nodes, (5) write the pointer array, "
     "(6) write argc."],
    ["machine/core.rs:100-105, 115-127, 130-146, 216-223",
     "Four short blocks inside CoreState::new that consume the function's "
     "result: pick the initial SP, write SP into x2, write argc into a0, "
     "attach the stack value to the stack segment."],
    ["benchmarks/argv-tests/",
     "test1_..c through test5_..c — the validating benchmarks."],
], col_widths=[5.6*cm, 10.6*cm]))

# --- Closing ---
story.append(Paragraph("Summary", H2))
story.append(p(
    "<b>One feature, one new function, three CLI flags, three Config fields, "
    "four short hooks in CoreState::new, five benchmark programs.</b> "
    "The rest of Rotor is unchanged. The bounded model checker now produces "
    "witness traces over command-line argument bytes, not just stdin."
))

# ---------------------------------------------------------------------------
doc.build(story)
print(f"Wrote {OUT_PATH}")
