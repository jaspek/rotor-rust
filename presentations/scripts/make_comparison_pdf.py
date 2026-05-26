"""Generate a PDF of the C-vs-Rust rotor comparison and symbolic-execution deep dive."""

from reportlab.lib.pagesizes import A4
from reportlab.lib.styles import getSampleStyleSheet, ParagraphStyle
from reportlab.lib.enums import TA_LEFT
from reportlab.lib.units import cm
from reportlab.lib import colors
from reportlab.platypus import (
    SimpleDocTemplate, Paragraph, Spacer, PageBreak,
    Table, TableStyle, Preformatted, KeepTogether,
)
from reportlab.lib.colors import HexColor

# ---------------------------------------------------------------------------
# Output
# ---------------------------------------------------------------------------
OUT_PATH = r"C:\Users\jasko\Programming\Rust\Project01\Rotor_C_vs_Rust_Comparison.pdf"

# ---------------------------------------------------------------------------
# Palette (Charcoal Minimal — readable, professional)
# ---------------------------------------------------------------------------
INK         = HexColor("#212121")
SUBTLE      = HexColor("#5a6470")
ACCENT      = HexColor("#1f5fa1")
RULE        = HexColor("#cfd6dd")
CODE_BG     = HexColor("#f3f4f6")
CODE_BORDER = HexColor("#dde1e6")
TABLE_HEAD  = HexColor("#e8edf2")

# ---------------------------------------------------------------------------
# Document
# ---------------------------------------------------------------------------
doc = SimpleDocTemplate(
    OUT_PATH,
    pagesize=A4,
    leftMargin=2.0 * cm, rightMargin=2.0 * cm,
    topMargin=1.8 * cm, bottomMargin=1.8 * cm,
    title="Rotor — C vs Rust comparison and symbolic execution deep dive",
    author="Project notes",
)

styles = getSampleStyleSheet()

H1 = ParagraphStyle("H1", parent=styles["Heading1"],
                    fontName="Helvetica-Bold", fontSize=18, leading=22,
                    textColor=INK, spaceBefore=10, spaceAfter=8, keepWithNext=True)
H2 = ParagraphStyle("H2", parent=styles["Heading2"],
                    fontName="Helvetica-Bold", fontSize=14, leading=18,
                    textColor=ACCENT, spaceBefore=14, spaceAfter=6, keepWithNext=True)
H3 = ParagraphStyle("H3", parent=styles["Heading3"],
                    fontName="Helvetica-Bold", fontSize=11.5, leading=15,
                    textColor=INK, spaceBefore=10, spaceAfter=4, keepWithNext=True)
H4 = ParagraphStyle("H4", parent=styles["Heading4"],
                    fontName="Helvetica-Bold", fontSize=10.5, leading=14,
                    textColor=SUBTLE, spaceBefore=8, spaceAfter=3, keepWithNext=True)
BODY = ParagraphStyle("Body", parent=styles["BodyText"],
                      fontName="Helvetica", fontSize=10, leading=14,
                      textColor=INK, spaceAfter=6, alignment=TA_LEFT)
BODY_SM = ParagraphStyle("BodySm", parent=BODY, fontSize=9, leading=12.5)
NOTE = ParagraphStyle("Note", parent=BODY, fontSize=9.5, leading=13,
                      textColor=SUBTLE, leftIndent=10, rightIndent=10,
                      borderColor=RULE, borderWidth=0,
                      backColor=HexColor("#f7f8fa"),
                      borderPadding=(6, 8, 6, 8), spaceBefore=4, spaceAfter=8)
BULLET = ParagraphStyle("Bullet", parent=BODY,
                        leftIndent=14, bulletIndent=2, spaceAfter=2)

# Code style — small, monospaced, light background, soft border
CODE = ParagraphStyle("Code", parent=styles["Code"],
                      fontName="Courier", fontSize=8.2, leading=10.5,
                      textColor=INK, leftIndent=0, rightIndent=0,
                      backColor=CODE_BG, borderColor=CODE_BORDER, borderWidth=0.5,
                      borderPadding=(6, 7, 6, 7), spaceBefore=4, spaceAfter=8)

def code(text):
    """A code block. Uses Preformatted for fixed-width fidelity."""
    block = Preformatted(text.rstrip("\n"), CODE)
    return KeepTogether([Spacer(1, 2), block])

def p(text, style=BODY):
    return Paragraph(text, style)

def bullets(items):
    return [Paragraph(f"&bull;&nbsp;&nbsp;{t}", BULLET) for t in items]

def section_table(rows, col_widths=None, header=True):
    tbl = Table(rows, colWidths=col_widths, hAlign="LEFT")
    base = [
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
        base += [
            ("FONT", (0, 0), (-1, 0), "Helvetica-Bold", 9),
            ("BACKGROUND", (0, 0), (-1, 0), TABLE_HEAD),
        ]
    tbl.setStyle(TableStyle(base))
    return KeepTogether([Spacer(1, 2), tbl, Spacer(1, 6)])

# ---------------------------------------------------------------------------
# Build story
# ---------------------------------------------------------------------------
story = []

# --- Title ---
story.append(Paragraph("Rotor: C vs Rust comparison &amp; symbolic execution deep dive",
                       ParagraphStyle("Title", parent=H1, fontSize=22, leading=26,
                                      spaceAfter=4)))
story.append(Paragraph("Feature-by-feature comparison of the original <font face='Courier'>rotor.c</font> "
                       "and the Rust reimplementation, with a detailed look at how symbolic execution "
                       "is implemented.",
                       ParagraphStyle("Sub", parent=BODY, fontSize=10.5,
                                      textColor=SUBTLE, spaceAfter=14)))

story.append(p("A literal line-by-line comparison of 13,822 lines of C against 5,400 lines of Rust would "
               "be useless to read. The comparison below is by feature: same functionality, both files, "
               "real code, with annotation of every important difference."))

# --- Section: setup ---
story.append(Paragraph("Setup: what the two files even look like", H2))

setup_rows = [
    ["", "C (rotor.c)", "Rust (rotor/src/)"],
    ["Total size", "13,822 lines, one file", "~5,400 lines, 26 files"],
    ["Language", "C* (selfie subset: only uint64_t, no structs)", "Rust 2021"],
    ["State storage", "hundreds of file-level globals", "structs passed by reference"],
    ["Per-core state", "parallel arrays indexed by core int",
                       "Vec<CoreState>, each holds its own fields"],
    ["BTOR2 node",
     "heap array of 15 uint64_t slots, accessed via get_*/set_* functions",
     "enum Op inside a Node struct, indexed by NodeId into an arena"],
    ["Deduplication",
     "optional reuse_lines flag, linear scan",
     "always-on HashMap lookup in the builder"],
    ["Output",
     "direct dprintf to fd while building",
     "two-pass: build node graph, then printer assigns nids and emits"],
]
story.append(section_table(setup_rows, col_widths=[3.4*cm, 6.8*cm, 6.8*cm]))
story.append(p("That is the architectural difference in one table. Everything below follows from those rows."))

# --- Feature 1 ---
story.append(Paragraph("Feature 1 — How a single BTOR2 node is created", H2))
story.append(p("This is the smallest unit. If you understand this, everything bigger composes from it."))

story.append(Paragraph("C — rotor.c:74-104", H3))
story.append(code(
"""uint64_t* allocate_line() {
  // returns a 15-slot heap array
}

uint64_t  get_nid(uint64_t* line)      { return *line; }
char*     get_op(uint64_t* line)       { return (char*)     *(line + 1); }
uint64_t* get_sid(uint64_t* line)      { return (uint64_t*) *(line + 2); }
uint64_t* get_arg1(uint64_t* line)     { return (uint64_t*) *(line + 3); }
uint64_t* get_arg2(uint64_t* line)     { return (uint64_t*) *(line + 4); }
uint64_t* get_arg3(uint64_t* line)     { return (uint64_t*) *(line + 5); }
char*     get_comment(uint64_t* line)  { return (char*)     *(line + 6); }
... // 8 more slots
void set_nid(uint64_t* line, uint64_t nid)       { *line = nid; }
... // matching setters

uint64_t* new_line(char* op, uint64_t* sid, uint64_t* arg1, uint64_t* arg2,
                   uint64_t* arg3, char* comment);"""
))
story.append(p("A node is a flat heap array. Field access is <font face='Courier'>*(line + offset)</font> "
               "with the offset hard-coded. The C* subset doesn't allow structs, so this is the only way. "
               "Type information is reconstructed at every call site by casting a "
               "<font face='Courier'>uint64_t</font> to whatever pointer type you want."))

story.append(Paragraph("Rust — btor2/node.rs", H3))
story.append(code(
"""#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(NonZeroU32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BinaryOp {
    Implies, Eq, Neq, Sgt, Ugt, Sgte, Ugte, Slt, Ult, Slte, Ulte,
    And, Or, Xor, Sll, Srl, Sra, Add, Sub, Mul, Sdiv, Udiv, Srem, Urem,
}

pub enum Op {
    Const  { sort: NodeId, value: u64 },
    Binary { sort: NodeId, op: BinaryOp, lhs: NodeId, rhs: NodeId },
    Write  { sort: NodeId, array: NodeId, addr: NodeId, val: NodeId },
    State  { sort: NodeId, name: String },
    Init   { sort: NodeId, state: NodeId, value: NodeId },
    // ...
}"""
))

story.append(Paragraph("Side-by-side meaning", H3))
sbs1 = [
    ["Concept", "C", "Rust"],
    ["Identity of a node", "a heap pointer (uint64_t*)", "a small NodeId(NonZeroU32) value"],
    ["Field access", "offset arithmetic (*(line + 3))", "enum match on Op variant"],
    ["Type safety", "none — every field is uint64_t reinterpreted",
                    "wrong shape won't compile (each variant has fixed fields)"],
    ["Comment", "extra string slot per line", "optional String on the Node"],
    ["BTOR2 nid",
     "stored on the node itself, set when emitted",
     "not stored — the printer assigns nids in a topological pass"],
]
story.append(section_table(sbs1, col_widths=[3.0*cm, 6.6*cm, 7.4*cm]))
story.append(p("That last row is huge. The C version interleaves 'build the node' and 'decide its nid.' "
               "The Rust version separates the two completely. That separation is what makes "
               "deduplication free and what avoids the C version's many "
               "<font face='Courier'>eval_init</font> / <font face='Courier'>set_succ</font> "
               "workarounds for nid ordering."))

# --- Feature 2 ---
story.append(Paragraph("Feature 2 — Sort declarations", H2))
story.append(p("Both files declare the same sorts (Boolean, byte, machine word, register-file array, "
               "memory arrays). The difference is how."))

story.append(Paragraph("C — scattered file-level globals (rotor.c:863+)", H3))
story.append(code(
"""uint64_t* SID_BOOLEAN = (uint64_t*) 0;
uint64_t* SID_BYTE    = (uint64_t*) 0;
... // many more

void init_register_file_sorts() {
  // these globals get assigned by calling new_bitvec(...) etc.
}"""
))
story.append(p("To find what sorts exist, you grep the whole file. To pass sorts around, you don't — "
               "they're just always in scope, like everything else."))

story.append(Paragraph("Rust — one struct, all sorts in one place (machine/sorts.rs:9-63)", H3))
story.append(code(
"""pub struct MachineSorts {
    pub sid_boolean:        NodeId,
    pub sid_byte:           NodeId,
    pub sid_machine_word:   NodeId,
    pub sid_register_state: NodeId,
    pub sid_stack_state:    NodeId,
    // ... ~30 fields
}

impl MachineSorts {
    pub fn new(builder: &mut Btor2Builder, config: &Config) -> Self { ... }
}"""
))
story.append(p("Same sorts, same BTOR2 output. The Rust version turns 'everything is a global' into "
               "'one struct constructed once and passed by &amp; everywhere.' That is why every Rust "
               "function takes <font face='Courier'>sorts: &amp;MachineSorts</font> as a parameter — "
               "it replaces what would have been file-level state in C."))

# --- Feature 3 ---
story.append(Paragraph("Feature 3 — Per-core state", H2))
story.append(p("This one really shows the structural shift."))

story.append(Paragraph("C — parallel arrays indexed by core number", H3))
story.append(code(
"""uint64_t* state_data_segment_nids  = (uint64_t*) 0;   // line 1270
uint64_t* state_heap_segment_nids  = (uint64_t*) 0;
uint64_t* state_stack_segment_nids = (uint64_t*) 0;
uint64_t* init_register_file_nids  = (uint64_t*) 0;
uint64_t* init_stack_segment_nids  = (uint64_t*) 0;
// ... dozens more

// Allocated as flat arrays sized by number_of_cores:
init_register_file_nids = allocate_lines(number_of_cores);  // line 907

// Read with set_for/get_for:
set_for(core, init_register_file_nids, init_register_file_nid);
get_for(core, state_data_segment_nids);"""
))
story.append(p("Per-core state is N parallel arrays. To talk about 'core 0's data segment,' "
               "you index <font face='Courier'>state_data_segment_nids[0]</font>. To talk about all of "
               "core 0's state, you index N different arrays at the same position. There is no "
               "'core 0 object.'"))

story.append(Paragraph("Rust — one struct per core, kept in a Vec", H3))
story.append(code(
"""// machine/core.rs:10-43
pub struct CoreState {
    pub pc_state:            NodeId,
    pub register_file_state: NodeId,
    pub code_segment_state:  NodeId,
    pub data_segment_state:  NodeId,
    pub heap_segment_state:  NodeId,
    pub stack_segment_state: NodeId,
    pub kernel:              KernelState,
    pub core_id:             usize,
    // ...
}

// model/generator.rs:42-45
let mut cores = Vec::new();
for core_id in 0..config.num_cores {
    let core = CoreState::new(...);
    cores.push(core);
}"""
))
story.append(p("Same data, opposite layout: rows of structs instead of columns of arrays. To talk about "
               "core 0, you grab <font face='Courier'>&amp;cores[0]</font> and access fields by name. The "
               "compiler knows you can't reach for a field that doesn't exist; the C version cannot."))
story.append(p("This is the single biggest reason the Rust file is shorter than the C one: thousands of "
               "<font face='Courier'>set_for</font>/<font face='Courier'>get_for</font> calls disappear."))

# --- Feature 4 ---
story.append(Paragraph("Feature 4 — Initializing memory segments from the binary", H2))
story.append(p("This is where the algorithmic differences become visible."))

story.append(Paragraph("C — initialize_memory_segment at rotor.c:6719-6782", H3))
story.append(p("The C version walks address-by-address, stops at every nonzero word, and chains a write "
               "into a linked list via <font face='Courier'>set_succ</font> (mutating the previous "
               "write's 'successor' field):"))
story.append(code(
"""void initialize_memory_segment(uint64_t core, uint64_t* state_segment_nid,
  uint64_t MEMORY_ADDRESS_SPACE, uint64_t segment_start, uint64_t segment_size) {
  // ...
  initial_head_nid = UNUSED;
  initial_tail_nid = state_segment_nid;
  vaddr = segment_start;
  while (vaddr - segment_start < address_space_size) {
    data = 0;
    if (core < number_of_binaries)
      if (vaddr - segment_start < segment_size)
        if (is_virtual_address_mapped(get_pt(current_context), vaddr))
          data = load_virtual_memory(get_pt(current_context), vaddr);
    if (data != 0) {
      laddr_nid = new_constant(OP_CONSTH, SID_VIRTUAL_ADDRESS,
                               vaddr - segment_start, ...);
      data_nid  = new_constant(OP_CONSTH, SID_MACHINE_WORD, data, ...);
      store_nid = store_machine_word_at_virtual_address(
                      laddr_nid, data_nid, initial_tail_nid, state_segment_nid);
      if (initial_head_nid == UNUSED)
        initial_head_nid = store_nid;
      else
        set_succ(initial_tail_nid, store_nid);   // linked list
      initial_tail_nid = store_nid;
      // sanity: read back must equal data
      if (eval_line(load_machine_word_at_virtual_address(
              laddr_nid, store_nid)) != data) {
        printf("...initial segment value mismatch...");
        exit(EXITCODE_SYSTEMERROR);
      }
    }
    vaddr = vaddr + WORDSIZE;
  }
}"""
))
story.append(p("Things to notice:"))
for b in [
    "<font face='Courier'>initial_head_nid</font> and <font face='Courier'>initial_tail_nid</font> are "
    "file-level globals, not return values. The function communicates by side effect.",
    "The chain of writes is also stored as a linked list via "
    "<font face='Courier'>set_succ</font>/<font face='Courier'>get_succ</font> slots on each line, "
    "separate from BTOR2 — used later by the printer to walk the init chain iteratively, "
    "'to avoid stack overflow.'",
    "<font face='Courier'>eval_line</font> runs the constructed expression <i>during generation</i> "
    "to sanity-check it, which means the C rotor has a built-in BTOR2 evaluator (more globals, more code).",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))

story.append(Paragraph("Rust — initialize_data_segment / initialize_code_segment in machine/core.rs", H3))
story.append(code(
"""fn initialize_code_segment(builder: &mut Btor2Builder, sorts: &MachineSorts,
                          _consts: &MachineConstants, binary: &LoadedBinary) -> NodeId
{
    // base state for the init chain
    // (BTOR2 does not allow 'input' in init expressions)
    let code_seg = builder.state(sorts.sid_code_state, "initial-code-base", ...);
    let mut current = code_seg;

    let num_instrs = binary.code.len() / 4;
    for i in 0..num_instrs {
        let offset = i * 4;
        let instr_word = u32::from_le_bytes([
            binary.code[offset],     binary.code[offset + 1],
            binary.code[offset + 2], binary.code[offset + 3],
        ]);
        let addr = builder.constd(sorts.sid_code_address,
                                  binary.code_start + offset as u64, None);
        let word = builder.constd(sorts.sid_code_word, instr_word as u64, ...);
        current  = builder.write(sorts.sid_code_state, current, addr, word, None);
    }
    current
}"""
))

story.append(Paragraph("Side-by-side meaning", H3))
sbs4 = [
    ["", "C", "Rust"],
    ["Result of 'the initial segment'",
     "side-effecting global (initial_head_nid / initial_tail_nid)",
     "NodeId returned by the function"],
    ["Chain mechanism",
     "linked list via set_succ slot on each node",
     "each write already takes the previous write's id — chain is in the BTOR2 expression"],
    ["Dedicated evaluator",
     "eval_line runs the expression during generation as a self-check",
     "none — printer doesn't need it because write ids are deterministic and unique"],
    ["Input-into-init issue",
     "muddled — uses inputs in init, emitter rewrites the order",
     "explicit: builder.state() with no init is the base"],
]
story.append(section_table(sbs4, col_widths=[3.5*cm, 6.6*cm, 6.9*cm]))
story.append(p("The Rust function is a pure function returning a single "
               "<font face='Courier'>NodeId</font>. That is the chain. No side channel."))

# --- Feature 5 ---
story.append(Paragraph("Feature 5 — Top-level pipeline", H2))

story.append(Paragraph("C — int main at rotor.c:13805", H3))
story.append(code(
"""int main(int argc, char** argv) {
  init_selfie((uint64_t) argc, (uint64_t*) argv);
  init_rotor(argc, argv);
  // ... (calls many init_* and new_* functions, each mutating globals)
  // ... eventually emits BTOR2 by walking the constructed lines
}"""
))
story.append(p("The C version's main is short, but misleading — almost everything happens inside "
               "the <font face='Courier'>init_*</font> and <font face='Courier'>new_*</font> functions "
               "through mutation of globals. There is no clear 'phase 1: build, phase 2: print' split."))

story.append(Paragraph("Rust — model_rotor at model/generator.rs:15", H3))
story.append(code(
"""pub fn model_rotor(binary_path: &Path, config: &Config, output: &mut dyn Write)
    -> Result<(), Box<dyn std::error::Error>>
{
    let binary = elf_loader::load_elf(binary_path)?;
    let mut builder = Btor2Builder::new();

    // Phase 1: sorts and constants
    let sorts  = MachineSorts::new(&mut builder, config);
    let consts = MachineConstants::new(&mut builder, &sorts, config);

    // Phase 2: per-core state
    let mut cores = Vec::new();
    for core_id in 0..config.num_cores {
        cores.push(CoreState::new(&mut builder, &sorts, &consts, config,
                                  &binary, core_id));
    }

    // Phase 3: combinational + sequential + properties, per core
    for core in &cores {
        let comb = rotor_combinational(&mut builder, &sorts, &consts, config, core);
        rotor_sequential( &mut builder, &sorts, &consts, config, core, &comb);
        rotor_properties( &mut builder, &sorts, &consts, config, core, &comb);
    }

    // Phase 4: print
    let printer = Btor2Printer::new(config.print_comments);
    printer.print(&builder, &mut *output)?;
    Ok(())
}"""
))
story.append(p("Four phases, named, in order. Every input is explicit in a function parameter. Every "
               "output goes into the builder and finally into the printer. No globals."))

# --- Feature 6 ---
story.append(PageBreak())
story.append(Paragraph("Feature 6 — Symbolic argv: the part that doesn't have a C equivalent", H2))
story.append(p("The C rotor has <b>no symbolic-argv feature</b>. It has "
               "<font face='Courier'>rotor_argv</font> (the host's argv to the rotor binary itself, used "
               "to record the invocation in the model header — see rotor.c:3479-3493 and 12156-12158), "
               "but no mechanism to make the <i>guest program's</i> command-line arguments symbolic. The "
               "guest program's stack is initialized to zero, and its argc/argv come from whatever the "
               "binary's start code reads from a fixed memory layout."))
story.append(p("So this is not a refactor — it is a new feature. There is no C side to compare to. What "
               "we can compare is 'what the C version does to the stack' vs 'what the Rust version does "
               "to the stack.'"))

story.append(Paragraph("C — stack initialization is just zero", H3))
story.append(code(
"""init_zeroed_stack_segment_nid = new_init(SID_STACK_STATE,
  state_stack_segment_nid, NID_MEMORY_WORD_0, "zeroing stack segment");
// ...
if (initial_head_nid != UNUSED) {
  // only used for binaries that have a .data section to load into stack,
  // not for argv
  init_stack_segment_nid = new_init(SID_STACK_STATE,
    state_stack_segment_nid, initial_tail_nid, "loaded stack");
}"""
))
story.append(p("The stack starts as the all-zero array. There is no path to put symbolic bytes there."))

story.append(Paragraph("Rust — initialize_symbolic_argv at machine/core.rs:272-431", H3))
story.append(p("This is the one new piece of logic, walked step by step."))

story.append(Paragraph("Step A — layout calculation (pure arithmetic, no BTOR2 yet)", H4))
story.append(code(
"""let total_argc = config.symbolic_argc + 1;          // +1 for argv[0]
let max_arglen = config.max_arglen;
let prog_name  = b"prog";

let argv0_len   = prog_name.len() + 1;              // "prog\\0" = 5 bytes
let sym_arg_len = max_arglen + 1;                   // bytes per arg + null
let string_area_size    = argv0_len + config.symbolic_argc * sym_arg_len;
let string_area_aligned = (string_area_size as u64 + word_size - 1)
                          & !(word_size - 1);

let pointer_area_size = (total_argc + 1) as u64 * word_size; // +1 for NULL
let total_stack_usage = string_area_aligned + pointer_area_size + word_size;

let string_area_start  = stack_top - string_area_aligned;
let pointer_area_start = string_area_start - pointer_area_size;
let sp                 = pointer_area_start - word_size;     // argc at SP"""
))
story.append(p("This computes, for every byte we are about to write, its exact address. Nothing is "
               "symbolic yet. Nothing has been written to the BTOR2 builder. This is just integers "
               "describing the layout that a real OS would have placed on the stack."))

story.append(Paragraph("Step B — create the base array (uninitialized state)", H4))
story.append(code(
"""let stack_seg = builder.state(
    sorts.sid_stack_state,
    "initial-stack-base",
    Some("base stack segment for argv initialization".to_string()),
);
let mut current = stack_seg;"""
))
story.append(p("<font face='Courier'>builder.state()</font> produces a BTOR2 <font face='Courier'>state</font> "
               "node of array sort with no <font face='Courier'>init</font> attached. By BTOR2 semantics, "
               "an uninitialized state is unconstrained — btormc may pick any value. This is the array we "
               "will <i>partially</i> override with our writes; everything we don't overwrite stays free."))
story.append(Paragraph("Why state, not input? Because we need to use this node inside an init expression "
                       "below, and BTOR2 forbids input nodes inside init expressions. An uninitialized "
                       "state is the legal substitute.", NOTE))

story.append(Paragraph("Step C — write argv[0] = \"prog\\0\" (concrete bytes)", H4))
story.append(code(
"""let mut str_addr = string_area_start;
addrs.push(str_addr);                        // argv[0] string address

for &byte_val in prog_name {
    let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
    let val  = builder.constd(sorts.sid_byte, byte_val as u64, ...);
    current  = builder.write(sorts.sid_stack_state, current, addr, val, None);
    str_addr += 1;
}
// null terminator
let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
let null = builder.constd(sorts.sid_byte, 0, ...);
current  = builder.write(sorts.sid_stack_state, current, addr, null, None);
str_addr += 1;"""
))
story.append(p("Each byte of \"prog\\0\" becomes a concrete <font face='Courier'>constd</font> byte node, "
               "and <font face='Courier'>builder.write(arr, addr, val)</font> returns a new BTOR2 node "
               "representing 'the array arr, except at addr it holds val.' The variable "
               "<font face='Courier'>current</font> always holds the latest such node."))

story.append(Paragraph("Step D — write the symbolic bytes (the only step that matters for symbolic execution)", H4))
story.append(code(
"""for arg_idx in 0..config.symbolic_argc {
    addrs.push(str_addr);
    for byte_idx in 0..max_arglen {
        let addr = builder.constd(sorts.sid_stack_address, str_addr, None);

        // Each byte is a symbolic (unconstrained) state — solver picks
        // any value 0..255. We use state (not input) because BTOR2 forbids
        // inputs in init expressions. An uninitialized state is unconstrained,
        // which is exactly what we want.
        let sym_byte = builder.state(
            sorts.sid_byte,
            &format!("argv[{}][{}]", arg_idx + 1, byte_idx),
            Some(format!("symbolic byte argv[{}][{}]", arg_idx + 1, byte_idx)),
        );
        current = builder.write(sorts.sid_stack_state, current, addr, sym_byte, None);
        str_addr += 1;
    }
    // null terminator stays concrete so C string semantics survive
    let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
    let null = builder.constd(sorts.sid_byte, 0, ...);
    current  = builder.write(sorts.sid_stack_state, current, addr, null, None);
    str_addr += 1;
}"""
))
story.append(p("This is the one place in the entire codebase where symbolic data gets minted. The trick "
               "is one line: <font face='Courier'>let sym_byte = builder.state(sorts.sid_byte, "
               "\"argv[i][j]\", ...);</font>"))
story.append(p("It creates a new BTOR2 <font face='Courier'>state</font> of sort byte (8 bits) with no "
               "<font face='Courier'>init</font> and (later) no <font face='Courier'>next</font>. In "
               "btormc semantics that means: 'this is a free 8-bit variable; pick any value 0..255 at "
               "step 0; it stays that value forever.' Perfect for 'a byte the user typed on the command "
               "line.'"))
story.append(p("We then <font face='Courier'>write</font> that free state into the array at the correct "
               "address, building up our <font face='Courier'>current</font> expression. The free state "
               "is now a <i>cell</i> of the array's init expression. When the program later does "
               "<font face='Courier'>argv[1][0]</font>, it is reading that exact cell."))

story.append(Paragraph("Step E — write the pointer area (concrete pointers to those addresses)", H4))
story.append(code(
"""let mut ptr_addr = pointer_area_start;
for (i, &string_addr) in argv_string_addrs.iter().enumerate() {
    for byte_idx in 0..word_size {
        let byte_val = (string_addr >> (byte_idx * 8)) & 0xFF;
        let addr = builder.constd(sorts.sid_stack_address, ptr_addr + byte_idx, None);
        let val  = builder.constd(sorts.sid_byte, byte_val, ...);
        current  = builder.write(sorts.sid_stack_state, current, addr, val, None);
    }
    ptr_addr += word_size;
}
// NULL pointer terminator
for byte_idx in 0..word_size {
    let addr = builder.constd(sorts.sid_stack_address, ptr_addr + byte_idx, None);
    let val  = builder.constd(sorts.sid_byte, 0, ...);
    current  = builder.write(sorts.sid_stack_state, current, addr, val, None);
}"""
))
story.append(p("The pointer at address <font face='Courier'>pointer_area_start + i*word_size</font> "
               "holds the address of the i'th string. Nothing symbolic — these are constants, since "
               "we control the layout entirely."))

story.append(Paragraph("Step F — write argc at SP", H4))
story.append(code(
"""let argc_value = total_argc as u64;
for byte_idx in 0..word_size {
    let byte_val = (argc_value >> (byte_idx * 8)) & 0xFF;
    let addr = builder.constd(sorts.sid_stack_address, sp + byte_idx, None);
    let val  = builder.constd(sorts.sid_byte, byte_val, ...);
    current  = builder.write(sorts.sid_stack_state, current, addr, val, None);
}

(sp, Some(current))"""
))
story.append(p("The function returns <font face='Courier'>(sp, init_expression)</font>. The caller "
               "in <font face='Courier'>CoreState::new</font> attaches that expression to the real "
               "stack state:"))
story.append(code(
"""let stack_segment_state = builder.state(sorts.sid_stack_state, "stack-segment", ...);
if let Some(stack_val) = stack_init_val {
    builder.init(sorts.sid_stack_state, stack_segment_state, stack_val,
                 Some("init stack segment with argv"));
}"""
))
story.append(p("That is the whole feature."))

# ---------------------------------------------------------------------------
# Symbolic execution deep dive
# ---------------------------------------------------------------------------
story.append(PageBreak())
story.append(Paragraph("How &quot;symbolic execution&quot; actually works in this project", H1))
story.append(p("The phrase 'symbolic execution' is overloaded; what <i>this</i> project does is one "
               "specific thing."))

story.append(Paragraph("What it is — and isn't", H2))
story.append(p("'Symbolic execution' usually conjures tools like KLEE or angr: the engine maintains a "
               "<i>path constraint</i>, walks the program with placeholder variables, and forks at every "
               "branch on a symbolic value. That is <b>not</b> what rotor does. Rotor does not run "
               "anything. The whole 'exploration' happens inside btormc, and it is done by SAT solving, "
               "not by walking."))
story.append(p("The right name for what rotor does is <b>bounded model checking with unconstrained "
               "inputs</b>. It produces the same kind of result (an input that triggers a bug, or 'no "
               "bug up to depth k'), but the mechanism is:"))
for b in [
    "Rotor emits a transition system in BTOR2.",
    "btormc unrolls the transition system k times, producing one large propositional formula.",
    "btormc asks a SAT solver: 'is there an assignment to the unconstrained inputs that makes any "
    "<font face=\"Courier\">bad</font> property true at any of the k steps?'",
    "If yes, it returns the witness — values for every input at every step.",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))

story.append(Paragraph("What &quot;unconstrained&quot; means at the BTOR2 level", H2))
story.append(p("A BTOR2 model defines:"))
for b in [
    "A set of <b>state</b> variables (the system's memory between steps).",
    "A set of <b>input</b> variables (chosen anew each step).",
    "An <b>init</b> expression for each state (its value at step 0).",
    "A <b>next</b> expression for each state (its value at step t+1 in terms of state at step t and "
    "inputs at step t).",
    "One or more <b>bad</b> expressions that should never become true.",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))
story.append(p("When rotor wants the solver to be free to choose something, it has exactly two levers:"))
for b in [
    "<b>Per-step input</b> (e.g. stdin bytes that arrive over time): use an "
    "<font face='Courier'>input</font> node. Each step, the solver picks a fresh value.",
    "<b>Once-set, never-changes input</b> (e.g. argv bytes, fixed at program start): use a "
    "<font face='Courier'>state</font> node with no <font face='Courier'>init</font> and no "
    "<font face='Courier'>next</font>. Each run, the solver picks a value at step 0; it stays that "
    "value forever.",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))
story.append(p("That is the entire vocabulary."))

story.append(Paragraph("How rotor uses those levers", H2))
story.append(p("There are exactly three places where rotor leaves something free for the solver:"))
for n, t in [
    ("stdin via the kernel read syscall.",
     "machine/kernel.rs creates an input_buffer array and lets the kernel module pull bytes out as the "
     "program reads them. Those bytes are unconstrained."),
    ("Heap segment, before the program writes it.",
     "machine/core.rs:204-208 creates heap_segment_state with no init. If the program reads heap before "
     "writing, the read returns whatever the solver wants. Usually irrelevant — most code writes before "
     "reading — but free."),
    ("Symbolic argv bytes.",
     "Each byte of argv[1..N] is a fresh state node with no init, embedded in the stack init "
     "expression. Discussed in Feature 6 above."),
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;<b>{n}</b> {t}", BULLET))
story.append(p("Everything else in the model — code, register file initial values, segmentation "
               "constants, instruction decoding — is fully concrete. There is no other source of "
               "symbolic-ness."))

story.append(Paragraph("What &quot;executes&quot; the symbolic stuff", H2))
story.append(p("The model itself. Once the symbolic bytes are sitting in the stack array, the program "
               "loads them through the <i>same</i> BTOR2 <font face='Courier'>read(stack, addr)</font> "
               "operation as any other load. The result of that read is symbolic because the array "
               "contains symbolic cells; the solver sees a read whose result is a free byte, and "
               "propagates that freedom into whatever uses it."))
story.append(p("So when the program does <font face='Courier'>if (argv[1][0] == 'X')</font>:"))
for b in [
    "The compiled instruction is something like <font face='Courier'>lb t0, 0(a1)</font> "
    "(<font face='Courier'>a1</font> holds <font face='Courier'>argv[1]</font>).",
    "Rotor's load handling computes <font face='Courier'>read(stack_segment, "
    "sp_relative_address_of_argv_1_byte_0)</font>.",
    "The result is the symbolic byte <font face='Courier'>state(\"argv[1][0]\")</font>.",
    "The compare instruction becomes <font face='Courier'>eq(symbolic_byte, const(0x58))</font> ('X').",
    "The branch becomes an <font face='Courier'>ite</font> over that boolean.",
    "Both branches are part of the unrolled formula. SAT will pick whichever satisfies a "
    "<font face='Courier'>bad</font>.",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))
story.append(p("There is no fork. There is no walk. There is one giant formula, and SAT decides."))

story.append(Paragraph("Why no next on the symbolic state", H2))
story.append(p("If we attached a <font face='Courier'>next</font> expression to "
               "<font face='Courier'>argv[1][0]</font> (e.g. <font face='Courier'>next(argv[1][0]) = "
               "argv[1][0]</font>), it would still work but be redundant — BTOR2 already treats a "
               "<font face='Courier'>state</font> with no <font face='Courier'>next</font> as 'value "
               "persists.' Leaving <font face='Courier'>next</font> off keeps the model smaller and "
               "signals intent."))

story.append(Paragraph("Why state-without-init instead of input", H2))
story.append(p("Two reasons, one semantic and one mechanical."))
for b in [
    "<b>Semantic.</b> argv is fixed at program start — it does not get a fresh value each step. "
    "<font face='Courier'>state</font> matches that semantics; <font face='Courier'>input</font> would not.",
    "<b>Mechanical.</b> BTOR2 specifically forbids <font face='Courier'>input</font> nodes inside "
    "<font face='Courier'>init</font> expressions. The argv bytes need to live inside the stack's "
    "<font face='Courier'>init</font> expression (so they're part of the array at step 0). So "
    "<font face='Courier'>state</font> is the only choice that is both legal and correct.",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))

story.append(Paragraph("Why the bytes get embedded in the stack array specifically", H2))
story.append(p("The CPU model performs every memory access as <font face='Courier'>read(some_segment_array, "
               "address)</font>. For the symbolic byte to influence the program's behavior, the program "
               "must read it <i>as if it were a normal stack byte</i>. Embedding the symbolic state "
               "inside the stack segment's init expression at the exact address the program will read "
               "makes the magic transparent: the program does its standard argv lookup; the address "
               "resolves to the address of our embedded symbolic cell; the cell yields a free byte; the "
               "byte propagates through the rest of the formula."))
story.append(p("Nothing in the program itself, in the decoder, in the kernel module, in the property "
               "checks, has to know about symbolic argv. It is entirely localized to "
               "<font face='Courier'>initialize_symbolic_argv</font>."))

story.append(Paragraph("Why this approach instead of &quot;just use BTOR2 input nodes for argv&quot;", H2))
story.append(p("You could try: emit an <font face='Courier'>input</font> node per argv byte, then on the "
               "first cycle read it and write it into the stack. Two problems:"))
for b in [
    "<b>Init-expression restriction.</b> As above, you can't have inputs in init, so they would have to "
    "be written <i>during execution</i>, which means changing the kernel/loader to model 'OS hands me "
    "argv on cycle 0.' Expensive.",
    "<b>Stability.</b> An <font face='Courier'>input</font> is <i>re-chosen</i> every step. argv bytes "
    "should not change after step 0. You would have to add constraints saying 'the argv bytes at step t "
    "equal those at step 0,' wasting solver budget.",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))
story.append(p("The state-without-init approach sidesteps both. Total cost: one "
               "<font face='Courier'>state</font> line per byte plus one <font face='Courier'>write</font> "
               "line per byte. Net: the BTOR2 file is bigger by "
               "<font face='Courier'>8 * symbolic_argc * max_arglen</font> lines, which for typical "
               "settings (1 arg of 8 bytes) is 64 lines of overhead. That is why models stay small."))

story.append(Paragraph("The full request → witness pipeline, end to end", H2))
story.append(p("Take <font face='Courier'>benchmarks/argv-tests/test4_multi_arg.c</font>:"))
story.append(code(
"""if (argc > 2)
    if (argv[1][0] == 88)        // 'X'
        if (argv[2][0] == 89)    // 'Y'
            return 1;
return 0;"""
))
story.append(code(
"""$ selfie -c test4_multi_arg.c -o test4.m
                                  # produces a RISC-V binary

$ rotor test4.m --symbolic-argv --symbolic-argc 2 --max-arglen 8 -o test4.btor2
                                  # rotor builds the model:
                                  #   - decodes instructions, builds combinational logic
                                  #   - emits state nodes for each argv byte (no init)
                                  #   - emits stack init expression with those states embedded
                                  #   - emits bad := (exit_status != 0)

$ btormc -kmax 200 test4.btor2 > test4.wit
                                  # btormc unrolls 200 steps, SAT-solves,
                                  # finds an assignment where argv[1][0]=88,
                                  # argv[2][0]=89 makes the program reach
                                  # return 1 -> exit_status nonzero -> bad true.

$ visualizer/index.html  <-  load test4.btor2 + test4.wit
                                  # see the trace step by step"""
))
story.append(p("That is the whole loop."))

# ---------------------------------------------------------------------------
doc.build(story)
print(f"Wrote {OUT_PATH}")
