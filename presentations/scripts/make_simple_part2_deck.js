// Simple presentation covering Part 2 of Rotor_Deep_Dive.pdf
// (sections 2.1 through 2.7 — symbolic arguments, end to end)
// Plain style: white backgrounds, simple typography, no decorative shapes.

const pptxgen = require("pptxgenjs");
const pres = new pptxgen();

pres.layout = "LAYOUT_WIDE";
const W = 13.333, H = 7.5;

// ---- minimal palette -----------------------------------------------------
const INK    = "1A1A1A";
const SUBTLE = "606060";
const ACCENT = "2A5DA0";
const RULE   = "D0D0D0";
const CODE   = "F5F5F5";

// ---- helpers -------------------------------------------------------------
function plain(slide) {
    slide.background = { color: "FFFFFF" };
}

function pageNum(slide, n, total) {
    slide.addText(`${n} / ${total}`, {
        x: 12.0, y: 7.05, w: 1.2, h: 0.3,
        fontSize: 9, fontFace: "Calibri", color: SUBTLE,
        align: "right", margin: 0,
    });
}

function title(slide, text) {
    slide.addText(text, {
        x: 0.7, y: 0.55, w: 12, h: 0.7,
        fontSize: 26, fontFace: "Calibri", bold: true,
        color: INK, align: "left", valign: "top", margin: 0,
    });
}

// thin underline below title — single 1px rule, no decorative line
function rule(slide) {
    slide.addShape("line", {
        x: 0.7, y: 1.35, w: 11.9, h: 0,
        line: { color: RULE, width: 0.75 },
    });
}

const TOTAL = 15;

// ===========================================================================
// 1 — Title
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);

    s.addText("Symbolic arguments", {
        x: 0.7, y: 2.7, w: 12, h: 1.0,
        fontSize: 44, fontFace: "Calibri", bold: true,
        color: INK, align: "left", valign: "top", margin: 0,
    });
    s.addText("End to end — CLI flag to witness trace", {
        x: 0.7, y: 3.7, w: 12, h: 0.6,
        fontSize: 20, fontFace: "Calibri",
        color: SUBTLE, align: "left", valign: "top", margin: 0,
    });

    s.addShape("line", {
        x: 0.7, y: 4.6, w: 1.2, h: 0,
        line: { color: ACCENT, width: 1.5 },
    });

    s.addText("Rotor (Rust) — Part 2 of the deep dive", {
        x: 0.7, y: 4.85, w: 12, h: 0.4,
        fontSize: 12, fontFace: "Calibri", italic: true,
        color: SUBTLE, align: "left", margin: 0,
    });
}

// ===========================================================================
// 2 — What "symbolic arguments" actually means
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "What 'symbolic arguments' actually means");
    rule(s);

    s.addText("The user wants to ask:", {
        x: 0.7, y: 1.8, w: 12, h: 0.5,
        fontSize: 14, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addText(
        "“Can my program crash, exit nonzero, or hit some bad state for any " +
        "command-line argument the user might type?”",
        {
            x: 1.0, y: 2.3, w: 11.5, h: 1.0,
            fontSize: 18, fontFace: "Calibri", italic: true, color: ACCENT,
            valign: "top", margin: 0,
        }
    );

    s.addText("Symbolic arguments make argv an open question:", {
        x: 0.7, y: 3.6, w: 12, h: 0.5,
        fontSize: 14, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addText([
        { text: "•  Each character of argv[1..N] is left free.\n", options: {} },
        { text: "•  The solver fills it in to find a bug.\n", options: {} },
        { text: "•  Rotor lays out a normal argv on the stack, but writes ", options: {} },
        { text: "‘unknown bytes’", options: { italic: true } },
        { text: " where the user's argument characters would go.\n", options: {} },
        { text: "•  The program reads argv exactly as in a real OS — it doesn't know anything is symbolic.", options: {} },
    ], {
        x: 1.0, y: 4.15, w: 11.5, h: 2.5,
        fontSize: 14, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    pageNum(s, 2, TOTAL);
}

// ===========================================================================
// 3 — 2.1 CLI flags
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "2.1  CLI flags — main.rs");
    rule(s);

    const flags = [
        ["--symbolic-argv",
         "Turn the feature on. Without it, argv is empty and the program runs as if invoked with no arguments."],
        ["--symbolic-argc N    (default 1)",
         "Number of symbolic arguments. argv[0] is always the literal “prog”. Total argc = N + 1."],
        ["--max-arglen K       (default 8)",
         "Bytes of each symbolic argument that are free. Each string is K + 1 bytes including the null terminator."],
    ];

    flags.forEach(([flag, desc], i) => {
        const y = 1.8 + i * 1.4;
        s.addText(flag, {
            x: 0.7, y, w: 12, h: 0.4,
            fontSize: 16, fontFace: "Consolas", bold: true, color: ACCENT,
            valign: "top", margin: 0,
        });
        s.addText(desc, {
            x: 0.7, y: y + 0.5, w: 12, h: 0.85,
            fontSize: 13, fontFace: "Calibri", color: INK,
            valign: "top", margin: 0,
        });
    });

    s.addText(
        "After parsing, main stuffs these into a Config and calls generator::model_rotor(...). " +
        "The CLI never touches symbolic argv again.",
        {
            x: 0.7, y: 6.3, w: 12, h: 0.6,
            fontSize: 12, fontFace: "Calibri", italic: true, color: SUBTLE,
            valign: "top", margin: 0,
        }
    );

    pageNum(s, 3, TOTAL);
}

// ===========================================================================
// 4 — 2.2 Config fields
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "2.2  Config fields — config.rs");
    rule(s);

    s.addText("The three flags become three fields on Config:", {
        x: 0.7, y: 1.7, w: 12, h: 0.5,
        fontSize: 14, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addShape("rect", {
        x: 0.7, y: 2.3, w: 12, h: 1.95,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "pub struct Config {\n" +
        "    // ...\n" +
        "    pub symbolic_argv: bool,\n" +
        "    pub symbolic_argc: usize,\n" +
        "    pub max_arglen:    usize,\n" +
        "    // ...\n" +
        "}",
        {
            x: 0.95, y: 2.4, w: 11.7, h: 1.85,
            fontSize: 14, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    s.addText(
        "Read once, much later, by CoreState::new. Nothing else in config.rs " +
        "matters for symbolic arguments.",
        {
            x: 0.7, y: 4.5, w: 12, h: 0.6,
            fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
        }
    );

    pageNum(s, 4, TOTAL);
}

// ===========================================================================
// 5 — 2.3 CoreState::new — four hooks
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "2.3  CoreState::new — four hooks");
    rule(s);

    s.addText("CoreState::new wires symbolic arguments into the machine. Four short blocks:", {
        x: 0.7, y: 1.65, w: 12, h: 0.5,
        fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    const hooks = [
        ["(a)  Choose the initial stack pointer",
         "If --symbolic-argv is on, call initialize_symbolic_argv. Use the SP it returns. " +
         "Otherwise default SP to one word below stack top."],
        ["(b)  Set the stack-pointer register (x2)",
         "Write the SP value into register x2. The program enters main with its stack pointer " +
         "already pointing at the argv layout we built."],
        ["(c)  Set a0 = argc",
         "RISC-V Linux calling convention: write symbolic_argc + 1 into x10. " +
         "main now sees the right argument count."],
        ["(d)  Bind the stack value to the stack segment",
         "Make the stack value built by initialize_symbolic_argv the stack segment's starting value. " +
         "From this line on, dereferencing **(sp+8) finds the argv we built."],
    ];

    hooks.forEach(([h, body], i) => {
        const y = 2.25 + i * 1.18;
        s.addText(h, {
            x: 0.7, y, w: 12, h: 0.35,
            fontSize: 13.5, fontFace: "Calibri", bold: true, color: ACCENT,
            valign: "top", margin: 0,
        });
        s.addText(body, {
            x: 0.7, y: y + 0.4, w: 12, h: 0.7,
            fontSize: 12, fontFace: "Calibri", color: INK,
            valign: "top", margin: 0,
        });
    });

    pageNum(s, 5, TOTAL);
}

// ===========================================================================
// 6 — 2.4 initialize_symbolic_argv — signature
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "2.4  initialize_symbolic_argv — the headline function");
    rule(s);

    s.addText("Signature:", {
        x: 0.7, y: 1.7, w: 12, h: 0.4,
        fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addShape("rect", {
        x: 0.7, y: 2.15, w: 12, h: 1.7,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "fn initialize_symbolic_argv(\n" +
        "    builder:    &mut Btor2Builder,\n" +
        "    sorts:      &MachineSorts,\n" +
        "    config:     &Config,\n" +
        "    stack_top:  u64,\n" +
        "    word_size:  u64,\n" +
        ") -> (u64, Option<NodeId>)",
        {
            x: 0.95, y: 2.25, w: 11.7, h: 1.6,
            fontSize: 13, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    s.addText("Returns (initial_sp, stack_value).", {
        x: 0.7, y: 4.1, w: 12, h: 0.4,
        fontSize: 13, fontFace: "Calibri", bold: true, color: INK, valign: "top", margin: 0,
    });

    s.addText([
        { text: "•  ", options: {} }, { text: "initial_sp", options: { fontFace: "Consolas" } }, { text: " — where SP should start.\n", options: {} },
        { text: "•  ", options: {} }, { text: "stack_value", options: { fontFace: "Consolas" } }, { text: " — what the stack array should look like at step 0.", options: {} },
    ], {
        x: 1.0, y: 4.55, w: 11.5, h: 1.0,
        fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addText("Walked phase by phase on the next slides. Six phases.", {
        x: 0.7, y: 5.85, w: 12, h: 0.5,
        fontSize: 13, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
    });

    pageNum(s, 6, TOTAL);
}

// ===========================================================================
// 7 — Phase 1 — compute the layout (text + diagram)
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "Phase 1 — compute the layout");
    rule(s);

    s.addText("Pure arithmetic. No machine state is touched yet.", {
        x: 0.7, y: 1.7, w: 12, h: 0.4,
        fontSize: 13, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
    });

    s.addText("Decide the address of every argv byte, every pointer, and argc:", {
        x: 0.7, y: 2.15, w: 6, h: 0.5,
        fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    // Text-style ASCII layout on the left
    s.addShape("rect", {
        x: 0.7, y: 2.7, w: 6.0, h: 4.0,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "high address\n" +
        "  +---------------------------+\n" +
        "  |  argv[0] bytes  \"prog\\0\" |\n" +
        "  |  argv[1] bytes  K + null  |\n" +
        "  |  ...                      |\n" +
        "  |  argv[N] bytes  K + null  |\n" +
        "  +---------------------------+\n" +
        "  |  word-alignment padding   |\n" +
        "  +---------------------------+\n" +
        "  |  pointer to argv[0]       |\n" +
        "  |  pointer to argv[1]       |\n" +
        "  |  ...                      |\n" +
        "  |  pointer to argv[N]       |\n" +
        "  |  NULL                     |\n" +
        "  +---------------------------+\n" +
        "  |  argc                     |   <- SP\n" +
        "low address",
        {
            x: 0.85, y: 2.8, w: 5.85, h: 3.85,
            fontSize: 10, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    // Right column — what we compute
    s.addText("Variables computed:", {
        x: 7.0, y: 2.15, w: 5.7, h: 0.4,
        fontSize: 13, fontFace: "Calibri", bold: true, color: INK, valign: "top", margin: 0,
    });

    s.addText([
        { text: "total_argc",         options: { fontFace: "Consolas", bold: true } },
        { text: " = symbolic_argc + 1\n", options: {} },
        { text: "string_area_size\n",  options: { fontFace: "Consolas", bold: true } },
        { text: "string_area_aligned (word-aligned)\n", options: { fontFace: "Consolas", bold: true } },
        { text: "pointer_area_size\n", options: { fontFace: "Consolas", bold: true } },
        { text: "string_area_start\n", options: { fontFace: "Consolas", bold: true } },
        { text: "pointer_area_start\n", options: { fontFace: "Consolas", bold: true } },
        { text: "sp",                  options: { fontFace: "Consolas", bold: true } },
        { text: " = pointer_area_start − word_size", options: {} },
    ], {
        x: 7.0, y: 2.65, w: 5.7, h: 4.0,
        fontSize: 12, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    pageNum(s, 7, TOTAL);
}

// ===========================================================================
// 8 — Phase 2 — create the stack array
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "Phase 2 — create the stack array");
    rule(s);

    s.addText("Start with an empty state. We will fold writes into it.", {
        x: 0.7, y: 1.7, w: 12, h: 0.5,
        fontSize: 14, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addShape("rect", {
        x: 0.7, y: 2.3, w: 12, h: 1.4,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "let stack_seg = builder.state(sorts.sid_stack_state, \"initial-stack-base\", ...);\n" +
        "let mut current = stack_seg;",
        {
            x: 0.95, y: 2.45, w: 11.7, h: 1.2,
            fontSize: 14, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    s.addText("How `current` works:", {
        x: 0.7, y: 4.0, w: 12, h: 0.4,
        fontSize: 13, fontFace: "Calibri", bold: true, color: INK, valign: "top", margin: 0,
    });

    s.addText([
        { text: "•  Each ", options: {} },
        { text: "builder.write", options: { fontFace: "Consolas" } },
        { text: " returns a new value with one byte updated.\n", options: {} },
        { text: "•  We assign that result back to ", options: {} },
        { text: "current", options: { fontFace: "Consolas" } },
        { text: ".\n", options: {} },
        { text: "•  By the end of the function, ", options: {} },
        { text: "current", options: { fontFace: "Consolas" } },
        { text: " holds the entire initial stack.", options: {} },
    ], {
        x: 1.0, y: 4.5, w: 11.5, h: 1.5,
        fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addText(
        "Why an empty state, not zeros: bytes outside argv stay free, so the program may not " +
        "assume any particular value for them. In practice the program never reads those bytes.",
        {
            x: 0.7, y: 6.1, w: 12, h: 0.8,
            fontSize: 12, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
        }
    );

    pageNum(s, 8, TOTAL);
}

// ===========================================================================
// 9 — Phase 3 — write argv[0]
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "Phase 3 — write argv[0] = \"prog\\0\"");
    rule(s);

    s.addText("Concrete bytes. The program name is always the literal string \"prog\\0\".", {
        x: 0.7, y: 1.7, w: 12, h: 0.5,
        fontSize: 14, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addShape("rect", {
        x: 0.7, y: 2.3, w: 12, h: 3.5,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "let mut str_addr = string_area_start;\n" +
        "addrs.push(str_addr);            // remember where argv[0] starts\n" +
        "\n" +
        "for &byte_val in prog_name {\n" +
        "    let addr = builder.constd(sorts.sid_stack_address, str_addr, None);\n" +
        "    let val  = builder.constd(sorts.sid_byte, byte_val as u64, ...);\n" +
        "    current  = builder.write(sorts.sid_stack_state, current, addr, val, None);\n" +
        "    str_addr += 1;\n" +
        "}\n" +
        "// null terminator\n" +
        "let addr = builder.constd(sorts.sid_stack_address, str_addr, None);\n" +
        "let null = builder.constd(sorts.sid_byte, 0, ...);\n" +
        "current  = builder.write(sorts.sid_stack_state, current, addr, null, None);",
        {
            x: 0.95, y: 2.4, w: 11.7, h: 3.35,
            fontSize: 11.5, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    s.addText(
        "Programs reading argv[0] see the exact characters 'p', 'r', 'o', 'g', '\\0'.",
        {
            x: 0.7, y: 6.0, w: 12, h: 0.5,
            fontSize: 13, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
        }
    );

    pageNum(s, 9, TOTAL);
}

// ===========================================================================
// 10 — Phase 4 — symbolic bytes (KEY SLIDE)
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "Phase 4 — write argv[1..N]   (the symbolic part)");
    rule(s);

    s.addText("This is where ‘symbolic’ is born. Free bytes — solver picks any value 0..255.", {
        x: 0.7, y: 1.7, w: 12, h: 0.5,
        fontSize: 14, fontFace: "Calibri", bold: true, color: ACCENT, valign: "top", margin: 0,
    });

    s.addShape("rect", {
        x: 0.7, y: 2.3, w: 12, h: 3.4,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "for arg_idx in 0..config.symbolic_argc {\n" +
        "    addrs.push(str_addr);\n" +
        "    for byte_idx in 0..max_arglen {\n" +
        "        let addr = builder.constd(sorts.sid_stack_address, str_addr, None);\n" +
        "\n" +
        "        // the actual symbolic byte — fresh free 8-bit value\n" +
        "        let sym_byte = builder.state(\n" +
        "            sorts.sid_byte,\n" +
        "            &format!(\"argv[{}][{}]\", arg_idx + 1, byte_idx),\n" +
        "            ...\n" +
        "        );\n" +
        "\n" +
        "        current = builder.write(sorts.sid_stack_state, current, addr, sym_byte, None);\n" +
        "        str_addr += 1;\n" +
        "    }\n" +
        "    // null terminator stays concrete\n" +
        "    ...\n" +
        "}",
        {
            x: 0.95, y: 2.4, w: 11.7, h: 3.25,
            fontSize: 11, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    s.addText([
        { text: "One line creates each free byte:  ", options: {} },
        { text: "let sym_byte = builder.state(sorts.sid_byte, \"argv[i][j]\", ...);", options: { fontFace: "Consolas", bold: true } },
    ], {
        x: 0.7, y: 5.85, w: 12, h: 0.5,
        fontSize: 12, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addText(
        "There are symbolic_argc × max_arglen of these in total. Every other byte on the stack " +
        "is concrete; only these ones are free. That is the entire mechanism.",
        {
            x: 0.7, y: 6.4, w: 12, h: 0.7,
            fontSize: 12, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
        }
    );

    pageNum(s, 10, TOTAL);
}

// ===========================================================================
// 11 — Phase 5 — pointer area
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "Phase 5 — write the pointer area");
    rule(s);

    s.addText("All concrete. Pointers to the strings we wrote, plus a NULL terminator.", {
        x: 0.7, y: 1.7, w: 12, h: 0.5,
        fontSize: 14, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addShape("rect", {
        x: 0.7, y: 2.3, w: 12, h: 3.6,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "let mut ptr_addr = pointer_area_start;\n" +
        "for (i, &string_addr) in argv_string_addrs.iter().enumerate() {\n" +
        "    // little-endian, byte by byte\n" +
        "    for byte_idx in 0..word_size {\n" +
        "        let byte_val = (string_addr >> (byte_idx * 8)) & 0xFF;\n" +
        "        let addr = builder.constd(sorts.sid_stack_address, ptr_addr + byte_idx, None);\n" +
        "        let val  = builder.constd(sorts.sid_byte, byte_val, ...);\n" +
        "        current  = builder.write(sorts.sid_stack_state, current, addr, val, None);\n" +
        "    }\n" +
        "    ptr_addr += word_size;\n" +
        "}\n" +
        "// + a NULL pointer terminator at the end (POSIX requires it)",
        {
            x: 0.95, y: 2.4, w: 11.7, h: 3.45,
            fontSize: 11.5, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    s.addText(
        "Each entry of the pointer array points to the string we wrote in Phase 3 / 4. " +
        "The trailing NULL is what `argv` arrays are required to end with in POSIX.",
        {
            x: 0.7, y: 6.05, w: 12, h: 0.7,
            fontSize: 12, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
        }
    );

    pageNum(s, 11, TOTAL);
}

// ===========================================================================
// 12 — Phase 6 — argc at SP
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "Phase 6 — write argc at SP");
    rule(s);

    s.addText("Concrete integer. Then return.", {
        x: 0.7, y: 1.7, w: 12, h: 0.5,
        fontSize: 14, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addShape("rect", {
        x: 0.7, y: 2.3, w: 12, h: 2.5,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "let argc_value = total_argc as u64;\n" +
        "for byte_idx in 0..word_size {\n" +
        "    let byte_val = (argc_value >> (byte_idx * 8)) & 0xFF;\n" +
        "    let addr = builder.constd(sorts.sid_stack_address, sp + byte_idx, None);\n" +
        "    let val  = builder.constd(sorts.sid_byte, byte_val, ...);\n" +
        "    current  = builder.write(sorts.sid_stack_state, current, addr, val, None);\n" +
        "}\n" +
        "\n" +
        "(sp, Some(current))",
        {
            x: 0.95, y: 2.4, w: 11.7, h: 2.35,
            fontSize: 12, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    s.addText(
        "argc is now in two places: in register a0 (set by hook (c)) and on the stack at SP. " +
        "RISC-V startup code can find it either way; both hold the same value.",
        {
            x: 0.7, y: 5.0, w: 12, h: 0.8,
            fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
        }
    );

    s.addText(
        "Return: SP and the fully built stack value. Caller (CoreState::new) attaches the value " +
        "to the stack segment.",
        {
            x: 0.7, y: 5.95, w: 12, h: 0.7,
            fontSize: 12, fontFace: "Calibri", italic: true, color: SUBTLE, valign: "top", margin: 0,
        }
    );

    pageNum(s, 12, TOTAL);
}

// ===========================================================================
// 13 — 2.5 What the running program sees
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "2.5  What the running program actually sees");
    rule(s);

    s.addText(
        "The program runs normally. There is no special path for symbolic data. " +
        "Trace one access: the program reads argv[1][0].",
        {
            x: 0.7, y: 1.7, w: 12, h: 0.7,
            fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
        }
    );

    const steps = [
        ["1.", "Load argv from the stack",
         "ld a1, 8(sp)  →  Memory::load_double_word returns the pointer-area value (a concrete address)."],
        ["2.", "Load argv[1]",
         "Another ld at a1+8  →  the address of the first symbolic string."],
        ["3.", "Load argv[1][0]",
         "lb t0, 0(a1)  →  Memory::load_byte returns the free 8-bit value created in Phase 4."],
        ["4.", "Branch on the byte",
         "if (t0 == 'X')  →  comparison whose result is itself a free boolean. Both sides enter the model."],
        ["5.", "Solver picks values",
         "btormc finds an assignment for every symbolic byte that satisfies the chain of branch conditions."],
    ];

    steps.forEach(([n, h, body], i) => {
        const y = 2.55 + i * 0.85;
        s.addText(n, {
            x: 0.7, y, w: 0.5, h: 0.4,
            fontSize: 13, fontFace: "Calibri", bold: true, color: ACCENT, valign: "top", margin: 0,
        });
        s.addText(h, {
            x: 1.2, y, w: 11, h: 0.4,
            fontSize: 13, fontFace: "Calibri", bold: true, color: INK, valign: "top", margin: 0,
        });
        s.addText(body, {
            x: 1.2, y: y + 0.35, w: 11, h: 0.5,
            fontSize: 11.5, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
        });
    });

    pageNum(s, 13, TOTAL);
}

// ===========================================================================
// 14 — 2.6 End-to-end on a real benchmark
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "2.6  End-to-end on a real benchmark");
    rule(s);

    s.addText("test4_multi_arg.c — exits 1 only when argv[1][0]=='X' AND argv[2][0]=='Y'.", {
        x: 0.7, y: 1.7, w: 12, h: 0.5,
        fontSize: 13, fontFace: "Calibri", color: INK, valign: "top", margin: 0,
    });

    s.addShape("rect", {
        x: 0.7, y: 2.25, w: 12, h: 4.55,
        fill: { color: CODE }, line: { color: RULE, width: 0.5 },
    });
    s.addText(
        "$ selfie -c test4_multi_arg.c -o test4.m\n" +
        "    # produces a RISC-V binary\n" +
        "\n" +
        "$ rotor test4.m --symbolic-argv --symbolic-argc 2 --max-arglen 8 -o test4.btor2\n" +
        "    # CoreState::new is called once.\n" +
        "    # initialize_symbolic_argv lays out argv:\n" +
        "    #    argv[0] = \"prog\\0\"   (concrete)\n" +
        "    #    argv[1] = 8 free bytes + null\n" +
        "    #    argv[2] = 8 free bytes + null\n" +
        "    # SP, a0, stack segment all wired up.\n" +
        "\n" +
        "$ btormc -kmax 200 test4.btor2 > test4.wit\n" +
        "    # solver finds:  argv[1][0] = 88 ('X'),  argv[2][0] = 89 ('Y')\n" +
        "    # program reaches return 1  →  exit_status nonzero  →  bad-exit triggered.\n" +
        "\n" +
        "$ visualizer/index.html  ←  load test4.btor2 + test4.wit\n" +
        "    # see the witness step by step",
        {
            x: 0.95, y: 2.35, w: 11.7, h: 4.4,
            fontSize: 11, fontFace: "Consolas", color: INK,
            valign: "top", margin: 0,
        }
    );

    pageNum(s, 14, TOTAL);
}

// ===========================================================================
// 15 — 2.7 Recap
// ===========================================================================
{
    const s = pres.addSlide();
    plain(s);
    title(s, "2.7  Recap — every function on the symbolic-argv path");
    rule(s);

    const rows = [
        [{ text: "Layer", options: { bold: true } },
         { text: "Function", options: { bold: true } },
         { text: "What it does", options: { bold: true } }],
        ["CLI", "main()",
         "Parses --symbolic-argv, --symbolic-argc, --max-arglen; builds Config."],
        ["Pipeline", "generator::model_rotor",
         "Loads the binary; sets up sorts/consts; calls CoreState::new for each core."],
        ["Core setup", "CoreState::new",
         "Branches on config.symbolic_argv. If on, calls initialize_symbolic_argv, writes SP and a0=argc, attaches the stack value."],
        ["Argv layout", "initialize_symbolic_argv",
         "Six phases: layout → empty stack → argv[0] → argv[1..N] free → pointer array → argc."],
        ["Runtime reads", "Memory::load_byte / load_word / ...",
         "When the program reads argv, this fires. No symbolic-aware code; the freedom is already in the bytes."],
        ["Properties", "rotor_properties",
         "Defines what counts as a bug. Reachability of these is what btormc proves or refutes."],
    ];

    s.addTable(rows, {
        x: 0.7, y: 1.85, w: 12.0,
        colW: [2.0, 4.0, 6.0],
        fontSize: 12, fontFace: "Calibri", color: INK,
        border: { type: "solid", pt: 0.5, color: RULE },
        rowH: 0.6,
        valign: "top",
        margin: 0.08,
    });

    s.addText(
        "The whole feature is one new function plus four short hooks in CoreState::new. " +
        "Every other module is unchanged.",
        {
            x: 0.7, y: 6.5, w: 12, h: 0.6,
            fontSize: 13, fontFace: "Calibri", italic: true, color: ACCENT, valign: "top", margin: 0,
        }
    );

    pageNum(s, 15, TOTAL);
}

// --- write -----------------------------------------------------------------
pres.writeFile({
    fileName: "C:/Users/jasko/Programming/Rust/Project01/Symbolic_Arguments_Simple_Deck.pptx"
}).then(fn => console.log("Wrote " + fn));
