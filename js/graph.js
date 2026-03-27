/**
 * BTOR2 Graph Visualization — Cytoscape.js integration.
 * Inspired by beatle (Diller, 2022) with subgraph views,
 * collapse/expand, and clumping.
 */

// Category → color mapping (refined dark theme)
const CATEGORY_COLORS = {
    sort:       { bg: '#3d3d50', border: '#5a5a70', text: '#9999bb' },
    constant:   { bg: '#2a3a50', border: '#4a7aaa', text: '#7ab4f8' },
    state:      { bg: '#1e4a2e', border: '#3a9a5a', text: '#6fdc8c' },
    input:      { bg: '#2a3a5a', border: '#5a8abb', text: '#82b4e8' },
    memory:     { bg: '#4a3a1a', border: '#aa8a3a', text: '#f0c674' },
    logic:      { bg: '#2e2e36', border: '#555566', text: '#c8c8d8' },
    bad:        { bg: '#5a1a1a', border: '#dd4444', text: '#ff7777' },
    constraint: { bg: '#4a4a1a', border: '#bbbb3a', text: '#e8e070' },
    clump:      { bg: '#3a2a50', border: '#8a6ab0', text: '#c8a0f0' },
};

// Shape per category for visual differentiation
const CATEGORY_SHAPES = {
    sort:       'ellipse',
    constant:   'diamond',
    state:      'roundrectangle',
    input:      'barrel',
    memory:     'pentagon',
    logic:      'roundrectangle',
    bad:        'octagon',
    constraint: 'hexagon',
    clump:      'roundrectangle',
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
 * Supports subgraph mode (only show cone of influence) and clumping.
 */
function buildElements(nodes, filter = {}, options = {}) {
    const elements = [];
    const visibleNids = new Set();

    // If subgraph root is specified, only show its cone of influence
    let allowedNids = null;
    if (options.subgraphRoot) {
        allowedNids = getConeOfInfluence(nodes, options.subgraphRoot);
    }

    // Determine visible nodes
    for (const [nid, node] of nodes) {
        if (filter.hideSorts && node.category === 'sort') continue;
        if (filter.hideConstants && node.category === 'constant') continue;
        if (allowedNids && !allowedNids.has(nid)) continue;

        // Skip collapsed descendants
        if (options.collapsedNodes && isCollapsedDescendant(nodes, nid, options.collapsedNodes)) continue;

        visibleNids.add(nid);
    }

    // Apply clumping if enabled
    if (options.clumpCategories && options.clumpCategories.size > 0) {
        return buildClumpedElements(nodes, visibleNids, options.clumpCategories, options.subgraphRoot);
    }

    // Create Cytoscape nodes
    for (const nid of visibleNids) {
        const node = nodes.get(nid);
        const colors = CATEGORY_COLORS[node.category] || CATEGORY_COLORS.logic;
        const shape = CATEGORY_SHAPES[node.category] || 'roundrectangle';

        // Check if this node has collapsed children
        const isCollapsed = options.collapsedNodes && options.collapsedNodes.has(nid);
        const hiddenCount = isCollapsed ? countHiddenDescendants(nodes, nid, visibleNids, options.collapsedNodes) : 0;

        let label = nodeLabel(node);
        if (isCollapsed && hiddenCount > 0) {
            label += `\n[+${hiddenCount}]`;
        }

        elements.push({
            group: 'nodes',
            data: {
                id: String(nid),
                label: label,
                category: node.category,
                op: node.op,
                nid: nid,
                bgColor: colors.bg,
                borderColor: colors.border,
                textColor: colors.text,
                nodeShape: shape,
                isCollapsed: isCollapsed,
            },
        });
    }

    // Create edges
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
 * Build elements with clumping: group nodes of specified categories into meta-nodes.
 */
function buildClumpedElements(nodes, visibleNids, clumpCategories, subgraphRoot) {
    const elements = [];
    const clumps = new Map(); // category → Set<nid>
    const clumpedNids = new Set();

    // Group nodes by clump category
    for (const nid of visibleNids) {
        const node = nodes.get(nid);
        if (clumpCategories.has(node.category)) {
            if (!clumps.has(node.category)) clumps.set(node.category, new Set());
            clumps.get(node.category).add(nid);
            clumpedNids.add(nid);
        }
    }

    // Create non-clumped nodes
    for (const nid of visibleNids) {
        if (clumpedNids.has(nid)) continue;
        const node = nodes.get(nid);
        const colors = CATEGORY_COLORS[node.category] || CATEGORY_COLORS.logic;
        const shape = CATEGORY_SHAPES[node.category] || 'roundrectangle';
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
                nodeShape: shape,
            },
        });
    }

    // Create clump meta-nodes
    for (const [cat, nids] of clumps) {
        const colors = CATEGORY_COLORS.clump;
        const clumpId = `clump-${cat}`;
        elements.push({
            group: 'nodes',
            data: {
                id: clumpId,
                label: `${cat}\n(${nids.size} nodes)`,
                category: 'clump',
                op: 'clump',
                nid: clumpId,
                bgColor: colors.bg,
                borderColor: colors.border,
                textColor: colors.text,
                nodeShape: 'roundrectangle',
                clumpCategory: cat,
                clumpCount: nids.size,
            },
        });
    }

    // Create edges — redirect clumped node edges to clump meta-nodes
    const edgeSet = new Set(); // prevent duplicate edges
    for (const nid of visibleNids) {
        const node = nodes.get(nid);
        for (let i = 0; i < node.operands.length; i++) {
            const opNid = node.operands[i];
            if (!visibleNids.has(opNid)) continue;

            let src = String(opNid);
            let tgt = String(nid);

            // Replace with clump ID if clumped
            const srcNode = nodes.get(opNid);
            const tgtNode = nodes.get(nid);
            if (srcNode && clumpCategories.has(srcNode.category)) {
                src = `clump-${srcNode.category}`;
            }
            if (tgtNode && clumpCategories.has(tgtNode.category)) {
                tgt = `clump-${tgtNode.category}`;
            }

            if (src === tgt) continue; // skip self-loops within same clump

            const edgeKey = `${src}->${tgt}`;
            if (edgeSet.has(edgeKey)) continue;
            edgeSet.add(edgeKey);

            // Dotted line for clump-to-clump edges (multiple consolidated edges)
            const isClumpEdge = src.startsWith('clump-') || tgt.startsWith('clump-');

            elements.push({
                group: 'edges',
                data: {
                    id: `e-${edgeKey}`,
                    source: src,
                    target: tgt,
                    isClumpEdge: isClumpEdge,
                },
            });
        }
    }

    return elements;
}

/**
 * Check if a node is a descendant of any collapsed node.
 */
function isCollapsedDescendant(nodes, nid, collapsedNodes) {
    const node = nodes.get(nid);
    if (!node) return false;
    // Walk up the dependent chain — if any ancestor is collapsed, hide this node
    // More efficient: check if any of the node's operands (ancestors in data flow) are collapsed
    // and the path from a collapsed node leads to this node
    for (const opNid of node.operands) {
        if (collapsedNodes.has(opNid)) return true;
        // Don't recurse deeply — just check immediate parents
    }
    return false;
}

/**
 * Count how many descendants are hidden by collapsing a node.
 */
function countHiddenDescendants(nodes, nid, visibleNids, collapsedNodes) {
    // Count operands that would become hidden
    const node = nodes.get(nid);
    if (!node) return 0;
    let count = 0;
    const visited = new Set();
    const queue = [...node.operands];
    while (queue.length > 0) {
        const opNid = queue.pop();
        if (visited.has(opNid)) continue;
        visited.add(opNid);
        const opNode = nodes.get(opNid);
        if (!opNode) continue;
        count++;
        // Only recurse if this operand isn't also collapsed
        if (!collapsedNodes.has(opNid)) {
            queue.push(...opNode.operands);
        }
    }
    return count;
}

/**
 * Initialize Cytoscape instance.
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
                    'width': 60,
                    'height': 30,
                    'padding': '8px',
                    'shape': 'data(nodeShape)',
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
                },
            },
            {
                selector: 'node[category="sort"]',
                style: {
                    'font-size': '8px',
                    'opacity': 0.5,
                },
            },
            {
                selector: 'node[?isCollapsed]',
                style: {
                    'border-style': 'double',
                    'border-width': 4,
                    'font-weight': 'bold',
                },
            },
            {
                selector: 'node[category="clump"]',
                style: {
                    'font-size': '11px',
                    'border-style': 'dashed',
                    'border-width': 3,
                    'padding': '14px',
                    'shape': 'roundrectangle',
                },
            },
            {
                selector: 'edge',
                style: {
                    'width': 1.5,
                    'line-color': '#444466',
                    'target-arrow-color': '#444466',
                    'target-arrow-shape': 'triangle',
                    'curve-style': 'bezier',
                    'arrow-scale': 0.7,
                },
            },
            {
                selector: 'edge[?isClumpEdge]',
                style: {
                    'line-style': 'dashed',
                    'line-dash-pattern': [6, 3],
                    'width': 2,
                    'line-color': '#7a6aaa',
                    'target-arrow-color': '#7a6aaa',
                },
            },
            {
                selector: '.highlighted',
                style: {
                    'border-color': '#ff8800',
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
                    'opacity': 0.12,
                },
            },
            {
                selector: '.neighbor',
                style: {
                    'border-color': '#ffaa00',
                    'border-width': 3,
                },
            },
            {
                selector: '.longest-path',
                style: {
                    'border-color': '#ff44ff',
                    'border-width': 4,
                    'z-index': 500,
                },
            },
            {
                selector: '.longest-path-edge',
                style: {
                    'line-color': '#ff44ff',
                    'target-arrow-color': '#ff44ff',
                    'width': 3,
                },
            },
            // ── Witness trace styles ──
            {
                selector: '.witness-inactive',
                style: {
                    'opacity': 0.15,
                },
            },
            {
                selector: '.witness-active',
                style: {
                    'opacity': 1,
                    'z-index': 100,
                },
            },
            {
                selector: '.witness-state',
                style: {
                    'border-color': '#00ff88',
                    'border-width': 4,
                    'label': 'data(witnessLabel)',
                    'font-size': '10px',
                },
            },
            {
                selector: '.witness-input',
                style: {
                    'border-color': '#44aaff',
                    'border-width': 4,
                    'label': 'data(witnessLabel)',
                    'font-size': '10px',
                },
            },
            {
                selector: '.witness-bad',
                style: {
                    'border-color': '#ff2222',
                    'border-width': 5,
                    'background-color': '#4a1a1a',
                    'label': 'data(witnessLabel)',
                    'font-size': '11px',
                    'font-weight': 'bold',
                },
            },
        ],
        layout: { name: 'preset' },
        wheelSensitivity: 1,
        minZoom: 0.05,
        maxZoom: 3,
    });

    return cy;
}

/**
 * Run the dagre layout.
 */
function runLayout(cy) {
    const layout = cy.layout({
        name: 'dagre',
        rankDir: 'BT',
        nodeSep: 30,
        rankSep: 45,
        edgeSep: 12,
        ranker: 'tight-tree',
        animate: false,
    });
    layout.run();
}

/**
 * Highlight the cone of influence for a given node.
 */
function highlightCone(cy, parsedNodes, nid) {
    cy.elements().removeClass('cone cone-edge dimmed');
    const cone = getConeOfInfluence(parsedNodes, nid);
    cy.elements().addClass('dimmed');
    for (const coneNid of cone) {
        const node = cy.getElementById(String(coneNid));
        if (node.length) {
            node.removeClass('dimmed').addClass('cone');
        }
    }
    cy.edges().forEach(edge => {
        const src = parseInt(edge.data('source'));
        const tgt = parseInt(edge.data('target'));
        if (cone.has(src) && cone.has(tgt)) {
            edge.removeClass('dimmed').addClass('cone-edge');
        }
    });
}

/**
 * Highlight the longest path from a root node.
 */
function highlightLongestPath(cy, parsedNodes, rootNid) {
    cy.elements().removeClass('longest-path longest-path-edge dimmed');
    cy.elements().addClass('dimmed');

    // Find longest path via DFS with memoization
    const memo = new Map();
    function longestFrom(nid) {
        if (memo.has(nid)) return memo.get(nid);
        const node = parsedNodes.get(nid);
        if (!node || node.operands.length === 0) {
            memo.set(nid, [nid]);
            return [nid];
        }
        let best = [nid];
        for (const opNid of node.operands) {
            const sub = longestFrom(opNid);
            if (sub.length + 1 > best.length) {
                best = [nid, ...sub];
            }
        }
        memo.set(nid, best);
        return best;
    }

    const path = longestFrom(rootNid);

    // Highlight path nodes
    for (const nid of path) {
        const cyNode = cy.getElementById(String(nid));
        if (cyNode.length) {
            cyNode.removeClass('dimmed').addClass('longest-path');
        }
    }

    // Highlight path edges
    for (let i = 0; i < path.length - 1; i++) {
        cy.edges().forEach(edge => {
            const src = parseInt(edge.data('source'));
            const tgt = parseInt(edge.data('target'));
            if ((src === path[i + 1] && tgt === path[i]) ||
                (src === path[i] && tgt === path[i + 1])) {
                edge.removeClass('dimmed').addClass('longest-path-edge');
            }
        });
    }

    return path;
}

/**
 * Clear all highlights.
 */
function clearHighlights(cy) {
    cy.elements().removeClass('cone cone-edge dimmed highlighted neighbor longest-path longest-path-edge');
}
