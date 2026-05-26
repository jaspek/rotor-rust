// Symbolic Console Arguments — presentation deck
// Palette: Midnight Executive (navy / ice blue / white)
// Typography: Georgia (headers), Calibri (body), Consolas (code)

const pptxgen = require("pptxgenjs");
const pres = new pptxgen();

pres.layout = "LAYOUT_WIDE";          // 13.333 x 7.5
const W = 13.333, H = 7.5;

// ---- palette --------------------------------------------------------------
const NAVY     = "1E2761";
const NAVY_DK  = "0F1633";
const ICE      = "CADCFC";
const ICE_DK   = "9FB6E0";
const WHITE    = "FFFFFF";
const SUBTLE   = "5A6470";
const ACCENT   = "F2C94C";  // single yellow accent for emphasis
const CODE_BG  = "F3F4F6";
const RULE     = "D6DDE6";

// ---- helpers --------------------------------------------------------------
function bgWhite(slide) {
  slide.background = { color: WHITE };
}
function bgNavy(slide) {
  slide.background = { color: NAVY };
}

// A small number badge ("01", "02", ...) for section slides
function numberBadge(slide, num, x, y, color = ACCENT, textColor = NAVY) {
  slide.addShape("ellipse", {
    x, y, w: 0.6, h: 0.6, fill: { color }, line: { color, width: 0 },
  });
  slide.addText(num, {
    x, y, w: 0.6, h: 0.6,
    fontSize: 14, fontFace: "Georgia", bold: true,
    color: textColor, align: "center", valign: "middle",
    margin: 0,
  });
}

// Title-slide / section-divider headers
function title(slide, text, opts = {}) {
  slide.addText(text, Object.assign({
    x: 0.7, y: 0.55, w: 12, h: 0.9,
    fontSize: 30, fontFace: "Georgia", bold: true,
    color: NAVY, align: "left", valign: "top", margin: 0,
  }, opts));
}

// Subtitle/eyebrow under titles
function eyebrow(slide, text, color = SUBTLE) {
  slide.addText(text, {
    x: 0.7, y: 0.32, w: 12, h: 0.3,
    fontSize: 11, fontFace: "Calibri", bold: true,
    color, align: "left", valign: "top", margin: 0, charSpacing: 2,
  });
}

// Footer with deck info
function footer(slide, num, total) {
  slide.addText("Symbolic Console Arguments  ·  Rotor (Rust)", {
    x: 0.7, y: 7.05, w: 9, h: 0.3,
    fontSize: 9, fontFace: "Calibri", color: SUBTLE, align: "left", margin: 0,
  });
  slide.addText(`${num} / ${total}`, {
    x: 11.7, y: 7.05, w: 0.9, h: 0.3,
    fontSize: 9, fontFace: "Calibri", color: SUBTLE, align: "right", margin: 0,
  });
}

const TOTAL = 16;

// ===========================================================================
// SLIDE 1 — title
// ===========================================================================
{
  const s = pres.addSlide();
  bgNavy(s);

  // accent block (left vertical band)
  s.addShape("rect", {
    x: 0, y: 0, w: 0.35, h: H, fill: { color: ACCENT }, line: { color: ACCENT, width: 0 },
  });

  // small eyebrow
  s.addText("ADVANCED SYSTEMS ENGINEERING  ·  UNIVERSITY OF SALZBURG", {
    x: 0.9, y: 1.4, w: 12, h: 0.4,
    fontSize: 11, fontFace: "Calibri", bold: true, color: ICE,
    align: "left", margin: 0, charSpacing: 3,
  });

  s.addText("Symbolic Console Arguments", {
    x: 0.9, y: 1.95, w: 12, h: 1.4,
    fontSize: 50, fontFace: "Georgia", bold: true, color: WHITE,
    align: "left", margin: 0,
  });

  s.addText("Adding argv to the BTOR2 model", {
    x: 0.9, y: 3.35, w: 12, h: 0.7,
    fontSize: 24, fontFace: "Georgia", italic: true, color: ICE,
    align: "left", margin: 0,
  });

  // divider line
  s.addShape("line", {
    x: 0.9, y: 4.4, w: 1.6, h: 0,
    line: { color: ACCENT, width: 2 },
  });

  s.addText(
    [
      { text: "Jasmin Begic & Daniel Wassie", options: { bold: true } },
      { text: "\nSupervised by Prof. Christoph Kirsch" },
    ],
    {
      x: 0.9, y: 4.6, w: 12, h: 1.0,
      fontSize: 14, fontFace: "Calibri", color: WHITE,
      align: "left", margin: 0,
    }
  );

  s.addText("Project contribution to Rotor", {
    x: 0.9, y: 6.6, w: 12, h: 0.4,
    fontSize: 12, fontFace: "Calibri", italic: true, color: ICE_DK,
    align: "left", margin: 0,
  });
}

// ===========================================================================
// SLIDE 2 — problem
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "PROBLEM");
  title(s, "What was missing in Rotor's BTOR2 model");

  // body text — left column
  s.addText(
    "Rotor is a front end: it translates a RISC-V binary into a BTOR2 model that a bounded model checker (btormc) consumes.",
    {
      x: 0.7, y: 1.7, w: 6.0, h: 0.9,
      fontSize: 14, fontFace: "Calibri", color: NAVY_DK, valign: "top", margin: 0,
    }
  );

  s.addText(
    [
      { text: "But the model only ever exposed ", options: {} },
      { text: "stdin", options: { bold: true } },
      { text: " to the solver.", options: {} },
    ],
    {
      x: 0.7, y: 2.7, w: 6.0, h: 0.5,
      fontSize: 14, fontFace: "Calibri", color: NAVY_DK, valign: "top", margin: 0,
    }
  );

  s.addText(
    [
      { text: "argv was always zero. argc was zero. ", options: {} },
      { text: "Any program behavior that depended on a command-line argument was unreachable in the witness trace.", options: { bold: true } },
    ],
    {
      x: 0.7, y: 3.3, w: 6.0, h: 1.6,
      fontSize: 14, fontFace: "Calibri", color: NAVY_DK, valign: "top", margin: 0,
    }
  );

  // right column — illustration card
  s.addShape("rect", {
    x: 7.4, y: 1.7, w: 5.2, h: 4.6,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });
  s.addShape("rect", {
    x: 7.4, y: 1.7, w: 5.2, h: 0.5,
    fill: { color: NAVY_DK }, line: { color: NAVY_DK, width: 0 },
  });
  s.addText("Solver's freedom — before", {
    x: 7.4, y: 1.7, w: 5.2, h: 0.5,
    fontSize: 11, fontFace: "Calibri", bold: true, color: ICE,
    align: "center", valign: "middle", charSpacing: 2, margin: 0,
  });

  // checklist style
  const items = [
    ["stdin bytes", true],
    ["heap (before first write)", true],
    ["argv[0..N]", false],
    ["argc", false],
  ];
  items.forEach(([label, on], i) => {
    const y = 2.45 + i * 0.85;
    s.addShape(on ? "ellipse" : "ellipse", {
      x: 7.7, y, w: 0.4, h: 0.4,
      fill: { color: on ? ACCENT : "3A4566" },
      line: { color: on ? ACCENT : "3A4566", width: 0 },
    });
    s.addText(on ? "✓" : "✗", {
      x: 7.7, y, w: 0.4, h: 0.4,
      fontSize: 14, fontFace: "Calibri", bold: true,
      color: on ? NAVY : ICE_DK, align: "center", valign: "middle", margin: 0,
    });
    s.addText(label, {
      x: 8.3, y: y - 0.05, w: 4.1, h: 0.5,
      fontSize: 14, fontFace: "Calibri", color: WHITE, valign: "middle", margin: 0,
    });
  });

  footer(s, 2, TOTAL);
}

// ===========================================================================
// SLIDE 3 — before / after, side by side
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "DELTA");
  title(s, "Before vs after, in one picture");

  function panel(x, headerText, headerBg, headerFg, lines) {
    const W_PANEL = 5.8;
    s.addShape("rect", {
      x, y: 1.7, w: W_PANEL, h: 4.6,
      fill: { color: WHITE }, line: { color: RULE, width: 1 },
    });
    s.addShape("rect", {
      x, y: 1.7, w: W_PANEL, h: 0.6,
      fill: { color: headerBg }, line: { color: headerBg, width: 0 },
    });
    s.addText(headerText, {
      x, y: 1.7, w: W_PANEL, h: 0.6,
      fontSize: 14, fontFace: "Georgia", bold: true,
      color: headerFg, align: "center", valign: "middle", margin: 0,
    });
    lines.forEach((line, i) => {
      s.addText(line, {
        x: x + 0.4, y: 2.55 + i * 0.55, w: W_PANEL - 0.8, h: 0.5,
        fontSize: 13, fontFace: "Calibri", color: NAVY_DK, valign: "top", margin: 0,
      });
    });
  }

  panel(0.7, "BEFORE", "E5E9F0", NAVY, [
    "stack initialised to all zeros",
    "argc = 0",
    "argv = NULL",
    "branches on argv: unreachable",
    "branches on stdin: reachable",
    "Witness trace: stdin bytes only",
  ]);

  panel(6.85, "AFTER", NAVY, WHITE, [
    "stack laid out as a real argv image",
    "argc = N + 1  (concrete)",
    "argv[0] = \"prog\\0\"   (concrete)",
    "argv[1..N] bytes: free for the solver",
    "branches on argv: now reachable",
    "Witness trace: stdin AND argv bytes",
  ]);

  footer(s, 3, TOTAL);
}

// ===========================================================================
// SLIDE 4 — the headline contribution (stat callout)
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "CONTRIBUTION");
  title(s, "One feature added to Rotor");

  // Big stat block
  s.addShape("rect", {
    x: 0.7, y: 1.9, w: 12, h: 4.0,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });

  // accent stripe
  s.addShape("rect", {
    x: 0.7, y: 1.9, w: 0.18, h: 4.0,
    fill: { color: ACCENT }, line: { color: ACCENT, width: 0 },
  });

  s.addText("symbolic console arguments", {
    x: 1.2, y: 2.2, w: 11.2, h: 0.6,
    fontSize: 14, fontFace: "Calibri", color: ICE, bold: true,
    align: "left", margin: 0, charSpacing: 3,
  });

  s.addText("argv becomes a question for the solver", {
    x: 1.2, y: 2.8, w: 11.2, h: 1.6,
    fontSize: 40, fontFace: "Georgia", bold: true, color: WHITE,
    align: "left", margin: 0,
  });

  s.addText(
    "Each character of argv[1..N] is left free; btormc returns a witness trace " +
    "with concrete byte values that drive the program into a bad state.",
    {
      x: 1.2, y: 4.6, w: 11.2, h: 1.0,
      fontSize: 16, fontFace: "Calibri", italic: true, color: ICE,
      align: "left", margin: 0,
    }
  );

  // tiny stats row
  const stats = [
    ["1", "new function"],
    ["3", "CLI flags"],
    ["5", "benchmark programs"],
    ["~160", "lines added"],
  ];
  stats.forEach(([n, lbl], i) => {
    const x = 0.7 + i * 3.1;
    s.addText(n, {
      x, y: 6.1, w: 3.0, h: 0.5,
      fontSize: 28, fontFace: "Georgia", bold: true, color: NAVY,
      align: "center", margin: 0,
    });
    s.addText(lbl, {
      x, y: 6.6, w: 3.0, h: 0.4,
      fontSize: 11, fontFace: "Calibri", color: SUBTLE,
      align: "center", margin: 0,
    });
  });

  footer(s, 4, TOTAL);
}

// ===========================================================================
// SLIDE 5 — CLI flags
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "USER-FACING SURFACE");
  title(s, "Three new flags on the rotor command line");

  const flags = [
    {
      name: "--symbolic-argv",
      sub: "switch",
      desc: "Turn the feature on. Without it, Rotor behaves exactly as before.",
    },
    {
      name: "--symbolic-argc N",
      sub: "default 1",
      desc: "Number of symbolic arguments. Total argc seen by the program is N + 1.",
    },
    {
      name: "--max-arglen K",
      sub: "default 8",
      desc: "Bytes of each argv[i] left free for the solver. Each string is K + 1 bytes incl. null.",
    },
  ];

  flags.forEach((f, i) => {
    const x = 0.7 + i * 4.2;
    // card
    s.addShape("rect", {
      x, y: 1.85, w: 3.95, h: 3.7,
      fill: { color: WHITE }, line: { color: NAVY, width: 1 },
    });
    // top stripe
    s.addShape("rect", {
      x, y: 1.85, w: 3.95, h: 0.4,
      fill: { color: NAVY }, line: { color: NAVY, width: 0 },
    });
    // number
    s.addText(`0${i + 1}`, {
      x: x + 0.1, y: 1.85, w: 0.8, h: 0.4,
      fontSize: 12, fontFace: "Georgia", bold: true, color: ACCENT,
      align: "left", valign: "middle", margin: 0,
    });
    s.addText(f.sub.toUpperCase(), {
      x: x + 0.5, y: 1.85, w: 3.4, h: 0.4,
      fontSize: 9, fontFace: "Calibri", bold: true, color: ICE,
      align: "right", valign: "middle", margin: 0, charSpacing: 2,
    });
    // flag name
    s.addText(f.name, {
      x: x + 0.25, y: 2.45, w: 3.7, h: 0.6,
      fontSize: 16, fontFace: "Consolas", bold: true, color: NAVY,
      align: "left", valign: "top", margin: 0,
    });
    // description
    s.addText(f.desc, {
      x: x + 0.25, y: 3.1, w: 3.55, h: 2.3,
      fontSize: 12, fontFace: "Calibri", color: NAVY_DK,
      align: "left", valign: "top", margin: 0,
    });
  });

  // example invocation
  s.addText("Example:", {
    x: 0.7, y: 5.85, w: 5, h: 0.35,
    fontSize: 11, fontFace: "Calibri", bold: true, color: NAVY,
    align: "left", margin: 0, charSpacing: 1,
  });
  s.addShape("rect", {
    x: 0.7, y: 6.2, w: 12.0, h: 0.65,
    fill: { color: CODE_BG }, line: { color: RULE, width: 0.75 },
  });
  s.addText(
    "rotor program.elf --symbolic-argv --symbolic-argc 2 --max-arglen 8 -o program.btor2",
    {
      x: 0.85, y: 6.2, w: 11.7, h: 0.65,
      fontSize: 13, fontFace: "Consolas", color: NAVY_DK,
      align: "left", valign: "middle", margin: 0,
    }
  );

  footer(s, 5, TOTAL);
}

// ===========================================================================
// SLIDE 6 — files changed
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "FOOTPRINT");
  title(s, "What changed in the codebase");

  const rows = [
    [{ text: "File", options: { bold: true, color: WHITE } },
     { text: "Change", options: { bold: true, color: WHITE } },
     { text: "What it adds", options: { bold: true, color: WHITE } }],
    [{ text: "main.rs", options: { fontFace: "Consolas" } },
     { text: "modified" },
     { text: "Three new CLI flags." }],
    [{ text: "config.rs", options: { fontFace: "Consolas" } },
     { text: "modified" },
     { text: "Three new Config fields." }],
    [{ text: "machine/core.rs", options: { fontFace: "Consolas" } },
     { text: "modified" },
     { text: "One new function (~160 lines) + four short hooks in CoreState::new." }],
    [{ text: "benchmarks/argv-tests/", options: { fontFace: "Consolas" } },
     { text: "added" },
     { text: "Five C programs, each containing a bug reachable only via specific argv bytes." }],
  ];

  s.addTable(rows, {
    x: 0.7, y: 1.85, w: 12.0,
    colW: [3.6, 2.0, 6.4],
    fontSize: 13, fontFace: "Calibri", color: NAVY_DK,
    border: { type: "solid", pt: 0.5, color: RULE },
    fill: { color: WHITE },
    rowH: 0.55,
    valign: "middle",
    margin: 0.08,
  });

  // header row override (pptxgenjs limitation: emulate via shape)
  s.addShape("rect", {
    x: 0.7, y: 1.85, w: 12.0, h: 0.55,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });
  s.addText("File", {
    x: 0.78, y: 1.85, w: 3.55, h: 0.55,
    fontSize: 12, fontFace: "Calibri", bold: true, color: WHITE,
    align: "left", valign: "middle", margin: 0, charSpacing: 1,
  });
  s.addText("Change", {
    x: 4.3, y: 1.85, w: 2.0, h: 0.55,
    fontSize: 12, fontFace: "Calibri", bold: true, color: WHITE,
    align: "left", valign: "middle", margin: 0, charSpacing: 1,
  });
  s.addText("What it adds", {
    x: 6.3, y: 1.85, w: 6.4, h: 0.55,
    fontSize: 12, fontFace: "Calibri", bold: true, color: WHITE,
    align: "left", valign: "middle", margin: 0, charSpacing: 1,
  });

  // takeaway box
  s.addShape("rect", {
    x: 0.7, y: 5.35, w: 12.0, h: 1.3,
    fill: { color: ICE }, line: { color: ICE_DK, width: 1 },
  });
  s.addShape("rect", {
    x: 0.7, y: 5.35, w: 0.18, h: 1.3,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });
  s.addText(
    [
      { text: "The whole feature lives in one function and a few hooks. ", options: { bold: true } },
      { text: "Five files exist (or change) for it. The rest of Rotor is untouched.", options: {} },
    ],
    {
      x: 1.0, y: 5.4, w: 11.6, h: 1.2,
      fontSize: 14, fontFace: "Calibri", color: NAVY_DK,
      align: "left", valign: "middle", margin: 0,
    }
  );

  footer(s, 6, TOTAL);
}

// ===========================================================================
// SLIDE 7 — what didn't change
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "FOOTPRINT (CONT.)");
  title(s, "What was NOT touched");

  s.addText(
    "Symbolic argv is a property of the initial state. " +
    "It is not a special path through the rest of Rotor. " +
    "These modules required no changes:",
    {
      x: 0.7, y: 1.7, w: 12, h: 0.85,
      fontSize: 14, fontFace: "Calibri", italic: true, color: SUBTLE,
      valign: "top", margin: 0,
    }
  );

  const untouched = [
    ["RISC-V decoder", "riscv/decode.rs, compressed.rs"],
    ["Memory loads/stores", "machine/memory.rs"],
    ["Register file", "machine/registers.rs"],
    ["Kernel & syscalls", "machine/kernel.rs"],
    ["Property checks", "model/properties.rs"],
    ["BTOR2 builder", "btor2/builder.rs"],
    ["BTOR2 printer", "btor2/printer.rs"],
    ["Combinational/sequential logic", "model/combinational.rs, sequential.rs"],
  ];

  // 2x4 grid
  untouched.forEach(([label, file], i) => {
    const col = i % 2, row = Math.floor(i / 2);
    const x = 0.7 + col * 6.15;
    const y = 2.85 + row * 0.95;
    // card
    s.addShape("rect", {
      x, y, w: 5.95, h: 0.8,
      fill: { color: WHITE }, line: { color: RULE, width: 1 },
    });
    // dot
    s.addShape("ellipse", {
      x: x + 0.25, y: y + 0.25, w: 0.3, h: 0.3,
      fill: { color: ACCENT }, line: { color: ACCENT, width: 0 },
    });
    // label
    s.addText(label, {
      x: x + 0.7, y: y + 0.1, w: 5.1, h: 0.32,
      fontSize: 13, fontFace: "Calibri", bold: true, color: NAVY,
      align: "left", margin: 0,
    });
    // file
    s.addText(file, {
      x: x + 0.7, y: y + 0.42, w: 5.1, h: 0.32,
      fontSize: 10.5, fontFace: "Consolas", color: SUBTLE,
      align: "left", margin: 0,
    });
  });

  footer(s, 7, TOTAL);
}

// ===========================================================================
// SLIDE 8 — the stack layout (visual diagram)
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "WHAT THE NEW STACK LOOKS LIKE");
  title(s, "argv layout written into the BTOR2 init expression");

  const cx = 4.5;       // diagram column center
  const left = 2.3, right = 6.7;
  const top = 2.0;
  // axis label
  s.addText("high address", {
    x: 0.4, y: 1.85, w: 1.6, h: 0.3,
    fontSize: 10, fontFace: "Calibri", italic: true, color: SUBTLE,
    align: "right", margin: 0,
  });
  s.addText("low address", {
    x: 0.4, y: 6.7, w: 1.6, h: 0.3,
    fontSize: 10, fontFace: "Calibri", italic: true, color: SUBTLE,
    align: "right", margin: 0,
  });

  // boxes from high to low — concrete vs symbolic
  const blocks = [
    { label: "argv[0] bytes:  \"prog\\0\"", note: "concrete", concrete: true,  h: 0.55 },
    { label: "argv[1] bytes  (K bytes + null)", note: "K bytes free",        concrete: false, h: 0.65 },
    { label: "...", note: "",        concrete: false, h: 0.4 },
    { label: "argv[N] bytes  (K bytes + null)", note: "K bytes free",        concrete: false, h: 0.65 },
    { label: "alignment padding", note: "concrete",      concrete: true,  h: 0.45 },
    { label: "pointer array (argv[0..N], NULL)", note: "concrete",      concrete: true,  h: 0.55 },
    { label: "argc", note: "concrete",      concrete: true,  h: 0.4 },
  ];
  let y = top;
  blocks.forEach(b => {
    s.addShape("rect", {
      x: left, y, w: right - left, h: b.h,
      fill: { color: b.concrete ? ICE : ACCENT },
      line: { color: b.concrete ? ICE_DK : "C8A93B", width: 0.75 },
    });
    s.addText(b.label, {
      x: left + 0.1, y, w: right - left - 0.2, h: b.h,
      fontSize: 11, fontFace: "Calibri", bold: !b.concrete, color: NAVY,
      align: "left", valign: "middle", margin: 0,
    });
    if (b.note) {
      s.addText(b.note, {
        x: right + 0.15, y, w: 1.4, h: b.h,
        fontSize: 10, fontFace: "Calibri", italic: true,
        color: b.concrete ? SUBTLE : "B8860B",
        align: "left", valign: "middle", margin: 0,
      });
    }
    y += b.h + 0.05;
  });

  // arrow indicating "SP starts here"
  s.addShape("line", {
    x: left, y: y - 0.1, w: -0.8, h: 0,
    line: { color: NAVY, width: 1.5, endArrowType: "triangle" },
  });
  s.addText("SP", {
    x: left - 1.2, y: y - 0.4, w: 0.6, h: 0.5,
    fontSize: 13, fontFace: "Consolas", bold: true, color: NAVY,
    align: "right", valign: "middle", margin: 0,
  });

  // legend on the right
  const lx = 9.2, ly = 2.2;
  s.addText("Legend", {
    x: lx, y: ly, w: 3.5, h: 0.4,
    fontSize: 12, fontFace: "Calibri", bold: true, color: NAVY,
    align: "left", margin: 0, charSpacing: 1,
  });
  // ice swatch
  s.addShape("rect", {
    x: lx, y: ly + 0.55, w: 0.5, h: 0.35,
    fill: { color: ICE }, line: { color: ICE_DK, width: 0.75 },
  });
  s.addText("Concrete bytes — Rotor writes specific values", {
    x: lx + 0.7, y: ly + 0.5, w: 3.3, h: 0.45,
    fontSize: 11, fontFace: "Calibri", color: NAVY_DK, valign: "middle", margin: 0,
  });
  // accent swatch
  s.addShape("rect", {
    x: lx, y: ly + 1.1, w: 0.5, h: 0.35,
    fill: { color: ACCENT }, line: { color: "C8A93B", width: 0.75 },
  });
  s.addText("Free bytes — solver picks any value 0..255", {
    x: lx + 0.7, y: ly + 1.05, w: 3.3, h: 0.45,
    fontSize: 11, fontFace: "Calibri", color: NAVY_DK, valign: "middle", margin: 0,
  });

  // takeaway
  s.addText(
    "The program reads this stack as if it were a normal OS-prepared argv. " +
    "Only the highlighted bytes are open questions for the solver.",
    {
      x: lx, y: ly + 1.85, w: 3.7, h: 2.0,
      fontSize: 11, fontFace: "Calibri", italic: true, color: SUBTLE,
      valign: "top", margin: 0,
    }
  );

  footer(s, 8, TOTAL);
}

// ===========================================================================
// SLIDE 9 — the mechanism
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "MECHANISM");
  title(s, "How the bytes become symbolic");

  s.addText(
    "For each free argv byte, Rotor emits one BTOR2 state node with no init expression.",
    {
      x: 0.7, y: 1.7, w: 12, h: 0.5,
      fontSize: 15, fontFace: "Calibri", color: NAVY_DK, valign: "top", margin: 0,
    }
  );
  s.addText(
    "BTOR2 semantics: a state without an init is unconstrained. The bounded model checker is " +
    "free to choose any value for it at step 0, and it stays that value for the whole run.",
    {
      x: 0.7, y: 2.2, w: 12, h: 0.85,
      fontSize: 13, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
    }
  );

  // code box
  s.addShape("rect", {
    x: 0.7, y: 3.2, w: 12, h: 1.7,
    fill: { color: NAVY_DK }, line: { color: NAVY_DK, width: 0 },
  });
  s.addText(
    [
      { text: "// machine/core.rs — inside initialize_symbolic_argv\n", options: { color: ICE_DK } },
      { text: "let ", options: { color: "FF9CB6" } },
      { text: "sym_byte = builder.", options: { color: WHITE } },
      { text: "state", options: { color: ACCENT, bold: true } },
      { text: "(\n    sorts.sid_byte,\n    &", options: { color: WHITE } },
      { text: "format!", options: { color: "FF9CB6" } },
      { text: "(\"argv[{}][{}]\", arg_idx + 1, byte_idx),\n    ", options: { color: WHITE } },
      { text: "// no init expression — unconstrained\n", options: { color: ICE_DK } },
      { text: ");", options: { color: WHITE } },
    ],
    {
      x: 0.95, y: 3.3, w: 11.5, h: 1.5,
      fontSize: 13, fontFace: "Consolas", color: WHITE,
      align: "left", valign: "top", margin: 0,
    }
  );

  // bottom row — three small "consequences"
  const conseq = [
    { num: "1", title: "Solver picks the byte",
      body: "Each step-0 SAT assignment includes a value 0..255 for every free byte." },
    { num: "2", title: "Reads see a free value",
      body: "When the program loads argv[i][j] with lb, the result is that free byte." },
    { num: "3", title: "Freedom propagates",
      body: "Branches on the byte become symbolic; both sides enter the unrolled formula." },
  ];
  conseq.forEach((c, i) => {
    const x = 0.7 + i * 4.2;
    s.addShape("rect", {
      x, y: 5.2, w: 3.95, h: 1.5,
      fill: { color: WHITE }, line: { color: NAVY, width: 1 },
    });
    s.addShape("ellipse", {
      x: x + 0.25, y: 5.35, w: 0.4, h: 0.4,
      fill: { color: NAVY }, line: { color: NAVY, width: 0 },
    });
    s.addText(c.num, {
      x: x + 0.25, y: 5.35, w: 0.4, h: 0.4,
      fontSize: 12, fontFace: "Georgia", bold: true, color: ACCENT,
      align: "center", valign: "middle", margin: 0,
    });
    s.addText(c.title, {
      x: x + 0.8, y: 5.35, w: 3.0, h: 0.4,
      fontSize: 13, fontFace: "Calibri", bold: true, color: NAVY,
      align: "left", valign: "middle", margin: 0,
    });
    s.addText(c.body, {
      x: x + 0.25, y: 5.85, w: 3.55, h: 0.85,
      fontSize: 11, fontFace: "Calibri", color: NAVY_DK,
      align: "left", valign: "top", margin: 0,
    });
  });

  footer(s, 9, TOTAL);
}

// ===========================================================================
// SLIDE 10 — design choice: state vs input
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "DESIGN CHOICE");
  title(s, "Why state nodes, not input nodes");

  function compare(x, header, headerFg, headerBg, items) {
    const w = 5.8;
    s.addShape("rect", {
      x, y: 1.85, w, h: 4.7,
      fill: { color: WHITE }, line: { color: RULE, width: 1 },
    });
    s.addShape("rect", {
      x, y: 1.85, w, h: 0.6,
      fill: { color: headerBg }, line: { color: headerBg, width: 0 },
    });
    s.addText(header, {
      x, y: 1.85, w, h: 0.6,
      fontSize: 14, fontFace: "Georgia", bold: true, color: headerFg,
      align: "center", valign: "middle", margin: 0,
    });
    items.forEach((line, i) => {
      // bullet dot
      s.addShape("ellipse", {
        x: x + 0.35, y: 2.7 + i * 0.65 + 0.1, w: 0.18, h: 0.18,
        fill: { color: headerBg }, line: { color: headerBg, width: 0 },
      });
      s.addText(line, {
        x: x + 0.65, y: 2.7 + i * 0.65, w: w - 0.85, h: 0.55,
        fontSize: 12, fontFace: "Calibri", color: NAVY_DK, valign: "top", margin: 0,
      });
    });
  }

  compare(0.7, "input nodes — REJECTED", ICE_DK, "EBC9C9", [
    "Forbidden inside init expressions.",
    "Re-chosen at every step. argv must be fixed at step 0.",
    "Would force argv setup during execution — invasive.",
    "Adds equality constraints across steps, wasting solver budget.",
  ]);

  compare(6.85, "state without init — CHOSEN", WHITE, NAVY, [
    "Legal inside init expressions.",
    "Picked once at step 0; persists for the whole run.",
    "Matches argv semantics directly.",
    "Cost: 8 × symbolic_argc × max_arglen extra BTOR2 lines (~64 at defaults).",
  ]);

  // bottom takeaway band
  s.addShape("rect", {
    x: 0.7, y: 6.7, w: 12, h: 0.55,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });
  s.addText(
    "Mechanical legality + correct semantics — the natural fit, not a clever trick.",
    {
      x: 0.7, y: 6.7, w: 12, h: 0.55,
      fontSize: 13, fontFace: "Calibri", italic: true, color: ICE,
      align: "center", valign: "middle", margin: 0,
    }
  );

  footer(s, 10, TOTAL);
}

// ===========================================================================
// SLIDE 11 — the function, in phases
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "THE NEW FUNCTION");
  title(s, "initialize_symbolic_argv — six phases");

  s.addText(
    "machine/core.rs:272-431. Pure function: input is the config, output is the new initial stack value and the SP.",
    {
      x: 0.7, y: 1.65, w: 12, h: 0.45,
      fontSize: 12, fontFace: "Consolas", color: SUBTLE, valign: "top", margin: 0,
    }
  );

  const phases = [
    { num: "1", title: "Compute the layout",
      body: "Pure arithmetic. Decide the address of every argv byte, every pointer, and argc.\nNo BTOR2 emitted yet." },
    { num: "2", title: "Create the empty stack",
      body: "One state node, no init. The chain of writes that follows will fold into this." },
    { num: "3", title: "Write argv[0] = \"prog\\0\"",
      body: "Concrete bytes. The literal program name. Programs reading argv[0] see exactly these characters." },
    { num: "4", title: "Write argv[1..N]",
      body: "FREE bytes. One state-without-init per byte. This is where symbolic-ness is born." },
    { num: "5", title: "Write the pointer array",
      body: "Concrete pointers, each pointing at the corresponding string above. Plus a NULL terminator." },
    { num: "6", title: "Write argc",
      body: "Concrete integer at SP. Returns (sp, stack_value) to the caller." },
  ];

  phases.forEach((p, i) => {
    const col = i % 3, row = Math.floor(i / 3);
    const x = 0.7 + col * 4.2;
    const y = 2.3 + row * 2.2;
    // card
    s.addShape("rect", {
      x, y, w: 3.95, h: 2.05,
      fill: { color: WHITE }, line: { color: RULE, width: 1 },
    });
    // numbered circle
    s.addShape("ellipse", {
      x: x + 0.2, y: y + 0.2, w: 0.55, h: 0.55,
      fill: { color: NAVY }, line: { color: NAVY, width: 0 },
    });
    s.addText(p.num, {
      x: x + 0.2, y: y + 0.2, w: 0.55, h: 0.55,
      fontSize: 16, fontFace: "Georgia", bold: true, color: ACCENT,
      align: "center", valign: "middle", margin: 0,
    });
    // title
    s.addText(p.title, {
      x: x + 0.85, y: y + 0.2, w: 3.0, h: 0.55,
      fontSize: 13, fontFace: "Calibri", bold: true, color: NAVY,
      align: "left", valign: "middle", margin: 0,
    });
    // body
    s.addText(p.body, {
      x: x + 0.2, y: y + 0.85, w: 3.55, h: 1.15,
      fontSize: 11, fontFace: "Calibri", color: NAVY_DK,
      align: "left", valign: "top", margin: 0,
    });
    // highlight phase 4
    if (p.num === "4") {
      s.addShape("rect", {
        x, y, w: 3.95, h: 2.05,
        fill: { type: "none" }, line: { color: ACCENT, width: 2 },
      });
    }
  });

  footer(s, 11, TOTAL);
}

// ===========================================================================
// SLIDE 12 — the four hooks in CoreState::new
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "WIRING IT UP");
  title(s, "Four short hooks in CoreState::new");

  s.addText(
    "Each hook is a few lines. They are the entire integration with the rest of Rotor.",
    {
      x: 0.7, y: 1.65, w: 12, h: 0.45,
      fontSize: 13, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
    }
  );

  const hooks = [
    { letter: "A", at: "core.rs:100-105", title: "Choose the initial stack pointer",
      body: "If --symbolic-argv is set, call initialize_symbolic_argv and take the SP it returns. Otherwise default to vaddr_top - word_size." },
    { letter: "B", at: "core.rs:115-127", title: "Write SP into register x2",
      body: "RISC-V calling convention: the program reads its stack pointer from x2 on entry. Set it to the value from (A)." },
    { letter: "C", at: "core.rs:130-146", title: "Write argc into register a0",
      body: "RISC-V Linux startup convention. Set a0 = symbolic_argc + 1 so main() sees the right argument count." },
    { letter: "D", at: "core.rs:216-223", title: "Attach the stack value to the stack segment",
      body: "Bind the stack init expression returned by initialize_symbolic_argv to the real stack segment state." },
  ];

  hooks.forEach((h, i) => {
    const y = 2.25 + i * 1.15;
    s.addShape("rect", {
      x: 0.7, y, w: 12, h: 1.05,
      fill: { color: WHITE }, line: { color: RULE, width: 1 },
    });
    // big letter
    s.addShape("rect", {
      x: 0.7, y, w: 0.95, h: 1.05,
      fill: { color: NAVY }, line: { color: NAVY, width: 0 },
    });
    s.addText(h.letter, {
      x: 0.7, y, w: 0.95, h: 1.05,
      fontSize: 32, fontFace: "Georgia", bold: true, color: ACCENT,
      align: "center", valign: "middle", margin: 0,
    });
    // title + at
    s.addText(h.title, {
      x: 1.85, y: y + 0.1, w: 8.0, h: 0.4,
      fontSize: 13.5, fontFace: "Calibri", bold: true, color: NAVY,
      align: "left", valign: "middle", margin: 0,
    });
    s.addText(h.at, {
      x: 9.85, y: y + 0.1, w: 2.7, h: 0.4,
      fontSize: 10.5, fontFace: "Consolas", color: SUBTLE,
      align: "right", valign: "middle", margin: 0,
    });
    s.addText(h.body, {
      x: 1.85, y: y + 0.5, w: 10.7, h: 0.55,
      fontSize: 11.5, fontFace: "Calibri", color: NAVY_DK,
      align: "left", valign: "top", margin: 0,
    });
  });

  footer(s, 12, TOTAL);
}

// ===========================================================================
// SLIDE 13 — validation: benchmarks
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "VALIDATION");
  title(s, "Five benchmarks. Each bug is reachable only via argv.");

  const benches = [
    { name: "test1", desc: "argv[1][0] == 'X'" },
    { name: "test2", desc: "Two-byte: argv[1][0..1] == \"XY\"" },
    { name: "test3", desc: "Multi-character match in argv[1]" },
    { name: "test4", desc: "Two args: argv[1][0]=='X' AND argv[2][0]=='Y'" },
    { name: "test5", desc: "Length-dependent branch on argv[1]" },
  ];

  benches.forEach((b, i) => {
    const y = 2.0 + i * 0.85;
    // numbered circle
    s.addShape("ellipse", {
      x: 0.95, y: y + 0.05, w: 0.55, h: 0.55,
      fill: { color: NAVY }, line: { color: NAVY, width: 0 },
    });
    s.addText(`${i + 1}`, {
      x: 0.95, y: y + 0.05, w: 0.55, h: 0.55,
      fontSize: 16, fontFace: "Georgia", bold: true, color: ACCENT,
      align: "center", valign: "middle", margin: 0,
    });
    s.addText(b.name, {
      x: 1.7, y: y + 0.1, w: 1.7, h: 0.5,
      fontSize: 14, fontFace: "Consolas", bold: true, color: NAVY,
      align: "left", valign: "middle", margin: 0,
    });
    s.addText(b.desc, {
      x: 3.5, y: y + 0.1, w: 6.5, h: 0.5,
      fontSize: 13, fontFace: "Calibri", color: NAVY_DK,
      align: "left", valign: "middle", margin: 0,
    });
    // ✓ tag
    s.addShape("rect", {
      x: 10.2, y: y + 0.05, w: 2.4, h: 0.55,
      fill: { color: "EAF5EE" }, line: { color: "9DD1B2", width: 0.75 },
    });
    s.addText("✓ btormc finds it", {
      x: 10.2, y: y + 0.05, w: 2.4, h: 0.55,
      fontSize: 11.5, fontFace: "Calibri", bold: true, color: "1F7A45",
      align: "center", valign: "middle", margin: 0,
    });
  });

  s.addText(
    "Each benchmark contains a return-1 path guarded by a constraint on specific argv bytes. " +
    "btormc returns a witness trace assigning exactly those bytes; the program reaches the bad-exit branch.",
    {
      x: 0.7, y: 6.5, w: 12, h: 0.7,
      fontSize: 12, fontFace: "Calibri", italic: true, color: SUBTLE,
      align: "left", valign: "top", margin: 0,
    }
  );

  footer(s, 13, TOTAL);
}

// ===========================================================================
// SLIDE 14 — the test4 walkthrough
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "EXAMPLE WALKTHROUGH");
  title(s, "test4_multi_arg.c — bug needs two argv bytes");

  // C code box
  s.addShape("rect", {
    x: 0.7, y: 1.7, w: 6.0, h: 3.6,
    fill: { color: NAVY_DK }, line: { color: NAVY_DK, width: 0 },
  });
  s.addText("test4_multi_arg.c", {
    x: 0.7, y: 1.7, w: 6.0, h: 0.4,
    fontSize: 10.5, fontFace: "Consolas", color: ICE_DK,
    align: "center", valign: "middle", margin: 0, charSpacing: 1,
  });
  s.addText(
    "uint64_t main(uint64_t argc, uint64_t* argv) {\n" +
    "    if (argc > 2) {\n" +
    "        if (((uint64_t*) *(argv + 1))[0]\n" +
    "                              == 'X')\n" +
    "            if (((uint64_t*) *(argv + 2))[0]\n" +
    "                              == 'Y')\n" +
    "                return 1;       // bad exit\n" +
    "    }\n" +
    "    return 0;\n" +
    "}",
    {
      x: 1.0, y: 2.1, w: 5.5, h: 3.1,
      fontSize: 13, fontFace: "Consolas", color: WHITE,
      align: "left", valign: "top", margin: 0,
    }
  );

  // arrow
  s.addShape("rightTriangle", {
    x: 6.85, y: 3.3, w: 0.4, h: 0.4,
    fill: { color: ACCENT }, line: { color: ACCENT, width: 0 },
    rotate: 0,
  });

  // witness box
  s.addShape("rect", {
    x: 7.4, y: 1.7, w: 5.2, h: 3.6,
    fill: { color: WHITE }, line: { color: NAVY, width: 1 },
  });
  s.addShape("rect", {
    x: 7.4, y: 1.7, w: 5.2, h: 0.45,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });
  s.addText("Witness trace from btormc", {
    x: 7.4, y: 1.7, w: 5.2, h: 0.45,
    fontSize: 11, fontFace: "Calibri", bold: true, color: ICE,
    align: "center", valign: "middle", margin: 0, charSpacing: 1,
  });
  s.addText(
    [
      { text: "Solver assignment:\n\n", options: { color: NAVY } },
      { text: "argv[1][0] = ", options: { color: SUBTLE } },
      { text: "'X'  (0x58)\n", options: { color: NAVY, bold: true } },
      { text: "argv[2][0] = ", options: { color: SUBTLE } },
      { text: "'Y'  (0x59)\n\n", options: { color: NAVY, bold: true } },
      { text: "Result:  exit code = 1\n", options: { color: NAVY } },
      { text: "Property:  bad exit triggered ✗", options: { color: "B8323A", bold: true } },
    ],
    {
      x: 7.7, y: 2.4, w: 4.7, h: 2.7,
      fontSize: 14, fontFace: "Consolas",
      align: "left", valign: "top", margin: 0,
    }
  );

  // bottom command line
  s.addShape("rect", {
    x: 0.7, y: 5.6, w: 11.9, h: 1.3,
    fill: { color: CODE_BG }, line: { color: RULE, width: 0.75 },
  });
  s.addText(
    [
      { text: "$ ", options: { color: SUBTLE } },
      { text: "selfie -c test4_multi_arg.c -o test4.m\n", options: { color: NAVY_DK } },
      { text: "$ ", options: { color: SUBTLE } },
      { text: "rotor   test4.m  --symbolic-argv --symbolic-argc 2 --max-arglen 8  -o test4.btor2\n", options: { color: NAVY_DK } },
      { text: "$ ", options: { color: SUBTLE } },
      { text: "btormc  -kmax 200  test4.btor2  >  test4.wit", options: { color: NAVY_DK } },
    ],
    {
      x: 0.95, y: 5.7, w: 11.5, h: 1.1,
      fontSize: 12, fontFace: "Consolas",
      align: "left", valign: "top", margin: 0,
    }
  );

  footer(s, 14, TOTAL);
}

// ===========================================================================
// SLIDE 15 — end-to-end pipeline
// ===========================================================================
{
  const s = pres.addSlide();
  bgWhite(s);
  eyebrow(s, "PIPELINE");
  title(s, "From C source to a witness over argv bytes");

  const stages = [
    { label: "C source", sub: "with argv", color: ICE,    fg: NAVY },
    { label: "RISC-V binary", sub: "selfie",   color: ICE,    fg: NAVY },
    { label: "BTOR2 model", sub: "rotor (Rust)", color: NAVY,  fg: WHITE,
      tag: "+ symbolic argv" },
    { label: "Witness trace", sub: "btormc",   color: NAVY,   fg: WHITE },
    { label: "Bytes for argv", sub: "the answer", color: ACCENT, fg: NAVY },
  ];

  const startX = 0.7, w = 2.35, gap = 0.18, y = 2.5, h = 1.6;
  stages.forEach((st, i) => {
    const x = startX + i * (w + gap);
    s.addShape("rect", {
      x, y, w, h,
      fill: { color: st.color },
      line: { color: st.color === ACCENT ? "C8A93B" : st.color === ICE ? ICE_DK : NAVY, width: 1 },
    });
    s.addText(st.label, {
      x, y: y + 0.25, w, h: 0.5,
      fontSize: 16, fontFace: "Georgia", bold: true, color: st.fg,
      align: "center", valign: "middle", margin: 0,
    });
    s.addText(st.sub, {
      x, y: y + 0.75, w, h: 0.4,
      fontSize: 11, fontFace: "Calibri", italic: true,
      color: st.fg === WHITE ? ICE : SUBTLE,
      align: "center", valign: "middle", margin: 0,
    });
    if (st.tag) {
      s.addShape("rect", {
        x: x + 0.2, y: y + h - 0.45, w: w - 0.4, h: 0.35,
        fill: { color: ACCENT }, line: { color: "C8A93B", width: 0 },
      });
      s.addText(st.tag, {
        x: x + 0.2, y: y + h - 0.45, w: w - 0.4, h: 0.35,
        fontSize: 10.5, fontFace: "Calibri", bold: true, color: NAVY,
        align: "center", valign: "middle", margin: 0,
      });
    }
    // arrow between stages
    if (i < stages.length - 1) {
      const ax = x + w + 0.01;
      s.addShape("line", {
        x: ax, y: y + h / 2, w: gap - 0.04, h: 0,
        line: { color: NAVY, width: 1.5, endArrowType: "triangle" },
      });
    }
  });

  // bottom band — what each stage contributes
  s.addText(
    "Rotor's symbolic-argv mode is the only stage in this pipeline that changed. " +
    "Selfie, btormc, and the visualizer are unchanged third-party tools.",
    {
      x: 0.7, y: 4.6, w: 12, h: 0.6,
      fontSize: 13, fontFace: "Calibri", italic: true, color: SUBTLE,
      align: "left", valign: "top", margin: 0,
    }
  );

  // visualizer mention
  s.addShape("rect", {
    x: 0.7, y: 5.5, w: 12, h: 1.3,
    fill: { color: ICE }, line: { color: ICE_DK, width: 1 },
  });
  s.addShape("rect", {
    x: 0.7, y: 5.5, w: 0.18, h: 1.3,
    fill: { color: NAVY }, line: { color: NAVY, width: 0 },
  });
  s.addText(
    [
      { text: "On top of this pipeline ", options: {} },
      { text: "we also built a browser-based visualizer ", options: { bold: true } },
      { text: "that reads a witness trace and displays it step by step — depth-limited subgraphs, cone-of-influence, per-step inspector. Companion work, separate from the symbolic-argv contribution.", options: {} },
    ],
    {
      x: 1.0, y: 5.55, w: 11.6, h: 1.2,
      fontSize: 13, fontFace: "Calibri", color: NAVY_DK,
      align: "left", valign: "middle", margin: 0,
    }
  );

  footer(s, 15, TOTAL);
}

// ===========================================================================
// SLIDE 16 — summary
// ===========================================================================
{
  const s = pres.addSlide();
  bgNavy(s);

  // accent block
  s.addShape("rect", {
    x: 0, y: 0, w: 0.35, h: H, fill: { color: ACCENT }, line: { color: ACCENT, width: 0 },
  });

  s.addText("ONE-LINE SUMMARY", {
    x: 0.9, y: 1.0, w: 12, h: 0.4,
    fontSize: 11, fontFace: "Calibri", bold: true, color: ACCENT,
    align: "left", margin: 0, charSpacing: 3,
  });

  s.addText("The bounded model checker now produces witness traces over command-line arguments, not just stdin.", {
    x: 0.9, y: 1.6, w: 12, h: 2.5,
    fontSize: 32, fontFace: "Georgia", bold: true, color: WHITE,
    align: "left", margin: 0,
  });

  // counts
  const stats = [
    ["1", "new function"],
    ["3", "CLI flags"],
    ["3", "Config fields"],
    ["4", "hooks in CoreState::new"],
    ["5", "benchmark programs"],
  ];
  stats.forEach(([n, lbl], i) => {
    const x = 0.9 + i * 2.45;
    s.addText(n, {
      x, y: 4.4, w: 2.35, h: 0.7,
      fontSize: 38, fontFace: "Georgia", bold: true, color: ACCENT,
      align: "center", margin: 0,
    });
    s.addText(lbl, {
      x, y: 5.15, w: 2.35, h: 0.5,
      fontSize: 11, fontFace: "Calibri", color: ICE,
      align: "center", margin: 0,
    });
  });

  s.addShape("line", {
    x: 0.9, y: 6.0, w: 1.6, h: 0,
    line: { color: ACCENT, width: 2 },
  });
  s.addText(
    "The rest of Rotor is unchanged. The contribution is contained to the entry-point setup of the model.",
    {
      x: 0.9, y: 6.2, w: 12, h: 0.6,
      fontSize: 14, fontFace: "Calibri", italic: true, color: ICE,
      align: "left", margin: 0,
    }
  );

  s.addText("Thank you.", {
    x: 0.9, y: 6.85, w: 12, h: 0.4,
    fontSize: 12, fontFace: "Calibri", color: ICE_DK,
    align: "left", margin: 0,
  });
}

// ---- write ---------------------------------------------------------------
pres.writeFile({ fileName: "C:/Users/jasko/Programming/Rust/Project01/Symbolic_Console_Arguments_Deck.pptx" })
    .then(fn => console.log("Wrote " + fn));
