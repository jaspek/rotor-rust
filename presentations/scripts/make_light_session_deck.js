// Single-slide deck for the online light session.
// All abbreviations expanded; jargon stripped where possible.

const pptxgen = require("pptxgenjs");
const pres = new pptxgen();
pres.layout = "LAYOUT_WIDE";

const NAVY = "1E2761", INK = "1A1A1A", SUBTLE = "606060",
      RULE = "C8C8C8", BG = "F7F8FA",
      ICE = "CADCFC", ICE_DK = "9FB6E0",
      ACCENT = "F2C94C", ACCENT_DK = "C8A93B",
      GREEN_TX = "1F7A45", AMBER_TX = "8B6914",
      GREEN_BG = "EAF5EE", AMBER_BG = "FFF8D6";

const s = pres.addSlide();
s.background = { color: "FFFFFF" };

// eyebrow
s.addText("LIGHT SESSION  ·  FEEDBACK REQUEST", {
  x: 0.6, y: 0.35, w: 12.2, h: 0.3,
  fontSize: 11, fontFace: "Calibri", bold: true,
  color: SUBTLE, charSpacing: 3, margin: 0,
});

// title
s.addText("What we added to your Rotor project", {
  x: 0.6, y: 0.65, w: 12.2, h: 0.85,
  fontSize: 26, fontFace: "Georgia", bold: true, color: INK, margin: 0,
});

// subtitle
s.addText("Three pieces. Mapping each one to what we understood from the first meeting.", {
  x: 0.6, y: 1.55, w: 12.2, h: 0.4,
  fontSize: 13, fontFace: "Calibri", italic: true, color: SUBTLE, margin: 0,
});

// --- three cards ----------------------------------------------------------
const items = [
  {
    num: "1",
    name: "The Rust rewrite",
    asked:
      "Rewrite Rotor in Rust. Make the code easier to read, easier to " +
      "extend, easier to use with modern tools.",
    delivered:
      "Done. The new code is split into separate files, each handling one " +
      "job. Produces the same models as the original on every test program " +
      "we have — and the models come out about 3.5× smaller, because the " +
      "new builder spots when the same expression appears more than once " +
      "and writes it only once.",
    status: "DONE", statusBg: GREEN_BG, statusFg: GREEN_TX,
  },
  {
    num: "2",
    name: "Symbolic command-line arguments",
    asked:
      "Let the bounded model checker reason about command-line input. " +
      "Right now it would have been blind to anything coming from the " +
      "command line.",
    delivered:
      "Done. When the program reads its command-line arguments, those " +
      "bytes in memory are unknown — the bounded model checker picks " +
      "them. The program reads them through a normal load. We wrote five " +
      "small test programs that each have a bug only reachable through a " +
      "specific input; the model checker finds the right characters " +
      "within seconds on every one.",
    status: "DONE", statusBg: GREEN_BG, statusFg: GREEN_TX,
  },
  {
    num: "3",
    name: "The witness viewer",
    asked:
      "A way to look at the bounded model checker's answer — the " +
      "witness — step by step. The witness shows the input that breaks " +
      "the program and how the program reaches the bad state.",
    delivered:
      "Mostly done. Browser tool that opens a model and a witness. Can " +
      "show only a slice of the model (instead of thousands of nodes at " +
      "once), highlights only the parts that mattered for the bug, walks " +
      "the trace one step at a time. Still being finished: showing " +
      "memory-array values nicely at each step.",
    status: "IN PROGRESS", statusBg: AMBER_BG, statusFg: AMBER_TX,
  },
];

items.forEach((it, i) => {
  const x = 0.6 + i * 4.13, w = 3.95, y = 2.15, h = 4.0;
  s.addShape("rect", { x, y, w, h,
    fill: { color: "FFFFFF" }, line: { color: NAVY, width: 1 } });
  s.addShape("rect", { x, y, w, h: 0.5,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 } });
  s.addShape("ellipse", { x: x + 0.2, y: y + 0.07, w: 0.36, h: 0.36,
    fill: { color: ACCENT }, line: { color: ACCENT, width: 0 } });
  s.addText(it.num, { x: x + 0.2, y: y + 0.07, w: 0.36, h: 0.36,
    fontSize: 12, fontFace: "Georgia", bold: true, color: NAVY,
    align: "center", valign: "middle", margin: 0 });
  s.addText(it.name, { x: x + 0.62, y, w: w - 0.7, h: 0.5,
    fontSize: 12.5, fontFace: "Calibri", bold: true, color: "FFFFFF",
    align: "left", valign: "middle", charSpacing: 1, margin: 0 });

  s.addText("WHAT WE WANTED", { x: x + 0.25, y: y + 0.6, w: w - 0.5, h: 0.25,
    fontSize: 9, fontFace: "Calibri", bold: true, color: NAVY,
    charSpacing: 1.5, margin: 0 });
  s.addText(it.asked, { x: x + 0.25, y: y + 0.87, w: w - 0.5, h: 1.2,
    fontSize: 10.5, fontFace: "Calibri", color: INK, italic: true,
    valign: "top", margin: 0 });

  s.addText("WHAT WE BUILT", { x: x + 0.25, y: y + 2.12, w: w - 0.5, h: 0.25,
    fontSize: 9, fontFace: "Calibri", bold: true, color: NAVY,
    charSpacing: 1.5, margin: 0 });
  s.addText(it.delivered, { x: x + 0.25, y: y + 2.39, w: w - 0.5, h: 1.15,
    fontSize: 10.5, fontFace: "Calibri", color: INK, valign: "top", margin: 0 });

  s.addShape("rect", { x: x + 0.25, y: y + h - 0.5, w: w - 0.5, h: 0.32,
    fill: { color: it.statusBg }, line: { color: it.statusBg, width: 0 } });
  s.addText(it.status, { x: x + 0.25, y: y + h - 0.5, w: w - 0.5, h: 0.32,
    fontSize: 10.5, fontFace: "Calibri", bold: true, color: it.statusFg,
    align: "center", valign: "middle", charSpacing: 2, margin: 0 });
});

// --- ask band -------------------------------------------------------------
s.addShape("rect", { x: 0.6, y: 6.4, w: 12.15, h: 0.6,
  fill: { color: AMBER_BG }, line: { color: ACCENT_DK, width: 1 } });
s.addShape("rect", { x: 0.6, y: 6.4, w: 0.15, h: 0.6,
  fill: { color: NAVY }, line: { color: NAVY, width: 0 } });
s.addText(
  [
    { text: "★ ASK:  ", options: { bold: true, color: NAVY, charSpacing: 2 } },
    { text: "Does this match what you had in mind? ", options: { color: INK } },
    { text: "For the witness viewer — what should we prioritise to finish well?",
      options: { italic: true, color: SUBTLE } },
  ],
  { x: 0.85, y: 6.4, w: 11.8, h: 0.6,
    fontSize: 13, fontFace: "Calibri",
    align: "left", valign: "middle", margin: 0 }
);

s.addText("Begic & Wassie  ·  Rotor in Rust + Symbolic command-line arguments", {
  x: 0.6, y: 7.15, w: 12.15, h: 0.25,
  fontSize: 9, fontFace: "Calibri", color: SUBTLE, margin: 0,
});

pres.writeFile({ fileName: "C:/Users/jasko/Programming/Rust/Project01/Light_Session_Deck_v5.pptx" })
    .then(fn => console.log("Wrote " + fn));
