/**
 * BTOR2 Graph Visualization — Cytoscape.js integration.
 */

// Category → color mapping (dark theme)
const CATEGORY_COLORS = {
    sort:       { bg: '#4a4a5a', border: '#6a6a7a', text: '#aaa' },
    constant:   { bg: '#3a4a5a', border: '#5a7a9a', text: '#8ab4f8' },
    state:      { bg: '#2a5a3a', border: '#4a9a5a', text: '#81c995' },
    input:      { bg: '#3a4a6a', border: '#5a7aaa', text: '#93b4e8' },
    memory:     { bg: '#5a4a2a', border: '#9a7a3a', text: '#f0c674' },
    logic:      { bg: '#3a3a3a', border: '#6a6a6a', text: '#d4d4d4' },
    bad:        { bg: '#6a2a2a', border: '#cc4444', text: '#ff8888' },
    constraint: { bg: '#5a5a2a', border: '#aaaa3a', text: '#e8e070' },
};

/**
 * Build a short display label for a node.
 */
function nodeLabel(node) {
    if (node.op === 'sort') {
        if (node.extra.sortKind === 'bitvec') return `bv${node.extra.width}`;
        return `array`;
    }
    if (node.op === 'constd' || node.op === 'consth' || node.op === 'const') {
        return node.name || node.op;
    }
    if (node.name) return `${node.op}\n${node.name}`;
    return node.op;
}

/**
 * Create Cytoscape elements (nodes + edges) from parsed BTOR2 data.
 * @param {Map} nodes - Parsed node map from parser
 * @param {object} filter - { hideSorts, hideConstants, hideWriteChains }
 */
function buildElements(nodes, filter = {}) {
    const elements = [];
    const visibleNids = new Set();

    // Determine visible nodes
    for (const [nid, node] of nodes) {
        if (filter.hideSorts && node.category === 'sort') continue;
        if (filter.hideConstants && node.category === 'constant') continue;
        visibleNids.add(nid);
    }

    // Create Cytoscape nodes
    for (const nid of visibleNids) {
        const node = nodes.get(nid);
        const colors = CATEGORY_COLORS[node.category] || CATEGORY_COLORS.logic;

        elements.push({
            group: 'nodes',
            data: {
                id: String(nid),
                label: nodeLabel(node),
                category: node.category,
                op: node.op,
                nid: nid,
                bgColor: colors.bg,
                borderColor: colors.border,
                textColor: colors.text,
            },
        });
    }

    // Create edges (from operand → this node)
    for (const nid of visibleNids) {
        const node = nodes.get(nid);
        for (let i = 0; i < node.operands.length; i++) {
            const opNid = node.operands[i];
            if (visibleNids.has(opNid)) {
                elements.push({
                    group: 'edges',
                    data: {
                        id: `e${opNid}-${nid}-${i}`,
                        source: String(opNid),
                        target: String(nid),
                    },
                });
            }
        }
    }

    return elements;
}

/**
 * Initialize Cytoscape instance with BTOR2 elements.
 */
function initGraph(container, elements) {
    const cy = cytoscape({
        container: container,
        elements: elements,
        style: [
            {
                selector: 'node',
                style: {
                    'label': 'data(label)',
                    'text-valign': 'center',
                    'text-halign': 'center',
                    'font-size': '9px',
                    'font-family': '"JetBrains Mono", "Fira Code", monospace',
                    'color': 'data(textColor)',
                    'background-color': 'data(bgColor)',
                    'border-color': 'data(borderColor)',
                    'border-width': 2,
                    'width': 'label',
                    'height': 'label',
                    'padding': '8px',
                    'shape': 'roundrectangle',
                    'text-wrap': 'wrap',
                    'text-max-width': '120px',
                },
            },
            {
                selector: 'node[category="bad"]',
                style: {
                    'border-width': 3,
                    'font-weight': 'bold',
                    'font-size': '11px',
                },
            },
            {
                selector: 'node[category="state"]',
                style: {
                    'border-width': 2,
                    'shape': 'roundrectangle',
                },
            },
            {
                selector: 'node[category="sort"]',
                style: {
                    'font-size': '8px',
                    'opacity': 0.6,
                },
            },
            {
                selector: 'edge',
                style: {
                    'width': 1,
                    'line-color': '#555',
                    'target-arrow-color': '#555',
                    'target-arrow-shape': 'triangle',
                    'curve-style': 'bezier',
                    'arrow-scale': 0.6,
                },
            },
            {
                selector: '.highlighted',
                style: {
                    'border-color': '#ff6600',
                    'border-width': 4,
                    'z-index': 999,
                },
            },
            {
                selector: '.cone',
                style: {
                    'border-color': '#ff4444',
                    'border-width': 3,
                    'opacity': 1,
                },
            },
            {
                selector: '.cone-edge',
                style: {
                    'line-color': '#ff4444',
                    'target-arrow-color': '#ff4444',
                    'width': 2,
                },
            },
            {
                selector: '.dimmed',
                style: {
                    'opacity': 0.15,
                },
            },
            {
                selector: '.neighbor',
                style: {
                    'border-color': '#ffaa00',
                    'border-width': 3,
                },
            },
        ],
        layout: { name: 'preset' }, // We'll run layout after
        wheelSensitivity: 0.3,
        minZoom: 0.05,
        maxZoom: 3,
    });

    return cy;
}

/**
 * Run the dagre layout on the Cytoscape instance.
 */
function runLayout(cy) {
    const layout = cy.layout({
        name: 'dagre',
        rankDir: 'BT',       // bottom to top (inputs at bottom, properties at top)
        nodeSep: 25,
        rankSep: 40,
        edgeSep: 10,
        ranker: 'tight-tree',
        animate: false,
    });
    layout.run();
}

/**
 * Highlight the cone of influence for a given node.
 */
function highlightCone(cy, parsedNodes, nid) {
    // Clear previous
    cy.elements().removeClass('cone cone-edge dimmed');

    const cone = getConeOfInfluence(parsedNodes, nid);

    // Dim everything
    cy.elements().addClass('dimmed');

    // Highlight cone nodes
    for (const coneNid of cone) {
        const node = cy.getElementById(String(coneNid));
        if (node.length) {
            node.removeClass('dimmed').addClass('cone');
        }
    }

    // Highlight edges within the cone
    cy.edges().forEach(edge => {
        const src = parseInt(edge.data('source'));
        const tgt = parseInt(edge.data('target'));
        if (cone.has(src) && cone.has(tgt)) {
            edge.removeClass('dimmed').addClass('cone-edge');
        }
    });
}

/**
 * Clear all highlights.
 */
function clearHighlights(cy) {
    cy.elements().removeClass('cone cone-edge dimmed highlighted neighbor');
}
