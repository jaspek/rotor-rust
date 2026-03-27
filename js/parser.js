/**
 * BTOR2 Parser — converts BTOR2 text into structured node/edge objects.
 */

// Operations that have no sort_nid (sort is implicit or absent)
const NO_SORT_OPS = new Set(['bad', 'constraint', 'justice', 'fair']);
// Operations with a named symbolic argument
const NAMED_OPS = new Set(['state', 'input', 'bad', 'constraint']);
// Unary operations
const UNARY_OPS = new Set(['not', 'inc', 'dec', 'neg', 'redand', 'redor', 'redxor']);
// Binary operations
const BINARY_OPS = new Set([
    'add', 'sub', 'mul', 'udiv', 'sdiv', 'urem', 'srem',
    'and', 'or', 'xor', 'nand', 'nor', 'xnor',
    'sll', 'srl', 'sra', 'rol', 'ror',
    'eq', 'neq', 'slt', 'ult', 'slte', 'ulte', 'sgt', 'ugt', 'sgte', 'ugte',
    'implies', 'iff',
    'saddo', 'uaddo', 'ssubo', 'usubo', 'smulo', 'umulo', 'sdivo',
]);

/**
 * Categorize a BTOR2 operation for visual styling.
 */
function categorize(op) {
    if (op === 'sort') return 'sort';
    if (op === 'constd' || op === 'consth' || op === 'const' || op === 'one' || op === 'ones' || op === 'zero') return 'constant';
    if (op === 'state' || op === 'init' || op === 'next') return 'state';
    if (op === 'input') return 'input';
    if (op === 'read' || op === 'write') return 'memory';
    if (op === 'bad') return 'bad';
    if (op === 'constraint') return 'constraint';
    return 'logic';
}

/**
 * Parse a BTOR2 file into a list of nodes.
 * @param {string} text - Raw BTOR2 file content
 * @returns {{ nodes: Map<number, object>, errors: string[] }}
 */
function parseBtor2(text) {
    const nodes = new Map();
    const errors = [];

    const lines = text.split('\n');
    for (let lineNum = 0; lineNum < lines.length; lineNum++) {
        let line = lines[lineNum].trim();
        if (!line || line.startsWith(';')) continue;

        // Extract comment
        let comment = null;
        const semicolonIdx = line.indexOf(' ; ');
        if (semicolonIdx >= 0) {
            comment = line.slice(semicolonIdx + 3).trim();
            line = line.slice(0, semicolonIdx).trim();
        } else if (line.endsWith(';')) {
            line = line.slice(0, -1).trim();
        }

        const tokens = line.split(/\s+/);
        if (tokens.length < 2) continue;

        const nid = parseInt(tokens[0], 10);
        if (isNaN(nid)) continue;

        const op = tokens[1];
        let sortNid = null;
        let operands = [];
        let name = null;
        let extra = {};

        try {
            if (op === 'sort') {
                // sort bitvec N  or  sort array idx_sort val_sort
                extra.sortKind = tokens[2];
                if (tokens[2] === 'bitvec') {
                    extra.width = parseInt(tokens[3], 10);
                } else if (tokens[2] === 'array') {
                    operands = [parseInt(tokens[3], 10), parseInt(tokens[4], 10)];
                    extra.indexSort = operands[0];
                    extra.elementSort = operands[1];
                }
            } else if (op === 'constd') {
                sortNid = parseInt(tokens[2], 10);
                extra.value = tokens[3];
                name = tokens[3];
            } else if (op === 'consth') {
                sortNid = parseInt(tokens[2], 10);
                extra.value = '0x' + tokens[3];
                name = '0x' + tokens[3];
            } else if (op === 'const') {
                sortNid = parseInt(tokens[2], 10);
                extra.value = '0b' + tokens[3];
                name = '0b' + tokens[3];
            } else if (op === 'zero' || op === 'one' || op === 'ones') {
                sortNid = parseInt(tokens[2], 10);
                name = op;
            } else if (op === 'state' || op === 'input') {
                sortNid = parseInt(tokens[2], 10);
                if (tokens.length > 3) name = tokens.slice(3).join(' ');
            } else if (op === 'init' || op === 'next') {
                sortNid = parseInt(tokens[2], 10);
                operands = [parseInt(tokens[3], 10), parseInt(tokens[4], 10)];
            } else if (op === 'bad' || op === 'constraint') {
                operands = [parseInt(tokens[2], 10)];
                if (tokens.length > 3) name = tokens.slice(3).join(' ');
            } else if (op === 'sext' || op === 'uext') {
                sortNid = parseInt(tokens[2], 10);
                operands = [parseInt(tokens[3], 10)];
                extra.width = parseInt(tokens[4], 10);
            } else if (op === 'slice') {
                sortNid = parseInt(tokens[2], 10);
                operands = [parseInt(tokens[3], 10)];
                extra.upper = parseInt(tokens[4], 10);
                extra.lower = parseInt(tokens[5], 10);
            } else if (op === 'ite') {
                sortNid = parseInt(tokens[2], 10);
                operands = [parseInt(tokens[3], 10), parseInt(tokens[4], 10), parseInt(tokens[5], 10)];
            } else if (op === 'write') {
                sortNid = parseInt(tokens[2], 10);
                operands = [parseInt(tokens[3], 10), parseInt(tokens[4], 10), parseInt(tokens[5], 10)];
            } else if (op === 'read' || op === 'concat') {
                sortNid = parseInt(tokens[2], 10);
                operands = [parseInt(tokens[3], 10), parseInt(tokens[4], 10)];
            } else if (UNARY_OPS.has(op)) {
                sortNid = parseInt(tokens[2], 10);
                operands = [parseInt(tokens[3], 10)];
            } else if (BINARY_OPS.has(op)) {
                sortNid = parseInt(tokens[2], 10);
                operands = [parseInt(tokens[3], 10), parseInt(tokens[4], 10)];
            } else {
                // Unknown op — best effort: treat remaining tokens as operands
                sortNid = parseInt(tokens[2], 10);
                for (let i = 3; i < tokens.length; i++) {
                    const v = parseInt(tokens[i], 10);
                    if (!isNaN(v)) operands.push(v);
                }
            }
        } catch (e) {
            errors.push(`Line ${lineNum + 1}: Parse error — ${e.message}`);
            continue;
        }

        const category = categorize(op);

        nodes.set(nid, {
            nid,
            op,
            sortNid,
            operands,
            name,
            comment,
            category,
            extra,
            rawLine: lines[lineNum].trim(),
            dependents: [], // filled in post-processing
        });
    }

    // Build reverse dependency map
    for (const [nid, node] of nodes) {
        for (const opNid of node.operands) {
            const dep = nodes.get(opNid);
            if (dep) dep.dependents.push(nid);
        }
        // Sort reference is also a dependency (but we don't draw edges for it)
    }

    return { nodes, errors };
}

/**
 * Compute statistics about a parsed BTOR2 model.
 */
function computeStats(nodes) {
    const stats = {
        total: nodes.size,
        byCategory: {},
        byOp: {},
        badCount: 0,
        constraintCount: 0,
        stateCount: 0,
        inputCount: 0,
    };

    for (const [, node] of nodes) {
        stats.byCategory[node.category] = (stats.byCategory[node.category] || 0) + 1;
        stats.byOp[node.op] = (stats.byOp[node.op] || 0) + 1;
        if (node.op === 'bad') stats.badCount++;
        if (node.op === 'constraint') stats.constraintCount++;
        if (node.op === 'state') stats.stateCount++;
        if (node.op === 'input') stats.inputCount++;
    }

    return stats;
}

/**
 * Get the cone of influence (all transitive dependencies) for a given node.
 */
function getConeOfInfluence(nodes, startNid, maxDepth) {
    if (maxDepth === undefined || maxDepth === null) maxDepth = Infinity;
    const visited = new Set();
    const queue = [[startNid, 0]];
    while (queue.length > 0) {
        const [nid, depth] = queue.shift();
        if (visited.has(nid)) continue;
        visited.add(nid);
        if (depth < maxDepth) {
            const node = nodes.get(nid);
            if (node) {
                for (const opNid of node.operands) {
                    if (!visited.has(opNid)) queue.push([opNid, depth + 1]);
                }
            }
        }
    }
    return visited;
}
