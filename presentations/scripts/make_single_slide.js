// Single dense slide: what was added to Rotor

const pptxgen = require("pptxgenjs");
const pres = new pptxgen();
pres.layout = "LAYOUT_WIDE"; // 13.333 x 7.5

const NAVY = "1E2761", INK = "1A1A1A", SUBTLE = "606060",
      RULE = "C8C8C8", BG = "F7F8FA", ACCENT = "F2C94C";

const s = pres.addSlide();
s.background = { color: "FFFFFF" };

// eyebrow
s.addText("CONTRIBUTION TO ROTOR", {
  x: 0.5, y: 0.35, w: 12.3, h: 0.3,
  fontSize: 11, fontFace: "Calibri", bold: true,
  color: SUBTLE, charSpacing: 3, margin: 0,
});

// title
s.addText("What was added to the codebase", {
  x: 0.5, y: 0.65, w: 12.3, h: 0.55,
  fontSize: 26, fontFace: "Georgia", bold: true, color: INK, margin: 0,
});

// one-liner band
s.addShape("rect", {
  x: 0.5, y: 1.3, w: 12.3, h: 0.65,
  fill: { color: NAVY }, line: { color: NAVY, width: 0 },
});
s.addShape("rect", {
  x: 0.5, y: 1.3, w: 0.15, h: 0.65,
  fill: { color: ACCENT }, line: { color: ACCENT, width: 0 },
});
s.addText(
  [
    { text: "One feature: ", options: { color: "CADCFC" } },
    { text: "symbolic console arguments", options: { bold: true, color: "FFFFFF" } },
    { text: "  ·  argv becomes an open question for the bounded model checker. Before: only stdin was symbolic.", options: { color: "CADCFC" } },
  ],
  {
    x: 0.85, y: 1.3, w: 11.9, h: 0.65,
    fontSize: 13, fontFace: "Calibri",
    align: "left", valign: "middle", margin: 0,
  }
);

// ===== left column: files + hooks =====
function section(x, y, w, h, title, body) {
  s.addShape("rect", {
    x, y, w, h,
    fill: { color: BG }, line: { color: RULE, width: 0.75 },
  });
  s.addText(title, {
    x: x + 0.2, y: y + 0.1, w: w - 0.4, h: 0.32,
    fontSize: 11, fontFace: "Calibri", bold: true,
    color: NAVY, charSpacing: 1, margin: 0,
  });
  s.addText(body, {
    x: x + 0.2, y: y + 0.45, w: w - 0.4, h: h - 0.55,
    fontSize: 10.5, fontFace: "Calibri", color: INK,
    valign: "top", margin: 0, paraSpaceAfter: 2,
  });
}

// FILES MODIFIED
section(0.5, 2.1, 4.05, 2.6, "FILES MODIFIED", [
  { text: "main.rs", options: { fontFace: "Consolas", bold: true } },
  { text: "  — 3 new CLI flags\n" },
  { text: "config.rs", options: { fontFace: "Consolas", bold: true } },
  { text: "  — 3 new Config fields\n" },
  { text: "machine/core.rs", options: { fontFace: "Consolas", bold: true } },
  { text: "  — 1 new function (~160 lines)\n             + 4 short hooks in CoreState::new\n" },
  { text: "benchmarks/argv-tests/", options: { fontFace: "Consolas", bold: true } },
  { text: "  — 5 new test programs" },
]);

// THE NEW FUNCTION
section(4.65, 2.1, 4.05, 2.6, "THE NEW FUNCTION", [
  { text: "initialize_symbolic_argv\n", options: { fontFace: "Consolas", bold: true, color: NAVY } },
  { text: "machine/core.rs : 272–431\n\n", options: { fontFace: "Consolas", color: SUBTLE, fontSize: 9 } },
  { text: "Lays out argv on the stack the way the OS would:\n" },
  { text: "  · ", options: { color: SUBTLE } }, { text: "argument count (concrete)\n" },
  { text: "  · ", options: { color: SUBTLE } }, { text: "pointer array (concrete)\n" },
  { text: "  · ", options: { color: SUBTLE } }, { text: "argv[0] = \"prog\" (concrete)\n" },
  { text: "  · ", options: { color: SUBTLE } }, { text: "argv[1..N] bytes — ", options: {} },
  { text: "free", options: { bold: true } }, { text: " for the solver\n" },
  { text: "  · ", options: { color: SUBTLE } }, { text: "null terminators (concrete)" },
]);

// CLI FLAGS
section(8.8, 2.1, 4.05, 2.6, "CLI FLAGS", [
  { text: "--symbolic-argv\n", options: { fontFace: "Consolas", bold: true } },
  { text: "turn the feature on\n\n", options: { color: SUBTLE } },
  { text: "--symbolic-argc N\n", options: { fontFace: "Consolas", bold: true } },
  { text: "how many symbolic arguments\n\n", options: { color: SUBTLE } },
  { text: "--max-arglen K\n", options: { fontFace: "Consolas", bold: true } },
  { text: "bytes free per argument", options: { color: SUBTLE } },
]);

// ===== middle row: hooks + validation + untouched =====

// HOOKS
section(0.5, 4.85, 4.05, 2.0, "4 HOOKS IN CoreState::new", [
  { text: "(a) ", options: { bold: true, color: NAVY } },
  { text: "pick initial stack pointer\n" },
  { text: "(b) ", options: { bold: true, color: NAVY } },
  { text: "write SP into the stack pointer register\n" },
  { text: "(c) ", options: { bold: true, color: NAVY } },
  { text: "write argument count into a0\n" },
  { text: "(d) ", options: { bold: true, color: NAVY } },
  { text: "attach stack value to stack segment" },
]);

// VALIDATION
section(4.65, 4.85, 4.05, 2.0, "VALIDATION — 5 BENCHMARKS", [
  { text: "Each program: a bug reachable ", options: {} },
  { text: "only via specific argv bytes.\n\n", options: { bold: true } },
  { text: "test4_multi_arg.c\n", options: { fontFace: "Consolas", color: NAVY } },
  { text: "needs argv[1][0] = 'X' AND argv[2][0] = 'Y'\n", options: { color: SUBTLE, fontSize: 9.5 } },
  { text: "btormc finds: ", options: {} },
  { text: "0x58, 0x59 ", options: { fontFace: "Consolas", bold: true } },
  { text: "(both within seconds).", options: { color: SUBTLE } },
]);

// UNTOUCHED
section(8.8, 4.85, 4.05, 2.0, "UNTOUCHED MODULES", [
  { text: "decoder, kernel, memory, registers,\n" },
  { text: "BTOR2 builder, BTOR2 printer,\n" },
  { text: "combinational & sequential logic,\n" },
  { text: "property checks\n\n" },
  { text: "Symbolic argv is a property of the\ninitial state — not a code path.", options: { italic: true, color: SUBTLE } },
]);

// ===== bottom: the mechanism, one line =====
s.addShape("rect", {
  x: 0.5, y: 7.0, w: 12.3, h: 0.4,
  fill: { color: "FFF8D6" }, line: { color: "E6CC55", width: 0.75 },
});
s.addText(
  [
    { text: "MECHANISM:  ", options: { bold: true, color: NAVY, charSpacing: 1 } },
    { text: "each free byte is one BTOR2 state node with no init — the bounded model checker picks any value 0..255, never changes.", options: { color: INK } },
  ],
  {
    x: 0.7, y: 7.0, w: 12.1, h: 0.4,
    fontSize: 11, fontFace: "Calibri",
    align: "left", valign: "middle", margin: 0,
  }
);

pres.writeFile({ fileName: "C:/Users/jasko/Programming/Rust/Project01/Rotor_Contribution_One_Slide.pptx" })
    .then(fn => console.log("Wrote " + fn));
