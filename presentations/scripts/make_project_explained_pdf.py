"""
Plain-language explanation of the entire Rotor-Rust project.
Audience: no prior knowledge of formal verification.
Output: Rotor_Project_Explained.pdf at the project root.
"""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, PageBreak, Table, TableStyle,
    Preformatted, KeepTogether,
)
from reportlab.lib.colors import HexColor

OUT = r"C:\Users\jasko\Programming\Rust\Project01\Rotor_Project_Explained.pdf"

INK    = HexColor("#1A1D26")
SOFT   = HexColor("#4A5160")
MUTED  = HexColor("#7A8095")
ACCENT = HexColor("#2A5DA0")
RULE   = HexColor("#C9CEDA")
CODEBG = HexColor("#F2F4F8")
KEYBG  = HexColor("#FFF8DF")
KEYBRD = HexColor("#E3D9A5")
THEAD  = HexColor("#E8EDF5")

doc = SimpleDocTemplate(
    OUT, pagesize=A4,
    leftMargin=2.3 * cm, rightMargin=2.3 * cm,
    topMargin=2.0 * cm, bottomMargin=2.0 * cm,
    title="The Rotor-Rust Project, Explained",
    author="Jasmin Begic & Daniel Wassie",
)
ss = getSampleStyleSheet()

TITLE = ParagraphStyle("T", parent=ss["Title"], fontName="Helvetica-Bold",
                       fontSize=24, leading=29, textColor=INK, spaceAfter=4,
                       alignment=TA_LEFT)
SUB = ParagraphStyle("S", parent=ss["BodyText"], fontName="Helvetica",
                     fontSize=12, leading=16, textColor=MUTED, spaceAfter=18)
H1 = ParagraphStyle("H1", parent=ss["Heading1"], fontName="Helvetica-Bold",
                    fontSize=16, leading=20, textColor=ACCENT,
                    spaceBefore=18, spaceAfter=8, keepWithNext=True)
H2 = ParagraphStyle("H2", parent=ss["Heading2"], fontName="Helvetica-Bold",
                    fontSize=12.5, leading=16, textColor=INK,
                    spaceBefore=12, spaceAfter=5, keepWithNext=True)
BODY = ParagraphStyle("B", parent=ss["BodyText"], fontName="Helvetica",
                      fontSize=11, leading=16.5, textColor=INK, spaceAfter=9)
KEY = ParagraphStyle("K", parent=BODY, backColor=KEYBG, borderColor=KEYBRD,
                     borderWidth=0.8, borderPadding=(8, 10, 8, 10),
                     spaceBefore=6, spaceAfter=12)
CODE = ParagraphStyle("C", parent=ss["Code"], fontName="Courier", fontSize=9,
                      leading=12, textColor=INK, backColor=CODEBG,
                      borderColor=RULE, borderWidth=0.5,
                      borderPadding=(7, 8, 7, 8), spaceBefore=4, spaceAfter=10)
BUL = ParagraphStyle("UL", parent=BODY, leftIndent=14, bulletIndent=2,
                     spaceAfter=4)

def p(t, s=BODY): return Paragraph(t, s)
def bullets(items): return [Paragraph(f"&bull;&nbsp;&nbsp;{t}", BUL) for t in items]
def code(t): return KeepTogether([Preformatted(t.rstrip("\n"), CODE)])

def table(rows, widths=None):
    t = Table(rows, colWidths=widths, hAlign="LEFT")
    t.setStyle(TableStyle([
        ("FONT", (0, 0), (-1, -1), "Helvetica", 9.5),
        ("FONT", (0, 0), (-1, 0), "Helvetica-Bold", 9.5),
        ("TEXTCOLOR", (0, 0), (-1, -1), INK),
        ("BACKGROUND", (0, 0), (-1, 0), THEAD),
        ("VALIGN", (0, 0), (-1, -1), "TOP"),
        ("LINEABOVE", (0, 0), (-1, 0), 0.7, RULE),
        ("LINEBELOW", (0, -1), (-1, -1), 0.7, RULE),
        ("LINEBELOW", (0, 0), (-1, 0), 0.5, RULE),
        ("ROWBACKGROUNDS", (0, 1), (-1, -1), [None, HexColor("#F7F9FC")]),
        ("LEFTPADDING", (0, 0), (-1, -1), 7),
        ("RIGHTPADDING", (0, 0), (-1, -1), 7),
        ("TOPPADDING", (0, 0), (-1, -1), 5),
        ("BOTTOMPADDING", (0, 0), (-1, -1), 5),
    ]))
    return KeepTogether([Spacer(1, 2), t, Spacer(1, 8)])

story = []

# ============================ COVER ============================
story += [
    Paragraph("The Rotor-Rust Project, Explained", TITLE),
    Paragraph("A plain-language guide to what this project is, the problem it "
              "solves, and how its three parts work together — written for "
              "readers with no background in formal verification.", SUB),

    p("This project asks a deceptively simple question: <b>“Is there any input "
      "that makes my program crash?”</b> Not “did it crash when I tried it,” "
      "but “could it ever crash, for anything a user might type.” The project "
      "rebuilds a research tool called <b>Rotor</b> in the Rust programming "
      "language, teaches it to reason about command-line arguments (something "
      "the original could not do), and adds a browser tool that turns the "
      "answer — normally an unreadable wall of text — into something a person "
      "can actually watch, step by step."),

    Paragraph("Three parts, one sentence each", H2),
    *bullets([
        "<b>Rotor in Rust</b> — a translator that turns a compiled program "
        "into a mathematical model a solver can analyze, rebuilt from a single "
        "14,000-line C file into a clean, fast, modular Rust program.",
        "<b>Symbolic command-line arguments</b> — a new capability: the solver "
        "can now <i>search over everything a user could type on the command "
        "line</i>, instead of treating those inputs as fixed.",
        "<b>The witness visualizer</b> — a browser app that replays the "
        "solver's answer like a film: which instruction ran, what changed, "
        "and how the program reached the bug.",
    ]),
]

# ============================ PROBLEM ============================
story += [
    Paragraph("1 · The problem: testing can only try some inputs", H1),

    p("The usual way to find bugs is testing: run the program with some "
      "inputs and see if it misbehaves. Testing is useful, but it has a "
      "built-in blind spot — it can only ever try a tiny fraction of the "
      "possible inputs. A program that reads just eight bytes of input has "
      "more possible inputs than there are stars in the observable universe. "
      "If the one input that triggers the crash isn't among the ones you "
      "tried, testing tells you nothing about it."),

    p("There is a different approach, called <b>verification</b>: instead of "
      "running the program over and over, you describe the program as "
      "mathematics and let a specialized tool — a <b>solver</b> — reason about "
      "<i>all</i> inputs at once. The solver doesn't try inputs one by one. "
      "It treats the unknown input as algebra, the way you solve "
      "“x + 3 = 7” for x without trying every number. If any input can reach "
      "the crash, the solver finds one and shows it to you. If no input can "
      "(within a given number of steps), the solver proves that, too."),

    Paragraph(
        "<b>The core trade.</b> Testing answers “did it break when I tried "
        "it?” Verification answers “can it break at all, for any input, "
        "within N steps?” — and when the answer is yes, it hands you the "
        "exact input that does it.", KEY),

    p("The catch is that solvers do not understand programs — they understand "
      "logic formulas. Someone has to translate the program into a form the "
      "solver accepts. That translator is what Rotor is."),
]

# ============================ WHAT ROTOR IS ============================
story += [
    Paragraph("2 · What Rotor is", H1),

    p("Rotor is a translator. It takes a <b>compiled program</b> — the actual "
      "binary instructions a RISC-V processor would execute, not the source "
      "code — and produces a precise mathematical description of the machine "
      "running it. That description is written in a standard format called "
      "<b>BTOR2</b>, which bounded model checkers (a kind of solver) accept "
      "as input. The original Rotor was written by Prof. Christoph Kirsch's "
      "group at the University of Salzburg as part of the selfie project."),

    p("The mathematical model describes the machine the way you would "
      "describe a board game: the <b>state</b> (where all the pieces are) and "
      "the <b>rules</b> (what moves are allowed). For a computer, the state "
      "is the program counter, the 32 processor registers, and the memory "
      "(code, data, heap, and stack). The rules say exactly how one "
      "instruction transforms the current state into the next one. Run the "
      "rules over and over and you have execution — except the model never "
      "actually runs; it just <i>describes</i> every possible run."),

    p("On top of the state and rules, the model lists <b>bad states</b> — "
      "situations that should never happen: dividing by zero, reading memory "
      "that doesn't belong to the program, exiting with an error code, and "
      "twenty-odd more. The solver's job is to answer one question: "
      "<i>“Starting from the initial state and following the rules, can the "
      "machine ever reach a bad state within k steps — for any input?”</i>"),

    Paragraph("Why rebuild it in Rust?", H2),
    p("The original Rotor is one 14,000-line C file — powerful, but hard to "
      "read, extend, or build upon. This project rebuilt it as a modular Rust "
      "program where each concern (building the model, simulating the "
      "machine, decoding instructions, the safety checks) lives in its own "
      "file. The rebuild also turned out dramatically faster: modeling "
      "selfie itself (a 43,000-instruction program) takes the C version "
      "about 106 seconds and 431 MB of memory; the Rust version does the "
      "same job in 47 milliseconds using 20 MB. The speedup comes from one "
      "data-structure choice (a hash table instead of a linear list for "
      "detecting duplicate formula pieces) — not from cutting corners: the "
      "Rust version performs the same checks, and its models have been "
      "verified to behave identically (more on that in section 7)."),
]

# ============================ SYMBOLIC ARGV ============================
story += [
    Paragraph("3 · Why symbolic command-line arguments were added", H1),

    p("Here is the gap this project fills. In the original Rotor, the solver "
      "could only explore inputs the program reads from <b>standard input</b> "
      "(the keyboard stream, “stdin”). But programs take input another way "
      "too: <b>command-line arguments</b> — the words you type after the "
      "program's name, like <font face='Courier'>./program hello world</font>. "
      "In the original models those arguments were fixed: the model always "
      "described a machine started with no arguments at all."),

    p("The consequence: any bug that only triggers for a specific "
      "command-line argument was <i>structurally invisible</i>. The solver "
      "wasn't failing to find such bugs — it was never even allowed to look. "
      "The part of the input space where those bugs live simply did not "
      "exist in the model."),

    Paragraph("What “symbolic” means", H2),
    p("When a program starts, the operating system writes the argument "
      "characters into the program's memory (on the stack) as ordinary "
      "bytes, along with a count and pointers to each argument string. The "
      "new <font face='Courier'>--symbolic-argv</font> mode builds that same "
      "memory layout — but instead of writing fixed characters, it leaves "
      "the argument bytes <b>open</b>: each one becomes an unknown the "
      "solver is free to choose. Think of a Sudoku grid where most cells are "
      "filled in (the program, the pointers, the count) and a few are blank "
      "(the argument characters). The solver fills in the blanks — looking "
      "specifically for values that drive the program into a bad state."),

    p("Crucially, the program inside the model has no idea anything is "
      "special. It reads its arguments through perfectly ordinary memory "
      "loads, exactly as it would on real hardware. Nothing else in Rotor "
      "had to change — the decoder, the memory system, and the safety checks "
      "are untouched. “Symbolic argv” is purely a property of how the "
      "machine's initial memory is described."),

    Paragraph("What capability this adds, concretely", H2),
    p("The project includes five small C programs, each hiding a bug that "
      "only fires for specific argument values. Three of them "
      "<b>cannot be found through stdin at all</b> — the programs never read "
      "stdin, so the original Rotor had no way to reach their bugs:"),

    table([
        ["Test program", "The hidden bug fires when…", "Solver finds"],
        ["test1_crash_string", 'argv[1] equals "CRASH"', "the 5 exact letters"],
        ["test2_numeric_overflow", "argv[1] starts with “AB” (→ ÷0)", "0x41, 0x42"],
        ["test3_length_dependent", "argv[1] is exactly 7 characters", "any 7 bytes"],
        ["test4_multi_arg", "argv[1][0]='X' AND argv[2][0]='Y'", "both at once"],
        ["test5_checksum", "first 4 bytes sum to 400", "e.g. 100+100+100+100"],
    ], widths=[4.3*cm, 6.6*cm, 4.2*cm]),

    p("On every one of these, the bounded model checker — given the symbolic "
      "model — finds the exact triggering characters within seconds. Test 4 "
      "is the most striking: the bug needs two specific characters in two "
      "<i>different</i> arguments simultaneously. No amount of stdin "
      "exploration could ever reach it; with symbolic argv it falls out of "
      "one solver run."),
]

# ============================ WORKFLOW ============================
story += [
    PageBreak(),
    Paragraph("4 · The complete workflow, start to finish", H1),

    p("Here is the entire journey from a C program to a verified answer, "
      "using test4 (the two-argument bug) as the running example."),

    Paragraph("Step 1 — Compile", H2),
    p("The C program is compiled into a RISC-V binary by <b>selfie</b> "
      "(Prof. Kirsch's teaching compiler) or gcc. From here on, nothing "
      "looks at the source code — Rotor works on the real machine "
      "instructions, so what gets verified is what actually runs."),
    code("$ selfie -c test4_multi_arg.c -o test4.m"),

    Paragraph("Step 2 — Translate (Rotor in Rust)", H2),
    p("Rotor reads the binary and emits the BTOR2 model: the machine state, "
      "the transition rules for every instruction in the program, the "
      "24 bad-state checks — and, because of the flags below, an argv area "
      "on the stack with two 8-byte arguments left open for the solver."),
    code("$ rotor test4.m --symbolic-argv --num-symbolic-args 2 \\\n"
         "        --max-arglen 8 --exit-code 1 -o test4.btor2"),

    Paragraph("Step 3 — Search (the bounded model checker)", H2),
    p("The solver, <b>btormc</b>, unrolls the rules step by step — “what are "
      "all reachable states after 1 step? after 2? after 3?…” — up to a "
      "chosen bound, and at each step asks whether any choice of the open "
      "bytes reaches a bad state. This is the “bounded” in bounded model "
      "checking: the guarantee covers all executions up to that many steps."),
    code("$ btormc -kmax 200 test4.btor2 > test4.wit"),

    Paragraph("Step 4 — The answer: a witness", H2),
    p("If a bad state is reachable, btormc prints <b>sat</b> (“satisfiable — "
      "yes, it can happen”) followed by a <b>witness</b>: the exact values "
      "of every open byte, plus a step-by-step record of the machine's "
      "states on the way to the bug. For test4 the witness contains "
      "precisely <font face='Courier'>argv[1][0]='X'</font> and "
      "<font face='Courier'>argv[2][0]='Y'</font> — the solver discovered, "
      "by logic alone, the only two characters that break the program. If "
      "nothing bad is reachable within the bound, it reports that instead — "
      "a guarantee, not a guess."),

    Paragraph("Step 5 — Understand it (the visualizer)", H2),
    p("The witness is honest but unreadable — thousands of lines of binary "
      "values keyed to internal node numbers. The visualizer turns it back "
      "into something human: load the model and the witness in a browser, "
      "press play, and watch the bug happen."),

    table([
        ["Stage", "Tool", "In", "Out"],
        ["Compile", "selfie / gcc", "C source", "RISC-V binary"],
        ["Translate", "Rotor (Rust) — this project", "binary", "BTOR2 model"],
        ["Search", "btormc (solver)", "model", "witness / “unreachable”"],
        ["Understand", "Visualizer — this project", "model + witness", "interactive replay"],
    ], widths=[2.6*cm, 5.6*cm, 3.4*cm, 4.0*cm]),
]

# ============================ VISUALIZER ============================
story += [
    Paragraph("5 · The visualizer: making the answer readable", H1),

    p("A verification result you can't read might as well not exist. The "
      "third part of the project is a browser application (no installation, "
      "also hosted online) that displays the model as an interactive graph: "
      "every piece of the machine is a node — green rounded boxes for state, "
      "red octagons for bad states, diamonds for constants — and the wires "
      "between them show what depends on what."),

    p("When you load a witness next to the model, the playback controls "
      "appear. Step through the trace and the display shows, at every step, "
      "which values changed, what the solver chose for the open input bytes, "
      "and — at the final step — which bad state fired. Because real models "
      "have a hundred thousand nodes, the visualizer offers focus tools: "
      "show only the slice of the graph that influences one chosen node "
      "(the “cone of influence”), limit the display depth, collapse groups "
      "of similar nodes into one, search by name, and export images. Twelve "
      "built-in examples — including all five argv tests with their solved "
      "witnesses — load with one click."),

    Paragraph(
        "<b>Why this matters:</b> with the visualizer, “the solver found a "
        "bug” stops being a wall of text and becomes a story anyone can "
        "follow: <i>here</i> are the two characters it chose, <i>here</i> is "
        "the comparison that went the wrong way, <i>here</i> is the exit "
        "with the error code.", KEY),
]

# ============================ HOW PARTS FIT ============================
story += [
    Paragraph("6 · How the three parts work together", H1),

    p("The three parts are stages of one pipeline, each making the next one "
      "possible or meaningful:"),

    *bullets([
        "<b>Rotor in Rust</b> produces the model. Its speed matters "
        "practically: at 47 milliseconds per model, regenerating after every "
        "change is free, which is what made the careful step-by-step "
        "verification of the project itself feasible.",
        "<b>Symbolic argv</b> widens what the model lets the solver explore. "
        "Same machine, same rules — but the initial memory now contains open "
        "bytes, so the solver searches over every possible command line "
        "instead of one fixed one.",
        "<b>The visualizer</b> closes the loop for humans. The solver's raw "
        "answer references the model's internal structure; the visualizer "
        "joins witness and model back together into a replay a person can "
        "watch and question.",
    ]),

    p("Remove any one stage and the story breaks: without the translator "
      "there is no model; without symbolic argv a whole class of bugs is "
      "invisible; without the visualizer the answer exists but persuades "
      "no one."),
]

# ============================ TRUST ============================
story += [
    Paragraph("7 · Can the new tool be trusted? Checking against the original", H1),

    p("A rewrite of a verification tool faces an awkward question: how do "
      "you know the <i>new</i> translator describes machines correctly? "
      "Claiming “it looks right” is not acceptable in this field — the whole "
      "point of verification is not having to take anyone's word."),

    p("The project's answer is to hold the Rust version up against the "
      "original C version, program by program, and demand that an "
      "independent referee — btormc itself — cannot tell the two models "
      "apart. For each benchmark, both rotors translate the same binary, "
      "the solver checks both models deeply, and the results must agree on "
      "<b>which</b> bad state is reached and at <b>exactly which step</b>. "
      "Every benchmark completed so far matches exactly — for instance, the "
      "division-by-zero program reaches that bad state at step 76 in both "
      "models, and the recursion-heavy benchmarks (hundreds of instructions "
      "through nested calls) match step-for-step as well. Getting there "
      "required making the Rust machine genuinely faithful: memory starts "
      "zeroed, the stack carries a real argv image at boot, the read system "
      "call delivers one input byte per machine step exactly like the "
      "original, and all 24 safety checks were re-derived from the original "
      "source line by line."),

    p("This evidence is reproducible: the comparison script and its results "
      "ship in the repository, so anyone can rerun the whole experiment and "
      "check the table themselves."),
]

# ============================ GLOSSARY ============================
story += [
    Paragraph("8 · Small glossary", H1),
    table([
        ["Term", "Plain meaning"],
        ["RISC-V", "An open instruction set — the vocabulary of machine "
                   "instructions the modeled processor executes."],
        ["BTOR2", "The standard text format for the mathematical machine "
                  "models; what Rotor writes and solvers read."],
        ["Bounded model checking", "Checking all possible executions up to a "
                                    "fixed number of steps (the bound k)."],
        ["btormc", "The bounded model checker (solver) used here."],
        ["Symbolic", "Left open for the solver to choose, instead of fixed "
                     "to one concrete value."],
        ["Bad state", "A situation that should never occur — division by "
                      "zero, an illegal memory access, exiting with the "
                      "target error code, etc."],
        ["Witness", "The solver's proof that a bad state is reachable: the "
                    "chosen input values plus the step-by-step run that "
                    "reaches it."],
        ["selfie", "Prof. Kirsch's minimal teaching system (compiler + "
                   "emulator) that the original Rotor is built on."],
    ], widths=[4.2*cm, 11.4*cm]),

    Spacer(1, 10),
    p("<i>Project by Jasmin Begic &amp; Daniel Wassie — Advanced Systems "
      "Engineering, University of Salzburg, supervised by Prof. Christoph "
      "Kirsch. Source: github.com/jaspek/rotor-rust · Live visualizer: "
      "jaspek.github.io/rotor-rust</i>", SUB),
]

doc.build(story)
print(f"Wrote {OUT}")
