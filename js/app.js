/**
 * BTOR2 Visualizer — Application logic.
 * Features: subgraph views, collapse/expand, clumping, longest path,
 * witness trace animation.
 */

let cy = null;
let parsedNodes = null;
let currentFilter = { hideSorts: true, hideConstants: false };
let subgraphRoot = null;
let maxDepth = Infinity;
let layoutMode = 'dagre';
let collapsedNodes = new Set();
let clumpCategories = new Set();
const player = new WitnessPlayer();

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
const selectRoot = document.getElementById('select-root');
const subgraphInfo = document.getElementById('subgraph-info');
const btnLongestPath = document.getElementById('btn-longest-path');
const depthControl = document.getElementById('depth-control');
const depthSlider = document.getElementById('depth-slider');
const depthValue = document.getElementById('depth-value');
const layoutDagre = document.getElementById('layout-dagre');
const layoutCose = document.getElementById('layout-cose');
const clumpHeader = document.getElementById('clump-header');
const clumpOptions = document.getElementById('clump-options');

// Witness UI
const witnessFileInput = document.getElementById('witness-file-input');
const btnWitnessUpload = document.getElementById('btn-witness-upload');
const btnWitnessPaste = document.getElementById('btn-witness-paste');
const witnessControls = document.getElementById('witness-controls');
const witnessStepEl = document.getElementById('witness-step');
const witnessTotalEl = document.getElementById('witness-total');
const btnWitnessStart = document.getElementById('btn-witness-start');
const btnWitnessBack = document.getElementById('btn-witness-back');
const btnWitnessPlay = document.getElementById('btn-witness-play');
const btnWitnessFwd = document.getElementById('btn-witness-fwd');
const btnWitnessEnd = document.getElementById('btn-witness-end');
const witnessSpeed = document.getElementById('witness-speed');
const witnessSpeedLabel = document.getElementById('witness-speed-label');
const btnWitnessClose = document.getElementById('btn-witness-close');
const witnessPasteOverlay = document.getElementById('witness-paste-overlay');
const witnessPasteTextarea = document.getElementById('witness-paste-textarea');
const btnWitnessPasteConfirm = document.getElementById('btn-witness-paste-confirm');
const btnWitnessPasteCancel = document.getElementById('btn-witness-paste-cancel');
const btnWitnessExample = document.getElementById('btn-witness-example');

// Trace panel
const tracePanel = document.getElementById('trace-panel');
const traceContent = document.getElementById('trace-content');

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
btnClearHighlight.addEventListener('click', () => {
    if (cy) clearHighlights(cy);
    if (player.loaded) player.goToStep(player.currentStep);
});

// Export buttons
const btnExportPng = document.getElementById('btn-export-png');
const btnExportSvg = document.getElementById('btn-export-svg');
btnExportPng.addEventListener('click', () => {
    if (!cy) return;
    const png = cy.png({ full: true, scale: 2, bg: '#12131a' });
    const a = document.createElement('a');
    a.href = png;
    a.download = 'btor2-graph.png';
    a.click();
    setStatus('Exported PNG');
});
btnExportSvg.addEventListener('click', () => {
    if (!cy) return;
    if (typeof cy.svg !== 'function') {
        setStatus('SVG export unavailable — cytoscape-svg plugin failed to load');
        return;
    }
    try {
        const svgContent = cy.svg({ full: true, scale: 1, bg: '#12131a' });
        const blob = new Blob([svgContent], { type: 'image/svg+xml' });
        const a = document.createElement('a');
        a.href = URL.createObjectURL(blob);
        a.download = 'btor2-graph.svg';
        a.click();
        URL.revokeObjectURL(a.href);
        setStatus('Exported SVG');
    } catch (err) {
        setStatus('SVG export failed: ' + err.message);
    }
});

// Subgraph root selector
selectRoot.addEventListener('change', () => {
    const val = selectRoot.value;
    subgraphRoot = val ? parseInt(val) : null;
    collapsedNodes.clear();
    if (subgraphRoot) {
        depthControl.classList.remove('hidden');
    } else {
        depthControl.classList.add('hidden');
        maxDepth = Infinity;
        depthSlider.value = 20;
        depthValue.textContent = 'All';
    }
    renderGraph();
    updateSubgraphInfo();
});

// Depth slider
depthSlider.addEventListener('input', () => {
    const val = parseInt(depthSlider.value);
    if (val >= 20) {
        maxDepth = Infinity;
        depthValue.textContent = 'All';
    } else {
        maxDepth = val;
        depthValue.textContent = String(val);
    }
    renderGraph();
    updateSubgraphInfo();
});

// Layout toggle
layoutDagre.addEventListener('change', () => {
    if (layoutDagre.checked) { layoutMode = 'dagre'; renderGraph(); }
});
layoutCose.addEventListener('change', () => {
    if (layoutCose.checked) { layoutMode = 'cose'; renderGraph(); }
});

// Longest path
btnLongestPath.addEventListener('click', () => {
    if (!cy || !parsedNodes) return;
    const root = subgraphRoot || findFirstBadNid();
    if (root) {
        const path = highlightLongestPath(cy, parsedNodes, root);
        setStatus(`Longest path: ${path.length} nodes from #${root}`);
    }
});

// Clumping controls
clumpHeader.addEventListener('click', () => {
    clumpOptions.classList.toggle('collapsed');
    clumpHeader.querySelector('span').textContent =
        clumpOptions.classList.contains('collapsed') ? '\u25B6' : '\u25BC';
});

['clump-logic', 'clump-state', 'clump-memory', 'clump-constant'].forEach(id => {
    document.getElementById(id).addEventListener('change', () => {
        clumpCategories.clear();
        if (document.getElementById('clump-logic').checked) clumpCategories.add('logic');
        if (document.getElementById('clump-state').checked) clumpCategories.add('state');
        if (document.getElementById('clump-memory').checked) clumpCategories.add('memory');
        if (document.getElementById('clump-constant').checked) clumpCategories.add('constant');
        renderGraph();
    });
});

// Witness event handlers
btnWitnessUpload.addEventListener('click', () => witnessFileInput.click());
witnessFileInput.addEventListener('change', handleWitnessUpload);
btnWitnessPaste.addEventListener('click', showWitnessPasteOverlay);
btnWitnessPasteConfirm.addEventListener('click', handleWitnessPaste);
btnWitnessPasteCancel.addEventListener('click', hideWitnessPasteOverlay);
btnWitnessExample.addEventListener('click', loadExampleWitness);
btnWitnessStart.addEventListener('click', () => player.goToStep(0));
btnWitnessBack.addEventListener('click', () => player.stepBackward());
btnWitnessPlay.addEventListener('click', toggleWitnessPlay);
btnWitnessFwd.addEventListener('click', () => player.stepForward());
btnWitnessEnd.addEventListener('click', () => player.goToStep(player.totalSteps - 1));
btnWitnessClose.addEventListener('click', closeWitness);
witnessSpeed.addEventListener('input', () => {
    const ms = parseInt(witnessSpeed.value);
    player.setSpeed(ms);
    witnessSpeedLabel.textContent = (ms / 1000).toFixed(1) + 's';
});

// Keyboard shortcuts
document.addEventListener('keydown', e => {
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA' || e.target.tagName === 'SELECT') return;

    if (player.loaded) {
        switch (e.key) {
            case 'ArrowLeft': e.preventDefault(); player.stepBackward(); break;
            case 'ArrowRight': e.preventDefault(); player.stepForward(); break;
            case ' ': e.preventDefault(); toggleWitnessPlay(); break;
            case 'Home': e.preventDefault(); player.goToStep(0); break;
            case 'End': e.preventDefault(); player.goToStep(player.totalSteps - 1); break;
            case 'Escape': if (player.playing) { player.pause(); updatePlayButton(); } break;
        }
    }
});

// Player callbacks
player.onStepChange = (step, total) => {
    witnessStepEl.textContent = step;
    witnessTotalEl.textContent = total - 1;
    updateWitnessDetailPanel(step);
    updateTracePanel();
    setStatus(`Witness step ${step} of ${total - 1}`);
};
player.onStop = () => updatePlayButton();

// ============ File Handling ============

function handleFileUpload(e) {
    const file = e.target.files[0];
    if (!file) return;
    setStatus(`Loading ${file.name}...`);
    const reader = new FileReader();
    reader.onload = ev => loadBtor2(ev.target.result, file.name);
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
    setStatus('Loading Rotor example...');
    try {
        const modelResp = await fetch('./examples/simple-assignment-1-35.btor2');
        const modelText = await modelResp.text();
        loadBtor2(modelText, 'simple-assignment-1-35.btor2');
    } catch (err) {
        setStatus('Failed to load example: ' + err.message);
        return;
    }
    try {
        const witResp = await fetch('./examples/simple-assignment-1-35.wit');
        const witText = await witResp.text();
        loadWitness(witText, 'simple-assignment-1-35.wit');
    } catch (err) {
        setStatus('Example loaded (witness unavailable)');
    }
}

// ============ Witness File Handling ============

function handleWitnessUpload(e) {
    const file = e.target.files[0];
    if (!file) return;
    if (!parsedNodes) { setStatus('Load a BTOR2 model first'); return; }
    const reader = new FileReader();
    reader.onload = ev => loadWitness(ev.target.result, file.name);
    reader.readAsText(file);
    witnessFileInput.value = '';
}

function showWitnessPasteOverlay() {
    if (!parsedNodes) { setStatus('Load a BTOR2 model first'); return; }
    witnessPasteOverlay.classList.remove('hidden');
    witnessPasteTextarea.focus();
}

function hideWitnessPasteOverlay() {
    witnessPasteOverlay.classList.add('hidden');
    witnessPasteTextarea.value = '';
}

function handleWitnessPaste() {
    const text = witnessPasteTextarea.value.trim();
    if (!text) return;
    hideWitnessPasteOverlay();
    loadWitness(text, 'pasted witness');
}

async function loadExampleWitness() {
    // Always load the counter-with-input model to match the witness
    setStatus('Loading counter model + witness...');
    try {
        const modelResp = await fetch('./examples/counter-with-input.btor2');
        const modelText = await modelResp.text();
        loadBtor2(modelText, 'counter-with-input.btor2');
    } catch (err) {
        setStatus('Failed to load example model: ' + err.message);
        return;
    }
    setStatus('Loading example witness...');
    try {
        const resp = await fetch('./examples/counter-with-input.wit');
        const text = await resp.text();
        loadWitness(text, 'counter-with-input.wit (btormc output)');
    } catch (err) {
        setStatus('Failed to load example witness: ' + err.message);
    }
}

function loadWitness(text, source) {
    if (!parsedNodes) { setStatus('No model loaded'); return; }
    setStatus(`Parsing witness from ${source}...`);
    const witness = parseWitness(text, parsedNodes);

    if (witness.errors.length > 0) {
        console.warn('Witness parse errors:', witness.errors);
        for (const err of witness.errors) {
            if (err.includes('UNSAT')) { setStatus('Witness: UNSAT — no counterexample'); return; }
        }
    }
    if (witness.frames.length === 0) { setStatus('No trace frames found'); return; }

    player.load(witness, parsedNodes, cy);
    witnessControls.classList.remove('hidden');
    tracePanel.classList.remove('hidden');
    witnessTotalEl.textContent = witness.frames.length - 1;
    witnessStepEl.textContent = '0';

    let badInfo = '';
    if (witness.bad !== null) {
        const badNid = badIndexToNid(parsedNodes, witness.bad);
        const badNode = badNid ? parsedNodes.get(badNid) : null;
        badInfo = ` — bad: ${badNode ? (badNode.name || '#' + badNid) : 'b' + witness.bad}`;
    }
    setStatus(`Witness: ${witness.frames.length} steps${badInfo}`);
    player.goToStep(0);
}

function closeWitness() {
    player.unload();
    witnessControls.classList.add('hidden');
    tracePanel.classList.add('hidden');
    if (cy) {
        cy.elements().removeClass('witness-active witness-state witness-input witness-bad witness-inactive witness-changed');
        cy.nodes().forEach(n => { n.data('witnessValue', null); n.data('witnessLabel', null); });
    }
    setStatus('Witness closed');
}

function toggleWitnessPlay() {
    if (player.playing) player.pause(); else player.play();
    updatePlayButton();
}

function updatePlayButton() {
    if (player.playing) {
        btnWitnessPlay.innerHTML = '&#9646;&#9646;';
        btnWitnessPlay.classList.add('playing');
    } else {
        btnWitnessPlay.innerHTML = '&#9654;';
        btnWitnessPlay.classList.remove('playing');
    }
}

// ============ Witness Detail Panel ============

function updateWitnessDetailPanel(step) {
    if (!player.loaded) return;

    const cumStates = player._cumStates || new Map();
    const cumInputs = player._cumInputs || new Map();
    const changed = player._changedNids || new Set();

    let html = `<div class="detail-header">
        <span class="detail-nid">Step ${step}</span>
        <span class="detail-op state">Witness</span>
    </div>`;

    if (player.witness.bad !== null) {
        const badNid = badIndexToNid(parsedNodes, player.witness.bad);
        const badNode = badNid ? parsedNodes.get(badNid) : null;
        const badName = badNode ? (badNode.name || badNode.op) : `b${player.witness.bad}`;
        const isViolated = step === player.totalSteps - 1;
        html += `<div class="detail-row">
            <span class="label">Bad:</span>
            <span style="color:var(--bad-color);font-weight:600;">${escHtml(badName)}${isViolated ? ' — VIOLATED' : ''}</span>
        </div>`;
    }

    if (cumStates.size > 0) {
        html += `<div class="detail-section">States (${cumStates.size})</div><div class="detail-links">`;
        for (const [nid, entry] of cumStates) {
            const node = parsedNodes.get(nid);
            const name = node ? (node.name || `#${nid}`) : `#${nid}`;
            const value = Array.isArray(entry) ? `[${entry.length} entries]` : formatWitnessValue(entry.bits);
            const isChanged = changed.has(nid);
            const style = isChanged ? 'color:var(--state-color);font-weight:700;' : 'color:var(--state-color);opacity:0.6;';
            html += `<a class="node-link" data-nid="${nid}">
                <span style="${style}">${escHtml(name)}</span>
                <span style="color:var(--text-muted);font-family:var(--font-mono);font-size:10px;"> = ${escHtml(String(value))}</span>
            </a>`;
        }
        html += `</div>`;
    }

    if (cumInputs.size > 0) {
        html += `<div class="detail-section">Inputs (${cumInputs.size})</div><div class="detail-links">`;
        for (const [nid, entry] of cumInputs) {
            const node = parsedNodes.get(nid);
            const name = node ? (node.name || `#${nid}`) : `#${nid}`;
            const value = Array.isArray(entry) ? `[array]` : formatWitnessValue(entry.bits);
            const isChanged = changed.has(nid);
            const style = isChanged ? 'color:var(--input-color);font-weight:700;' : 'color:var(--input-color);opacity:0.6;';
            html += `<a class="node-link" data-nid="${nid}">
                <span style="${style}">${escHtml(name)}</span>
                <span style="color:var(--text-muted);font-family:var(--font-mono);font-size:10px;"> = ${escHtml(String(value))}</span>
            </a>`;
        }
        html += `</div>`;
    }

    detailPanel.innerHTML = html;
    detailPanel.querySelectorAll('.node-link').forEach(link => {
        link.addEventListener('click', () => navigateToNode(parseInt(link.dataset.nid)));
    });
}

// ============ Trace Panel ============

function findPcNid(nodes) {
    for (const [nid, node] of nodes) {
        if (node.op === 'state' && node.name === 'pc') return nid;
    }
    return null;
}

function updateTracePanel() {
    if (!player.loaded || !tracePanel || !traceContent) {
        if (tracePanel) tracePanel.classList.add('hidden');
        return;
    }
    tracePanel.classList.remove('hidden');

    const step = player.currentStep;
    const total = player.totalSteps;
    const changed = player._changedNids || new Set();
    let html = '';

    html += `<div class="trace-row"><span class="label">Step:</span> <span class="trace-value">${step} / ${total - 1}</span></div>`;

    const pcNid = findPcNid(parsedNodes);
    if (pcNid !== null && player._cumStates) {
        const entry = player._cumStates.get(pcNid);
        if (entry && !Array.isArray(entry)) {
            const dec = formatWitnessValue(entry.bits);
            let hex = '';
            const pureBits = entry.bits;
            if (pureBits.length <= 52) {
                hex = '0x' + parseInt(pureBits, 2).toString(16);
            } else {
                hex = '0x' + BigInt('0b' + pureBits).toString(16);
            }
            html += `<div class="trace-row"><span class="label">PC:</span> <span class="trace-value" style="color:var(--state-color);">${escHtml(hex)}</span></div>`;
        }
    }

    if (player.witness.bad !== null) {
        const badNid = badIndexToNid(parsedNodes, player.witness.bad);
        const badNode = badNid ? parsedNodes.get(badNid) : null;
        const badName = badNode ? (badNode.name || badNode.op) : `b${player.witness.bad}`;
        const isViolated = step === total - 1;
        const style = isViolated ? 'color:var(--bad-color);font-weight:600;' : '';
        html += `<div class="trace-row"><span class="label">Bad:</span> <span style="${style}">${escHtml(badName)}${isViolated ? ' VIOLATED' : ''}</span></div>`;
    }

    html += `<div class="trace-section">Changed (${changed.size})</div>`;
    if (changed.size > 0) {
        const prevValues = player._prevValues || new Map();
        for (const nid of changed) {
            const node = parsedNodes.get(nid);
            const name = node ? (node.name || `#${nid}`) : `#${nid}`;
            const isState = player._cumStates && player._cumStates.has(nid);
            const entry = isState ? player._cumStates.get(nid) : (player._cumInputs ? player._cumInputs.get(nid) : null);
            const val = entry && !Array.isArray(entry) ? formatWitnessValue(entry.bits) : '...';
            const color = isState ? 'var(--state-color)' : 'var(--input-color)';
            const prevBits = prevValues.get(nid);
            let valDisplay;
            if (prevBits) {
                valDisplay = `${escHtml(formatWitnessValue(prevBits))} → ${escHtml(val)}`;
            } else {
                valDisplay = `= ${escHtml(val)}`;
            }
            html += `<a class="node-link trace-change" data-nid="${nid}"><span style="color:${color};">${escHtml(name)}</span> <span class="trace-val">${valDisplay}</span></a>`;
        }
    } else {
        html += `<span class="trace-none text-muted">(carried forward)</span>`;
    }

    traceContent.innerHTML = html;
    traceContent.querySelectorAll('.node-link').forEach(link => {
        link.addEventListener('click', () => navigateToNode(parseInt(link.dataset.nid)));
    });
}

// ============ Core Logic ============

function loadBtor2(text, filename) {
    setStatus(`Parsing ${filename}...`);
    if (player.loaded) closeWitness();

    const result = parseBtor2(text);
    parsedNodes = result.nodes;
    subgraphRoot = null;
    collapsedNodes.clear();

    if (result.errors.length > 0) console.warn('Parse errors:', result.errors);

    const stats = computeStats(parsedNodes);
    renderStats(stats);
    renderNodeList(parsedNodes);
    populateRootSelector(parsedNodes);
    renderGraph();

    setStatus(`Loaded ${filename}: ${stats.total} nodes, ${stats.badCount} bad, ${stats.stateCount} states`);
}

function renderGraph() {
    if (!parsedNodes) return;

    currentFilter.hideSorts = filterSorts.checked;
    currentFilter.hideConstants = filterConsts.checked;

    const options = {
        subgraphRoot: subgraphRoot,
        maxDepth: maxDepth,
        collapsedNodes: collapsedNodes,
        clumpCategories: clumpCategories,
    };

    const elements = buildElements(parsedNodes, currentFilter, options);

    if (cy) cy.destroy();
    cy = initGraph(graphContainer, elements);
    setStatus('Running layout...');

    setTimeout(() => {
        runLayout(cy, layoutMode);
        cy.fit(null, 30);
        if (layoutMode !== 'cose') cy.nodes().ungrabify();
        setupGraphInteraction();

        if (player.loaded) {
            player.cy = cy;
            player._lastRenderedStep = -1;
            player.goToStep(player.currentStep);
        }
        setStatus('Ready');
    }, 50);
}

function setupGraphInteraction() {
    // Click node → show detail; double-click → collapse/expand
    cy.on('tap', 'node', function (evt) {
        const nid = parseInt(evt.target.data('nid'));
        if (isNaN(nid)) return; // clump node

        if (player.loaded) {
            showWitnessNodeDetail(nid);
        } else {
            showNodeDetail(nid);
        }

        cy.elements().removeClass('highlighted neighbor');
        evt.target.addClass('highlighted');
        evt.target.neighborhood().addClass('neighbor');
    });

    // Double-click node → toggle collapse/expand
    cy.on('dbltap', 'node', function (evt) {
        const nid = parseInt(evt.target.data('nid'));
        if (isNaN(nid)) return;
        const node = parsedNodes.get(nid);
        if (!node) return;

        // Only allow collapse on non-leaf nodes (those with operands)
        if (node.operands.length === 0) return;

        if (collapsedNodes.has(nid)) {
            collapsedNodes.delete(nid);
            setStatus(`Expanded node #${nid}`);
        } else {
            collapsedNodes.add(nid);
            setStatus(`Collapsed node #${nid} (double-click to expand)`);
        }
        renderGraph();
    });

    // Right-click node → show cone of influence
    cy.on('cxttap', 'node', function (evt) {
        const nid = parseInt(evt.target.data('nid'));
        if (isNaN(nid)) return;
        highlightCone(cy, parsedNodes, nid);
        setStatus(`Cone of influence for #${nid}`);
    });

    // Click background → clear selection
    cy.on('tap', function (evt) {
        if (evt.target === cy) {
            if (player.loaded) {
                updateWitnessDetailPanel(player.currentStep);
            } else {
                detailPanel.innerHTML = '<p class="placeholder">Click a node to see details<br><span class="text-muted">Double-click to collapse/expand</span></p>';
            }
            cy.elements().removeClass('highlighted neighbor');
        }
    });
}

// ============ Subgraph Root Selector ============

function populateRootSelector(nodes) {
    // Clear existing options
    while (selectRoot.firstChild) selectRoot.removeChild(selectRoot.firstChild);
    const defaultOpt = document.createElement('option');
    defaultOpt.value = '';
    defaultOpt.textContent = 'Full graph';
    selectRoot.appendChild(defaultOpt);

    const badNodes = [];
    const stateNodes = [];

    for (const [nid, node] of nodes) {
        if (node.op === 'bad') badNodes.push(node);
        else if (node.op === 'state' && node.name) stateNodes.push(node);
    }

    // Add bad properties
    if (badNodes.length > 0) {
        const grp = document.createElement('optgroup');
        grp.label = 'Bad Properties (' + badNodes.length + ')';
        for (const node of badNodes) {
            const opt = document.createElement('option');
            opt.value = String(node.nid);
            opt.textContent = '#' + node.nid + ' ' + (node.name || 'bad');
            grp.appendChild(opt);
        }
        selectRoot.appendChild(grp);
    }

    // Add named states
    if (stateNodes.length > 0) {
        const grp = document.createElement('optgroup');
        grp.label = 'States (' + stateNodes.length + ')';
        for (const node of stateNodes.slice(0, 30)) {
            const opt = document.createElement('option');
            opt.value = String(node.nid);
            opt.textContent = '#' + node.nid + ' ' + node.name;
            grp.appendChild(opt);
        }
        selectRoot.appendChild(grp);
    }

}

function updateSubgraphInfo() {
    if (subgraphRoot) {
        const node = parsedNodes.get(subgraphRoot);
        const name = node ? (node.name || node.op) : subgraphRoot;
        const cone = getConeOfInfluence(parsedNodes, subgraphRoot, maxDepth);
        const depthLabel = maxDepth === Infinity ? '' : `, depth ${maxDepth}`;
        subgraphInfo.innerHTML = `<span style="color:var(--accent);">${escHtml(String(name))}</span> <span class="text-muted">(${cone.size} nodes${depthLabel})</span>`;
    } else {
        subgraphInfo.innerHTML = '<span class="text-muted">Showing full graph</span>';
    }
}

function findFirstBadNid() {
    for (const [nid, node] of parsedNodes) {
        if (node.op === 'bad') return nid;
    }
    return null;
}

// ============ Witness-Aware Node Detail ============

function showWitnessNodeDetail(nid) {
    const node = parsedNodes.get(nid);
    if (!node) return;

    const sortInfo = node.sortNid ? getSortDescription(node.sortNid) : '';
    let html = `<div class="detail-header">
        <span class="detail-nid">#${node.nid}</span>
        <span class="detail-op ${node.category}">${node.op}</span>
    </div>`;

    if (node.name) html += `<div class="detail-row"><span class="label">Name:</span> ${escHtml(node.name)}</div>`;
    if (sortInfo) html += `<div class="detail-row"><span class="label">Sort:</span> ${sortInfo}</div>`;

    // Witness value at current step (cumulative — includes carried-forward values)
    if (player.loaded) {
        const entry = (player._cumStates && player._cumStates.get(nid)) ||
                      (player._cumInputs && player._cumInputs.get(nid));
        if (entry && !Array.isArray(entry)) {
            const isState = player._cumStates && player._cumStates.has(nid);
            const color = isState ? 'var(--state-color)' : 'var(--input-color)';
            const isChanged = player._changedNids && player._changedNids.has(nid);
            const suffix = isChanged ? '' : ' (carried forward)';
            html += `<div class="detail-row"><span class="label">Value @${player.currentStep}:</span> <span style="color:${color};font-family:var(--font-mono);font-weight:600;">${escHtml(formatWitnessValue(entry.bits))}${suffix}</span></div>`;
            html += `<div class="detail-row"><span class="label">Binary:</span> <span style="font-family:var(--font-mono);font-size:10px;">${escHtml(entry.bits)}</span></div>`;
        }
    }

    // Operands
    if (node.operands.length > 0) {
        html += `<div class="detail-section">Operands</div><div class="detail-links">`;
        for (const opNid of node.operands) {
            const opNode = parsedNodes.get(opNid);
            const opLabel = opNode ? `#${opNid} (${opNode.op}${opNode.name ? ': ' + opNode.name : ''})` : `#${opNid}`;
            html += `<a class="node-link" data-nid="${opNid}">${escHtml(opLabel)}</a>`;
        }
        html += `</div>`;
    }

    html += `<div class="detail-section">Raw BTOR2</div>`;
    html += `<pre class="raw-line">${escHtml(node.rawLine)}</pre>`;

    detailPanel.innerHTML = html;
    detailPanel.querySelectorAll('.node-link').forEach(link => {
        link.addEventListener('click', () => navigateToNode(parseInt(link.dataset.nid)));
    });
}

// ============ Detail Panel ============

function showNodeDetail(nid) {
    const node = parsedNodes.get(nid);
    if (!node) return;

    const sortInfo = node.sortNid ? getSortDescription(node.sortNid) : '';
    let html = `<div class="detail-header">
        <span class="detail-nid">#${node.nid}</span>
        <span class="detail-op ${node.category}">${node.op}</span>
    </div>`;

    if (node.name) html += `<div class="detail-row"><span class="label">Name:</span> ${escHtml(node.name)}</div>`;
    if (sortInfo) html += `<div class="detail-row"><span class="label">Sort:</span> ${sortInfo}</div>`;
    if (node.comment) html += `<div class="detail-row"><span class="label">Comment:</span> ${escHtml(node.comment)}</div>`;

    if (node.extra.width !== undefined && node.op !== 'sort')
        html += `<div class="detail-row"><span class="label">Width:</span> ${node.extra.width}</div>`;
    if (node.extra.upper !== undefined)
        html += `<div class="detail-row"><span class="label">Slice:</span> [${node.extra.upper}:${node.extra.lower}]</div>`;
    if (node.extra.value !== undefined)
        html += `<div class="detail-row"><span class="label">Value:</span> ${escHtml(String(node.extra.value))}</div>`;

    if (node.operands.length > 0) {
        html += `<div class="detail-section">Operands</div><div class="detail-links">`;
        for (const opNid of node.operands) {
            const opNode = parsedNodes.get(opNid);
            const opLabel = opNode ? `#${opNid} (${opNode.op}${opNode.name ? ': ' + opNode.name : ''})` : `#${opNid}`;
            html += `<a class="node-link" data-nid="${opNid}">${escHtml(opLabel)}</a>`;
        }
        html += `</div>`;
    }

    if (node.dependents.length > 0) {
        html += `<div class="detail-section">Used by (${node.dependents.length})</div><div class="detail-links">`;
        const show = node.dependents.slice(0, 20);
        for (const depNid of show) {
            const depNode = parsedNodes.get(depNid);
            const depLabel = depNode ? `#${depNid} (${depNode.op})` : `#${depNid}`;
            html += `<a class="node-link" data-nid="${depNid}">${escHtml(depLabel)}</a>`;
        }
        if (node.dependents.length > 20) html += `<span class="text-muted">... and ${node.dependents.length - 20} more</span>`;
        html += `</div>`;
    }

    // Collapse hint
    if (node.operands.length > 0) {
        const isCollapsed = collapsedNodes.has(nid);
        html += `<div class="detail-section">Actions</div>`;
        html += `<button class="btn btn-small" onclick="toggleCollapseNode(${nid})">${isCollapsed ? 'Expand' : 'Collapse'} descendants</button>`;
        html += `<button class="btn btn-small" style="margin-left:4px;" onclick="viewAsSubgraph(${nid})">View subgraph</button>`;
    }

    html += `<div class="detail-section">Raw BTOR2</div>`;
    html += `<pre class="raw-line">${escHtml(node.rawLine)}</pre>`;

    detailPanel.innerHTML = html;
    detailPanel.querySelectorAll('.node-link').forEach(link => {
        link.addEventListener('click', () => navigateToNode(parseInt(link.dataset.nid)));
    });
}

// Global functions for inline event handlers
window.toggleCollapseNode = function(nid) {
    if (collapsedNodes.has(nid)) collapsedNodes.delete(nid);
    else collapsedNodes.add(nid);
    renderGraph();
};

window.viewAsSubgraph = function(nid) {
    subgraphRoot = nid;
    selectRoot.value = String(nid);
    collapsedNodes.clear();
    renderGraph();
    updateSubgraphInfo();
};

function navigateToNode(nid) {
    if (!cy) return;
    const cyNode = cy.getElementById(String(nid));
    if (cyNode.length) {
        cy.animate({ center: { eles: cyNode }, zoom: 1.5 }, { duration: 300 });
        cy.elements().removeClass('highlighted neighbor');
        cyNode.addClass('highlighted');
        cyNode.neighborhood().addClass('neighbor');
        if (player.loaded) showWitnessNodeDetail(nid);
        else showNodeDetail(nid);
    }
}

function getSortDescription(sortNid) {
    const sortNode = parsedNodes.get(sortNid);
    if (!sortNode) return `sort #${sortNid}`;
    if (sortNode.extra.sortKind === 'bitvec') return `bitvec[${sortNode.extra.width}]`;
    if (sortNode.extra.sortKind === 'array') {
        return `array[${getSortDescription(sortNode.extra.indexSort)} \u2192 ${getSortDescription(sortNode.extra.elementSort)}]`;
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
        if (catNodes.length > maxShow) html += `<span class="text-muted list-item">... ${catNodes.length - maxShow} more</span>`;
        html += `</div>`;
    }

    nodeList.innerHTML = html;
    nodeList.querySelectorAll('.node-link').forEach(link => {
        link.addEventListener('click', () => navigateToNode(parseInt(link.dataset.nid)));
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
    if (sortedOps.length > 15) html += `<div class="text-muted">... ${sortedOps.length - 15} more</div>`;
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

    if (matches.length === 0) { setStatus(`No matches for "${query}"`); return; }

    if (cy) {
        cy.elements().removeClass('highlighted neighbor');
        const matchEles = cy.collection();
        for (const nid of matches) {
            const el = cy.getElementById(String(nid));
            el.addClass('highlighted');
            matchEles.merge(el);
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

function setStatus(msg) { statusBar.textContent = msg; }

function escHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
}

// ============ Init ============

setStatus('Ready \u2014 upload a BTOR2 file or load the example');
