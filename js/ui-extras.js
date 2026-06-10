/**
 * UI extras — additive enhancements layered over app.js:
 *   - empty-state hero (hidden once a model loads)
 *   - drag & drop loading for .btor2 / .wit files
 *   - toast notifications for load events and errors
 *   - witness timeline scrubber synced with the player
 *   - extra keyboard shortcuts (F fit, +/- zoom, / search, ? help, Esc)
 *
 * Everything here is defensive: if a hook is missing, the feature
 * silently disables itself rather than breaking the app.
 */
(function () {
    'use strict';

    // ---------- toasts ----------
    const toastContainer = document.getElementById('toast-container');

    window.toast = function (message, kind = 'info', ms = 3200) {
        if (!toastContainer) return;
        const el = document.createElement('div');
        el.className = 'toast' + (kind === 'error' ? ' error' : kind === 'success' ? ' success' : '');
        el.textContent = message;
        toastContainer.appendChild(el);
        setTimeout(() => {
            el.classList.add('leaving');
            setTimeout(() => el.remove(), 350);
        }, ms);
        // keep at most 4 toasts
        while (toastContainer.children.length > 4) toastContainer.firstChild.remove();
    };

    // ---------- hero empty state ----------
    const hero = document.getElementById('hero');

    function hideHero() {
        if (hero) hero.classList.add('hidden');
    }

    const heroUpload = document.getElementById('hero-btn-upload');
    const heroExample = document.getElementById('hero-btn-example');
    if (heroUpload) heroUpload.addEventListener('click', () => {
        const real = document.getElementById('btn-upload');
        if (real) real.click();
    });
    if (heroExample) heroExample.addEventListener('click', () => {
        const picker = document.getElementById('example-picker');
        // Prefer a SMALL example for a legible first impression — a 1400-node
        // CPU model is unreadable cold. Fall back through progressively larger
        // ones, then the first manifest entry.
        if (picker && window.__exampleManifest && window.__exampleManifest.length > 0) {
            const prefer = ['counter-with-input', 'tiny-counter',
                            'simple-assignment-1-35', 'argv_test4_multi_arg'];
            let pick = null;
            for (const id of prefer) {
                if (window.__exampleManifest.some(e => e.id === id)) { pick = id; break; }
            }
            pick = pick || window.__exampleManifest[0].id;
            picker.value = pick;
            picker.dispatchEvent(new Event('change'));
        } else {
            const real = document.getElementById('btn-example');
            if (real) real.click();
        }
    });

    // Wrap loadBtor2 / loadWitness to hide the hero + toast.
    if (typeof window.loadBtor2 === 'function' || typeof loadBtor2 === 'function') {
        try {
            const orig = loadBtor2;
            // eslint-disable-next-line no-global-assign
            loadBtor2 = function (text, filename) {
                const r = orig(text, filename);
                hideHero();
                window.toast(`Model loaded: ${filename}`, 'success');
                return r;
            };
        } catch (e) { /* leave as-is */ }
    }
    if (typeof loadWitness === 'function') {
        try {
            const orig = loadWitness;
            // eslint-disable-next-line no-global-assign
            loadWitness = function (text, source) {
                const r = orig(text, source);
                window.toast(`Witness loaded: ${source}`, 'success');
                return r;
            };
        } catch (e) { /* leave as-is */ }
    }

    // ---------- drag & drop ----------
    const dropTarget = document.body;
    const graphArea = document.getElementById('graph-area');
    let dragDepth = 0;

    dropTarget.addEventListener('dragenter', (e) => {
        e.preventDefault();
        dragDepth++;
        if (graphArea) graphArea.classList.add('dragover');
    });
    dropTarget.addEventListener('dragleave', (e) => {
        e.preventDefault();
        dragDepth = Math.max(0, dragDepth - 1);
        if (dragDepth === 0 && graphArea) graphArea.classList.remove('dragover');
    });
    dropTarget.addEventListener('dragover', (e) => e.preventDefault());
    dropTarget.addEventListener('drop', (e) => {
        e.preventDefault();
        dragDepth = 0;
        if (graphArea) graphArea.classList.remove('dragover');

        const file = e.dataTransfer && e.dataTransfer.files && e.dataTransfer.files[0];
        if (!file) return;

        const name = file.name.toLowerCase();
        const reader = new FileReader();
        reader.onload = (ev) => {
            const text = ev.target.result;
            try {
                if (name.endsWith('.wit') || name.endsWith('.witness') || name.endsWith('.out')) {
                    if (typeof parsedNodes === 'undefined' || !parsedNodes) {
                        window.toast('Load a BTOR2 model before dropping a witness', 'error');
                        return;
                    }
                    loadWitness(text, file.name);
                } else {
                    loadBtor2(text, file.name);
                }
            } catch (err) {
                window.toast(`Failed to load ${file.name}: ${err.message}`, 'error', 5200);
            }
        };
        reader.readAsText(file);
    });

    // ---------- witness scrubber ----------
    const scrubber = document.getElementById('witness-scrubber');
    if (scrubber && typeof player !== 'undefined' && player) {
        // chain onto the existing step-change callback
        const prev = player.onStepChange;
        player.onStepChange = function (step, total) {
            if (typeof prev === 'function') prev(step, total);
            scrubber.max = Math.max(0, total - 1);
            scrubber.value = step;
        };
        scrubber.addEventListener('input', () => {
            if (player.loaded) {
                if (player.playing && typeof player.pause === 'function') player.pause();
                player.goToStep(parseInt(scrubber.value, 10));
            }
        });
    }

    // ---------- keyboard shortcuts ----------
    const helpOverlay = document.getElementById('help-overlay');
    const btnHelp = document.getElementById('btn-help');
    const btnHelpClose = document.getElementById('btn-help-close');

    function toggleHelp(force) {
        if (!helpOverlay) return;
        const show = force !== undefined ? force : helpOverlay.classList.contains('hidden');
        helpOverlay.classList.toggle('hidden', !show);
    }
    if (btnHelp) btnHelp.addEventListener('click', () => toggleHelp());
    if (btnHelpClose) btnHelpClose.addEventListener('click', () => toggleHelp(false));
    if (helpOverlay) helpOverlay.addEventListener('click', (e) => {
        if (e.target === helpOverlay) toggleHelp(false);
    });

    document.addEventListener('keydown', (e) => {
        const tag = e.target.tagName;
        if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') {
            if (e.key === 'Escape') e.target.blur();
            return;
        }

        switch (e.key) {
            case '?':
                e.preventDefault();
                toggleHelp();
                break;
            case '/':
                e.preventDefault();
                {
                    const s = document.getElementById('search-input');
                    if (s) s.focus();
                }
                break;
            case 'f': case 'F':
                {
                    const fit = document.getElementById('btn-fit');
                    if (fit) fit.click();
                }
                break;
            case '+': case '=':
                {
                    const z = document.getElementById('btn-zoom-in');
                    if (z) z.click();
                }
                break;
            case '-': case '_':
                {
                    const z = document.getElementById('btn-zoom-out');
                    if (z) z.click();
                }
                break;
            case 'Escape':
                // close overlays first, then clear highlights
                if (helpOverlay && !helpOverlay.classList.contains('hidden')) {
                    toggleHelp(false);
                } else {
                    const clear = document.getElementById('btn-clear-highlight');
                    if (clear) clear.click();
                }
                break;
        }
    });

    // ---------- keep the graph framed on window resize ----------
    // Cytoscape needs an explicit resize() when its container changes size,
    // otherwise the graph stays anchored to the old viewport and looks empty.
    let resizeTimer = null;
    window.addEventListener('resize', () => {
        clearTimeout(resizeTimer);
        resizeTimer = setTimeout(() => {
            if (typeof cy !== 'undefined' && cy) {
                cy.resize();
                cy.fit(null, 30);
            }
        }, 150);
    });

    // If a model was somehow loaded before this script ran, hide the hero.
    if (typeof parsedNodes !== 'undefined' && parsedNodes) hideHero();
})();
