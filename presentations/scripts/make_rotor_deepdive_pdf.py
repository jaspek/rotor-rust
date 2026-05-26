"""
Deep-dive PDF: every function of the rotor machine model and the symbolic-arguments
feature, explained in plain machine-level terms (no BTOR2 internals).
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

OUT_PATH = r"C:\Users\jasko\Programming\Rust\Project01\Rotor_Deep_Dive.pdf"

# --- Palette ---------------------------------------------------------------
INK         = HexColor("#1d232b")
SUBTLE      = HexColor("#5a6470")
ACCENT      = HexColor("#1f5fa1")
ACCENT2     = HexColor("#0f6e58")
RULE        = HexColor("#cfd6dd")
CODE_BG     = HexColor("#f3f4f6")
CODE_BORDER = HexColor("#dde1e6")
TABLE_HEAD  = HexColor("#e8edf2")
NOTE_BG     = HexColor("#f7f8fa")
KEY_BG      = HexColor("#fffbe6")
KEY_BORDER  = HexColor("#e6dca0")

# --- Document --------------------------------------------------------------
doc = SimpleDocTemplate(
    OUT_PATH, pagesize=A4,
    leftMargin=2.0 * cm, rightMargin=2.0 * cm,
    topMargin=1.8 * cm, bottomMargin=1.8 * cm,
    title="Rotor — machine model and symbolic arguments deep dive",
    author="Project notes",
)

base = getSampleStyleSheet()

H1 = ParagraphStyle("H1", parent=base["Heading1"],
                    fontName="Helvetica-Bold", fontSize=20, leading=24,
                    textColor=INK, spaceBefore=8, spaceAfter=10, keepWithNext=True)
H2 = ParagraphStyle("H2", parent=base["Heading2"],
                    fontName="Helvetica-Bold", fontSize=14.5, leading=18,
                    textColor=ACCENT, spaceBefore=14, spaceAfter=6, keepWithNext=True)
H3 = ParagraphStyle("H3", parent=base["Heading3"],
                    fontName="Helvetica-Bold", fontSize=11.5, leading=15,
                    textColor=INK, spaceBefore=10, spaceAfter=4, keepWithNext=True)
H4 = ParagraphStyle("H4", parent=base["Heading4"],
                    fontName="Helvetica-Bold", fontSize=10.5, leading=14,
                    textColor=ACCENT2, spaceBefore=6, spaceAfter=2, keepWithNext=True)

BODY = ParagraphStyle("Body", parent=base["BodyText"],
                      fontName="Helvetica", fontSize=10, leading=14,
                      textColor=INK, spaceAfter=6, alignment=TA_LEFT)
SMALL = ParagraphStyle("Small", parent=BODY, fontSize=9, leading=12.5)

NOTE = ParagraphStyle("Note", parent=BODY, fontSize=9.5, leading=13,
                      textColor=SUBTLE, leftIndent=10, rightIndent=10,
                      backColor=NOTE_BG,
                      borderPadding=(6, 8, 6, 8), spaceBefore=4, spaceAfter=8)

KEY = ParagraphStyle("Key", parent=BODY, fontSize=10, leading=13.5,
                     leftIndent=10, rightIndent=10,
                     backColor=KEY_BG, borderColor=KEY_BORDER, borderWidth=0.6,
                     borderPadding=(6, 8, 6, 8), spaceBefore=6, spaceAfter=8)

BULLET = ParagraphStyle("Bullet", parent=BODY,
                        leftIndent=14, bulletIndent=2, spaceAfter=2)

CODE = ParagraphStyle("Code", parent=base["Code"],
                      fontName="Courier", fontSize=8.2, leading=10.5,
                      textColor=INK, leftIndent=0, rightIndent=0,
                      backColor=CODE_BG, borderColor=CODE_BORDER, borderWidth=0.5,
                      borderPadding=(6, 7, 6, 7), spaceBefore=4, spaceAfter=8)

SIG = ParagraphStyle("Sig", parent=BODY, fontName="Courier-Bold", fontSize=9.5,
                     leading=12, textColor=INK, spaceBefore=6, spaceAfter=2,
                     keepWithNext=True)


def code(text):
    return KeepTogether([Spacer(1, 2), Preformatted(text.rstrip("\n"), CODE)])


def p(text, style=BODY):
    return Paragraph(text, style)


def bullets(items):
    return [Paragraph(f"&bull;&nbsp;&nbsp;{t}", BULLET) for t in items]


def section_table(rows, col_widths=None, header=True):
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


def fn(sig, *blocks):
    """A function entry: signature + description blocks."""
    parts = [Paragraph(sig, SIG)]
    parts.extend(blocks)
    return KeepTogether(parts)


# ---------------------------------------------------------------------------
story = []

# ===========================================================================
# Title
# ===========================================================================
story.append(Paragraph(
    "Rotor in Rust: machine model and symbolic arguments",
    ParagraphStyle("Title", parent=H1, fontSize=22, leading=27, spaceAfter=4)
))
story.append(Paragraph(
    "Every function of the simulated machine and of the symbolic-arguments feature, "
    "explained at the machine level. This document does not discuss BTOR2 internals — "
    "it describes what the code <i>means</i> in terms of registers, memory, instructions, "
    "and command-line arguments.",
    ParagraphStyle("Sub", parent=BODY, fontSize=10.5, textColor=SUBTLE, spaceAfter=14),
))

story.append(p(
    "Rotor reads a compiled RISC-V program and produces a precise model of the machine "
    "that would run it: the program counter, the 32 registers, four memory regions, and a "
    "small kernel for system calls. A separate solver then asks 'can this machine reach a "
    "bad state for some choice of input?' — but that solver is not the subject of this "
    "document. Here we look at the machine itself, function by function, and at the new "
    "<b>symbolic arguments</b> feature, which lets a user say 'argv[1] can be anything; "
    "find me an argv that breaks the program.'"
))

# ===========================================================================
# Part 1 — The rotor machine model
# ===========================================================================
story.append(Paragraph("Part 1 — The rotor machine model", H1))

story.append(p(
    "Rotor is built around one central object — the <b>core</b> — and five supporting "
    "modules that describe how the core's pieces behave. The core represents one CPU. "
    "Everything a real CPU has — instruction pointer, registers, memory, knowledge of where "
    "code/data/heap/stack live, kernel state for syscalls — lives on the core. We start "
    "with the core, then walk each piece outward."
))

# ----- 1.1 CoreState --------------------------------------------------------
story.append(Paragraph("1.1 CoreState — the whole machine in one struct", H2))
story.append(p(
    "The file <font face='Courier'>machine/core.rs</font> defines a struct called "
    "<font face='Courier'>CoreState</font>. One instance equals one CPU. The struct's "
    "fields are exactly the things a programmer thinks of when they say 'the state of "
    "this machine right now':"
))

story.append(code(
"""pub struct CoreState {
    pub pc_state:            NodeId,   // program counter (instruction pointer)
    pub pc_nid:              NodeId,   // its initial value (entry point)
    pub ir:                  Option<NodeId>,  // last fetched instruction
    pub c_ir:                Option<NodeId>,  // its 16-bit compressed form, if any
    pub instruction_id:      Option<NodeId>,  // decoded instruction kind
    pub register_file_state: NodeId,   // the 32 RISC-V registers
    pub code_segment_state:  NodeId,   // .text — read-only after init
    pub data_segment_state:  NodeId,   // .data + .bss
    pub heap_segment_state:  NodeId,   // grows up via brk syscall
    pub stack_segment_state: NodeId,   // grows down from a fixed top
    pub segmentation:        Segmentation,  // segment boundaries
    pub kernel:              KernelState,   // syscall + heap pointer + stdin
    pub core_id:             usize,
}"""
))
story.append(p(
    "If you imagine the running program as a movie: the core is everything visible in the "
    "frame. The other modules describe how each piece changes from frame to frame."
))

story.append(Paragraph("CoreState::new", H3))
story.append(p(
    "Builds one core from a loaded binary. The function is long because it has to set up "
    "every field in the right order, but conceptually it does five things:"
))
for i, t in enumerate([
    "<b>Compute the entry point and stack top.</b> Entry point comes from the ELF file. "
    "Stack top is at <font face='Courier'>2^(virtual_address_space-1)</font> by default — "
    "high in the address space, growing downward.",
    "<b>Decide what argv looks like.</b> If <font face='Courier'>--symbolic-argv</font> "
    "is set, call <font face='Courier'>initialize_symbolic_argv</font> to lay out argv on "
    "the stack and get back the resulting initial stack pointer. If not, just put SP one "
    "word below the top.",
    "<b>Initialize the register file.</b> Start with all registers zero, then set "
    "<font face='Courier'>x2 (sp)</font> to the stack pointer computed above. If symbolic "
    "argv is on, also set <font face='Courier'>x10 (a0)</font> to argc — that is what the "
    "OS would pass to <font face='Courier'>main</font>. Force <font face='Courier'>x0</font> "
    "to zero (RISC-V hardwires it).",
    "<b>Initialize each segment.</b> Code segment gets the binary's <font face='Courier'>.text</font> "
    "bytes. Data segment gets <font face='Courier'>.data</font>. Heap is left empty. Stack "
    "is whatever <font face='Courier'>initialize_symbolic_argv</font> built (or zero).",
    "<b>Build the kernel state.</b> See section 1.6.",
], 1):
    story.append(Paragraph(f"{i}.&nbsp;&nbsp;{t}", BULLET))

story.append(p(
    "Two private helpers belong to the core constructor:"
))
story.append(fn(
    "fn initialize_code_segment(builder, sorts, consts, binary) -> NodeId",
    p("Walks the binary's instruction bytes, four at a time (one RISC-V instruction is "
      "four bytes when not compressed), and writes each instruction word to its address "
      "in the code segment. Result: a code segment that holds the program."),
))
story.append(fn(
    "fn initialize_data_segment(builder, sorts, binary) -> NodeId",
    p("Walks the binary's data bytes one at a time and writes each non-zero byte to its "
      "address. Zero bytes are skipped because the segment is treated as zero by default. "
      "Result: a data segment matching <font face='Courier'>.data</font> in the ELF."),
))

# ----- 1.2 Segmentation -----------------------------------------------------
story.append(Paragraph("1.2 Segmentation — where each region lives in memory", H2))
story.append(p(
    "<font face='Courier'>machine/segmentation.rs</font> defines a struct that just stores "
    "the start and end address of each of the four memory regions. Code starts where the "
    "ELF says; data starts where the ELF says; heap starts immediately after data; stack "
    "ends at the top of the address space and grows downward."
))
story.append(code(
"""pub struct Segmentation {
    pub code_start, code_end,
    pub data_start, data_end,
    pub heap_start, heap_end,
    pub stack_start, stack_end: NodeId,
}"""
))
story.append(p("All useful operations on this struct are address-classification helpers."))

story.append(fn(
    "fn new(builder, sorts, consts, binary, heap_allowance, stack_allowance) -> Self",
    p("Computes all eight boundary constants from the loaded binary plus the heap/stack "
      "size the user requested on the command line. Heap follows data; stack ends at "
      "<font face='Courier'>2GB (1 &lt;&lt; 31)</font> by default and is sized by "
      "<font face='Courier'>--stack</font>."),
))
for sig, desc in [
    ("fn is_in_data_segment(builder, sorts, vaddr) -> NodeId",
     "True iff the given address lies within <font face='Courier'>[data_start, data_end)</font>."),
    ("fn is_in_heap_segment(builder, sorts, vaddr) -> NodeId",
     "True iff the address is in the heap."),
    ("fn is_in_stack_segment(builder, sorts, vaddr) -> NodeId",
     "True iff the address is in the stack."),
    ("fn is_valid_write_address(builder, sorts, vaddr) -> NodeId",
     "True iff the address is in any writable segment — that is, data, heap, or stack. "
     "Code is read-only and excluded."),
    ("fn is_valid_read_address(builder, sorts, vaddr) -> NodeId",
     "True iff the address is in any readable segment — code, data, heap, or stack."),
    ("fn select_segment(builder, sorts, vaddr, data_state, heap_state, stack_state, array_sid) -> NodeId",
     "Given an address and the three writable segment values, return the right segment. "
     "Used by every memory operation: 'this address lives in stack, so the load reads "
     "from the stack array; that one is in heap, so write goes to the heap array.' "
     "Defaults to data when the address is in none of them — the segfault check upstream "
     "catches that case."),
]:
    story.append(fn(sig, p(desc)))

# ----- 1.3 RegisterFile ----------------------------------------------------
story.append(Paragraph("1.3 RegisterFile — read and write the 32 registers", H2))
story.append(p(
    "<font face='Courier'>machine/registers.rs</font> is a unit-struct with four "
    "associated functions. RISC-V has 32 integer registers, indexed 0..31. Register 0 is "
    "hardwired to zero — writes to it are silently dropped. The whole file is "
    "boilerplate to model that reliably."
))
for sig, desc in [
    ("fn load_register(builder, sorts, reg_state, reg_addr, comment) -> NodeId",
     "Read the value of the register whose address is <font face='Courier'>reg_addr</font>. "
     "The address here is itself a value because the RISC-V instruction format encodes the "
     "register number in the instruction bits."),
    ("fn store_register(builder, sorts, reg_state, reg_addr, value, comment) -> NodeId",
     "Update the register file: the new state is the same as before except register "
     "<font face='Courier'>reg_addr</font> now holds <font face='Courier'>value</font>. "
     "Returned as a new register-file value."),
    ("fn load_register_by_index(builder, sorts, consts, reg_state, reg_index, comment) -> NodeId",
     "Same as <font face='Courier'>load_register</font> but with a compile-time-known "
     "index (0..31). Used when rotor itself knows which register it wants — for example, "
     "when reading <font face='Courier'>a0</font>, <font face='Courier'>a7</font>, "
     "<font face='Courier'>sp</font>."),
    ("fn store_register_by_index(builder, sorts, consts, reg_state, reg_index, value, comment) -> NodeId",
     "Same idea for stores. If the index is 0, returns the unmodified state — the x0 "
     "no-op rule."),
    ("fn conditional_store(builder, sorts, consts, reg_state, rd_addr, value, comment) -> NodeId",
     "The version used during instruction execution. The destination register "
     "<font face='Courier'>rd</font> is decoded from the instruction at runtime (its "
     "index is a value, not known to rotor). This function writes the value, then asks "
     "'is rd actually x0?'; if yes, returns the original state instead. This is the "
     "x0-as-zero rule applied dynamically."),
]:
    story.append(fn(sig, p(desc)))

# ----- 1.4 Memory -----------------------------------------------------------
story.append(Paragraph("1.4 Memory — load and store at byte/half/word/double widths", H2))
story.append(p(
    "<font face='Courier'>machine/memory.rs</font> is the hottest file in the machine "
    "model — every load and store goes through here. RISC-V is byte-addressable, so all "
    "memory is a flat array of bytes. Wider loads (<font face='Courier'>lh</font>, "
    "<font face='Courier'>lw</font>, <font face='Courier'>ld</font>) are built by reading "
    "consecutive bytes and concatenating them little-endian; wider stores split a value "
    "into bytes and write them at consecutive addresses."
))
story.append(p("Loads:"))
for sig, desc in [
    ("fn load_byte(builder, sorts, memory, vaddr, comment) -> NodeId",
     "Read one byte at <font face='Courier'>vaddr</font>."),
    ("fn load_half_word(builder, sorts, consts, memory, vaddr) -> NodeId",
     "Read two consecutive bytes; return them as a 16-bit value, low byte at "
     "<font face='Courier'>vaddr</font>."),
    ("fn load_word(builder, sorts, consts, memory, vaddr) -> NodeId",
     "Same with four bytes -> 32-bit value."),
    ("fn load_double_word(builder, sorts, consts, memory, vaddr) -> NodeId",
     "Same with eight bytes -> 64-bit value."),
]:
    story.append(fn(sig, p(desc)))

story.append(p("Stores are the symmetrical functions:"))
for sig, desc in [
    ("fn store_byte(builder, sorts, memory, vaddr, value, comment) -> NodeId",
     "Write one byte."),
    ("fn store_half_word(builder, sorts, consts, memory, vaddr, value) -> NodeId",
     "Split a 16-bit value into two bytes; write them at <font face='Courier'>vaddr</font> "
     "and <font face='Courier'>vaddr+1</font>."),
    ("fn store_word(builder, sorts, consts, memory, vaddr, value) -> NodeId",
     "Same idea with four bytes."),
    ("fn store_double_word(builder, sorts, consts, memory, vaddr, value) -> NodeId",
     "Same with eight bytes."),
]:
    story.append(fn(sig, p(desc)))

story.append(p(
    "<b>Width-dispatch helpers.</b> The instruction decoder produces a 'width' value "
    "(1, 2, 4, or 8) for each load/store. The memory module exposes one function that "
    "picks the right per-width function based on that runtime value, so the per-instruction "
    "execution code can just say 'load some-width bytes from this address':"
))
for sig, desc in [
    ("fn load_value(builder, sorts, consts, memory, vaddr, width, sign_extend) -> NodeId",
     "Picks <font face='Courier'>load_byte</font>/<font face='Courier'>half_word</font>/"
     "<font face='Courier'>word</font>/<font face='Courier'>double_word</font> based on "
     "<font face='Courier'>width</font>. <font face='Courier'>sign_extend</font> controls "
     "whether smaller-than-machine-word loads sign-extend into the destination register "
     "(<font face='Courier'>lb</font> sign-extends, <font face='Courier'>lbu</font> does "
     "not)."),
    ("fn store_value(builder, sorts, consts, memory, vaddr, value, width) -> NodeId",
     "The equivalent for stores."),
]:
    story.append(fn(sig, p(desc)))

# ----- 1.5 Sorts and constants ---------------------------------------------
story.append(Paragraph("1.5 Sorts and constants — naming the types and known values", H2))
story.append(p(
    "<font face='Courier'>machine/sorts.rs</font> exists for one reason: every function in "
    "the project takes a <font face='Courier'>&amp;MachineSorts</font> argument so it can "
    "say 'I want a byte' or 'I want an array indexed by addresses.' Two structs:"
))
for sig, desc in [
    ("MachineSorts (struct)",
     "A bag of about 30 type tags: <font face='Courier'>sid_boolean</font>, "
     "<font face='Courier'>sid_byte</font>, <font face='Courier'>sid_machine_word</font> "
     "(32 or 64 bit depending on <font face='Courier'>--xlen</font>), the various register- "
     "and memory-array types, immediate-extraction types of every bit width the decoder "
     "uses."),
    ("fn MachineSorts::new(builder, config) -> Self",
     "Creates each type once and stores its handle. Called once per run from "
     "<font face='Courier'>model_rotor</font>."),
    ("MachineConstants (struct)",
     "Holds the small handful of known integer values that show up everywhere: 0, 1, 2, "
     "3, 4, 8 (used as offsets), the ID for each instruction kind, the syscall numbers, "
     "the index of each register."),
    ("fn MachineConstants::new(builder, sorts, config) -> Self",
     "Creates each constant once. Subsequent uses just hand back the same handle, so the "
     "model never has duplicate '0' entries."),
    ("fn MachineConstants::nid_register(reg_index) -> NodeId",
     "Returns the address node for register <font face='Courier'>reg_index</font> (0..31). "
     "Used everywhere the rotor code knows which register it means by name "
     "(<font face='Courier'>SP</font>, <font face='Courier'>A0</font>, "
     "<font face='Courier'>A7</font>, <font face='Courier'>ZR</font>)."),
    ("fn MachineConstants::nid_instr_id(InstrId) -> NodeId",
     "Returns the integer ID for an instruction kind (e.g., <font face='Courier'>ADD</font>, "
     "<font face='Courier'>LW</font>, <font face='Courier'>ECALL</font>). Used to compare "
     "the decoded instruction against a specific kind."),
]:
    story.append(fn(sig, p(desc)))

# ----- 1.6 KernelState ------------------------------------------------------
story.append(Paragraph("1.6 KernelState — system calls, the heap pointer, and stdin", H2))
story.append(p(
    "<font face='Courier'>machine/kernel.rs</font> models the only operating-system "
    "behavior the program ever sees: the <font face='Courier'>ecall</font> instruction "
    "and the handful of syscalls selfie programs use (<font face='Courier'>read</font>, "
    "<font face='Courier'>write</font>, <font face='Courier'>openat</font>, "
    "<font face='Courier'>brk</font>, <font face='Courier'>exit</font>)."
))

story.append(code(
"""pub struct KernelState {
    pub program_break:      NodeId,  // current heap top, advances on brk()
    pub program_break_init: NodeId,  // its initial value
    pub input_buffer:       NodeId,  // bytes the program will read via read()
    pub readable_bytes:     NodeId,  // how many of them are still available
    pub read_bytes:         NodeId,  // how many have been read so far
}"""
))

for sig, desc in [
    ("fn KernelState::new(builder, sorts, consts, initial_brk, bytes_to_read) -> Self",
     "Creates the kernel state. Initial program break sits right after the data segment. "
     "The input buffer holds <font face='Courier'>bytes_to_read</font> bytes (default 4) "
     "and is left totally unconstrained — the solver picks what stdin returns. "
     "<font face='Courier'>read_bytes</font> starts at zero."),
    ("fn KernelState::is_ecall(builder, sorts, consts, instruction_id) -> NodeId",
     "Tells whether the current instruction is an <font face='Courier'>ecall</font>. The "
     "decoder produces the instruction ID; this just compares it to the "
     "<font face='Courier'>Ecall</font> ID."),
    ("fn KernelState::decode_syscall(builder, sorts, consts, a7_value) -> SyscallDecode",
     "Linux/RISC-V passes the syscall number in register <font face='Courier'>a7</font>. "
     "This returns a struct of five booleans: <font face='Courier'>is_exit</font>, "
     "<font face='Courier'>is_read</font>, <font face='Courier'>is_write</font>, "
     "<font face='Courier'>is_openat</font>, <font face='Courier'>is_brk</font>. Only one "
     "of them is true at a time."),
    ("fn KernelState::next_program_break(builder, sorts, consts, current_brk, a0_value, "
     "is_brk_syscall, is_ecall) -> NodeId",
     "Computes the heap pointer's value after the next step. Standard "
     "<font face='Courier'>brk</font> semantics: if <font face='Courier'>a0 == 0</font>, "
     "querying — return the current break unchanged. If "
     "<font face='Courier'>a0 != 0</font>, set the break to <font face='Courier'>a0</font>. "
     "If the current instruction isn't a <font face='Courier'>brk</font> ecall at all, "
     "leave the break unchanged."),
    ("fn KernelState::ecall_return_value(builder, sorts, consts, syscall, a0_value, "
     "current_brk, readable_bytes) -> NodeId",
     "What the kernel writes into <font face='Courier'>a0</font> after the ecall returns. "
     "<font face='Courier'>brk</font> returns the new break; <font face='Courier'>read</font> "
     "returns how many bytes it gave the program; <font face='Courier'>write</font> returns "
     "the byte count it pretended to write; <font face='Courier'>openat</font> returns 0 "
     "as a stub fd; <font face='Courier'>exit</font> doesn't return at all."),
    ("SyscallDecode (struct)",
     "Five booleans, one per recognized syscall. Returned by "
     "<font face='Courier'>decode_syscall</font> and consumed by the next-state logic."),
]:
    story.append(fn(sig, p(desc)))

# ----- 1.7 model::generator -------------------------------------------------
story.append(Paragraph("1.7 The pipeline — generator.rs", H2))
story.append(p(
    "<font face='Courier'>model/generator.rs</font> orchestrates everything. Given a "
    "binary path and a config, it loads the ELF, sets up the machine, runs three logic "
    "passes per core, and emits the final model."
))
for sig, desc in [
    ("fn model_rotor(binary_path, config, output) -> Result&lt;()&gt;",
     "The normal entry point. Loads the binary, builds <font face='Courier'>MachineSorts</font> "
     "and <font face='Courier'>MachineConstants</font>, creates one "
     "<font face='Courier'>CoreState</font> per requested core, and for each core runs the "
     "three logic phases: <b>combinational</b> (what does this step compute?), "
     "<b>sequential</b> (what's the new state?), <b>properties</b> (which expressions are "
     "we asking the solver to falsify?)."),
    ("fn model_rotor_synthesis(config, output) -> Result&lt;()&gt;",
     "Same shape, but with no binary — used for code-synthesis experiments. Builds an "
     "almost-empty 'binary' with a single NOP and lets the rest run on top."),
]:
    story.append(fn(sig, p(desc)))

# ----- 1.8 Combinational ---------------------------------------------------
story.append(Paragraph("1.8 Combinational logic — what one step computes", H2))
story.append(p(
    "<font face='Courier'>model/combinational.rs</font> describes one step of the machine "
    "as pure functions of the current state. The result is a struct, "
    "<font face='Courier'>CombinationalResult</font>, holding everything the next-state "
    "logic will need: which value goes into <font face='Courier'>rd</font>, what the next "
    "PC is, whether memory is being written, and so on."
))
story.append(code(
"""pub struct CombinationalResult {
    pub instruction_id:    NodeId,  // which RISC-V instruction
    pub rd_value:          NodeId,  // value to write to rd
    pub rd_addr:           NodeId,  // which register rd is
    pub next_pc:           NodeId,  // PC after this instruction
    pub store_addr:        NodeId,  // memory address (if storing)
    pub store_value:       NodeId,  // value to store
    pub store_width:       NodeId,  // 1, 2, 4 or 8 bytes
    pub writes_rd:         NodeId,  // is this a register-writing instr?
    pub writes_memory:     NodeId,  // is this a store?
    pub is_ecall:          NodeId,
    pub ir:                NodeId,  // raw 32-bit instruction word
    pub is_compressed:     NodeId,
    pub division_by_zero:  NodeId,  // for property checks
    pub invalid_address:   NodeId,
}"""
))

story.append(fn(
    "fn rotor_combinational(builder, sorts, consts, config, core) -> CombinationalResult",
    p("One huge function that, in order:"),
    *bullets([
        "<b>Fetches</b> the four bytes at <font face='Courier'>PC</font> from the code segment.",
        "Decides whether the instruction is <b>compressed</b> (low two bits != "
        "<font face='Courier'>11</font>); if so, expands the 16-bit form into the 32-bit "
        "form using <font face='Courier'>riscv/compressed.rs</font>.",
        "<b>Decodes</b> the instruction via <font face='Courier'>riscv/decode.rs</font>, "
        "producing an instruction ID and the various operand fields (rs1, rs2, rd, "
        "immediate).",
        "<b>Reads operands</b> from the register file: <font face='Courier'>rs1</font> and "
        "<font face='Courier'>rs2</font>.",
        "<b>Computes the data result</b> per instruction kind: ALU, multiply/divide, load, "
        "compare, branch target, jump target. All stitched together with cascaded selects "
        "based on the instruction ID.",
        "<b>Computes the next PC</b>: PC+4 by default, branch target if the branch is "
        "taken, jump target for jumps, or a 'do not advance' value for syscalls that "
        "don't.",
        "<b>Computes store address/value/width</b> if the instruction is a store.",
        "<b>Sets the property flags</b>: is this a divide where rs2 is zero? Is the load/"
        "store address out of every valid segment?",
    ]),
    p("The output is the <font face='Courier'>CombinationalResult</font> bag — pure "
      "functions of the current state, no commitments yet."),
))

# ----- 1.9 Sequential -------------------------------------------------------
story.append(Paragraph("1.9 Sequential logic — committing to the next state", H2))
story.append(p(
    "<font face='Courier'>model/sequential.rs</font> takes the bag from combinational and "
    "tells the model 'the next value of each piece of state equals this expression.' One "
    "function:"
))
story.append(fn(
    "fn rotor_sequential(builder, sorts, consts, config, core, comb)",
    p("Wires next-state for every mutable thing in the machine:"),
    *bullets([
        "<b>PC.</b> next = <font face='Courier'>comb.next_pc</font>.",
        "<b>Register file.</b> next = the old register file with "
        "<font face='Courier'>rd_addr</font> := <font face='Courier'>rd_value</font>, but "
        "only when the instruction actually writes a register. Ecall return values are "
        "folded in here (the kernel's <font face='Courier'>ecall_return_value</font> goes "
        "into <font face='Courier'>a0</font>).",
        "<b>Data segment.</b> next = old segment with the store applied, but only if the "
        "store target is in the data segment.",
        "<b>Heap segment.</b> Same, conditional on store-in-heap.",
        "<b>Stack segment.</b> Same, conditional on store-in-stack.",
        "<b>Program break.</b> next = <font face='Courier'>kernel.next_program_break(...)</font>.",
        "<b>readable_bytes.</b> next = old minus however many bytes the read syscall "
        "consumed this step.",
        "<b>read_bytes.</b> next = old plus those bytes.",
    ]),
))
story.append(p(
    "Note the discipline: combinational <i>builds</i> the candidate next values; "
    "sequential <i>names them</i>. Splitting these makes the per-instruction logic easy "
    "to reason about."
))

# ----- 1.10 Properties ------------------------------------------------------
story.append(Paragraph("1.10 Properties — what counts as 'a bug'", H2))
story.append(p(
    "<font face='Courier'>model/properties.rs</font> defines what the solver should try to "
    "falsify. Each enabled check becomes one boolean expression that, when true, means "
    "the program reached a bad state."
))
story.append(fn(
    "fn rotor_properties(builder, sorts, consts, config, core, comb)",
    p("Inspects the config and emits a check for each enabled property:"),
    *bullets([
        "<font face='Courier'>--check-bad-exit</font>: bug = the program called "
        "<font face='Courier'>exit</font> with a non-zero code.",
        "<font face='Courier'>--check-good-exit</font>: bug = the program called "
        "<font face='Courier'>exit(0)</font>. Useful for finding inputs that make a "
        "program <i>succeed</i>.",
        "<font face='Courier'>--check-exit</font>: bug = the program exited at all.",
        "<font face='Courier'>--check-div-zero</font>: bug = a division or remainder "
        "instruction had <font face='Courier'>rs2 == 0</font>.",
        "<font face='Courier'>--check-seg-faults</font>: bug = an instruction tried to "
        "load from or store to an address outside every segment.",
    ]),
))

# ----- 1.11 RISC-V layer ----------------------------------------------------
story.append(Paragraph("1.11 The RISC-V layer (decode and ELF loading)", H2))
story.append(p(
    "Three files under <font face='Courier'>riscv/</font> handle anything specific to the "
    "ISA. They are referenced by the combinational logic but are otherwise self-contained."
))
for sig, desc in [
    ("riscv/elf_loader.rs::load_elf(path) -> Result&lt;LoadedBinary&gt;",
     "Parses the ELF file, finds <font face='Courier'>.text</font> and "
     "<font face='Courier'>.data</font> sections, returns a flat struct with code bytes, "
     "data bytes, addresses, entry point, and 32/64-bit flag."),
    ("riscv/decode.rs::decode_instruction(builder, sorts, consts, config, ir) -> NodeId",
     "Given the 32-bit instruction word, returns its instruction ID. The function is the "
     "RISC-V decoder expressed as a giant cascade of opcode/funct3/funct7 comparisons. "
     "Each <font face='Courier'>InstrId::Add</font>, <font face='Courier'>Sub</font>, "
     "<font face='Courier'>Lw</font>, etc. gets one branch."),
    ("riscv/compressed.rs::expand_compressed(builder, sorts, consts, c_ir) -> NodeId",
     "Translates a 16-bit C-extension instruction into its equivalent 32-bit form so the "
     "main decoder can handle it uniformly."),
    ("riscv/isa.rs",
     "Constants for register indices (<font face='Courier'>SP=2</font>, "
     "<font face='Courier'>A0=10</font>, <font face='Courier'>A7=17</font>, etc.), "
     "instruction IDs as a flat enum, and bit-field constants (opcodes, funct3 values)."),
]:
    story.append(fn(sig, p(desc)))

# ===========================================================================
# Part 2 — Symbolic arguments
# ===========================================================================
story.append(PageBreak())
story.append(Paragraph("Part 2 — Symbolic arguments, end to end", H1))

story.append(Paragraph(KEY_INTRO :=
    "<b>What 'symbolic arguments' actually means.</b> A user wants to ask: 'can my "
    "program crash, exit nonzero, or hit some bad state for <i>any</i> command-line "
    "argument the user might type?' Symbolic arguments make argv an open question. "
    "Each character of <font face='Courier'>argv[1..N]</font> is left free; the solver "
    "fills it in to find a bug. Concretely: rotor lays out a normal argv on the stack, "
    "but instead of writing concrete characters, it writes 'unknown bytes' that the "
    "solver gets to choose. The program reads argv exactly as it would in a real OS — "
    "it doesn't know anything is symbolic.", KEY))

story.append(p(
    "Below: every function on the path from the command line to a program reading "
    "<font face='Courier'>argv[1][0]</font> in its main."
))

# ---------- 2.1 CLI ---------------------------------------------------------
story.append(Paragraph("2.1 CLI flags — main.rs", H2))
story.append(p("Three flags control the feature, defined in <font face='Courier'>main.rs</font>:"))
story.append(code(
"""/// Enable symbolic command-line arguments (argv)
#[arg(long)]
symbolic_argv: bool,                        // default: false

/// Number of symbolic arguments (requires --symbolic-argv)
#[arg(long, default_value_t = 1)]
symbolic_argc: usize,

/// Maximum length of each symbolic argument in bytes
#[arg(long, default_value_t = 8)]
max_arglen: usize,"""
))
for b in [
    "<b><font face='Courier'>--symbolic-argv</font></b>: turn the feature on. Without "
    "this, argv is empty and the program runs as if invoked with no arguments.",
    "<b><font face='Courier'>--symbolic-argc N</font></b>: how many symbolic arguments "
    "to provide (separately from <font face='Courier'>argv[0]</font>, which is always "
    "the literal string <font face='Courier'>\"prog\"</font>). Total argc seen by the "
    "program is <font face='Courier'>N + 1</font>.",
    "<b><font face='Courier'>--max-arglen K</font></b>: how many bytes of each symbolic "
    "argument are free. The argument is always K+1 bytes long counting the null "
    "terminator.",
]:
    story.append(Paragraph(f"&bull;&nbsp;&nbsp;{b}", BULLET))
story.append(p(
    "After parsing, <font face='Courier'>main</font> stuffs these three flags into a "
    "<font face='Courier'>Config</font> and calls <font face='Courier'>generator::model_rotor(...)</font>. "
    "The CLI never touches symbolic argv again."
))

# ---------- 2.2 Config ------------------------------------------------------
story.append(Paragraph("2.2 Config fields — config.rs", H2))
story.append(p(
    "The three flags become three fields on <font face='Courier'>Config</font> "
    "(<font face='Courier'>config.rs</font>):"
))
story.append(code(
"""pub struct Config {
    // ...
    pub symbolic_argv: bool,
    pub symbolic_argc: usize,
    pub max_arglen:    usize,
    // ...
}"""
))
story.append(p(
    "These are read once, much later, by <font face='Courier'>CoreState::new</font>. "
    "Nothing else in <font face='Courier'>config.rs</font> matters for symbolic arguments."
))

# ---------- 2.3 CoreState::new ----------------------------------------------
story.append(Paragraph("2.3 CoreState::new — three places it touches the feature", H2))
story.append(p(
    "<font face='Courier'>CoreState::new</font> wires symbolic arguments into the machine. "
    "Three short blocks, in order:"
))

story.append(Paragraph("(a) Choose the initial stack pointer", H4))
story.append(code(
"""let vaddr_top = 1u64 << (config.virtual_address_space - 1);

let (initial_sp, stack_init_val) =
    if config.symbolic_argv && config.symbolic_argc > 0 {
        Self::initialize_symbolic_argv(builder, sorts, config, vaddr_top, word_size)
    } else {
        let sp = vaddr_top - word_size;
        (sp, None)
    };"""
))
story.append(p(
    "If symbolic argv is enabled, hand off to <font face='Courier'>initialize_symbolic_argv</font> "
    "(section 2.4). It returns two things: <b>where the stack pointer should start</b> and "
    "<b>what the stack should look like at that point</b>. If symbolic argv is disabled, "
    "set SP to one word below the top and don't bother building a stack."
))

story.append(Paragraph("(b) Set the stack-pointer register", H4))
story.append(code(
"""let sp_val = builder.constd(sorts.sid_machine_word, initial_sp,
                            Some(format!("initial stack pointer 0x{:x}", initial_sp)));
let sp_addr = consts.nid_register(crate::riscv::isa::regs::SP);
let reg_with_sp = builder.write(
    sorts.sid_register_state, base_regs, sp_addr, sp_val,
    Some("set initial SP".to_string()),
);"""
))
story.append(p(
    "Take the SP value computed in (a), write it into register x2 (the stack pointer in "
    "RISC-V calling convention). When the program starts, it sees its stack pointer "
    "already pointing at the argv layout we built."
))

story.append(Paragraph("(c) Set a0 = argc", H4))
story.append(code(
"""let reg_after_argc = if config.symbolic_argv && config.symbolic_argc > 0 {
    let argc_val = builder.constd(sorts.sid_machine_word,
                                  (config.symbolic_argc + 1) as u64,
                                  Some(format!("argc = {}", config.symbolic_argc + 1)));
    let a0_addr = consts.nid_register(crate::riscv::isa::regs::A0);
    builder.write(sorts.sid_register_state, reg_with_sp, a0_addr, argc_val,
                  Some("set a0 = argc".to_string()))
} else {
    reg_with_sp
};"""
))
story.append(p(
    "RISC-V Linux startup code expects argc in <font face='Courier'>a0</font>. We write "
    "<font face='Courier'>symbolic_argc + 1</font> (because argv[0] is concrete and "
    "counts) into x10. Done. The program now has both pieces it needs to read its "
    "command line."
))

story.append(Paragraph("(d) Hand the stack value to the real stack segment", H4))
story.append(code(
"""let stack_segment_state = builder.state(sorts.sid_stack_state,
                                       &format!("{}stack-segment", prefix), ...);
if let Some(stack_val) = stack_init_val {
    let _stack_init = builder.init(
        sorts.sid_stack_state,
        stack_segment_state,
        stack_val,
        Some("init stack segment with argv".to_string()),
    );
}"""
))
story.append(p(
    "Bind the stack value built by <font face='Courier'>initialize_symbolic_argv</font> "
    "to be the stack segment's starting value. After this line, the running program will "
    "see the argv layout when it dereferences <font face='Courier'>**(sp+8)</font> and "
    "friends."
))

# ---------- 2.4 initialize_symbolic_argv -----------------------------------
story.append(Paragraph(
    "2.4 initialize_symbolic_argv — the headline function", H2
))
story.append(p(
    "Signature:"
))
story.append(code(
"""fn initialize_symbolic_argv(
    builder:    &mut Btor2Builder,
    sorts:      &MachineSorts,
    config:     &Config,
    stack_top:  u64,
    word_size:  u64,
) -> (u64, Option<NodeId>)"""
))
story.append(p(
    "Returns <font face='Courier'>(initial_sp, stack_value)</font>. The stack pointer is "
    "where SP should start; the stack value is what the stack array should look like at "
    "step 0. The function has no side effects beyond producing builder nodes."
))
story.append(p(
    "Walked phase by phase:"
))

story.append(Paragraph("Phase 1 — compute the layout", H3))
story.append(p(
    "Pure arithmetic. No machine state is touched yet."
))
story.append(code(
"""let total_argc      = config.symbolic_argc + 1;     // +1 for argv[0]
let max_arglen      = config.max_arglen;
let prog_name       = b"prog";

let argv0_len       = prog_name.len() + 1;          // "prog\\0" = 5 bytes
let sym_arg_len     = max_arglen + 1;               // K + null
let string_area_size    = argv0_len + config.symbolic_argc * sym_arg_len;
let string_area_aligned = (string_area_size as u64 + word_size - 1)
                          & !(word_size - 1);

let pointer_area_size  = (total_argc + 1) as u64 * word_size;
let total_stack_usage  = string_area_aligned + pointer_area_size + word_size;

let string_area_start  = stack_top - string_area_aligned;
let pointer_area_start = string_area_start - pointer_area_size;
let sp                 = pointer_area_start - word_size;     // argc lives at SP"""
))
story.append(p(
    "What this is doing: deciding the exact layout of argv on the stack, in the same "
    "shape a real OS would lay it out. Top of stack: the bytes of every argv string. "
    "Just below that: the array of pointers to those strings. At SP: the integer argc."
))
story.append(p("Picture (high address at the top):"))
story.append(code(
"""high address  +-----------------------------+
              |  bytes of argv[0]           |  "prog\\0"   (concrete)
              |  bytes of argv[1]           |  K bytes + null  (K symbolic)
              |  ...                        |
              |  bytes of argv[N]           |
              +-----------------------------+  string_area_start
              |  word-alignment padding     |
              +-----------------------------+
              |  pointer to argv[0] string  |  word_size bytes
              |  pointer to argv[1] string  |
              |  ...                        |
              |  pointer to argv[N] string  |
              |  NULL                        |  argv terminator
              +-----------------------------+  pointer_area_start
              |  argc                        |  word_size bytes
low address   +-----------------------------+  sp <-- initial stack pointer"""
))

story.append(Paragraph("Phase 2 — create the stack array", H3))
story.append(code(
"""let stack_seg = builder.state(sorts.sid_stack_state, "initial-stack-base", ...);
let mut current = stack_seg;"""
))
story.append(p(
    "<font face='Courier'>current</font> will track the stack value as we lay bytes into "
    "it. Each subsequent <font face='Courier'>builder.write</font> returns a new value "
    "that we assign back to <font face='Courier'>current</font>, so by the end of the "
    "function <font face='Courier'>current</font> holds the entire initial stack."
))
story.append(Paragraph(
    "Why we start with an empty <font face='Courier'>state</font> rather than a fixed "
    "array of zeros: the bytes we never touch (anywhere outside argv) are left free, "
    "so the program may not assume the rest of the stack starts at any particular value. "
    "In practice the program never reads those untouched bytes, so it never matters.",
    NOTE,
))

story.append(Paragraph("Phase 3 — write argv[0] = \"prog\\0\"", H3))
story.append(code(
"""let mut str_addr = string_area_start;
addrs.push(str_addr);                      // remember where argv[0] starts

for &byte_val in prog_name {
    let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
    let val  = builder.constd(sorts.sid_byte, byte_val as u64,
                              Some(format!("argv[0][{}] = '{}'",
                                           str_addr - string_area_start,
                                           byte_val as char)));
    current  = builder.write(sorts.sid_stack_state, current, addr, val, None);
    str_addr += 1;
}
// null terminator
let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
let null = builder.constd(sorts.sid_byte, 0,
                          Some("argv[0] null terminator".to_string()));
current  = builder.write(sorts.sid_stack_state, current, addr, null, None);
str_addr += 1;"""
))
story.append(p(
    "Concrete bytes. The program name is always the literal string "
    "<font face='Courier'>\"prog\\0\"</font>. Programs that read "
    "<font face='Courier'>argv[0]</font> see those exact characters."
))

story.append(Paragraph(
    "Phase 4 — write argv[1..N] (the symbolic part)", H3
))
story.append(code(
"""for arg_idx in 0..config.symbolic_argc {
    addrs.push(str_addr);                  // pointer-area entry
    for byte_idx in 0..max_arglen {
        let addr = builder.constd(sorts.sid_stack_address, str_addr, None);

        // The actual symbolic byte. Each one is a fresh free 8-bit value:
        // the solver picks any 0..255.
        let sym_byte = builder.state(
            sorts.sid_byte,
            &format!("argv[{}][{}]", arg_idx + 1, byte_idx),
            Some(format!("symbolic byte argv[{}][{}]", arg_idx + 1, byte_idx)),
        );

        current = builder.write(sorts.sid_stack_state, current, addr, sym_byte, None);
        str_addr += 1;
    }
    // null terminator stays concrete
    let addr = builder.constd(sorts.sid_stack_address, str_addr, None);
    let null = builder.constd(sorts.sid_byte, 0,
                              Some(format!("argv[{}] null terminator", arg_idx + 1)));
    current  = builder.write(sorts.sid_stack_state, current, addr, null, None);
    str_addr += 1;
}"""
))
story.append(Paragraph(
    "<b>This is where 'symbolic' is born.</b> One line creates each free byte: "
    "<font face='Courier'>let sym_byte = builder.state(sorts.sid_byte, "
    "\"argv[i][j]\", ...);</font>. There are <font face='Courier'>symbolic_argc * "
    "max_arglen</font> of these in total. Every other byte on the stack is concrete; "
    "only these ones are free. That is the entire mechanism.",
    KEY,
))
story.append(p(
    "After this loop, the stack has the program name in concrete bytes, the symbolic "
    "argument strings in free bytes, and null terminators between them. The strings "
    "are addressable as if they were normal C strings."
))

story.append(Paragraph("Phase 5 — write the pointer area", H3))
story.append(code(
"""let mut ptr_addr = pointer_area_start;
for (i, &string_addr) in argv_string_addrs.iter().enumerate() {
    // Write a word-sized pointer in little-endian byte order
    for byte_idx in 0..word_size {
        let byte_val = (string_addr >> (byte_idx * 8)) & 0xFF;
        let addr = builder.constd(sorts.sid_stack_address, ptr_addr + byte_idx, None);
        let val  = builder.constd(sorts.sid_byte, byte_val,
                                  Some(format!("argv_ptr[{}] byte {}", i, byte_idx)));
        current  = builder.write(sorts.sid_stack_state, current, addr, val, None);
    }
    ptr_addr += word_size;
}
// NULL pointer terminator
for byte_idx in 0..word_size {
    let addr = builder.constd(sorts.sid_stack_address, ptr_addr + byte_idx, None);
    let val  = builder.constd(sorts.sid_byte, 0,
                              Some("argv NULL terminator byte".to_string()));
    current  = builder.write(sorts.sid_stack_state, current, addr, val, None);
}"""
))
story.append(p(
    "All concrete. Every entry in the argv pointer array points to the corresponding "
    "string we wrote in phases 3 and 4, plus a NULL pointer at the end (POSIX requires "
    "the array to be NULL-terminated)."
))

story.append(Paragraph("Phase 6 — write argc at SP", H3))
story.append(code(
"""let argc_value = total_argc as u64;
for byte_idx in 0..word_size {
    let byte_val = (argc_value >> (byte_idx * 8)) & 0xFF;
    let addr = builder.constd(sorts.sid_stack_address, sp + byte_idx, None);
    let val  = builder.constd(sorts.sid_byte, byte_val,
                              Some(format!("argc byte {}", byte_idx)));
    current  = builder.write(sorts.sid_stack_state, current, addr, val, None);
}

(sp, Some(current))"""
))
story.append(p(
    "argc itself is also written to memory at SP — RISC-V startup code can either find "
    "it in <font face='Courier'>a0</font> (set by section 2.3c) or load it from the "
    "stack. Either way it sees the same value."
))
story.append(p(
    "Then we return: SP and the fully built stack value."
))

# ---------- 2.5 What happens when the program reads argv -------------------
story.append(Paragraph("2.5 What the running program actually sees", H2))
story.append(p(
    "Once the model is set up, the program runs normally. There is no special path for "
    "symbolic data. Take a typical access pattern: the program reads "
    "<font face='Courier'>argv[1][0]</font>."
))
for i, t in enumerate([
    "The program loads <font face='Courier'>argv</font> from the stack: "
    "<font face='Courier'>ld a1, 8(sp)</font> (or similar). This goes through "
    "<font face='Courier'>Memory::load_double_word</font> on the stack segment, returns "
    "the pointer-area value we wrote in Phase 5 — a concrete address.",
    "The program loads <font face='Courier'>argv[1]</font>: another "
    "<font face='Courier'>ld</font> at <font face='Courier'>a1+8</font>, returning the "
    "address of the first symbolic string.",
    "The program loads <font face='Courier'>argv[1][0]</font>: "
    "<font face='Courier'>lb t0, 0(a1)</font>, which calls "
    "<font face='Courier'>Memory::load_byte</font> on the stack at the address of the "
    "first symbolic byte. The result is the free 8-bit value we created in Phase 4.",
    "The program does <font face='Courier'>if (t0 == 'X')</font>: this becomes a "
    "comparison whose result is <i>itself</i> a free boolean. The branch becomes a "
    "select — both sides are part of the model.",
    "The solver, when asked 'is there an argv that makes the program reach the bad "
    "exit?', picks values for every symbolic byte that satisfy the chain of branch "
    "conditions leading to the bug.",
], 1):
    story.append(Paragraph(f"{i}.&nbsp;&nbsp;{t}", BULLET))
story.append(p(
    "Crucially: nothing in <font face='Courier'>Memory</font>, "
    "<font face='Courier'>RegisterFile</font>, the decoder, the kernel, or the "
    "combinational/sequential logic has to know about symbolic argv. They all just do "
    "their normal job. The freedom in the bytes flows through the model wherever the "
    "data flows."
))

# ---------- 2.6 End-to-end --------------------------------------------------
story.append(Paragraph("2.6 End-to-end on a real benchmark", H2))
story.append(p(
    "The benchmark <font face='Courier'>test4_multi_arg.c</font> is intentionally "
    "minimal. It exits 1 only when <font face='Courier'>argv[1][0] == 'X'</font> "
    "<i>and</i> <font face='Courier'>argv[2][0] == 'Y'</font> at the same time."
))
story.append(code(
"""uint64_t main(uint64_t argc, uint64_t* argv) {
    if (argc > 2) {
        if (((uint64_t*) *(argv + 1))[0] - (((uint64_t*) *(argv + 1))[0] / 256) * 256 == 88)
            if (((uint64_t*) *(argv + 2))[0] - (((uint64_t*) *(argv + 2))[0] / 256) * 256 == 89)
                return 1;
    }
    return 0;
}"""
))
story.append(p("From source to witness:"))
story.append(code(
"""$ selfie -c test4_multi_arg.c -o test4.m
                                    # produces a RISC-V binary

$ rotor test4.m --symbolic-argv --symbolic-argc 2 --max-arglen 8 -o test4.btor2
                                    # CoreState::new is called once.
                                    # initialize_symbolic_argv lays out argv:
                                    #    argv[0] = "prog\\0"   (concrete)
                                    #    argv[1] = 8 free bytes + null
                                    #    argv[2] = 8 free bytes + null
                                    # SP, a0, stack segment all wired up.
                                    # Combinational + sequential + properties
                                    # produce the rest of the model.

$ btormc -kmax 200 test4.btor2 > test4.wit
                                    # the solver finds an assignment:
                                    #    argv[1][0] = 88   ('X')
                                    #    argv[2][0] = 89   ('Y')
                                    # under which the program reaches return 1,
                                    # which makes exit_status nonzero,
                                    # which trips the "bad exit" property.

$ visualizer/index.html  <-  load test4.btor2 + test4.wit
                                    # see the witness step by step"""
))

# ---------- 2.7 Recap -------------------------------------------------------
story.append(Paragraph("2.7 Recap — every function on the symbolic-argv path", H2))
story.append(section_table([
    ["Layer", "Function", "What it does"],
    ["CLI", "main()",
     "Parses --symbolic-argv, --symbolic-argc, --max-arglen; builds Config."],
    ["Pipeline", "generator::model_rotor",
     "Loads the binary, sets up sorts/consts, calls CoreState::new for each core."],
    ["Core setup", "CoreState::new",
     "Branches on config.symbolic_argv. If on, calls initialize_symbolic_argv, "
     "writes SP and a0=argc into the register file, attaches the stack value to the "
     "stack segment."],
    ["Argv layout", "CoreState::initialize_symbolic_argv",
     "Computes the argv layout, creates the stack value: concrete bytes for argv[0] "
     "and pointers/argc, free bytes for argv[1..N]."],
    ["Reads at runtime",
     "Memory::load_byte / load_word / load_double_word on stack_segment_state",
     "When the program reads argv, this is what fires. No symbolic-aware code; the "
     "freedom is already in the bytes."],
    ["Properties", "rotor_properties",
     "Defines what counts as a bug. Reachability of any of these is what the solver "
     "tries to prove."],
], col_widths=[2.6*cm, 6.0*cm, 7.6*cm]))

story.append(Paragraph(
    "<b>The whole feature is one new file-local function plus three short blocks in "
    "<font face='Courier'>CoreState::new</font>.</b> Every other module in the codebase "
    "is unchanged. That is what makes the design clean: 'symbolic argv' is a property of "
    "the initial stack contents, not a special path through the rest of the machine.",
    KEY,
))

# ---------------------------------------------------------------------------
doc.build(story)
print(f"Wrote {OUT_PATH}")
