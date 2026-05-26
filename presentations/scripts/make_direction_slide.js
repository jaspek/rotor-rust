// One slide for the "light session" — frame as a problem we need
// the professor's input on. NOT a presentation slide.

const pptxgen = require("pptxgenjs");
const pres = new pptxgen();
pres.layout = "LAYOUT_WIDE"; // 13.333 x 7.5

const NAVY = "1E2761", INK = "1A1A1A", SUBTLE = "606060",
      RULE = "C8C8C8", BG = "F7F8FA",
      ICE = "CADCFC", ICE_DK = "9FB6E0",
      ACCENT = "F2C94C", ACCENT_DK = "C8A93B";

const s = pres.addSlide();
s.background = { color: "FFFFFF" };

// eyebrow
s.addText("FEEDBACK REQUEST", {
  x: 0.5, y: 0.3, w: 12.3, h: 0.3,
  fontSize: 11, fontFace: "Calibri", bold: true,
  color: SUBTLE, charSpacing: 3, margin: 0,
});

// title — the question itself, big
s.addText("Symbolic arguments work. Where should we take symbolic execution next?", {
  x: 0.5, y: 0.6, w: 12.3, h: 1.0,
  fontSize: 24, fontFace: "Georgia", bold: true, color: INK, margin: 0,
});

// subtitle
s.addText(
  "We have time for one more extension. Which direction is most useful for your SMT-solver work?",
  {
    x: 0.5, y: 1.65, w: 12.3, h: 0.4,
    fontSize: 13, fontFace: "Calibri", italic: true, color: SUBTLE, margin: 0,
  }
);

// ----- status band ---------------------------------------------------------
s.addShape("rect", {
  x: 0.5, y: 2.15, w: 12.3, h: 0.95,
  fill: { color: BG }, line: { color: RULE, width: 0.75 },
});
s.addText("WHERE WE ARE", {
  x: 0.7, y: 2.22, w: 12, h: 0.3,
  fontSize: 10, fontFace: "Calibri", bold: true, color: NAVY,
  charSpacing: 2, margin: 0,
});

const statusItems = [
  { icon: "✓", t: "Symbolic argv complete", d: "5 benchmarks, each bug found", c: "1F7A45" },
  { icon: "✓", t: "Symbolic stdin (already in rotor)", d: "bytes-to-read flag", c: "1F7A45" },
  { icon: "✓", t: "Heap free until written", d: "built-in", c: "1F7A45" },
];
statusItems.forEach((it, i) => {
  const x = 0.7 + i * 4.1;
  s.addText(it.icon, {
    x, y: 2.5, w: 0.35, h: 0.55,
    fontSize: 18, fontFace: "Calibri", bold: true, color: it.c,
    align: "left", valign: "middle", margin: 0,
  });
  s.addText(it.t, {
    x: x + 0.35, y: 2.5, w: 3.6, h: 0.3,
    fontSize: 11.5, fontFace: "Calibri", bold: true, color: INK,
    align: "left", valign: "middle", margin: 0,
  });
  s.addText(it.d, {
    x: x + 0.35, y: 2.78, w: 3.6, h: 0.27,
    fontSize: 9.5, fontFace: "Calibri", color: SUBTLE,
    align: "left", valign: "middle", margin: 0,
  });
});

// ----- direction cards (the options) --------------------------------------
function dirCard(x, y, w, h, num, title, body) {
  s.addShape("rect", {
    x, y, w, h,
    fill: { color: "FFFFFF" }, line: { color: NAVY, width: 1 },
  });
  // number circle
  s.addShape("ellipse", {
    x: x + 0.2, y: y + 0.2, w: 0.55, h: 0.55,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });
  s.addText(num, {
    x: x + 0.2, y: y + 0.2, w: 0.55, h: 0.55,
    fontSize: 16, fontFace: "Georgia", bold: true, color: ACCENT,
    align: "center", valign: "middle", margin: 0,
  });
  s.addText(title, {
    x: x + 0.85, y: y + 0.22, w: w - 1.0, h: 0.55,
    fontSize: 12.5, fontFace: "Calibri", bold: true, color: NAVY,
    align: "left", valign: "middle", margin: 0,
  });
  s.addText(body, {
    x: x + 0.2, y: y + 0.85, w: w - 0.35, h: h - 0.95,
    fontSize: 10.5, fontFace: "Calibri", color: INK,
    valign: "top", margin: 0, paraSpaceAfter: 2,
  });
}

const CARD_Y = 3.3, CARD_H = 3.1, CARD_W = 3.95, CARD_GAP = 0.15;

dirCard(0.5, CARD_Y, CARD_W, CARD_H, "1",
  "Combined argv + stdin",
  [
    { text: "Let programs use ", options: {} },
    { text: "both symbolic argv and symbolic stdin", options: { bold: true } },
    { text: " in the same model.\n\n", options: {} },
    { text: "Use case: ", options: { color: SUBTLE } },
    { text: "config string from argv,\ndata stream from stdin.\n\n", options: {} },
    { text: "Lift: ", options: { color: SUBTLE } },
    { text: "small (both pieces exist).", options: {} },
  ]
);

dirCard(0.5 + (CARD_W + CARD_GAP), CARD_Y, CARD_W, CARD_H, "2",
  "Symbolic env vars",
  [
    { text: "Let ", options: {} },
    { text: "getenv() return free bytes", options: { bold: true } },
    { text: ".\n\n", options: {} },
    { text: "Use case: ", options: { color: SUBTLE } },
    { text: "programs configured by environment (PATH, LANG, custom keys).\n\n", options: {} },
    { text: "Lift: ", options: { color: SUBTLE } },
    { text: "medium — same idea as argv but a layer up in the stack.", options: {} },
  ]
);

dirCard(0.5 + 2 * (CARD_W + CARD_GAP), CARD_Y, CARD_W, CARD_H, "3",
  "Symbolic file contents",
  [
    { text: "Let ", options: {} },
    { text: "read() from a file return free bytes", options: { bold: true } },
    { text: ".\n\n", options: {} },
    { text: "Use case: ", options: { color: SUBTLE } },
    { text: "programs that parse a file and crash on malformed input.\n\n", options: {} },
    { text: "Lift: ", options: { color: SUBTLE } },
    { text: "bigger — need to model fds and the open syscall.", options: {} },
  ]
);

// ----- the ask at the bottom ----------------------------------------------
s.addShape("rect", {
  x: 0.5, y: 6.55, w: 12.3, h: 0.7,
  fill: { color: "FFF8D6" }, line: { color: ACCENT_DK, width: 1 },
});
s.addShape("rect", {
  x: 0.5, y: 6.55, w: 0.15, h: 0.7,
  fill: { color: NAVY }, line: { color: NAVY, width: 0 },
});
s.addText(
  [
    { text: "ASK:  ", options: { bold: true, color: NAVY, charSpacing: 2 } },
    { text: "Which direction is most useful for your SMT-solver work — or is there a fourth we're missing? ", options: { color: INK } },
    { text: "(We can do one before the final deliverable.)", options: { color: SUBTLE, italic: true } },
  ],
  {
    x: 0.8, y: 6.55, w: 12, h: 0.7,
    fontSize: 13, fontFace: "Calibri",
    align: "left", valign: "middle", margin: 0,
  }
);

pres.writeFile({ fileName: "C:/Users/jasko/Programming/Rust/Project01/Symbolic_Direction_Feedback.pptx" })
    .then(fn => console.log("Wrote " + fn));
