/**
 * BTOR2 Witness Trace Parser
 *
 * Parses btormc / btorsim counterexample witness format:
 *   sat
 *   b<bad_index>
 *   @<step>
 *   <index> <binary_value>[ <symbol_name>]
 *   ...
 *   #<step>
 *   @<step>
 *   ...
 *   .
 *
 * The witness has two sections per time step:
 *   - State assignments (only meaningful at step 0 for initial state values)
 *   - Input assignments (at every step)
 *
 * We build a mapping from state/input indices to BTOR2 nids by scanning
 * the parsed model's state and input nodes in declaration order.
 */

/**
 * Parse a btormc witness trace.
 * @param {string} text - Raw witness text
 * @param {Map} modelNodes - Parsed BTOR2 node map (from parser.js)
 * @returns {{ bad: number|null, frames: Array<{step: number, states: Map, inputs: Map}>, errors: string[] }}
 */
function parseWitness(text, modelNodes) {
    const errors = [];

    // Build ordered lists of state and input nids (declaration order = index order)
    const stateNids = [];
    const inputNids = [];
    for (const [nid, node] of modelNodes) {
        if (node.op === 'state') stateNids.push(nid);
        if (node.op === 'input') inputNids.push(nid);
    }

    const lines = text.split('\n').map(l => l.trim()).filter(l => l.length > 0);
    if (lines.length === 0) {
        errors.push('Empty witness trace');
        return { bad: null, frames: [], errors };
    }

    // First line should be 'sat'
    const header = lines[0].toLowerCase();
    if (header !== 'sat') {
        if (header === 'unsat') {
            errors.push('Witness indicates UNSAT — no counterexample exists');
            return { bad: null, frames: [], errors };
        }
        errors.push(`Unexpected header: "${lines[0]}" (expected "sat")`);
    }

    let bad = null;
    const frames = [];
    let currentStep = -1;
    let section = null; // 'state' or 'input'
    let currentFrame = null;

    for (let i = 1; i < lines.length; i++) {
        const line = lines[i];

        // End of witness
        if (line === '.') break;

        // Bad property: b<N>
        const badMatch = line.match(/^b(\d+)$/);
        if (badMatch) {
            bad = parseInt(badMatch[1], 10);
            continue;
        }

        // Justice property: j<N>
        if (line.match(/^j\d+$/)) continue;

        // Time step header: @<N>
        const stepMatch = line.match(/^@(\d+)$/);
        if (stepMatch) {
            const step = parseInt(stepMatch[1], 10);
            if (step !== currentStep) {
                // New frame
                currentStep = step;
                currentFrame = {
                    step: step,
                    states: new Map(),  // nid → { value, bits }
                    inputs: new Map(),  // nid → { value, bits }
                };
                frames.push(currentFrame);
                section = 'state'; // first section after @k is states
            } else {
                // Same step seen again → this is the input section
                section = 'input';
            }
            continue;
        }

        // End of section: #<N>
        const endMatch = line.match(/^#(\d+)$/);
        if (endMatch) {
            // Toggle from state section to input section
            section = 'input';
            continue;
        }

        // Assignment: <index> <value>[ <symbol>]
        const assignMatch = line.match(/^(\d+)\s+([01]+(?:\[[^\]]*\])?)(?:\s+(.*))?$/);
        if (assignMatch && currentFrame) {
            const idx = parseInt(assignMatch[1], 10);
            const bits = assignMatch[2];
            const symbol = assignMatch[3] || null;

            // Convert binary to decimal (handle as BigInt for wide bitvectors)
            let value;
            const pureBits = bits.replace(/\[.*\]/, ''); // remove array notation if any
            if (pureBits.length <= 52) {
                value = parseInt(pureBits, 2);
            } else {
                value = BigInt('0b' + pureBits);
            }

            const entry = { value, bits: pureBits, symbol };

            if (section === 'state' && idx < stateNids.length) {
                const nid = stateNids[idx];
                currentFrame.states.set(nid, entry);
            } else if (section === 'input' && idx < inputNids.length) {
                const nid = inputNids[idx];
                currentFrame.inputs.set(nid, entry);
            } else if (section === 'state') {
                errors.push(`Step ${currentStep}: state index ${idx} out of range (${stateNids.length} states)`);
            } else {
                errors.push(`Step ${currentStep}: input index ${idx} out of range (${inputNids.length} inputs)`);
            }
            continue;
        }

        // Array assignment: <index> [<idx>] <value>
        const arrayMatch = line.match(/^(\d+)\s+\[(\d+)\]\s+([01]+)(?:\s+(.*))?$/);
        if (arrayMatch && currentFrame) {
            const idx = parseInt(arrayMatch[1], 10);
            const arrayIdx = parseInt(arrayMatch[2], 10);
            const bits = arrayMatch[3];
            const symbol = arrayMatch[4] || null;

            const pureBits = bits;
            let value;
            if (pureBits.length <= 52) {
                value = parseInt(pureBits, 2);
            } else {
                value = BigInt('0b' + pureBits);
            }

            const entry = { value, bits: pureBits, symbol, arrayIndex: arrayIdx };

            if (section === 'state' && idx < stateNids.length) {
                const nid = stateNids[idx];
                // For arrays, store multiple entries
                if (!currentFrame.states.has(nid)) {
                    currentFrame.states.set(nid, []);
                }
                const existing = currentFrame.states.get(nid);
                if (Array.isArray(existing)) {
                    existing.push(entry);
                } else {
                    currentFrame.states.set(nid, [existing, entry]);
                }
            }
            continue;
        }
    }

    // If no explicit input sections found, try to infer from frame structure
    // (some btormc versions merge state+input in the same section)

    return { bad, frames, errors, stateNids, inputNids };
}

/**
 * Format a binary value for display.
 */
function formatWitnessValue(bits, maxLen) {
    maxLen = maxLen || 16;
    if (!bits) return '?';

    // For short values, show decimal
    if (bits.length <= 64) {
        const val = bits.length <= 52 ? parseInt(bits, 2) : BigInt('0b' + bits);
        // Also check for negative (two's complement) for display
        if (bits.length > 1 && bits[0] === '1') {
            // Could be negative in signed interpretation
            const unsigned = typeof val === 'bigint' ? val : BigInt(val);
            const mask = (1n << BigInt(bits.length)) - 1n;
            const signed = unsigned > (mask >> 1n) ? unsigned - mask - 1n : unsigned;
            if (signed < 0n) {
                return `${val} (${signed})`;
            }
        }
        return String(val);
    }

    // For wide values, show hex
    const hex = BigInt('0b' + bits).toString(16);
    if (hex.length > maxLen) {
        return '0x' + hex.slice(0, maxLen - 3) + '...';
    }
    return '0x' + hex;
}

/**
 * Find the bad property nid corresponding to a bad index.
 * Bad properties are numbered 0, 1, 2, ... in declaration order.
 */
function badIndexToNid(modelNodes, badIndex) {
    let count = 0;
    for (const [nid, node] of modelNodes) {
        if (node.op === 'bad') {
            if (count === badIndex) return nid;
            count++;
        }
    }
    return null;
}
