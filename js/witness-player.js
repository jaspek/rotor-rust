/**
 * BTOR2 Witness Trace Player
 *
 * Provides step-by-step animation of counterexample traces on the
 * Cytoscape graph. Highlights state/input nodes with their values
 * at each time step and shows the violated bad property.
 */

class WitnessPlayer {
    constructor() {
        this.witness = null;       // Parsed witness { bad, frames, stateNids, inputNids }
        this.modelNodes = null;    // Reference to parsed BTOR2 nodes
        this.cy = null;            // Cytoscape instance
        this.currentStep = 0;
        this.playing = false;
        this.playInterval = null;
        this.speed = 1000;         // ms per step
        this.onStepChange = null;  // Callback: (step, totalSteps) => void
        this.onStop = null;        // Callback: () => void

        // Track value overlays so we can remove them
        this._overlayEles = [];

        // Cumulative state accessible by the app for trace panel
        this._cumStates = new Map();
        this._cumInputs = new Map();
        this._changedNids = new Set();
        this._prevValues = new Map(); // nid → previous bits string (for old→new diffs)
        this._lastRenderedStep = -1;
    }

    /**
     * Load a witness trace for playback.
     */
    load(witness, modelNodes, cy) {
        this.stop();
        this.witness = witness;
        this.modelNodes = modelNodes;
        this.cy = cy;
        this.currentStep = 0;
        this._lastRenderedStep = -1;
        this._clearOverlays();
    }

    /**
     * Whether a witness is loaded and ready.
     */
    get loaded() {
        return this.witness && this.witness.frames.length > 0;
    }

    /**
     * Total number of steps in the trace.
     */
    get totalSteps() {
        return this.witness ? this.witness.frames.length : 0;
    }

    /**
     * Jump to a specific step and render it.
     */
    goToStep(step) {
        if (!this.loaded) return;
        step = Math.max(0, Math.min(step, this.totalSteps - 1));
        this.currentStep = step;
        this._renderStep();
        if (this.onStepChange) {
            this.onStepChange(this.currentStep, this.totalSteps);
        }
    }

    /**
     * Step forward.
     */
    stepForward() {
        if (this.currentStep < this.totalSteps - 1) {
            this.goToStep(this.currentStep + 1);
        } else {
            this.stop();
        }
    }

    /**
     * Step backward.
     */
    stepBackward() {
        if (this.currentStep > 0) {
            this.goToStep(this.currentStep - 1);
        }
    }

    /**
     * Start auto-playing the trace.
     */
    play() {
        if (!this.loaded || this.playing) return;
        this.playing = true;

        // If at the end, restart from beginning
        if (this.currentStep >= this.totalSteps - 1) {
            this.goToStep(0);
        }

        this.playInterval = setInterval(() => {
            if (this.currentStep < this.totalSteps - 1) {
                this.stepForward();
            } else {
                this.stop();
            }
        }, this.speed);
    }

    /**
     * Pause playback.
     */
    pause() {
        this.playing = false;
        if (this.playInterval) {
            clearInterval(this.playInterval);
            this.playInterval = null;
        }
    }

    /**
     * Stop playback and clear overlays.
     */
    stop() {
        this.pause();
        this._clearOverlays();
        if (this.onStop) this.onStop();
    }

    /**
     * Unload the witness entirely.
     */
    unload() {
        this.stop();
        this.witness = null;
        this.currentStep = 0;
    }

    /**
     * Set playback speed.
     * @param {number} ms - Milliseconds per step
     */
    setSpeed(ms) {
        this.speed = ms;
        // If currently playing, restart with new speed
        if (this.playing) {
            this.pause();
            this.play();
        }
    }

    // ─── Internal rendering ───

    _clearOverlays() {
        if (!this.cy) return;
        // Remove witness classes
        this.cy.elements().removeClass('witness-active witness-state witness-input witness-bad witness-inactive witness-value witness-changed');

        // Remove value labels
        this.cy.nodes().forEach(n => {
            n.data('witnessValue', null);
            n.data('witnessLabel', null);
        });
    }

    _renderStep() {
        if (!this.loaded || !this.cy) return;

        // Snapshot previous cumulative values for old→new diffs
        const prevCum = new Map();
        for (const [nid, entry] of this._cumStates) {
            if (!Array.isArray(entry)) prevCum.set(nid, entry.bits);
        }
        for (const [nid, entry] of this._cumInputs) {
            if (!Array.isArray(entry)) prevCum.set(nid, entry.bits);
        }

        // Build cumulative state: carry forward values from all frames up to current step
        this._cumStates = new Map();
        this._cumInputs = new Map();
        for (let i = 0; i <= this.currentStep; i++) {
            const f = this.witness.frames[i];
            if (!f) continue;
            for (const [nid, entry] of f.states) this._cumStates.set(nid, entry);
            for (const [nid, entry] of f.inputs) this._cumInputs.set(nid, entry);
        }

        // Track which nids changed in this specific step, and store previous values
        this._changedNids = new Set();
        this._prevValues = new Map();
        const currentFrame = this.witness.frames[this.currentStep];
        if (currentFrame) {
            for (const nid of currentFrame.states.keys()) {
                this._changedNids.add(nid);
                if (prevCum.has(nid)) this._prevValues.set(nid, prevCum.get(nid));
            }
            for (const nid of currentFrame.inputs.keys()) {
                this._changedNids.add(nid);
                if (prevCum.has(nid)) this._prevValues.set(nid, prevCum.get(nid));
            }
        }

        // Optimization: skip expensive graph re-render if nothing changed
        const atViolation = this.currentStep === this.totalSteps - 1;
        const wasAtViolation = this._lastRenderedStep === this.totalSteps - 1;
        if (this._changedNids.size === 0 && this._lastRenderedStep >= 0 &&
            atViolation === wasAtViolation) {
            this._lastRenderedStep = this.currentStep;
            return;
        }
        this._lastRenderedStep = this.currentStep;

        // Clear previous overlays
        this._clearOverlays();

        // Dim all nodes slightly
        this.cy.elements().addClass('witness-inactive');

        // Collect all nids that have cumulative values
        const activeNids = new Set();

        // Highlight state nodes with cumulative values
        for (const [nid, entry] of this._cumStates) {
            activeNids.add(nid);
            const cyNode = this.cy.getElementById(String(nid));
            if (cyNode.length) {
                cyNode.removeClass('witness-inactive');
                cyNode.addClass('witness-active witness-state');
                if (this._changedNids.has(nid)) cyNode.addClass('witness-changed');

                if (Array.isArray(entry)) {
                    cyNode.data('witnessValue', `[${entry.length} entries]`);
                } else {
                    cyNode.data('witnessValue', formatWitnessValue(entry.bits));
                }
                cyNode.data('witnessLabel', this._makeLabel(cyNode, nid));
            }
        }

        // Highlight input nodes with cumulative values
        for (const [nid, entry] of this._cumInputs) {
            activeNids.add(nid);
            const cyNode = this.cy.getElementById(String(nid));
            if (cyNode.length) {
                cyNode.removeClass('witness-inactive');
                cyNode.addClass('witness-active witness-input');
                if (this._changedNids.has(nid)) cyNode.addClass('witness-changed');

                if (!Array.isArray(entry)) {
                    cyNode.data('witnessValue', formatWitnessValue(entry.bits));
                }
                cyNode.data('witnessLabel', this._makeLabel(cyNode, nid));
            }
        }

        // Highlight the bad property node
        if (this.witness.bad !== null) {
            const badNid = badIndexToNid(this.modelNodes, this.witness.bad);
            if (badNid !== null) {
                const badNode = this.cy.getElementById(String(badNid));
                if (badNode.length) {
                    badNode.removeClass('witness-inactive');
                    badNode.addClass('witness-active witness-bad');

                    if (this.currentStep === this.totalSteps - 1) {
                        badNode.data('witnessValue', 'VIOLATED');
                    }
                    badNode.data('witnessLabel', this._makeLabel(badNode, badNid));
                }
            }
        }

        // Un-dim edges connected to active nodes
        this.cy.edges().forEach(edge => {
            const src = parseInt(edge.data('source'));
            const tgt = parseInt(edge.data('target'));
            if (activeNids.has(src) || activeNids.has(tgt)) {
                edge.removeClass('witness-inactive');
            }
        });

        // Un-dim operand neighbors of active nodes
        for (const nid of activeNids) {
            const cyNode = this.cy.getElementById(String(nid));
            if (cyNode.length) {
                const node = this.modelNodes.get(nid);
                if (node) {
                    for (const opNid of node.operands) {
                        const opNode = this.cy.getElementById(String(opNid));
                        if (opNode.length) {
                            opNode.removeClass('witness-inactive');
                        }
                    }
                }
            }
        }
    }

    _makeLabel(cyNode, nid) {
        const node = this.modelNodes.get(nid);
        if (!node) return '';
        const base = node.name || node.op;
        const val = cyNode.data('witnessValue');
        if (val) {
            return `${node.op}\n${node.name || ''}\n= ${val}`;
        }
        return base;
    }
}
