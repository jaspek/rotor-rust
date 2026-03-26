/**
 * BTOR2 Visualizer — Application logic.
 */

let cy = null;
let parsedNodes = null;
let currentFilter = { hideSorts: true, hideConstants: false };

// ============ UI References ============

const graphContainer = document.getElementById('cy');
const fileInput = document.getElementById('file-input');
const uploadBtn = document.getElementById('btn-upload');
const pasteBtn = document.getElementById('btn-paste');
const exampleBtn = document.getElementById('btn-example');
const filterSorts = document.getElementById('filter-sorts');
const filterConsts = document.getElementById('filter-consts');
const searchInput = document.getElementById('search-input');
const searchBtn = document.getElementById('btn-search');
const nodeList = document.getElementById('node-list');
const detailPanel = document.getElementById('detail-content');
const statsPanel = document.getElementById('stats-content');
const statusBar = document.getElementById('status-bar');
const pasteOverlay = document.getElementById('paste-overlay');
const pasteTextarea = document.getElementById('paste-textarea');
const pasteConfirm = document.getElementById('btn-paste-confirm');
const pasteCancel = document.getElementById('btn-paste-cancel');
const btnZoomIn = document.getElementById('btn-zoom-in');
const btnZoomOut = document.getElementById('btn-zoom-out');
const btnFit = document.getElementById('btn-fit');
const btnClearHighlight = document.getElementById('btn-clear-highlight');

// ============ Event Handlers ============

uploadBtn.addEventListener('click', () => fileInput.click());
fileInput.addEventListener('change', handleFileUpload);
pasteBtn.addEventListener('click', showPasteOverlay);
pasteConfirm.addEventListener('click', handlePaste);
pasteCancel.addEventListener('click', hidePasteOverlay);
exampleBtn.addEventListener('click', loadExample);
filterSorts.addEventListener('change', applyFilters);
filterConsts.addEventListener('change', applyFilters);
searchBtn.addEventListener('click', doSearch);
searchInput.addEventListener('keydown', e => { if (e.key === 'Enter') doSearch(); });
btnZoomIn.addEventListener('click', () => cy && cy.zoom(cy.zoom() * 1.3));
btnZoomOut.addEventListener('click', () => cy && cy.zoom(cy.zoom() / 1.3));
btnFit.addEventListener('click', () => cy && cy.fit(null, 30));
btnClearHighlight.addEventListener('click', () => { if (cy) clearHighlights(cy); });

// ============ File Handling ============

function handleFileUpload(e) {
    const file = e.target.files[0];
    if (!file) return;
    setStatus(`Loading ${file.name}...`);
    const reader = new FileReader();
    reader.onload = ev => {
        loadBtor2(ev.target.result, file.name);
    };
    reader.readAsText(file);
    fileInput.value = '';
}

function showPasteOverlay() {
    pasteOverlay.classList.remove('hidden');
    pasteTextarea.focus();
}

function hidePasteOverlay() {
    pasteOverlay.classList.add('hidden');
    pasteTextarea.value = '';
}

function handlePaste() {
    const text = pasteTextarea.value.trim();
    if (!text) return;
    hidePasteOverlay();
    loadBtor2(text, 'pasted input');
}

async function loadExample() {
    setStatus('Loading example...');
    try {
        const resp = await fetch('./examples/simple-assignment-1-35.btor2');
        const text = await resp.text();
        loadBtor2(text, 'simple-assignment-1-35.btor2');
    } catch (err) {
        setStatus('Failed to load example: ' + err.message);
    }
}

// ============ Core Logic ============

function loadBtor2(text, filename) {
    setStatus(`Parsing ${filename}...`);

    const result = parseBtor2(text);
    parsedNodes = result.nodes;

    if (result.errors.length > 0) {
        console.warn('Parse errors:', result.errors);
    }

    const stats = computeStats(parsedNodes);
    renderStats(stats);
    renderNodeList(parsedNodes);
    renderGraph();

    setStatus(`Loaded ${filename}: ${stats.total} nodes, ${stats.badCount} bad properties, ${stats.stateCount} states`);
}

function renderGraph() {
    if (!parsedNodes) return;

    currentFilter.hideSorts = filterSorts.checked;
    currentFilter.hideConstants = filterConsts.checked;

    const elements = buildElements(parsedNodes, currentFilter);

    if (cy) cy.destroy();

    cy = initGraph(graphContainer, elements);
    setStatus('Running layout...');

    // Run layout asynchronously for UI responsiveness
    setTimeout(() => {
        runLayout(cy);
        cy.fit(null, 30);
        setupGraphInteraction();
        setStatus('Ready');
    }, 50);
}

function setupGraphInteraction() {
    // Click node → show detail
    cy.on('tap', 'node', function (evt) {
        const nid = parseInt(evt.target.data('nid'));
        showNodeDetail(nid);

        // Highlight
        cy.elements().removeClass('highlighted neighbor');
        evt.target.addClass('highlighted');
        evt.target.neighborhood().addClass('neighbor');
    });

    // Right-click node → show cone of influence
    cy.on('cxttap', 'node', function (evt) {
        const nid = parseInt(evt.target.data('nid'));
        highlightCone(cy, parsedNodes, nid);
        setStatus(`Showing cone of influence for node ${nid}`);
    });

    // Click background → clear selection
    cy.on('tap', function (evt) {
        if (evt.target === cy) {
            detailPanel.innerHTML = '<p class="placeholder">Click a node to see details</p>';
            cy.elements().removeClass('highlighted neighbor');
        }
    });
}

// ============ Detail Panel ============

function showNodeDetail(nid) {
    const node = parsedNodes.get(nid);
    if (!node) return;

    const sortInfo = node.sortNid ? getSortDescription(node.sortNid) : '';

    let html = `
        <div class="detail-header">
            <span class="detail-nid">#${node.nid}</span>
            <span class="detail-op ${node.category}">${node.op}</span>
        </div>
    `;

    if (node.name) {
        html += `<div class="detail-row"><span class="label">Name:</span> ${escHtml(node.name)}</div>`;
    }

    if (sortInfo) {
        html += `<div class="detail-row"><span class="label">Sort:</span> ${sortInfo}</div>`;
    }

    if (node.comment) {
        html += `<div class="detail-row"><span class="label">Comment:</span> ${escHtml(node.comment)}</div>`;
    }

    // Extra info
    if (node.extra.width !== undefined && node.op !== 'sort') {
        html += `<div class="detail-row"><span class="label">Width:</span> ${node.extra.width}</div>`;
    }
    if (node.extra.upper !== undefined) {
        html += `<div class="detail-row"><span class="label">Slice:</span> [${node.extra.upper}:${node.extra.lower}]</div>`;
    }
    if (node.extra.value !== undefined) {
        html += `<div class="detail-row"><span class="label">Value:</span> ${escHtml(String(node.extra.value))}</div>`;
    }

    // Operands
    if (node.operands.length > 0) {
        html += `<div class="detail-section">Operands</div>`;
        html += `<div class="detail-links">`;
        for (const opNid of node.operands) {
            const opNode = parsedNodes.get(opNid);
            const opLabel = opNode ? `#${opNid} (${opNode.op}${opNode.name ? ': ' + opNode.name : ''})` : `#${opNid}`;
            html += `<a class="node-link" data-nid="${opNid}">${escHtml(opLabel)}</a>`;
        }
        html += `</div>`;
    }

    // Dependents
    if (node.dependents.length > 0) {
        html += `<div class="detail-section">Used by (${node.dependents.length})</div>`;
        html += `<div class="detail-links">`;
        const maxShow = 20;
        const show = node.dependents.slice(0, maxShow);
        for (const depNid of show) {
            const depNode = parsedNodes.get(depNid);
            const depLabel = depNode ? `#${depNid} (${depNode.op})` : `#${depNid}`;
            html += `<a class="node-link" data-nid="${depNid}">${escHtml(depLabel)}</a>`;
        }
        if (node.dependents.length > maxShow) {
            html += `<span class="text-muted">... and ${node.dependents.length - maxShow} more</span>`;
        }
        html += `</div>`;
    }

    // Raw line
    html += `<div class="detail-section">Raw BTOR2</div>`;
    html += `<pre class="raw-line">${escHtml(node.rawLine)}</pre>`;

    detailPanel.innerHTML = html;

    // Add click handlers for node links
    detailPanel.querySelectorAll('.node-link').forEach(link => {
        link.addEventListener('click', () => {
            const targetNid = parseInt(link.dataset.nid);
            navigateToNode(targetNid);
        });
    });
}

function navigateToNode(nid) {
    if (!cy) return;
    const cyNode = cy.getElementById(String(nid));
    if (cyNode.length) {
        cy.animate({
            center: { eles: cyNode },
            zoom: 1.5,
        }, { duration: 300 });
        cy.elements().removeClass('highlighted neighbor');
        cyNode.addClass('highlighted');
        cyNode.neighborhood().addClass('neighbor');
        showNodeDetail(nid);
    }
}

function getSortDescription(sortNid) {
    const sortNode = parsedNodes.get(sortNid);
    if (!sortNode) return `sort #${sortNid}`;
    if (sortNode.extra.sortKind === 'bitvec') {
        return `bitvec[${sortNode.extra.width}]`;
    } else if (sortNode.extra.sortKind === 'array') {
        const idxSort = getSortDescription(sortNode.extra.indexSort);
        const elemSort = getSortDescription(sortNode.extra.elementSort);
        return `array[${idxSort} → ${elemSort}]`;
    }
    return `sort #${sortNid}`;
}

// ============ Node List ============

function renderNodeList(nodes) {
    let html = '';
    const categories = ['bad', 'constraint', 'state', 'input', 'memory', 'logic', 'constant', 'sort'];

    for (const cat of categories) {
        const catNodes = [];
        for (const [, node] of nodes) {
            if (node.category === cat) catNodes.push(node);
        }
        if (catNodes.length === 0) continue;

        html += `<div class="list-category">
            <div class="list-cat-header ${cat}">${cat} (${catNodes.length})</div>`;

        const maxShow = cat === 'bad' || cat === 'constraint' || cat === 'state' || cat === 'input' ? catNodes.length : 10;
        for (let i = 0; i < Math.min(catNodes.length, maxShow); i++) {
            const n = catNodes[i];
            const label = n.name || n.comment || n.op;
            html += `<a class="list-item node-link" data-nid="${n.nid}">
                <span class="list-nid">#${n.nid}</span> ${escHtml(label)}
            </a>`;
        }
        if (catNodes.length > maxShow) {
            html += `<span class="text-muted list-item">... ${catNodes.length - maxShow} more</span>`;
        }
        html += `</div>`;
    }

    nodeList.innerHTML = html;

    // Add click handlers
    nodeList.querySelectorAll('.node-link').forEach(link => {
        link.addEventListener('click', () => {
            navigateToNode(parseInt(link.dataset.nid));
        });
    });
}

// ============ Statistics ============

function renderStats(stats) {
    let html = `<div class="stat-row"><span>Total nodes:</span><span>${stats.total}</span></div>`;
    html += `<div class="stat-row"><span>Bad properties:</span><span class="bad">${stats.badCount}</span></div>`;
    html += `<div class="stat-row"><span>Constraints:</span><span>${stats.constraintCount}</span></div>`;
    html += `<div class="stat-row"><span>States:</span><span class="state">${stats.stateCount}</span></div>`;
    html += `<div class="stat-row"><span>Inputs:</span><span>${stats.inputCount}</span></div>`;

    html += `<div class="detail-section">By Operation</div>`;
    const sortedOps = Object.entries(stats.byOp).sort((a, b) => b[1] - a[1]);
    for (const [op, count] of sortedOps.slice(0, 15)) {
        html += `<div class="stat-row"><span>${op}:</span><span>${count}</span></div>`;
    }
    if (sortedOps.length > 15) {
        html += `<div class="text-muted">... ${sortedOps.length - 15} more op types</div>`;
    }

    statsPanel.innerHTML = html;
}

// ============ Search ============

function doSearch() {
    const query = searchInput.value.trim().toLowerCase();
    if (!query || !parsedNodes) return;

    const matches = [];
    for (const [nid, node] of parsedNodes) {
        if (String(nid) === query ||
            node.op.toLowerCase().includes(query) ||
            (node.name && node.name.toLowerCase().includes(query)) ||
            (node.comment && node.comment.toLowerCase().includes(query))) {
            matches.push(nid);
        }
    }

    if (matches.length === 0) {
        setStatus(`No matches for "${query}"`);
        return;
    }

    if (cy) {
        cy.elements().removeClass('highlighted neighbor');
        for (const nid of matches) {
            cy.getElementById(String(nid)).addClass('highlighted');
        }
        // Fit to matches
        const matchEles = cy.collection();
        for (const nid of matches) {
            matchEles.merge(cy.getElementById(String(nid)));
        }
        if (matchEles.length) cy.fit(matchEles, 50);
    }

    setStatus(`Found ${matches.length} matches for "${query}"`);
}

// ============ Filters ============

function applyFilters() {
    renderGraph();
}

// ============ Utilities ============

function setStatus(msg) {
    statusBar.textContent = msg;
}

function escHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

// ============ Init ============

setStatus('Ready — upload a BTOR2 file or load the example');
