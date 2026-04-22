// Generate a presentation on symbolic arguments in Rotor.
// Requires: npm install -g pptxgenjs

const pptxgen = require("pptxgenjs");

let pres = new pptxgen();
pres.layout = "LAYOUT_WIDE"; // 13.3 x 7.5
pres.title = "Symbolic Arguments in Rotor";
pres.author = "Jasmin Begic";

// -------- Palette (Midnight Executive, tuned) --------
const NAVY   = "1E2761";
const NAVY_D = "141A47";
const ICE    = "CADCFC";
const WHITE  = "FFFFFF";
const GOLD   = "F2B705";
const MUTED  = "6B7280";
const DARK   = "111827";
const LIGHT  = "F7F9FC";
const CODE_BG = "0B1330";
const CODE_FG = "E6EDF7";
const GREEN  = "10B981";
const RED    = "EF4444";

const H_FONT = "Georgia";
const B_FONT = "Calibri";
const C_FONT = "Consolas";

// slide dimensions
const SW = 13.3;
const SH = 7.5;

// helper: footer bar
function addFooter(slide, pageNum, total) {
  slide.addShape(pres.shapes.RECTANGLE, {
    x: 0, y: SH - 0.3, w: SW, h: 0.3,
    fill: { color: NAVY_D }, line: { color: NAVY_D }
  });
  slide.addText("Symbolic Arguments in Rotor", {
    x: 0.4, y: SH - 0.3, w: 6, h: 0.3,
    fontSize: 10, fontFace: B_FONT, color: ICE, valign: "middle", margin: 0
  });
  slide.addText(`${pageNum} / ${total}`, {
    x: SW - 1.2, y: SH - 0.3, w: 0.8, h: 0.3,
    fontSize: 10, fontFace: B_FONT, color: ICE, align: "right", valign: "middle", margin: 0
  });
}

// helper: title bar for content slides
function addTitleBar(slide, title, subtitle) {
  slide.addShape(pres.shapes.RECTANGLE, {
    x: 0, y: 0, w: SW, h: 1.1,
    fill: { color: NAVY }, line: { color: NAVY }
  });
  // accent square
  slide.addShape(pres.shapes.RECTANGLE, {
    x: 0.4, y: 0.35, w: 0.4, h: 0.4,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  slide.addText(title, {
    x: 1.0, y: 0.15, w: SW - 1.4, h: 0.55,
    fontSize: 28, fontFace: H_FONT, color: WHITE, bold: true, valign: "middle", margin: 0
  });
  if (subtitle) {
    slide.addText(subtitle, {
      x: 1.0, y: 0.68, w: SW - 1.4, h: 0.35,
      fontSize: 13, fontFace: B_FONT, color: ICE, valign: "middle", margin: 0
    });
  }
}

// helper: code block
function addCodeBlock(slide, x, y, w, h, code, opts = {}) {
  slide.addShape(pres.shapes.RECTANGLE, {
    x, y, w, h, fill: { color: CODE_BG }, line: { color: CODE_BG }
  });
  slide.addText(code, {
    x: x + 0.15, y: y + 0.1, w: w - 0.3, h: h - 0.2,
    fontSize: opts.fontSize || 12, fontFace: C_FONT, color: CODE_FG,
    valign: "top", margin: 0
  });
}

// helper: card
function addCard(slide, x, y, w, h, title, body, accent) {
  slide.addShape(pres.shapes.RECTANGLE, {
    x, y, w, h, fill: { color: WHITE }, line: { color: "E2E8F0" },
    shadow: { type: "outer", color: "000000", blur: 6, offset: 2, angle: 135, opacity: 0.08 }
  });
  slide.addShape(pres.shapes.RECTANGLE, {
    x, y, w: 0.08, h, fill: { color: accent || NAVY }, line: { color: accent || NAVY }
  });
  slide.addText(title, {
    x: x + 0.25, y: y + 0.15, w: w - 0.4, h: 0.45,
    fontSize: 16, fontFace: H_FONT, color: NAVY, bold: true, valign: "middle", margin: 0
  });
  slide.addText(body, {
    x: x + 0.25, y: y + 0.6, w: w - 0.4, h: h - 0.75,
    fontSize: 12, fontFace: B_FONT, color: DARK, valign: "top", margin: 0, paraSpaceAfter: 4
  });
}

const TOTAL = 10;

// =================================================================
// Slide 1 — Title
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: NAVY };

  // gold accent
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.8, y: 2.8, w: 0.6, h: 0.08,
    fill: { color: GOLD }, line: { color: GOLD }
  });

  s.addText("Symbolic Arguments in Rotor", {
    x: 0.8, y: 3.0, w: 11.5, h: 1.2,
    fontSize: 54, fontFace: H_FONT, color: WHITE, bold: true, margin: 0
  });

  s.addText("How unconstrained argv lets a model checker discover triggering inputs", {
    x: 0.8, y: 4.3, w: 11.5, h: 0.7,
    fontSize: 22, fontFace: B_FONT, color: ICE, italic: true, margin: 0
  });

  s.addText([
    { text: "Rotor  ", options: { color: GOLD, bold: true } },
    { text: "·  BTOR2  ", options: { color: ICE } },
    { text: "·  btormc  ", options: { color: ICE } },
    { text: "·  RISC-V", options: { color: ICE } }
  ], {
    x: 0.8, y: 5.3, w: 11.5, h: 0.5,
    fontSize: 16, fontFace: B_FONT, margin: 0
  });

  s.addText("Jasmin Begic  ·  University of Salzburg", {
    x: 0.8, y: 6.6, w: 11.5, h: 0.4,
    fontSize: 13, fontFace: B_FONT, color: ICE, margin: 0
  });
}

// =================================================================
// Slide 2 — Concrete vs symbolic
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: LIGHT };
  addTitleBar(s, "Concrete vs Symbolic Arguments",
    "The key difference: who picks the input — you, or the solver?");

  // Two large cards
  // Concrete
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.7, y: 1.5, w: 5.8, h: 5.3,
    fill: { color: WHITE }, line: { color: "E2E8F0" },
    shadow: { type: "outer", color: "000000", blur: 6, offset: 2, angle: 135, opacity: 0.08 }
  });
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.7, y: 1.5, w: 5.8, h: 0.7, fill: { color: MUTED }, line: { color: MUTED }
  });
  s.addText("Concrete run", {
    x: 0.9, y: 1.5, w: 5.4, h: 0.7,
    fontSize: 20, fontFace: H_FONT, color: WHITE, bold: true, valign: "middle", margin: 0
  });

  s.addText([
    { text: "You pick the input.", options: { bold: true, breakLine: true } },
    { text: "The program runs with exactly those bytes.", options: { breakLine: true } },
    { text: "If the bug hides behind a specific argv, you must already know it.", options: {} }
  ], {
    x: 0.9, y: 2.3, w: 5.4, h: 1.3,
    fontSize: 13, fontFace: B_FONT, color: DARK, margin: 0, paraSpaceAfter: 4
  });

  addCodeBlock(s, 0.9, 3.7, 5.4, 2.9,
    "$ ./program HELLO\n" +
    "  argv[1] = \"HELLO\"\n" +
    "  exit code: 0   (no crash)\n\n" +
    "$ ./program CRASH\n" +
    "  argv[1] = \"CRASH\"\n" +
    "  exit code: 1   (bug!)\n\n" +
    "# You had to guess \"CRASH\".",
    { fontSize: 12 });

  // Symbolic
  s.addShape(pres.shapes.RECTANGLE, {
    x: 6.8, y: 1.5, w: 5.8, h: 5.3,
    fill: { color: WHITE }, line: { color: "E2E8F0" },
    shadow: { type: "outer", color: "000000", blur: 6, offset: 2, angle: 135, opacity: 0.08 }
  });
  s.addShape(pres.shapes.RECTANGLE, {
    x: 6.8, y: 1.5, w: 5.8, h: 0.7, fill: { color: NAVY }, line: { color: NAVY }
  });
  s.addText("Symbolic run", {
    x: 7.0, y: 1.5, w: 5.4, h: 0.7,
    fontSize: 20, fontFace: H_FONT, color: WHITE, bold: true, valign: "middle", margin: 0
  });

  s.addText([
    { text: "The solver picks the input.", options: { bold: true, breakLine: true } },
    { text: "Each argv byte is an unconstrained bitvector.", options: { breakLine: true } },
    { text: "btormc searches over ALL possible argv values for one that reaches a bad state.", options: {} }
  ], {
    x: 7.0, y: 2.3, w: 5.4, h: 1.3,
    fontSize: 13, fontFace: B_FONT, color: DARK, margin: 0, paraSpaceAfter: 4
  });

  addCodeBlock(s, 7.0, 3.7, 5.4, 2.9,
    "$ btormc prog.btor2\n" +
    "  sat\n" +
    "  b0   (bug reached)\n" +
    "  argv[1][0] = 0x43  ('C')\n" +
    "  argv[1][1] = 0x52  ('R')\n" +
    "  argv[1][2] = 0x41  ('A')\n" +
    "  argv[1][3] = 0x53  ('S')\n" +
    "  argv[1][4] = 0x48  ('H')",
    { fontSize: 12 });

  addFooter(s, 2, TOTAL);
}

// =================================================================
// Slide 3 — Why symbolic argv matters
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: LIGHT };
  addTitleBar(s, "Why Symbolic argv Matters",
    "Symbolic stdin is not enough. Some bugs are only reachable via command-line input.");

  // Three cards in a row
  addCard(s, 0.7, 1.6, 3.9, 2.6,
    "Beyond stdin",
    "Many binaries never call read() but still parse argv.\n\n" +
    "Without symbolic argv, those paths are invisible to the model checker.",
    RED);

  addCard(s, 4.7, 1.6, 3.9, 2.6,
    "Inputs the CPU sees",
    "argv bytes are memory the program actually loads with lb / lw.\n\n" +
    "Modelling them symbolically lets the solver reason about every dependent branch.",
    GOLD);

  addCard(s, 8.7, 1.6, 3.9, 2.6,
    "Witness = recipe",
    "A satisfying assignment is not a report — it is the exact argv that triggers the bug.\n\n" +
    "You can replay it on real hardware.",
    GREEN);

  // Bottom quote / banner
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.7, y: 4.5, w: 11.9, h: 2.3,
    fill: { color: NAVY }, line: { color: NAVY }
  });
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.7, y: 4.5, w: 0.15, h: 2.3,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  s.addText("What changes formally?", {
    x: 1.0, y: 4.6, w: 11.3, h: 0.45,
    fontSize: 18, fontFace: H_FONT, color: GOLD, bold: true, margin: 0
  });
  s.addText([
    { text: "Concrete:  ", options: { bold: true, color: ICE, breakLine: false } },
    { text: "M runs on one input x → check one final state.", options: { color: WHITE, breakLine: true } },
    { text: "Symbolic:  ", options: { bold: true, color: ICE, breakLine: false } },
    { text: "∃ argv . reachable(M[argv], bad)?  — a solver query over the whole input space, bounded by kmax steps.", options: { color: WHITE } }
  ], {
    x: 1.0, y: 5.1, w: 11.3, h: 1.6,
    fontSize: 15, fontFace: B_FONT, margin: 0, paraSpaceAfter: 8
  });

  addFooter(s, 3, TOTAL);
}

// =================================================================
// Slide 4 — How it works in Rotor (stack layout)
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: LIGHT };
  addTitleBar(s, "How Rotor Builds Symbolic argv",
    "The stack is laid out exactly as the SysV RISC-V ABI expects — but argv bytes are unconstrained states.");

  // Left: bullet list of steps
  s.addText("Rotor initializes the stack segment in four steps:", {
    x: 0.7, y: 1.4, w: 6.0, h: 0.4,
    fontSize: 14, fontFace: B_FONT, color: DARK, bold: true, margin: 0
  });

  s.addText([
    { text: "argv[0] = \"prog\"", options: { bold: true, color: NAVY, breakLine: true } },
    { text: "Fixed program name (concrete bytes + null terminator).", options: { color: DARK, breakLine: true } },
    { text: " ", options: { breakLine: true } },

    { text: "argv[1..N] = symbolic states", options: { bold: true, color: NAVY, breakLine: true } },
    { text: "Each byte is a fresh unconstrained BTOR2 state — the solver picks 0..255.", options: { color: DARK, breakLine: true } },
    { text: " ", options: { breakLine: true } },

    { text: "Pointer array + NULL terminator", options: { bold: true, color: NAVY, breakLine: true } },
    { text: "Concrete addresses pointing into the string area.", options: { color: DARK, breakLine: true } },
    { text: " ", options: { breakLine: true } },

    { text: "argc + sp", options: { bold: true, color: NAVY, breakLine: true } },
    { text: "argc is fixed at symbolic_argc + 1; register a0 = argc; sp points to argc.", options: { color: DARK } }
  ], {
    x: 0.7, y: 1.85, w: 6.0, h: 4.8,
    fontSize: 13, fontFace: B_FONT, margin: 0, paraSpaceAfter: 2
  });

  // Right: stack diagram
  const dx = 7.4, dy = 1.4, dw = 5.2;
  s.addText("Stack layout (high → low)", {
    x: dx, y: dy, w: dw, h: 0.35,
    fontSize: 13, fontFace: B_FONT, color: MUTED, italic: true, bold: true, margin: 0
  });

  const rows = [
    { label: "argv[1..N] symbolic bytes + '\\0'", color: GOLD,  fg: DARK },
    { label: "argv[0] = \"prog\\0\"",            color: MUTED, fg: WHITE },
    { label: "argv ptr [0]",                     color: ICE,   fg: DARK },
    { label: "argv ptr [1..N]",                  color: ICE,   fg: DARK },
    { label: "NULL terminator",                  color: ICE,   fg: DARK },
    { label: "argc  ← sp",                       color: NAVY,  fg: WHITE },
  ];
  const rowH = 0.65;
  rows.forEach((r, i) => {
    const y = dy + 0.4 + i * (rowH + 0.1);
    s.addShape(pres.shapes.RECTANGLE, {
      x: dx, y, w: dw, h: rowH,
      fill: { color: r.color }, line: { color: "E2E8F0" }
    });
    s.addText(r.label, {
      x: dx + 0.2, y, w: dw - 0.4, h: rowH,
      fontSize: 13, fontFace: C_FONT, color: r.fg, valign: "middle", margin: 0
    });
  });
  // Arrow label
  s.addText("↓  growing down", {
    x: dx, y: dy + 0.4 + rows.length * (rowH + 0.1) + 0.05, w: dw, h: 0.3,
    fontSize: 11, fontFace: B_FONT, color: MUTED, italic: true, align: "center", margin: 0
  });

  addFooter(s, 4, TOTAL);
}

// =================================================================
// Slide 5 — The one-line trick (state vs input)
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: LIGHT };
  addTitleBar(s, "The One-Line Trick: state, not input",
    "BTOR2 forbids inputs in init expressions — so we use an uninitialised state.");

  // Left text
  s.addText([
    { text: "An uninitialised ", options: { color: DARK } },
    { text: "state", options: { bold: true, color: NAVY } },
    { text: " in BTOR2 is ", options: { color: DARK } },
    { text: "unconstrained", options: { bold: true, color: NAVY } },
    { text: " — the solver is free to choose any value at step 0, and propagate it through the execution.", options: { color: DARK, breakLine: true } },
    { text: " ", options: { breakLine: true } },
    { text: "That is exactly the semantics we want for a symbolic byte.", options: { color: DARK, italic: true } }
  ], {
    x: 0.7, y: 1.5, w: 5.8, h: 2.0,
    fontSize: 14, fontFace: B_FONT, margin: 0
  });

  // Small callout
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.7, y: 3.7, w: 5.8, h: 2.9,
    fill: { color: WHITE }, line: { color: "E2E8F0" }
  });
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.7, y: 3.7, w: 0.1, h: 2.9,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  s.addText("Why not just use input?", {
    x: 0.95, y: 3.8, w: 5.4, h: 0.45,
    fontSize: 16, fontFace: H_FONT, color: NAVY, bold: true, margin: 0
  });
  s.addText([
    { text: "BTOR2's init operator requires a constant or a state — not an input node.", options: { breakLine: true } },
    { text: " ", options: { breakLine: true } },
    { text: "Using an uninitialised state gives us the same freedom (solver picks any bitvector) while staying inside the grammar.", options: {} }
  ], {
    x: 0.95, y: 4.3, w: 5.4, h: 2.2,
    fontSize: 13, fontFace: B_FONT, color: DARK, margin: 0, paraSpaceAfter: 4
  });

  // Right: code snippet
  addCodeBlock(s, 6.9, 1.5, 5.8, 5.1,
    "// rotor/src/machine/core.rs (excerpt)\n\n" +
    "for byte_idx in 0..max_arglen {\n" +
    "    let addr = builder.constd(\n" +
    "        sorts.sid_stack_address, str_addr, None);\n\n" +
    "    // Unconstrained state = symbolic byte.\n" +
    "    // BTOR2 forbids 'input' in init, so we use\n" +
    "    // an uninitialised state instead.\n" +
    "    let sym_byte = builder.state(\n" +
    "        sorts.sid_byte,\n" +
    "        &format!(\"argv[{}][{}]\",\n" +
    "                 arg_idx + 1, byte_idx),\n" +
    "        Some(\"symbolic byte\".into()),\n" +
    "    );\n\n" +
    "    current = builder.write(\n" +
    "        sorts.sid_stack_state,\n" +
    "        current, addr, sym_byte, None);\n" +
    "    str_addr += 1;\n" +
    "}",
    { fontSize: 13 });

  addFooter(s, 5, TOTAL);
}

// =================================================================
// Slide 6 — The five test programs
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: LIGHT };
  addTitleBar(s, "Five Test Programs",
    "Each one hides a bug that only triggers on a specific argv — a benchmark for the solver.");

  // Table
  const rows = [
    [
      { text: "#",          options: { bold: true, color: WHITE, fill: { color: NAVY }, valign: "middle", align: "center" } },
      { text: "File",       options: { bold: true, color: WHITE, fill: { color: NAVY }, valign: "middle" } },
      { text: "Bug condition",          options: { bold: true, color: WHITE, fill: { color: NAVY }, valign: "middle" } },
      { text: "Expected finding",       options: { bold: true, color: WHITE, fill: { color: NAVY }, valign: "middle" } },
    ],
    [
      { text: "1", options: { align: "center" } },
      "test1_crash_string.c",
      "argv[1][0] == 'C' (first byte)",
      "argv[1] = { 0x43, ... }",
    ],
    [
      { text: "2", options: { align: "center" } },
      "test2_numeric_overflow.c",
      "word(argv[1]) == 0x4142  ('A','B')",
      "argv[1] = { 0x41, 0x42 }",
    ],
    [
      { text: "3", options: { align: "center" } },
      "test3_length_dependent.c",
      "strlen(argv[1]) == 1  (b0≠0 ∧ b1=0)",
      "any 1-byte non-null string",
    ],
    [
      { text: "4", options: { align: "center" } },
      "test4_multi_arg.c",
      "argv[1][0]=='X' ∧ argv[2][0]=='Y'",
      "two coupled args  (argc=2)",
    ],
    [
      { text: "5", options: { align: "center" } },
      "test5_checksum.c",
      "byte0 + byte1 == 200",
      "e.g. { 100, 100, ... }",
    ],
  ];

  s.addTable(rows, {
    x: 0.7, y: 1.5, w: 11.9,
    colW: [0.7, 3.4, 4.3, 3.5],
    rowH: 0.55,
    fontFace: B_FONT, fontSize: 13, color: DARK,
    border: { pt: 0.5, color: "E2E8F0" },
    valign: "middle",
  });

  // Key insight callout
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.7, y: 5.9, w: 11.9, h: 1.1,
    fill: { color: NAVY }, line: { color: NAVY }
  });
  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.7, y: 5.9, w: 0.15, h: 1.1,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  s.addText([
    { text: "Key point.  ", options: { bold: true, color: GOLD } },
    { text: "Tests 2, 4 and 5 cannot be found by symbolic stdin alone — the programs never call read(). " +
            "Only symbolic argv exposes these branches.", options: { color: WHITE } }
  ], {
    x: 1.0, y: 6.0, w: 11.5, h: 0.9,
    fontSize: 14, fontFace: B_FONT, valign: "middle", margin: 0
  });

  addFooter(s, 6, TOTAL);
}

// =================================================================
// Slide 7 — A concrete test program (test1)
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: LIGHT };
  addTitleBar(s, "Example: test1_crash_string.c",
    "A minimal program whose exit code depends on one byte of argv[1].");

  // Left code
  addCodeBlock(s, 0.7, 1.5, 6.6, 5.3,
    "uint64_t main(uint64_t argc, uint64_t* argv) {\n" +
    "    uint64_t* arg1;\n" +
    "    uint64_t  first_word, first_byte;\n\n" +
    "    if (argc > 1) {\n" +
    "        arg1       = (uint64_t*) *(argv + 1);\n" +
    "        first_word = *arg1;\n\n" +
    "        // extract low byte (no bitwise & in C*)\n" +
    "        first_byte = first_word\n" +
    "                   - (first_word / 256) * 256;\n\n" +
    "        if (first_byte == 67)   // 'C'\n" +
    "            return 1;            // <- bad exit\n" +
    "    }\n" +
    "    return 0;\n" +
    "}",
    { fontSize: 14 });

  // Right: what the solver sees
  s.addText("What the solver sees", {
    x: 7.6, y: 1.5, w: 5.1, h: 0.45,
    fontSize: 18, fontFace: H_FONT, color: NAVY, bold: true, margin: 0
  });

  s.addText([
    { text: "argc  ", options: { bold: true, color: NAVY } },
    { text: "is fixed (= 2).", options: { color: DARK, breakLine: true } },
    { text: "argv[0]  ", options: { bold: true, color: NAVY } },
    { text: "= \"prog\"  (fixed).", options: { color: DARK, breakLine: true } },
    { text: "argv[1][0..7]  ", options: { bold: true, color: NAVY } },
    { text: "= 8 unconstrained bytes.", options: { color: DARK, breakLine: true } },
    { text: " ", options: { breakLine: true } },
    { text: "Bad state:  ", options: { bold: true, color: RED } },
    { text: "program exits with a non-zero status.", options: { color: DARK, breakLine: true } },
    { text: " ", options: { breakLine: true } },
    { text: "btormc answer:  ", options: { bold: true, color: GREEN } },
    { text: "assign argv[1][0] = 0x43 ('C') — all other bytes may be anything.", options: { color: DARK } }
  ], {
    x: 7.6, y: 2.0, w: 5.1, h: 3.2,
    fontSize: 14, fontFace: B_FONT, margin: 0, paraSpaceAfter: 3
  });

  // Small witness box
  addCodeBlock(s, 7.6, 5.3, 5.1, 1.5,
    "sat\n" +
    "b0\n" +
    "#0  state argv[1][0]  0x43   'C'\n" +
    "#0  state argv[1][1]  0x00   (any)",
    { fontSize: 12 });

  addFooter(s, 7, TOTAL);
}

// =================================================================
// Slide 8 — How to test it out (commands)
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: LIGHT };
  addTitleBar(s, "How to Test It Out",
    "Three steps: compile → generate BTOR2 → run the model checker.");

  // Step 1
  s.addShape(pres.shapes.OVAL, {
    x: 0.7, y: 1.5, w: 0.5, h: 0.5,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  s.addText("1", {
    x: 0.7, y: 1.5, w: 0.5, h: 0.5,
    fontSize: 20, fontFace: H_FONT, color: NAVY, bold: true, align: "center", valign: "middle", margin: 0
  });
  s.addText("Compile to RISC-V (selfie C*)", {
    x: 1.3, y: 1.5, w: 11.3, h: 0.5,
    fontSize: 17, fontFace: H_FONT, color: NAVY, bold: true, valign: "middle", margin: 0
  });
  addCodeBlock(s, 1.3, 2.05, 11.3, 0.55,
    "$ selfie -c benchmarks/argv-tests/test1_crash_string.c -o test1.m",
    { fontSize: 13 });

  // Step 2
  s.addShape(pres.shapes.OVAL, {
    x: 0.7, y: 2.8, w: 0.5, h: 0.5,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  s.addText("2", {
    x: 0.7, y: 2.8, w: 0.5, h: 0.5,
    fontSize: 20, fontFace: H_FONT, color: NAVY, bold: true, align: "center", valign: "middle", margin: 0
  });
  s.addText("Generate BTOR2 with symbolic argv", {
    x: 1.3, y: 2.8, w: 11.3, h: 0.5,
    fontSize: 17, fontFace: H_FONT, color: NAVY, bold: true, valign: "middle", margin: 0
  });
  addCodeBlock(s, 1.3, 3.35, 11.3, 1.2,
    "$ cargo run --release -- test1.m \\\n" +
    "      --symbolic-argv  --symbolic-argc 1  --max-arglen 8 \\\n" +
    "      -o test1.btor2",
    { fontSize: 13 });

  // Step 3
  s.addShape(pres.shapes.OVAL, {
    x: 0.7, y: 4.75, w: 0.5, h: 0.5,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  s.addText("3", {
    x: 0.7, y: 4.75, w: 0.5, h: 0.5,
    fontSize: 20, fontFace: H_FONT, color: NAVY, bold: true, align: "center", valign: "middle", margin: 0
  });
  s.addText("Verify with btormc (bounded model check)", {
    x: 1.3, y: 4.75, w: 11.3, h: 0.5,
    fontSize: 17, fontFace: H_FONT, color: NAVY, bold: true, valign: "middle", margin: 0
  });
  addCodeBlock(s, 1.3, 5.3, 11.3, 1.5,
    "$ docker run --rm -v \"$PWD:/work\" btormc \\\n" +
    "      btormc -kmax 200 /work/test1.btor2 > test1.wit\n\n" +
    "$ head -3 test1.wit\n" +
    "sat\nb0",
    { fontSize: 13 });

  addFooter(s, 8, TOTAL);
}

// =================================================================
// Slide 9 — Reading the witness
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: LIGHT };
  addTitleBar(s, "Reading the Witness",
    "btormc's output tells you exactly which argv bytes make the bad state reachable.");

  // Left: witness sample
  addCodeBlock(s, 0.7, 1.5, 6.6, 5.3,
    "sat\n" +
    "b0\n" +
    "#0\n" +
    "0  00000000  initial-stack-base#...\n" +
    "1  01000011  argv[1][0]      0x43   'C'\n" +
    "2  00000000  argv[1][1]      0x00\n" +
    "3  00000000  argv[1][2]      0x00\n" +
    "...\n" +
    "@0\n" +
    "0  <PC, regs, mem ...>\n" +
    ".",
    { fontSize: 14 });

  // Right: legend
  s.addText("How to read it", {
    x: 7.6, y: 1.5, w: 5.1, h: 0.45,
    fontSize: 18, fontFace: H_FONT, color: NAVY, bold: true, margin: 0
  });

  s.addText([
    { text: "sat", options: { bold: true, color: GREEN } },
    { text: "  —  a counterexample exists.", options: { color: DARK, breakLine: true } },
    { text: "b0", options: { bold: true, color: RED } },
    { text: "  —  which bad property was hit.", options: { color: DARK, breakLine: true } },
    { text: "#0", options: { bold: true, color: NAVY } },
    { text: "  —  initial state assignment (argv bytes here).", options: { color: DARK, breakLine: true } },
    { text: "@0, @1, …", options: { bold: true, color: NAVY } },
    { text: "  —  per-step input assignments.", options: { color: DARK, breakLine: true } },
    { text: ".", options: { bold: true, color: MUTED, italic: true } },
    { text: "  —  end of trace.", options: { color: DARK } }
  ], {
    x: 7.6, y: 2.0, w: 5.1, h: 3.0,
    fontSize: 14, fontFace: B_FONT, margin: 0, paraSpaceAfter: 4
  });

  // Tip box
  s.addShape(pres.shapes.RECTANGLE, {
    x: 7.6, y: 5.3, w: 5.1, h: 1.5,
    fill: { color: NAVY }, line: { color: NAVY }
  });
  s.addShape(pres.shapes.RECTANGLE, {
    x: 7.6, y: 5.3, w: 0.12, h: 1.5,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  s.addText([
    { text: "Visualize it.  ", options: { bold: true, color: GOLD } },
    { text: "Drop the .wit file into the BTOR2 web visualizer (Part 3) to step through the trace and see which register / memory byte flips at each step.", options: { color: WHITE } }
  ], {
    x: 7.85, y: 5.4, w: 4.8, h: 1.3,
    fontSize: 13, fontFace: B_FONT, valign: "middle", margin: 0
  });

  addFooter(s, 9, TOTAL);
}

// =================================================================
// Slide 10 — Recap
// =================================================================
{
  const s = pres.addSlide();
  s.background = { color: NAVY };

  s.addShape(pres.shapes.RECTANGLE, {
    x: 0.8, y: 0.8, w: 0.6, h: 0.08,
    fill: { color: GOLD }, line: { color: GOLD }
  });
  s.addText("Recap", {
    x: 0.8, y: 1.0, w: 11.5, h: 0.9,
    fontSize: 44, fontFace: H_FONT, color: WHITE, bold: true, margin: 0
  });

  // Four takeaways
  const items = [
    { h: "Symbolic = unconstrained", b: "Every argv byte becomes an unconstrained BTOR2 state; the solver chooses its value." },
    { h: "Rotor lays out the stack",  b: "argv[0], argv[1..N], pointers, argc, sp — all ABI-correct so the binary runs unmodified." },
    { h: "btormc finds the input",    b: "A sat witness is the exact argv that drives the program into a bad state." },
    { h: "Try it",                   b: "5 ready-made test programs in benchmarks/argv-tests/. Compile, generate BTOR2, run btormc." },
  ];
  const startY = 2.2;
  const cardH = 1.1;
  items.forEach((it, i) => {
    const y = startY + i * (cardH + 0.12);
    s.addShape(pres.shapes.RECTANGLE, {
      x: 0.8, y, w: 11.7, h: cardH,
      fill: { color: NAVY_D }, line: { color: NAVY_D }
    });
    s.addShape(pres.shapes.RECTANGLE, {
      x: 0.8, y, w: 0.12, h: cardH,
      fill: { color: GOLD }, line: { color: GOLD }
    });
    s.addText(it.h, {
      x: 1.1, y: y + 0.1, w: 11.2, h: 0.4,
      fontSize: 18, fontFace: H_FONT, color: GOLD, bold: true, valign: "middle", margin: 0
    });
    s.addText(it.b, {
      x: 1.1, y: y + 0.5, w: 11.2, h: 0.55,
      fontSize: 13, fontFace: B_FONT, color: ICE, valign: "top", margin: 0
    });
  });

  s.addText("Jasmin Begic  ·  Advanced Systems Engineering  ·  University of Salzburg", {
    x: 0.8, y: 7.05, w: 11.5, h: 0.35,
    fontSize: 11, fontFace: B_FONT, color: ICE, italic: true, margin: 0
  });
}

pres.writeFile({ fileName: "Symbolic_Arguments_Presentation.pptx" })
  .then(fn => console.log("Wrote " + fn));
