// Single dense slide v2 — everything I'd put on it if I were defending
// the work in front of the prof tomorrow morning.

const pptxgen = require("pptxgenjs");
const pres = new pptxgen();
pres.layout = "LAYOUT_WIDE"; // 13.333 x 7.5

const NAVY = "1E2761", NAVY_DK = "0F1633", INK = "1A1A1A",
      SUBTLE = "606060", RULE = "C8C8C8", BG = "F7F8FA",
      ICE = "CADCFC", ICE_DK = "9FB6E0",
      ACCENT = "F2C94C", ACCENT_DK = "C8A93B",
      GREEN = "EAF5EE", GREEN_DK = "9DD1B2", GREEN_TX = "1F7A45",
      RED   = "F8E5E5", RED_DK   = "D89494", RED_TX   = "8B2A2A";

const s = pres.addSlide();
s.background = { color: "FFFFFF" };

// ---------- header ---------------------------------------------------------
s.addText("CONTRIBUTION TO ROTOR  ·  EVERYTHING IN ONE SLIDE", {
  x: 0.4, y: 0.25, w: 12.5, h: 0.3,
  fontSize: 10, fontFace: "Calibri", bold: true,
  color: SUBTLE, charSpacing: 3, margin: 0,
});

s.addText("Symbolic console arguments", {
  x: 0.4, y: 0.5, w: 12.5, h: 0.5,
  fontSize: 22, fontFace: "Georgia", bold: true, color: INK, margin: 0,
});

// ---------- problem / fix band --------------------------------------------
function band(x, y, w, h, label, labelColor, text) {
  s.addShape("rect", {
    x, y, w, h,
    fill: { color: "F7F8FA" }, line: { color: RULE, width: 0.75 },
  });
  s.addShape("rect", {
    x, y, w: 1.2, h,
    fill: { color: labelColor }, line: { color: labelColor, width: 0 },
  });
  s.addText(label, {
    x, y, w: 1.2, h,
    fontSize: 10, fontFace: "Calibri", bold: true,
    color: "FFFFFF", align: "center", valign: "middle",
    charSpacing: 2, margin: 0,
  });
  s.addText(text, {
    x: x + 1.35, y, w: w - 1.5, h,
    fontSize: 11.5, fontFace: "Calibri", color: INK,
    align: "left", valign: "middle", margin: 0,
  });
}

band(0.4, 1.1, 6.25, 0.6, "BEFORE", "8B2A2A",
  [
    { text: "argv = zero, argc = 0. ", options: { bold: true } },
    { text: "Branches on command-line arguments were ", options: {} },
    { text: "unreachable", options: { italic: true } },
    { text: " in the witness trace." },
  ]);

band(6.75, 1.1, 6.15, 0.6, "AFTER", "1F7A45",
  [
    { text: "argv bytes are free for the solver. ", options: { bold: true } },
    { text: "btormc picks values that drive the program into a bad state." },
  ]);

// ---------- section builder ------------------------------------------------
function section(x, y, w, h, title, body) {
  s.addShape("rect", {
    x, y, w, h,
    fill: { color: "FFFFFF" }, line: { color: RULE, width: 0.75 },
  });
  s.addShape("rect", {
    x, y, w, h: 0.32,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });
  s.addText(title, {
    x: x + 0.15, y, w: w - 0.3, h: 0.32,
    fontSize: 10, fontFace: "Calibri", bold: true,
    color: "FFFFFF", charSpacing: 1.5, align: "left", valign: "middle", margin: 0,
  });
  s.addText(body, {
    x: x + 0.15, y: y + 0.4, w: w - 0.3, h: h - 0.45,
    fontSize: 9.5, fontFace: "Calibri", color: INK,
    valign: "top", margin: 0, paraSpaceAfter: 1,
  });
}

// ---------- mid row (4 columns) -------------------------------------------
const MID_Y = 1.85, MID_H = 2.85;

// FILES MODIFIED
section(0.4, MID_Y, 3.05, MID_H, "FILES MODIFIED", [
  { text: "main.rs", options: { fontFace: "Consolas", bold: true } },
  { text: "  · 3 new CLI flags\n" },
  { text: "config.rs", options: { fontFace: "Consolas", bold: true } },
  { text: "  · 3 new Config fields\n" },
  { text: "machine/core.rs", options: { fontFace: "Consolas", bold: true } },
  { text: "  · 1 new function (~160 lines)\n  · 4 hooks in CoreState::new\n" },
  { text: "benchmarks/argv-tests/", options: { fontFace: "Consolas", bold: true } },
  { text: "  · 5 new test programs\n\n" },
  { text: "Everything else: untouched.", options: { italic: true, color: SUBTLE } },
]);

// THE NEW FUNCTION
section(3.55, MID_Y, 3.05, MID_H, "NEW FUNCTION  —  6 PHASES", [
  { text: "initialize_symbolic_argv\n", options: { fontFace: "Consolas", bold: true, color: NAVY } },
  { text: "machine/core.rs : 272–431\n\n", options: { fontFace: "Consolas", color: SUBTLE, fontSize: 8.5 } },
  { text: "1. ", options: { bold: true, color: NAVY } }, { text: "compute the layout (math only)\n" },
  { text: "2. ", options: { bold: true, color: NAVY } }, { text: "create an empty stack\n" },
  { text: "3. ", options: { bold: true, color: NAVY } }, { text: "write \"prog\\0\"  (concrete)\n" },
  { text: "4. ", options: { bold: true, color: NAVY } },
  { text: "write argv[1..N] bytes  ", options: {} },
  { text: "(FREE)\n", options: { bold: true, color: ACCENT_DK } },
  { text: "5. ", options: { bold: true, color: NAVY } }, { text: "write pointer array (concrete)\n" },
  { text: "6. ", options: { bold: true, color: NAVY } }, { text: "write argument count at SP\n" },
]);

// CLI FLAGS  +  HOOKS  (combined)
section(6.7, MID_Y, 3.05, MID_H, "USER SURFACE & WIRING", [
  { text: "CLI flags\n", options: { bold: true, color: NAVY, charSpacing: 1 } },
  { text: "--symbolic-argv\n", options: { fontFace: "Consolas", bold: true } },
  { text: "--symbolic-argc N\n", options: { fontFace: "Consolas", bold: true } },
  { text: "--max-arglen K\n\n", options: { fontFace: "Consolas", bold: true } },
  { text: "4 hooks in CoreState::new\n", options: { bold: true, color: NAVY, charSpacing: 1 } },
  { text: "(a) ", options: { bold: true, color: NAVY } }, { text: "pick initial stack pointer\n" },
  { text: "(b) ", options: { bold: true, color: NAVY } }, { text: "write SP into sp register\n" },
  { text: "(c) ", options: { bold: true, color: NAVY } }, { text: "write argument count into a0\n" },
  { text: "(d) ", options: { bold: true, color: NAVY } }, { text: "attach stack value to segment" },
]);

// STACK LAYOUT (right column)
section(9.85, MID_Y, 3.05, MID_H, "STACK LAYOUT BUILT BY PHASES 1-6", []);

// Manually draw the layout boxes
const lx = 10.05, lw = 2.65;
const stackBoxes = [
  { l: "argv[N] bytes",        c: ACCENT, h: 0.30 },
  { l: "...",                  c: ACCENT, h: 0.18 },
  { l: "argv[1] bytes",        c: ACCENT, h: 0.30 },
  { l: "\"prog\\0\"",          c: ICE,    h: 0.26 },
  { l: "padding",              c: ICE,    h: 0.18 },
  { l: "pointer array + NULL", c: ICE,    h: 0.30 },
  { l: "argument count",       c: ICE,    h: 0.26 },
];
let ly = MID_Y + 0.42;
stackBoxes.forEach(b => {
  s.addShape("rect", {
    x: lx, y: ly, w: lw, h: b.h,
    fill: { color: b.c },
    line: { color: b.c === ACCENT ? ACCENT_DK : ICE_DK, width: 0.5 },
  });
  s.addText(b.l, {
    x: lx + 0.1, y: ly, w: lw - 0.15, h: b.h,
    fontSize: 9, fontFace: "Calibri",
    bold: b.c === ACCENT,
    color: NAVY, valign: "middle", align: "left", margin: 0,
  });
  ly += b.h + 0.02;
});
// SP arrow
s.addText("← SP", {
  x: lx + lw + 0.02, y: ly - 0.28, w: 0.5, h: 0.28,
  fontSize: 9, fontFace: "Consolas", bold: true, color: NAVY,
  valign: "middle", margin: 0,
});
// legend
s.addShape("rect", {
  x: lx, y: ly + 0.05, w: 0.18, h: 0.18,
  fill: { color: ACCENT }, line: { color: ACCENT_DK, width: 0.5 },
});
s.addText("free for solver", {
  x: lx + 0.25, y: ly + 0.05, w: 1.6, h: 0.18,
  fontSize: 8, fontFace: "Calibri", color: SUBTLE, valign: "middle", margin: 0,
});
s.addShape("rect", {
  x: lx + 1.55, y: ly + 0.05, w: 0.18, h: 0.18,
  fill: { color: ICE }, line: { color: ICE_DK, width: 0.5 },
});
s.addText("concrete", {
  x: lx + 1.78, y: ly + 0.05, w: 1.0, h: 0.18,
  fontSize: 8, fontFace: "Calibri", color: SUBTLE, valign: "middle", margin: 0,
});

// ---------- bottom row (3 columns) ----------------------------------------
const BOT_Y = 4.85, BOT_H = 1.95;

// PIPELINE
section(0.4, BOT_Y, 4.25, BOT_H, "PIPELINE — END TO END", []);
const stages = [
  { l: "C source",       c: ICE,    fg: NAVY  },
  { l: "RISC-V binary",  c: ICE,    fg: NAVY  },
  { l: "BTOR2 model",    c: NAVY,   fg: "FFF" },
  { l: "Witness trace",  c: NAVY,   fg: "FFF" },
  { l: "argv bytes",     c: ACCENT, fg: NAVY  },
];
const SX = 0.55, SY = BOT_Y + 0.55, SH = 0.55, SW = 0.72, SGAP = 0.075;
stages.forEach((st, i) => {
  const x = SX + i * (SW + SGAP);
  s.addShape("rect", {
    x, y: SY, w: SW, h: SH,
    fill: { color: st.c },
    line: { color: st.c === ACCENT ? ACCENT_DK : st.c === ICE ? ICE_DK : NAVY, width: 0.5 },
  });
  s.addText(st.l, {
    x, y: SY, w: SW, h: SH,
    fontSize: 8, fontFace: "Calibri", bold: true,
    color: st.fg === "FFF" ? "FFFFFF" : NAVY,
    align: "center", valign: "middle", margin: 0,
  });
  if (i < stages.length - 1) {
    s.addShape("line", {
      x: x + SW + 0.005, y: SY + SH / 2, w: SGAP - 0.01, h: 0,
      line: { color: NAVY, width: 1, endArrowType: "triangle" },
    });
  }
});
s.addText("selfie compiles · rotor builds the model · btormc solves · we read the bytes", {
  x: 0.55, y: SY + 0.7, w: 4.0, h: 0.45,
  fontSize: 9, fontFace: "Calibri", italic: true, color: SUBTLE,
  valign: "middle", margin: 0,
});
s.addText("Only rotor changed. selfie and btormc are unchanged third-party tools.", {
  x: 0.55, y: SY + 1.05, w: 4.0, h: 0.4,
  fontSize: 9, fontFace: "Calibri", color: SUBTLE,
  valign: "middle", margin: 0,
});

// VALIDATION
section(4.75, BOT_Y, 4.05, BOT_H, "VALIDATION", [
  { text: "5 benchmarks. Each bug reachable ", options: {} },
  { text: "only via specific argv bytes.\n\n", options: { bold: true } },
  { text: "test4_multi_arg.c\n", options: { fontFace: "Consolas", color: NAVY, bold: true } },
  { text: "    argv[1][0] == 'X'  &&  argv[2][0] == 'Y'\n", options: { fontFace: "Consolas", fontSize: 8.5 } },
  { text: "    ", options: {} },
  { text: "→ btormc returns 0x58, 0x59", options: { fontFace: "Consolas", color: GREEN_TX, bold: true } },
  { text: "\n\nAll 5 found within seconds.", options: { italic: true, color: SUBTLE } },
]);

// DESIGN CHOICE
section(8.9, BOT_Y, 4.0, BOT_H, "DESIGN CHOICE  —  state, not input", [
  { text: "Why not BTOR2 ", options: {} },
  { text: "input", options: { fontFace: "Consolas", bold: true } },
  { text: " nodes?\n", options: {} },
  { text: " · ", options: { color: SUBTLE } }, { text: "Forbidden inside init expressions\n" },
  { text: " · ", options: { color: SUBTLE } }, { text: "Re-chosen every step — argv is fixed\n\n" },
  { text: "Uninitialized ", options: {} },
  { text: "state", options: { fontFace: "Consolas", bold: true } },
  { text: " nodes ", options: {} },
  { text: "→ ", options: { color: NAVY } },
  { text: "picked once at step 0, persist forever, legal in init.", options: {} },
]);

// ---------- bottom mechanism band -----------------------------------------
s.addShape("rect", {
  x: 0.4, y: 6.95, w: 12.5, h: 0.45,
  fill: { color: "FFF8D6" }, line: { color: "E6CC55", width: 0.75 },
});
s.addText(
  [
    { text: "★ MECHANISM:  ", options: { bold: true, color: NAVY, charSpacing: 1 } },
    { text: "each free argv byte is a BTOR2 ", options: { color: INK } },
    { text: "state", options: { bold: true, fontFace: "Consolas", color: INK } },
    { text: " node with no init — solver picks any value 0..255, never changes. ", options: { color: INK } },
    { text: " The program reads them through ordinary byte loads; nothing else in Rotor knows anything is symbolic.", options: { color: SUBTLE, italic: true } },
  ],
  {
    x: 0.6, y: 6.95, w: 12.2, h: 0.45,
    fontSize: 10.5, fontFace: "Calibri",
    align: "left", valign: "middle", margin: 0,
  }
);

pres.writeFile({ fileName: "C:/Users/jasko/Programming/Rust/Project01/Rotor_Contribution_One_Slide_v2.pptx" })
    .then(fn => console.log("Wrote " + fn));
