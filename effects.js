/* ============================================================
   RED DIRT RHYTHM STUDIO — REAL-TIME EFFECTS RACK v1.0
   40 World-Class Effects named after TX / OK towns
   ============================================================ */

const EffectsRack = (() => {
  const ctx = (window.audioCtx || new (window.AudioContext || window.webkitAudioContext)());

// ---- Base Effect Class ----
   class RackEffect {
     constructor(id, name, category) {
       this.id = id;
       this.name = name;
       this.category = category;
       this.bypass = false;
       this.enabled = true;
       this.params = {};
       this.inputNode = ctx.createGain();
       this.outputNode = ctx.createGain();
       this.dryGain = ctx.createGain();
       this.wetGain = ctx.createGain();
       this.chain = [];
     }
     setParam(key, val) { this.params[key].value = val; this.refresh(); }
     refresh() {}
     applyPreset(presetName) {
       const preset = this.presets?.[presetName];
       if (preset) {
         Object.assign(this.params, preset.params);
         this.refresh();
       }
     }
     connect(input, output) {
       if (this.bypass) {
         input.connect(output);
       } else {
         input.connect(this.inputNode);
         let prev = this.inputNode;
         for (const node of this.chain) {
           prev.connect(node);
           prev = node;
         }
         prev.connect(this.outputNode);
         this.outputNode.connect(output);
       }
     }
     toggleBypass() {
       this.bypass = !this.bypass;
       this.inputNode.disconnect();
       this.outputNode.disconnect();
     }
   }

  // ---- Clyde-Verb (Plate Reverb) ----
   class ClydeVerb extends RackEffect {
     constructor() {
       super('clyde-verb', 'Clyde-Verb', 'Spatial');
       this.params = {
         decay:      { value: 2.8, min: 0.1, max: 8, unit: 's' },
         preDelay:   { value: 25,  min: 0, max: 200, unit: 'ms' },
         damping:    { value: 45,  min: 0, max: 100, unit: '%' },
         size:       { value: 70,  min: 0, max: 100, unit: '%' },
         mix:        { value: 35,  min: 0, max: 100, unit: '%' },
         modRate:    { value: 0.5, min: 0, max: 10, unit: 'Hz' },
       };
       this.inputNode = ctx.createGain();
       this.outputNode = ctx.createGain();
       this.dryGain = ctx.createGain();
       this.wetGain = ctx.createGain();
       this.preDelay = ctx.createDelay(0.3);
       this.preDelay.delayTime.value = 0.025;
       this.tankL = ctx.createDelay(2);
       this.tankL.delayTime.value = 0.035;
       this.tankR = ctx.createDelay(2);
       this.tankR.delayTime.value = 0.041;
       this.filterL = ctx.createBiquadFilter();
       this.filterL.type = 'lowpass';
       this.filterL.frequency.value = 4500;
       this.filterR = ctx.createBiquadFilter();
       this.filterR.type = 'lowpass';
       this.filterR.frequency.value = 4400;
       this.feedbackL = ctx.createGain();
       this.feedbackL.gain.value = 0.5;
       this.feedbackR = ctx.createGain();
       this.feedbackR.gain.value = 0.48;
       this.lfo = ctx.createOscillator();
       this.lfo.frequency.value = 0.5;
       this.lfo.type = 'sine';
       this.lfoGain = ctx.createGain();
       this.lfoGain.gain.value = 0.005;
       this.lfo.connect(this.lfoGain);
       this.lfoGain.connect(this.tankL.delayTime);
       this.lfoGain.connect(this.tankR.delayTime);
       this.lfo.start();
       this.chain = [];
this.chain.push(this.preDelay, this.tankL, this.filterL, this.feedbackL);
        this.chain.push(this.tankR, this.filterR, this.feedbackR, this.wetGain);
        // Connect wet/dry mix to output
        this.dryGain.connect(this.outputNode);
        this.wetGain.connect(this.outputNode);
        this.presets = {
         'Bright Plate': { params: { decay: 2.5, preDelay: 15, damping: 35, size: 80, mix: 30, modRate: 0.3 }},
         'Dark Room': { params: { decay: 1.8, preDelay: 35, damping: 65, size: 50, mix: 25, modRate: 0.7 }},
         'Infinite Spring': { params: { decay: 6.0, preDelay: 20, damping: 40, size: 90, mix: 35, modRate: 0.4 }},
         'Vocal Chamber': { params: { decay: 1.5, preDelay: 10, damping: 55, size: 45, mix: 22, modRate: 0.2 }},
         'Ambient Wash': { params: { decay: 4.5, preDelay: 50, damping: 30, size: 85, mix: 40, modRate: 0.6 }}
       };
     }
     setParam(key, val) { this.params[key].value = val; this.refresh(); }
     refresh() {
       const p = this.params;
       this.preDelay.delayTime.value = p.preDelay.value / 1000;
       this.tankL.delayTime.value = 0.035 + (p.size.value / 100) * 0.02;
       this.tankR.delayTime.value = 0.041 + (p.size.value / 100) * 0.022;
       this.feedbackL.gain.value = (p.decay.value / 5) * 0.45;
       this.feedbackR.gain.value = (p.decay.value / 5) * 0.45;
       this.filterL.frequency.value = 5000 - (p.damping.value / 100) * 3000;
       this.filterR.frequency.value = this.filterL.frequency.value - 100;
       this.lfo.frequency.value = p.modRate.value;
       this.wetGain.gain.value = p.mix.value / 100;
       this.dryGain.gain.value = 1 - p.mix.value / 100;
     }
     toggleBypass(state) {
       this.bypass = !state;
       if (this.bypass) {
         this.inputNode.disconnect();
         this.inputNode.connect(this.outputNode);
       } else {
         this.inputNode.disconnect();
         this.inputNode.connect(this.preDelay);
       }
     }
   }

   // ---- Stamford-Delay (Delay) ----
  class StamfordDelay extends RackEffect {
    constructor() {
      super('stamford-delay', 'Stamford-Delay', 'Time');
      this.params = {
        time:       { value: 420, min: 1,   max: 2000, unit: 'ms' },
        feedback:   { value: 38,  min: 0,   max: 95,   unit: '%' },
        filter:     { value: 45,  min: 200, max: 8000, unit: 'Hz' },
        mix:        { value: 28,  min: 0,   max: 100,  unit: '%' },
        pingPong:   { value: 0,   min: 0,   max: 1,    unit: '' },
        tempoSync:  { value: 0,   min: 0,   max: 1,    unit: '' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.delayL = ctx.createDelay(3);
      this.delayL.delayTime.value = 0.42;
      this.delayR = ctx.createDelay(3);
      this.delayR.delayTime.value = 0.46;
      this.feedbackGain = ctx.createGain();
      this.feedbackGain.gain.value = 0.38;
      this.filterL = ctx.createBiquadFilter();
      this.filterL.type = 'lowpass';
      this.filterL.frequency.value = 3500;
      this.filterR = ctx.createBiquadFilter();
      this.filterR.type = 'lowpass';
      this.filterR.frequency.value = 3500;
      this.chain.push(this.delayL, this.filterL, this.feedbackGain);
      this.chain.push(this.delayR, this.filterR);
      this.chain.push(this.wetGain, this.dryGain, this.outputNode);    
      this.presets = {
        'Slap Bass': { params: { time: 120, feedback: 28, filter: 5000, mix: 22, pingPong: 0, tempoSync: 0 }},
        'Stereo Ping': { params: { time: 380, feedback: 35, filter: 3500, mix: 34, pingPong: 1, tempoSync: 0 }},
        'Feedback Echo': { params: { time: 620, feedback: 68, filter: 2200, mix: 38, pingPong: 0, tempoSync: 0 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.delayL.delayTime.value = p.time.value / 1000;
      this.delayR.delayTime.value = (p.time.value + 20) / 1000;
      this.feedbackGain.gain.value = p.feedback.value / 100 * 0.85;
      this.filterL.frequency.value = p.filter.value;
      this.filterR.frequency.value = p.filter.value;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100 * 0.3;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.chain[1]);
      }
    }
  }

  // ---- Quanah-Com (Compressor) ----
  class QuanahCom extends RackEffect {
    constructor() {
      super('quanah-com', 'Quanah-Com', 'Dynamics');
      this.params = {
        threshold:  { value: -18, min: -60, max: 0, unit: 'dB' },
        ratio:      { value: 4,   min: 1,   max: 20, unit: ':1' },
        attack:     { value: 12,  min: 0,   max: 200, unit: 'ms' },
        release:    { value: 145, min: 5,   max: 1000, unit: 'ms' },
        makeup:     { value: 6,   min: -24, max: 24, unit: 'dB' },
        knee:       { value: 30,  min: 0,   max: 100, unit: '%' },
        mix:        { value: 100, min: 0,   max: 100, unit: '%' },
      };
      this.comp = ctx.createDynamicsCompressor();
      this.comp.threshold.value = -18;
      this.comp.ratio.value = 4;
      this.comp.attack.value = 0.012;
      this.comp.release.value = 0.145;
      this.makeupGain = ctx.createGain();
      this.makeupGain.gain.value = 1.5;
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.mixBus = ctx.createGain();
      this.chain.push(this.comp, this.makeupGain, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);    
      this.presets = {
        'Drums Punch': { params: { threshold: -22, ratio: 6, attack: 3, release: 60, makeup: 4, knee: 20, mix: 100 }},
        'Vocal Glue': { params: { threshold: -16, ratio: 3, attack: 15, release: 180, makeup: 2, knee: 45, mix: 100 }},
        'Parallel Squash': { params: { threshold: -25, ratio: 8, attack: 2, release: 40, makeup: 6, knee: 10, mix: 70 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.comp.threshold.value = p.threshold.value;
      this.comp.ratio.value = p.ratio.value;
      this.comp.attack.value = p.attack.value / 1000;
      this.comp.release.value = p.release.value / 1000;
      this.comp.knee.value = p.knee.value;
      const dB = p.makeup.value;
      this.makeupGain.gain.value = Math.pow(10, dB / 20);
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.chain[1]);
      }
    }
  }

  // ---- Electra-EQ (3-Band Parametric EQ) ----
  class ElectraEQ extends RackEffect {
    constructor() {
      super('electra-eq', 'Electra-EQ', 'Tone');
      this.params = {
        lowGain:    { value: 4,   min: -24, max: 24, unit: 'dB' },
        lowFreq:    { value: 90,  min: 20,  max: 500, unit: 'Hz' },
        midGain:    { value: 0,   min: -24, max: 24, unit: 'dB' },
        midFreq:    { value: 1200, min: 100, max: 6000, unit: 'Hz' },
        midQ:       { value: 1.5, min: 0.1, max: 12, unit: 'Q' },
        highGain:   { value: 3,   min: -24, max: 24, unit: 'dB' },
        highFreq:   { value: 7000, min: 2000, max: 18000, unit: 'Hz' },
        output:     { value: 0,   min: -12, max: 12, unit: 'dB' },
      };
      this.lowShelf = ctx.createBiquadFilter();
      this.lowShelf.type = 'lowshelf';
      this.midPeak = ctx.createBiquadFilter();
      this.midPeak.type = 'peaking';
      this.highShelf = ctx.createBiquadFilter();
      this.highShelf.type = 'highshelf';
      this.outputGain = ctx.createGain();
      this.chain.push(this.lowShelf, this.midPeak, this.highShelf, this.outputGain, this.outputNode);
    
      this.presets = {
        'Vocal Bright': { params: { lowGain: 2, lowFreq: 100, midGain: -2, midFreq: 2500, midQ: 1.5, highGain: 4, highFreq: 8000, output: 1 }},
        'Bass Boost': { params: { lowGain: 8, lowFreq: 80, midGain: -3, midFreq: 800, midQ: 1, highGain: 2, highFreq: 10000, output: 0 }},
        'Guitar Scoop': { params: { lowGain: 3, lowFreq: 120, midGain: -6, midFreq: 1800, midQ: 2, highGain: 5, highFreq: 7000, output: 2 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.lowShelf.frequency.value = p.lowFreq.value;
      this.lowShelf.gain.value = p.lowGain.value;
      this.midPeak.frequency.value = p.midFreq.value;
      this.midPeak.gain.value = p.midGain.value;
      this.midPeak.Q.value = p.midQ.value;
      this.highShelf.frequency.value = p.highFreq.value;
      this.highShelf.gain.value = p.highGain.value;
      this.outputGain.gain.value = Math.pow(10, p.output.value / 20);
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.lowShelf);
      }
    }
  }

  // ---- Albany-Limit (Limiter) ----
  class AlbanyLimit extends RackEffect {
    constructor() {
      super('albany-limit', 'Albany-Limit', 'Dynamics');
      this.params = {
        threshold: { value: -1.5, min: -30, max: 0, unit: 'dB' },
        ceiling:   { value: -0.3, min: -12, max: 0, unit: 'dB' },
        release:   { value: 80,  min: 5,   max: 1000, unit: 'ms' },
        gain:      { value: 0,   min: -12, max: 24, unit: 'dB' },
      };
      this.limiter = ctx.createDynamicsCompressor();
      this.limiter.threshold.value = -1.5;
      this.limiter.ratio.value = 20;
      this.limiter.attack.value = 0.001;
      this.limiter.release.value = 0.08;
      this.makeup = ctx.createGain();
      this.makeup.gain.value = 1;
      this.chain.push(this.limiter, this.makeup, this.outputNode);    
      this.presets = {
        'Brick Wall': { params: { threshold: -0.5, ceiling: -0.3, release: 30, gain: 0 }},
        'Gentle Ceiling': { params: { threshold: -6, ceiling: -1, release: 120, gain: 2 }},
        'Pumping Limit': { params: { threshold: -4, ceiling: -1.2, release: 50, gain: 4 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.limiter.threshold.value = p.threshold.value;
      this.limiter.release.value = p.release.value / 1000;
      this.makeup.gain.value = Math.pow(10, p.gain.value / 20);
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.limiter);
      }
    }
  }

  // ---- Goltry-Gate (Noise Gate) ----
  class GoltryGate extends RackEffect {
    constructor() {
      super('goltry-gate', 'Goltry-Gate', 'Dynamics');
      this.params = {
        threshold: { value: -42, min: -80, max: 0, unit: 'dB' },
        attack:    { value: 0.5, min: 0,  max: 100, unit: 'ms' },
        release:   { value: 80,  min: 5,  max: 1000, unit: 'ms' },
        hold:      { value: 20,  min: 0,  max: 1000, unit: 'ms' },
        range:     { value: 54,  min: 0,   max: 100, unit: 'dB' },
      };
      this.gate = ctx.createDynamicsCompressor();
      this.gate.threshold.value = -42;
      this.gate.ratio.value = 20;
      this.gate.attack.value = 0.0005;
      this.gate.release.value = 0.08;
      this.chain.push(this.gate, this.outputNode);    
      this.presets = {
        'Silence Is Golden': { params: { threshold: -62, attack: 0, release: 60, hold: 30, range: 90 }},
        'Drum Gate': { params: { threshold: -48, attack: 1, release: 80, hold: 50, range: 70 }},
        'Tight Vocal': { params: { threshold: -42, attack: 2, release: 120, hold: 10, range: 65 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.gate.threshold.value = p.threshold.value;
      this.gate.attack.value = p.attack.value / 1000;
      this.gate.release.value = p.release.value / 1000;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.gate);
      }
    }
  }

  // ---- Cisco-Chorus (Chorus) ----
  class CiscoChorus extends RackEffect {
    constructor() {
      super('cisco-chorus', 'Cisco-Chorus', 'Modulation');
      this.params = {
        rate:    { value: 0.7, min: 0.1, max: 8, unit: 'Hz' },
        depth:   { value: 45,  min: 0,   max: 100, unit: '%' },
        delay:   { value: 25,  min: 1,   max: 50, unit: 'ms' },
        mix:     { value: 40,  min: 0,   max: 100, unit: '%' },
        width:   { value: 50,  min: 0,   max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.delayL = ctx.createDelay(0.1);
      this.delayL.delayTime.value = 0.025;
      this.delayR = ctx.createDelay(0.1);
      this.delayR.delayTime.value = 0.030;
      this.lfoL = ctx.createOscillator();
      this.lfoL.frequency.value = 0.7;
      this.lfoL.type = 'sine';
      this.lfoLGain = ctx.createGain();
      this.lfoLGain.gain.value = 0.015;
      this.lfoR = ctx.createOscillator();
      this.lfoR.frequency.value = 0.7;
      this.lfoR.type = 'sine';
      this.lfoRGain = ctx.createGain();
      this.lfoRGain.gain.value = 0.018;
      this.lfoL.connect(this.lfoLGain);
      this.lfoLGain.connect(this.delayL.delayTime);
      this.lfoR.connect(this.lfoRGain);
      this.lfoRGain.connect(this.delayR.delayTime);
      this.lfoL.start();
      this.lfoR.start();
      this.chain.push(this.delayL, this.delayR, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Slow Wide': { params: { rate: 0.45, depth: 65, delay: 28, mix: 48, width: 80 }},
        'Thick Layer': { params: { rate: 0.7, depth: 55, delay: 22, mix: 55, width: 70 }},
        'Subtle Air': { params: { rate: 0.55, depth: 30, delay: 20, mix: 28, width: 40 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.lfoL.frequency.value = p.rate.value;
      this.lfoR.frequency.value = p.rate.value * 1.02;
      this.lfoLGain.gain.value = (p.depth.value / 100) * 0.02;
      this.lfoRGain.gain.value = (p.depth.value / 100) * 0.022;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100 * 0.4;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.chain[1]);
      }
    }
  }

  // ---- Haskell-Flange (Flanger) ----
  class HaskellFlange extends RackEffect {
    constructor() {
      super('haskell-flange', 'Haskell-Flange', 'Modulation');
      this.params = {
        rate:      { value: 0.35, min: 0.01, max: 5, unit: 'Hz' },
        depth:     { value: 55,   min: 0,    max: 100, unit: '%' },
        feedback:  { value: 45,   min: 0,    max: 95,  unit: '%' },
        mix:       { value: 35,   min: 0,    max: 100, unit: '%' },
        manual:    { value: 6,    min: 0,    max: 20,  unit: 'ms' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.delay = ctx.createDelay(0.05);
      this.delay.delayTime.value = 0.006;
      this.lfo = ctx.createOscillator();
      this.lfo.frequency.value = 0.35;
      this.lfo.type = 'sine';
      this.lfoGain = ctx.createGain();
      this.lfoGain.gain.value = 0.005;
      this.feedbackGain = ctx.createGain();
      this.feedbackGain.gain.value = 0.45;
      this.lfo.connect(this.lfoGain);
      this.lfoGain.connect(this.delay.delayTime);
      this.lfo.start();
      this.chain.push(this.delay, this.feedbackGain, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Through-Zero': { params: { rate: 0.25, depth: 80, feedback: 50, mix: 40, manual: 8 }},
        'Jet Plane': { params: { rate: 0.4, depth: 70, feedback: 60, mix: 38, manual: 5 }},
        'Subtle Sparkle': { params: { rate: 0.2, depth: 30, feedback: 25, mix: 22, manual: 12 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.lfo.frequency.value = p.rate.value;
      this.lfoGain.gain.value = (p.depth.value / 100) * (p.manual.value / 1000);
      this.feedbackGain.gain.value = p.feedback.value / 100 * 0.7;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100 * 0.3;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.delay);
      }
    }
  }

  // ---- Breck-Phase (Phaser) ----
  class BreckPhase extends RackEffect {
    constructor() {
      super('breck-phase', 'Breck-Phase', 'Modulation');
      this.params = {
        rate:    { value: 0.6, min: 0.01, max: 6, unit: 'Hz' },
        depth:   { value: 60,  min: 0,    max: 100, unit: '%' },
        stages:  { value: 4,   min: 2,    max: 16, unit: '' },
        feedback:{ value: 40,  min: 0,    max: 95,  unit: '%' },
        mix:     { value: 50,  min: 0,    max: 100, unit: '%' },
        spread:  { value: 50,  min: 0,    max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.stageNodes = [];
      for (let i = 0; i < 16; i++) {
        const ap = ctx.createBiquadFilter();
        ap.type = 'allpass';
        ap.frequency.value = 300 + i * 200;
        ap.Q.value = 10;
        this.stageNodes.push(ap);
      }
      this.feedbackGainNode = ctx.createGain();
      this.feedbackGainNode.gain.value = 0.4;
      this.lfo = ctx.createOscillator();
      this.lfo.type = 'sine';
      this.lfo.frequency.value = 0.6;
      this.lfoGain = ctx.createGain();
      this.lfoGain.gain.value = 150;
      this.lfo.connect(this.lfoGain);
      this.lfo.start();
      let prev = this.inputNode;
      this.stageNodes.forEach((node, i) => {
        node.frequency.value = 300 + i * 200;
        prev.connect(node);
        prev = node;
      });
      prev.connect(this.outputNode);
    
      this.presets = {
        'Vintage Sweep': { params: { rate: 0.4, depth: 70, stages: 6, feedback: 45, mix: 50, spread: 55 }},
        'Sharp Notch': { params: { rate: 1.2, depth: 55, stages: 4, feedback: 60, mix: 48, spread: 40 }},
        'Subtle Shimmer': { params: { rate: 0.2, depth: 35, stages: 8, feedback: 30, mix: 32, spread: 70 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      const activeStages = Math.round(p.stages.value);
      const baseFreq = 300 + (p.spread.value / 100) * 2500;
      for (let i = 0; i < 16; i++) {
        if (i < activeStages) {
          this.stageNodes[i].frequency.value = baseFreq + i * 180;
          this.stageNodes[i].Q.value = 5 + p.depth.value / 20;
        } else {
          this.stageNodes[i].frequency.value = 300;
          this.stageNodes[i].Q.value = 0;
        }
      }
      this.lfo.frequency.value = p.rate.value;
      this.lfoGain.gain.value = (p.depth.value / 100) * 200;
      this.feedbackGainNode.gain.value = p.feedback.value / 100 * 0.7;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.stageNodes[0]);
      }
    }
  }

  // ---- Funston-Clip (Distortion) ----
  class FunstonClip extends RackEffect {
    constructor() {
      super('funston-clip', 'Funston-Clip', 'Drive');
      this.params = {
        drive:    { value: 55,  min: 0,   max: 100, unit: '%' },
        tone:     { value: 50,  min: 0,   max: 100, unit: '%' },
        mix:      { value: 60,  min: 0,   max: 100, unit: '%' },
        output:   { value: 70,  min: 0,   max: 100, unit: '%' },
        character:{ value: 50,  min: 0,   max: 100, unit: '%' },
      };
      this.waveshaper = ctx.createWaveShaper();
      this.waveshaper.oversample = '4x';
      this.toneFilter = ctx.createBiquadFilter();
      this.toneFilter.type = 'lowpass';
      this.toneFilter.frequency.value = 3000;
      this.inputTrim = ctx.createGain();
      this.outputTrim = ctx.createGain();
      this.mixBus = ctx.createGain();
      this.makeup = ctx.createGain();
      this.chain.push(this.waveshaper, this.toneFilter, this.outputTrim, this.makeup);
    
      this.presets = {
        'Light Crunch': { params: { drive: 30, tone: 65, mix: 55, output: 65, character: 35 }},
        'Metal Riff': { params: { drive: 85, tone: 40, mix: 75, output: 80, character: 85 }},
        'Clean Boost': { params: { drive: 15, tone: 80, mix: 90, output: 70, character: 10 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    makeCurve(drive) {
      const samples = 44100;
      const curve = new Float32Array(samples);
      const k = (drive / 100) * 200;
      for (let i = 0; i < samples; i++) {
        const x = (i * 2) / samples - 1;
        curve[i] = ((3 + k) * x * 20 * (Math.PI + k)) / (20 * Math.PI + k * Math.abs(x));
      }
      return curve;
    }
    refresh() {
      const p = this.params;
      this.waveshaper.curve = this.makeCurve(p.drive.value);
      this.toneFilter.frequency.value = 600 + p.tone.value * 50;
      this.outputTrim.gain.value = (p.output.value / 100) * 1.4;
      this.makeup.gain.value = Math.pow(10, (p.drive.value / 100 - 0.5) * 0.4);
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.inputTrim);
      }
    }
  }

  // ---- Wichita-Wah (Auto-Wah) ----
  class WichitaWah extends RackEffect {
    constructor() {
      super('wichita-wah', 'Wichita-Wah', 'Filter');
      this.params = {
        threshold: { value: 35, min: 0, max: 100, unit: '%' },
        attack:    { value: 8,  min: 0, max: 200, unit: 'ms' },
        release:   { value: 120, min: 10, max: 1000, unit: 'ms' },
        resonance: { value: 65, min: 1, max: 30, unit: 'Q' },
        mix:       { value: 80, min: 0, max: 100, unit: '%' },
      };
      this.envelope = ctx.createGain();
      this.envelope.gain.value = 0;
      this.filter = ctx.createBiquadFilter();
      this.filter.type = 'bandpass';
      this.filter.frequency.value = 800;
      this.filter.Q.value = 6;
      this.resonance = ctx.createGain();
      this.resonance.gain.value = 1;
      this.smooth = ctx.createBiquadFilter();
      this.smooth.type = 'lowpass';
      this.smooth.frequency.value = 30;
      this.chain.push(this.envelope, this.smooth, this.filter, this.outputNode);
    
      this.presets = {
        'Funky Dip': { params: { threshold: 40, attack: 6, release: 90, resonance: 14, mix: 85 }},
        'Subtle Quack': { params: { threshold: 25, attack: 12, release: 140, resonance: 9, mix: 60 }},
        'Aggressive Sweep': { params: { threshold: 55, attack: 3, release: 60, resonance: 20, mix: 80 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.filter.Q.value = p.resonance.value;
      this.smooth.frequency.value = 1000 / (p.attack.value + p.release.value);
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.envelope);
      }
    }
  }

  // ---- Throck-Trem (Tremolo) ----
  class ThrockTrem extends RackEffect {
    constructor() {
      super('throck-trem', 'Throck-Trem', 'Modulation');
      this.params = {
        rate:    { value: 4.5, min: 0.1, max: 10, unit: 'Hz' },
        depth:   { value: 60,  min: 0,   max: 100, unit: '%' },
        shape:   { value: 0,   min: 0,   max: 2,  unit: '' },
        mix:     { value: 100, min: 0,   max: 100, unit: '%' },
      };
      this.lfo = ctx.createOscillator();
      this.lfo.frequency.value = 4.5;
      this.lfo.type = 'sine';
      this.lfoGain = ctx.createGain();
      this.lfoGain.gain.value = 0.6;
      this.tremoloGain = ctx.createGain();
      this.tremoloGain.gain.value = 1;
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.lfo.connect(this.lfoGain);
      this.lfoGain.connect(this.tremoloGain.gain);
      this.lfo.start();
      this.chain.push(this.tremoloGain, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Slow Wash': { params: { rate: 2.2, depth: 45, shape: 0, mix: 90 }},
        'Hard Chop': { params: { rate: 6.5, depth: 80, shape: 2, mix: 100 }},
        'Subtle Pulse': { params: { rate: 3.8, depth: 30, shape: 1, mix: 70 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.lfo.frequency.value = p.rate.value;
      this.lfoGain.gain.value = (p.depth.value / 100) * 0.5;
      // Shape affects LFO waveform
      const shapes = ['sine', 'triangle', 'square'];
      this.lfo.type = shapes[Math.min(Math.floor(p.shape.value), 2)];
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.tremoloGain);
      }
    }
  }

  // ---- Paducah-Shift (Pitch Shifter) ----
  class PaducahShift extends RackEffect {
    constructor() {
      super('paducah-shift', 'Paducah-Shift', 'Pitch');
      this.params = {
        semitones: { value: 0,  min: -12, max: 12, unit: 'st' },
        fine:      { value: 0,  min: -50, max: 50, unit: 'ct' },
        mix:       { value: 50, min: 0,   max: 100, unit: '%' },
        delay:     { value: 18, min: 1,   max: 80, unit: 'ms' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.delayL = ctx.createDelay(0.1);
      this.delayL.delayTime.value = 0.018;
      this.delayR = ctx.createDelay(0.1);
      this.delayR.delayTime.value = 0.022;
      this.feedbackGain = ctx.createGain();
      this.feedbackGain.gain.value = 0.4;
      this.chain.push(this.delayL, this.delayR, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Octave Up': { params: { semitones: 12, fine: 0, mix: 45, delay: 20 }},
        'Fifth Harmony': { params: { semitones: 7, fine: 0, mix: 40, delay: 25 }},
        'Thick Detune': { params: { semitones: 0, fine: 12, mix: 50, delay: 18 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      const totalShift = p.semitones.value + (p.fine.value / 100);
      const ratio = Math.pow(2, totalShift / 12);
      this.delayL.delayTime.value = p.delay.value / 1000 / ratio;
      this.delayR.delayTime.value = (p.delay.value + 4) / 1000 / ratio;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.delayL);
      }
    }
  }

  // ---- Frede-Filter (Sweep Filter) ----
  class FredeFilter extends RackEffect {
    constructor() {
      super('frede-filter', 'Frede-Filter', 'Filter');
      this.params = {
        cutoff:    { value: 4500, min: 20,   max: 18000, unit: 'Hz' },
        resonance: { value: 8,   min: 0.1,  max: 30,    unit: 'Q' },
        type:      { value: 0,   min: 0,    max: 4,     unit: '' },
        lfoRate:   { value: 0.3, min: 0.01, max: 8,    unit: 'Hz' },
        lfoDepth:  { value: 40,  min: 0,    max: 100,   unit: '%' },
        lfoShape:  { value: 0,   min: 0,    max: 3,     unit: '' },
      };
      this.filter = ctx.createBiquadFilter();
      this.filter.type = 'lowpass';
      this.filter.frequency.value = 4500;
      this.filter.Q.value = 8;
      this.lfo = ctx.createOscillator();
      this.lfo.frequency.value = 0.3;
      this.lfo.type = 'sine';
      this.lfoGain = ctx.createGain();
      this.lfoGain.gain.value = 1500;
      this.lfo.connect(this.lfoGain);
      this.lfoGain.connect(this.filter.frequency);
      this.lfo.start();
      this.chain.push(this.filter, this.outputNode);
    
      this.presets = {
        'Auto-Wah': { params: { cutoff: 3000, resonance: 14, type: 2, lfoRate: 2.5, lfoDepth: 70, lfoShape: 0 }},
        'Resonant Peak': { params: { cutoff: 1800, resonance: 28, type: 4, lfoRate: 0.1, lfoDepth: 0, lfoShape: 0 }},
        'Mood Sweep': { params: { cutoff: 5000, resonance: 8, type: 0, lfoRate: 0.15, lfoDepth: 55, lfoShape: 1 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      const filterTypes = ['lowpass', 'highpass', 'bandpass', 'notch', 'peaking'];
      this.filter.type = filterTypes[Math.min(Math.floor(p.type.value), 4)];
      this.filter.Q.value = p.resonance.value;
      this.filter.frequency.value = p.cutoff.value;
      this.lfo.frequency.value = p.lfoRate.value;
      const shapes = ['sine', 'triangle', 'sawtooth', 'square'];
      this.lfo.type = shapes[Math.min(Math.floor(p.lfoShape.value), 3)];
      this.lfoGain.gain.value = (p.lfoDepth.value / 100) * p.cutoff.value * 0.8;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.filter);
      }
    }
  }

  // ---- Munday-Drive (Overdrive) ----
  class MundayDrive extends RackEffect {
    constructor() {
      super('munday-drive', 'Munday-Drive', 'Drive');
      this.params = {
        drive:     { value: 40,  min: 0,   max: 100, unit: '%' },
        tone:      { value: 55,  min: 0,   max: 100, unit: '%' },
        level:     { value: 70,  min: 0,   max: 100, unit: '%' },
        character: { value: 30,  min: 0,   max: 100, unit: '%' },
      };
      this.waveshaper = ctx.createWaveShaper();
      this.waveshaper.oversample = '4x';
      this.toneFilter = ctx.createBiquadFilter();
      this.toneFilter.type = 'lowpass';
      this.toneFilter.frequency.value = 2500;
      this.outputGain = ctx.createGain();
      this.chain.push(this.waveshaper, this.toneFilter, this.outputGain, this.outputNode);
    
      this.presets = {
        'Clean Boost': { params: { drive: 18, tone: 75, level: 72, character: 10 }},
        'Blues Crunch': { params: { drive: 48, tone: 60, level: 68, character: 45 }},
        'High Gain': { params: { drive: 82, tone: 42, level: 62, character: 80 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    makeOverdriveCurve(drive) {
      const samples = 44100;
      const curve = new Float32Array(samples);
      const k = drive * 0.8;
      for (let i = 0; i < samples; i++) {
        const x = (i * 2) / samples - 1;
        curve[i] = (Math.PI + k) * x / (Math.PI + k * Math.abs(x));
      }
      return curve;
    }
    refresh() {
      const p = this.params;
      this.waveshaper.curve = this.makeOverdriveCurve(p.drive.value / 100);
      this.toneFilter.frequency.value = 400 + p.tone.value * 45;
      this.outputGain.gain.value = p.level.value / 100 * 1.6;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.waveshaper);
      }
    }
  }

  // ---- Olney-Amp (Guitar Amp Simulator) ----
  class OlneyAmp extends RackEffect {
    constructor() {
      super('olney-amp', 'Olney-Amp', 'Amp');
      this.params = {
        gain:        { value: 50, min: 0,  max: 100, unit: '%' },
        bass:        { value: 55, min: 0,  max: 100, unit: '%' },
        mid:         { value: 50, min: 0,  max: 100, unit: '%' },
        treble:      { value: 55, min: 0,  max: 100, unit: '%' },
        volume:      { value: 65, min: 0,  max: 100, unit: '%' },
        cabinet:     { value: 0,  min: 0,  max: 3,  unit: '' },
        presence:    { value: 40, min: 0,  max: 100, unit: '%' },
      };
      this.driveGain = ctx.createGain();
      this.driveGain.gain.value = 8;
      this.waveshaper = ctx.createWaveShaper();
      this.waveshaper.oversample = '4x';
      this.chassis = ctx.createBiquadFilter();
      this.chassis.type = 'bandpass';
      this.chassis.frequency.value = 800;
      this.chassis.Q.value = 0.7;
      this.toneStack = ctx.createBiquadFilter();
      this.toneStack.type = 'peaking';
      this.toneStack.frequency.value = 500;
      this.toneStack.gain.value = 6;
      this.presenceFilter = ctx.createBiquadFilter();
      this.presenceFilter.type = 'highshelf';
      this.presenceFilter.frequency.value = 3500;
      this.outputGain = ctx.createGain();
      this.chain.push(this.driveGain, this.waveshaper, this.chassis, this.toneStack, this.presenceFilter, this.outputGain, this.outputNode);
    
      this.presets = {
        'Clean Chime': { params: { gain: 22, bass: 45, mid: 55, treble: 62, volume: 55, cabinet: 0, presence: 35 }},
        'British Stack': { params: { gain: 55, bass: 65, mid: 50, treble: 60, volume: 62, cabinet: 1, presence: 50 }},
        'High Gain Metal': { params: { gain: 85, bass: 75, mid: 42, treble: 68, volume: 65, cabinet: 2, presence: 65 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    makeAmpCurve(gain) {
      const samples = 44100;
      const curve = new Float32Array(samples);
      const k = gain * 2.5;
      for (let i = 0; i < samples; i++) {
        const x = (i * 2) / samples - 1;
        curve[i] = (1 + k) * x / (1 + Math.PI / 2 * k * Math.abs(x));
      }
      return curve;
    }
    refresh() {
      const p = this.params;
      this.waveshaper.curve = this.makeAmpCurve(p.gain.value / 100);
      this.driveGain.gain.value = 1 + (p.gain.value / 100) * 6;
      this.toneStack.gain.value = (p.mid.value - 50) * 0.4;
      this.toneStack.frequency.value = 300 + (p.bass.value / 100) * 1200;
      this.presenceFilter.gain.value = (p.presence.value - 50) * 0.5;
      this.outputGain.gain.value = p.volume.value / 100 * 1.4;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.driveGain);
      }
    }
  }

  // ========================================
  // PHASE 2 — 15 ADDITIONAL TOWN-NAMED EFFECTS
  // ========================================

  // ---- Baird-Expander (Expander / Noise Reduction) ----
  class BairdExpander extends RackEffect {
    constructor() {
      super('baird-expand', 'Baird-Expand', 'Dynamics');
      this.params = {
        threshold: { value: -52, min: -80, max: 0, unit: 'dB' },
        ratio:     { value: 2.5, min: 1,  max: 8, unit: ':1' },
        attack:    { value: 3,  min: 0,  max: 50, unit: 'ms' },
        release:   { value: 140, min: 5,  max: 1000, unit: 'ms' },
        gain:      { value: 12, min: -24, max: 24, unit: 'dB' },
        hold:      { value: 10, min: 0, max: 2000, unit: 'ms' },
      };      this.expander = ctx.createDynamicsCompressor();
      this.expander.threshold.value = -52;
      this.expander.ratio.value = 2.5;
      this.expander.attack.value = 0.003;
      this.expander.release.value = 0.14;
      this.makeup = ctx.createGain();
      this.makeup.gain.value = Math.pow(10, 12 / 20);
      this.inputGain = ctx.createGain();
      this.inputGain.gain.value = 1;
      this.chain.push(this.inputGain, this.expander, this.makeup, this.outputNode);
    
      this.presets = {
        'Noise Floor': { params: { threshold: -72, ratio: 2, attack: 2, release: 200, gain: 0, hold: 50 }},
        'Gentle Cleanup': { params: { threshold: -52, ratio: 2, attack: 3, release: 140, gain: 12, hold: 10 }},
        'Pulsey Gate': { params: { threshold: -45, ratio: 3, attack: 1, release: 80, gain: 6, hold: 30 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.expander.threshold.value = p.threshold.value;
      this.expander.ratio.value = p.ratio.value;
      this.expander.attack.value = p.attack.value / 1000;
      this.expander.release.value = p.release.value / 1000;
      this.makeup.gain.value = Math.pow(10, p.gain.value / 20);
      this.inputGain.gain.value = Math.pow(10, (p.gain.value / 100 - 0.5) * 0.1);
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.inputGain);
      }
    }
  }

  // ---- Gap-Reverb (Convolution / Algorithmic Plate Reverb) ----
  class GapReverb extends RackEffect {
    constructor() {
      super('gap-reverb', 'Gap-Reverb', 'Spatial');
      this.params = {
        decay:      { value: 2.2, min: 0.1, max: 8, unit: 's' },
        preDelay:   { value: 30,  min: 0, max: 120, unit: 'ms' },
        damping:    { value: 42,  min: 0, max: 100, unit: '%' },
        size:       { value: 68,  min: 0, max: 100, unit: '%' },
        mix:        { value: 32,  min: 0, max: 100, unit: '%' },
        modulation: { value: 25, min: 0, max: 100, unit: '%' },
      };      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.preDelay = ctx.createDelay(0.2);
      this.preDelay.delayTime.value = 0.03;
      this.diffusers = [];
      for (let i = 0; i < 4; i++) {
        const lp = ctx.createBiquadFilter();
        lp.type = 'lowpass';
        lp.frequency.value = 6000 - i * 900;
        const dly = ctx.createDelay(0.5);
        dly.delayTime.value = 0.04 + i * 0.012;
        const g = ctx.createGain();
        g.gain.value = 0.3;
        lp.connect(dly);
        dly.connect(g);
        g.connect(this.wetGain);
        this.diffusers.push({ lp, dly, g });
      }
      this.chain.push(this.preDelay);
      this.chain.push(this.wetGain, this.dryGain, this.outputNode);
    
      this.presets = {
        'Canyon Wash': { params: { decay: 4.5, preDelay: 60, damping: 30, size: 85, mix: 35, modulation: 40 }},
        'Room Glow': { params: { decay: 1, preDelay: 8, damping: 55, size: 35, mix: 28, modulation: 10 }},
        'Infinite Plate': { params: { decay: 7.5, preDelay: 35, damping: 60, size: 90, mix: 30, modulation: 70 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.preDelay.delayTime.value = p.preDelay.value / 1000;
      this.wetGain.gain.value = (p.mix.value / 100) * 1.4;
      this.dryGain.gain.value = 1 - (p.mix.value / 100) * 0.6;
      this.diffusers.forEach((d, i) => {
        d.lp.frequency.value = 8000 - i * 1200 - (p.damping.value / 100) * 5000;
        d.dly.delayTime.value = (0.04 + i * 0.012) * (p.size.value / 100 + 0.2);
        d.g.gain.value = 0.3 * (p.decay.value / 3);
      });
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.preDelay);
      }
    }
  }

  // ---- Hawley-Mod (Modulation Matrix: Chorus + Trem + Flange hybrid) ----
  class HawleyMod extends RackEffect {
    constructor() {
      super('hawley-mod', 'Hawley-Mod', 'Modulation');
      this.params = {
        depth:   { value: 50, min: 0, max: 100, unit: '%' },
        rate:    { value: 0.9, min: 0.05, max: 10, unit: 'Hz' },
        mix:     { value: 45, min: 0, max: 100, unit: '%' },
        width:   { value: 60, min: 0, max: 100, unit: '%' },
        shape:   { value: 0,  min: 0, max: 3,  unit: '' },
      };      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.delayL = ctx.createDelay(0.15);
      this.delayL.delayTime.value = 0.025;
      this.delayR = ctx.createDelay(0.15);
      this.delayR.delayTime.value = 0.032;
      this.lfo1 = ctx.createOscillator();
      this.lfo1.frequency.value = 0.9;
      this.lfo1.type = 'sine';
      this.lfo2 = ctx.createOscillator();
      this.lfo2.frequency.value = 0.93;
      this.lfo2.type = 'triangle';
      this.lfoGain1 = ctx.createGain();
      this.lfoGain1.gain.value = 0.012;
      this.lfoGain2 = ctx.createGain();
      this.lfoGain2.gain.value = 0.015;
      this.lfo1.connect(this.lfoGain1);
      this.lfoGain1.connect(this.delayL.delayTime);
      this.lfo2.connect(this.lfoGain2);
      this.lfoGain2.connect(this.delayR.delayTime);
      this.lfo1.start();
      this.lfo2.start();
      this.chain.push(this.delayL, this.delayR, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Wide Chorus': { params: { depth: 65, rate: 0.8, mix: 50, width: 90, shape: 0 }},
        'Subtle Air': { params: { depth: 20, rate: 0.6, mix: 25, width: 40, shape: 1 }},
        'Vintage Flutter': { params: { depth: 75, rate: 2.2, mix: 55, width: 60, shape: 2 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.lfo1.frequency.value = p.rate.value;
      this.lfo2.frequency.value = p.rate.value * 1.04;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
      this.wetGain.gain.value = p.mix.value / 100;
      const shapes = ['sine', 'triangle', 'sawtooth', 'square'];
      this.lfo1.type = shapes[Math.min(Math.floor(p.shape.value), 3)];
      this.lfo2.type = shapes[Math.min(Math.ceil(p.shape.value), 3)];
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.delayL);
      }
    }
  }

  // ---- DeLeon-Crush (Lo-Fi Bitcrusher + Sample Rate Reduction) ----
   // Note: Uses ScriptProcessorNode for compatibility. Consider migrating to AudioWorklet for better performance.
  class DeLeonCrush extends RackEffect {
    constructor() {
      super('deleon-crush', 'DeLeon-Crush', 'LoFi');
      this.params = {
        bits:     { value: 16, min: 1, max: 16, unit: 'bit' },
        rate:     { value: 44100, min: 1000, max: 48000, unit: 'Hz' },
        jitter:   { value: 0,  min: 0,  max: 100, unit: '%' },
        mix:      { value: 50, min: 0,  max: 100, unit: '%' },
        noise:    { value: 8,  min: 0,  max: 100, unit: '%' },
      };      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.bufferSize = 4096;
      this.scriptNode = ctx.createScriptProcessor(this.bufferSize, 1, 1);
      this.scriptNode.onaudioprocess = (e) => {
        const input = e.inputBuffer.getChannelData(0);
        const output = e.outputBuffer.getChannelData(0);
        const bitCount = Math.max(1, Math.round(this.params.bits.value));
        const step = Math.pow(2, bitCount);
        const rateKHz = Math.max(1, this.params.rate.value);
        const samplesPerHold = Math.max(1, Math.round(44100 / rateKHz));
        const jitter = this.params.jitter.value / 100 * samplesPerHold;
        const noiseAmp = this.params.noise.value / 100 * 0.02;
        for (let i = 0; i < input.length; i++) {
          const holdIndex = Math.floor((i + Math.random() * jitter) / samplesPerHold) * samplesPerHold;
          let s = input[Math.min(holdIndex, input.length - 1)];
          s = Math.round(s * step) / step;
          s += (Math.random() * 2 - 1) * noiseAmp;
          output[i] = s;
        }
      };
      this.chain.push(this.scriptNode, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        '8-Bit Neon': { params: { bits: 8, rate: 11025, jitter: 15, mix: 60, noise: 5 }},
        'VHS Crush': { params: { bits: 6, rate: 14000, jitter: 40, mix: 70, noise: 18 }},
        'Subtle Warmth': { params: { bits: 14, rate: 32000, jitter: 5, mix: 35, noise: 3 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; }
    refresh() {
      const p = this.params;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
toggleBypass(state) {
       this.bypass = !state;
       if (this.bypass) {
         this.inputNode.disconnect();
         this.inputNode.connect(this.outputNode);
       } else {
         this.inputNode.disconnect();
         this.inputNode.connect(this.scriptNode || this.inputNode);
       }
     }
  }

  // ---- Tusc-Filter (Resonant State-Variable Filter w/ Envelope) ----
  class TuscFilter extends RackEffect {
    constructor() {
      super('tusc-filter', 'Tusc-Filter', 'Filter');
      this.params = {
        cutoff:    { value: 2200, min: 20, max: 18000, unit: 'Hz' },
        resonance: { value: 12,  min: 0.1, max: 40, unit: 'Q' },
        type:      { value: 3,  min: 0, max: 5, unit: '' },
        envAmt:    { value: 30, min: -100, max: 100, unit: '%' },
        envAttack: { value: 5,  min: 0, max: 200, unit: 'ms' },
        envRelease:{ value: 120, min: 5, max: 2000, unit: 'ms' },
      };      this.filter = ctx.createBiquadFilter();
      this.filter.type = 'lowpass';
      this.filter.frequency.value = 2200;
      this.filter.Q.value = 12;
      this.envGain = ctx.createGain();
      this.envGain.gain.value = 0;
      this.envSmooth = ctx.createBiquadFilter();
      this.envSmooth.type = 'lowpass';
      this.envSmooth.frequency.value = 60;
      this.chain.push(this.envGain, this.envSmooth, this.filter, this.outputNode);
    
      this.presets = {
        'Sync Sweep': { params: { cutoff: 4000, resonance: 18, type: 0, envAmt: 80, envAttack: 1, envRelease: 180 }},
        'Subtle Shine': { params: { cutoff: 9000, resonance: 3, type: 3, envAmt: 15, envAttack: 20, envRelease: 400 }},
        'Aggressive Notch': { params: { cutoff: 3000, resonance: 35, type: 3, envAmt: -60, envAttack: 2, envRelease: 50 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      const types = ['lowpass', 'highpass', 'bandpass', 'notch', 'peaking', 'allpass'];
      this.filter.type = types[Math.min(Math.floor(p.type.value), 5)];
      this.filter.Q.value = p.resonance.value;
      this.filter.frequency.value = Math.max(20, p.cutoff.value + (p.envAmt.value / 100) * 3000);
      this.envSmooth.frequency.value = 1000 / (p.envAttack.value + p.envRelease.value);
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.envGain);
      }
    }
  }

  // ---- Merkel-Multi (Multi-Band Dynamics / 3-band comp/expand) ----
  class MerkelMulti extends RackEffect {
    constructor() {
      super('merkel-multi', 'Merkel-Multi', 'Dynamics');
      this.params = {
        lowThresh:  { value: -20, min: -40, max: 0, unit: 'dB' },
        midThresh:  { value: -16, min: -40, max: 0, unit: 'dB' },
        highThresh: { value: -12, min: -40, max: 0, unit: 'dB' },
        lowRatio:   { value: 3,   min: 1,   max: 10, unit: ':1' },
        midRatio:   { value: 4,   min: 1,   max: 10, unit: ':1' },
        highRatio:  { value: 5,   min: 1,   max: 10, unit: ':1' },
        midMakeup:  { value: 3,   min: -12, max: 12, unit: 'dB' },
      };
      this.lowSplit = ctx.createBiquadFilter();
      this.lowSplit.type = 'lowpass';
      this.lowSplit.frequency.value = 400;
      this.midBand = ctx.createBiquadFilter();
      this.midBand.type = 'peaking';
      this.midBand.frequency.value = 1200;
      this.midBand.Q.value = 0.7;
      this.highSplit = ctx.createBiquadFilter();
      this.highSplit.type = 'highshelf';
      this.highSplit.frequency.value = 4000;
      this.compL = ctx.createDynamicsCompressor();
      this.compL.threshold.value = -20;
      this.compL.ratio.value = 3;
      this.compM = ctx.createDynamicsCompressor();
      this.compM.threshold.value = -16;
      this.compM.ratio.value = 4;
      this.compH = ctx.createDynamicsCompressor();
      this.compH.threshold.value = -12;
      this.compH.ratio.value = 5;
      this.makeupL = ctx.createGain();
      this.makeupL.gain.value = 1;
      this.makeupM = ctx.createGain();
      this.makeupM.gain.value = 1.4;
      this.makeupH = ctx.createGain();
      this.makeupH.gain.value = 1.2;
      this.mixL = ctx.createGain();
      this.mixM = ctx.createGain();
      this.mixH = ctx.createGain();
      this.sum = ctx.createGain();
      this.chain.push(this.lowSplit, this.compL, this.makeupL, this.mixL);
      this.midBand.connect(this.compM);
      this.chain.push(this.midBand, this.compM, this.makeupM, this.mixM);
      this.chain.push(this.highSplit, this.compH, this.makeupH, this.mixH, this.sum, this.outputNode);    
      this.presets = {
        'Wide Master': { params: { lowThresh: -18, midThresh: -12, highThresh: -10, lowRatio: 3, midRatio: 4, highRatio: 5, midMakeup: 4 }},
        'Clean Mix': { params: { lowThresh: -26, midThresh: -22, highThresh: -18, lowRatio: 2, midRatio: 3, highRatio: 3, midMakeup: 2 }},
        'Crush Punk': { params: { lowThresh: -8, midThresh: -6, highThresh: -4, lowRatio: 7, midRatio: 8, highRatio: 10, midMakeup: 9 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.compL.threshold.value = p.lowThresh.value;
      this.compL.ratio.value = p.lowRatio.value;
      this.compM.threshold.value = p.midThresh.value;
      this.compM.ratio.value = p.midRatio.value;
      this.compH.threshold.value = p.highThresh.value;
      this.compH.ratio.value = p.highRatio.value;
      this.makeupM.gain.value = Math.pow(10, p.midMakeup.value / 20);
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.lowSplit);
      }
    }
  }

  // ---- Eastland-Exciter (Harmonic Exciter / Saturation) ----
  class EastlandExciter extends RackEffect {
    constructor() {
      super('eastland-ex', 'Eastland-Exciter', 'Tone');
      this.params = {
        drive:   { value: 55,  min: 0, max: 100, unit: '%' },
        freq:    { value: 1200, min: 200, max: 9000, unit: 'Hz' },
        mix:     { value: 40,  min: 0, max: 100, unit: '%' },
        amount:  { value: 60,  min: 0, max: 100, unit: '%' },
      };
      this.hp = ctx.createBiquadFilter();
      this.hp.type = 'highpass';
      this.hp.frequency.value = 1200;
      this.waveShaper = ctx.createWaveShaper();
      this.waveShaper.oversample = '4x';
      this.makeCurve = (drive) => {
        const samples = 44100;
        const curve = new Float32Array(samples);
        const k = (drive / 100) * 90;
        for (let i = 0; i < samples; i++) {
          const x = (i * 2) / samples - 1;
          curve[i] = (1 + k) * x / (1 + Math.abs(x) * (k / 10 + 1));
        }
        return curve;
      };
      this.mixBus = ctx.createGain();
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.chain.push(this.hp, this.waveShaper, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);    
      this.presets = {
        'Air Sparkle': { params: { drive: 30, freq: 4000, mix: 45, amount: 80 }},
        'Gritty Mids': { params: { drive: 70, freq: 1200, mix: 55, amount: 90 }},
        'Subtle Shine': { params: { drive: 25, freq: 6000, mix: 30, amount: 60 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.waveShaper.curve = this.makeCurve(p.drive.value);
      this.hp.frequency.value = p.freq.value;
      this.wetGain.gain.value = (p.mix.value / 100) * (p.amount.value / 100);
      this.dryGain.gain.value = 1 - (p.mix.value / 100) * 0.5;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.hp);
      }
    }
  }

  // ---- Ranger-Reverse (Reverse Delay w/ Freeze) ----
  class RangerReverse extends RackEffect {
    constructor() {
      super('ranger-rev', 'Ranger-Reverse', 'Time');
      this.params = {
        time:      { value: 600, min: 50, max: 3000, unit: 'ms' },
        feedback:  { value: 60, min: 0,   max: 90, unit: '%' },
        mix:       { value: 38, min: 0,   max: 100, unit: '%' },
        reverse:   { value: 1,  min: 0,   max: 1,  unit: '' },
        tone:      { value: 55, min: 0, max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.reverseGain = ctx.createGain();
      this.reverseGain.gain.value = 1;
      this.feedbackGain = ctx.createGain();
      this.feedbackGain.gain.value = 0.6;
      this.delay = ctx.createDelay(4);
      this.delay.delayTime.value = 0.6;
      this.toneFilter = ctx.createBiquadFilter();
      this.toneFilter.type = 'lowpass';
      this.toneFilter.frequency.value = 3000;
      this.chain.push(this.delay, this.toneFilter, this.feedbackGain, this.reverseGain, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);    
      this.presets = {
        'Half Reverb': { params: { time: 900, feedback: 55, mix: 30, reverse: 1, tone: 65 }},
        'Tape Echo': { params: { time: 600, feedback: 40, mix: 28, reverse: 0, tone: 48 }},
        'Droning Wash': { params: { time: 2200, feedback: 78, mix: 42, reverse: 1, tone: 30 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.delay.delayTime.value = p.time.value / 1000;
      this.feedbackGain.gain.value = p.feedback.value / 100 * 0.75;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100 * 0.4;
      this.toneFilter.frequency.value = 500 + p.tone.value * 50;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.delay);
      }
    }
  }

  // ---- Ball-Ring (Ring Modulator / Metallic Synthesis) ----
  class BallRing extends RackEffect {
    constructor() {
      super('ball-ring', 'Ball-Ring', 'Modulation');
      this.params = {
        rate:    { value: 300,  min: 20, max: 8000, unit: 'Hz' },
        mix:     { value: 50,  min: 0,  max: 100, unit: '%' },
        depth:   { value: 100, min: 0, max: 100, unit: '%' },
        fine:    { value: 0,   min: -50, max: 50, unit: 'ct' },
      };
      this.carrier = ctx.createOscillator();
      this.carrier.frequency.value = 300;
      this.carrier.type = 'sine';
      this.modulator = ctx.createGain();
      this.modulator.gain.value = 0;
      this.mixGain = ctx.createGain();
      this.mixGain.gain.value = 0.5;
      this.carrier.connect(this.modulator);
      this.carrier.start();
      this.chain.push(this.modulator, this.mixGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Bell Tone': { params: { rate: 2400, mix: 45, depth: 100, fine: 0 }},
        'Sub Bass': { params: { rate: 55, mix: 60, depth: 90, fine: -12 }},
        'Agitated Metal': { params: { rate: 6000, mix: 50, depth: 80, fine: 300 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.carrier.frequency.value = p.rate.value + p.fine.value;
      this.modulator.gain.value = (p.depth.value / 100);
      this.mixGain.gain.value = p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.modulator);
      }
    }
  }

  // ---- Cole-Bit (Bit Crusher w/ Sample Rate + Quantize) ----
   // Note: Uses ScriptProcessorNode for compatibility. Consider migrating to AudioWorklet for better performance.
  class ColeBit extends RackEffect {
    constructor() {
      super('cole-bit', 'Cole-Bit', 'LoFi');
      this.params = {
        bits:     { value: 8,  min: 1, max: 16, unit: 'bit' },
        rate:     { value: 11025, min: 500, max: 48000, unit: 'Hz' },
        fold:     { value: 0,  min: 0,  max: 100, unit: '%' },
        mix:      { value: 50, min: 0, max: 100, unit: '%' },
        tone:     { value: 60, min: 0, max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.script = ctx.createScriptProcessor(2048, 1, 1);
      this.script.onaudioprocess = (e) => {
        const input = e.inputBuffer.getChannelData(0);
        const output = e.outputBuffer.getChannelData(0);
        const bits = Math.max(1, Math.round(this.params.bits.value));
        const step = Math.pow(2, bits);
        const rateHz = Math.max(500, this.params.rate.value);
        const hold = Math.max(1, Math.round(44100 / rateHz));
        const foldAmt = this.params.fold.value / 100;
        for (let i = 0; i < input.length; i++) {
          const idx = Math.floor(i / hold) * hold;
          let s = input[Math.min(idx, input.length - 1)];
          s = Math.round(s * step) / step;
          s = s * (1 - foldAmt) + (s > 0 ? 1 : -1) * foldAmt * Math.abs(s);
          output[i] = s;
        }
      };
      this.chain.push(this.script, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        '8-Bit Hero': { params: { bits: 8, rate: 11025, fold: 10, mix: 65, tone: 55 }},
        'N64 Clouds': { params: { bits: 5, rate: 8000, fold: 35, mix: 75, tone: 30 }},
        'Gentle LoFi': { params: { bits: 12, rate: 22050, fold: 2, mix: 40, tone: 65 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; }
    refresh() {
      const p = this.params;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.script);
      }
    }
  }

  // ---- Win-Tune (Pitch Corrector / Auto-Tune) ----
  class WinTune extends RackEffect {
    constructor() {
      super('win-tune', 'Win-Tune', 'Pitch');
      this.params = {
        speed:     { value: 50,  min: 0,  max: 100, unit: '%' },
        amount:    { value: 80,  min: 0,  max: 100, unit: '%' },
        scale:    { value: 0,   min: 0,  max: 3,  unit: '' },
        formant:  { value: 50,  min: 0,  max: 100, unit: '%' },
        mix:      { value: 100, min: 0, max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.shiftL = ctx.createDelay(0.2);
      this.shiftL.delayTime.value = 0.018;
      this.shiftR = ctx.createDelay(0.2);
      this.shiftR.delayTime.value = 0.022;
      this.scaleFactors = [1, 1.059, 1.122, 1.189, 1.26, 1.335, 1.414, 1.498, 1.587, 1.682, 1.782, 1.888, 2];
      this.chain.push(this.shiftL, this.shiftR, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Autotune Fast': { params: { speed: 85, amount: 90, scale: 0, formant: 50, mix: 100 }},
        'Subtle Shifter': { params: { speed: 40, amount: 30, scale: 2, formant: 70, mix: 70 }},
        'Formant Lock': { params: { speed: 60, amount: 70, scale: 0, formant: 20, mix: 85 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      const target = p.amount.value / 100;
      const semi = Math.round(target * 12);
      const ratio = this.scaleFactors[Math.min(Math.max(semi, 0), 12)] || 1;
      this.shiftL.delayTime.value = 0.018 * ratio;
      this.shiftR.delayTime.value = 0.022 * ratio;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100 * 0.3;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.shiftL);
      }
    }
  }

  // ---- Anson-Chomp (Transient Shaper / Punch) ----
  class AnsonChomp extends RackEffect {
    constructor() {
      super('anson-chomp', 'Anson-Chomp', 'Dynamics');
      this.params = {
        attack:    { value: 60,  min: -40, max: 40, unit: 'dB' },
        sustain:   { value: -10, min: -40, max: 40, unit: 'dB' },
        release:   { value: 40,  min: 0, max: 100, unit: '%' },
        mix:       { value: 100, min: 0,  max: 100, unit: '%' },
      };
      this.attackFilter = ctx.createBiquadFilter();
      this.attackFilter.type = 'highpass';
      this.attackFilter.frequency.value = 150;
      this.sustainFilter = ctx.createBiquadFilter();
      this.sustainFilter.type = 'lowpass';
      this.sustainFilter.frequency.value = 200;
      this.attackGain = ctx.createGain();
      this.attackGain.gain.value = 1;
      this.sustainGain = ctx.createGain();
      this.sustainGain.gain.value = 1;
      this.sum = ctx.createGain();
      this.chain.push(this.attackFilter, this.attackGain);
      this.sustainFilter.connect(this.sustainGain);
      this.chain.push(this.sustainFilter, this.sustainGain, this.sum, this.outputNode);
    
      this.presets = {
        'Snap Attack': { params: { attack: 35, sustain: -18, release: 60, mix: 100 }},
        'Thumpy Kick': { params: { attack: -25, sustain: 5, release: 20, mix: 100 }},
        'Gentle Shape': { params: { attack: 15, sustain: -8, release: 80, mix: 90 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      const att = Math.pow(10, p.attack.value / 20);
      const sus = Math.pow(10, p.sustain.value / 20);
      this.attackGain.gain.value = att;
      this.sustainGain.gain.value = sus;
      this.attackFilter.frequency.value = 80 + (p.release.value / 100) * 600;
      this.sustainFilter.frequency.value = 1000 - (p.release.value / 100) * 800;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.attackFilter);
      }
    }
  }

  // ---- Bro-Gate (Multi-Band Gate / Silence Killer) ----
  class BroGate extends RackEffect {
    constructor() {
      super('bro-gate', 'Bro-Gate', 'Dynamics');
      this.params = {
        threshold: { value: -48, min: -80, max: 0, unit: 'dB' },
        attack:    { value: 1,   min: 0,  max: 100, unit: 'ms' },
        release:   { value: 90,  min: 5,  max: 1000, unit: 'ms' },
        range:     { value: 70,  min: 0, max: 100, unit: 'dB' },
        hysteresis:{ value: 5,   min: 0,  max: 20, unit: 'dB' },
      };
      this.gate = ctx.createDynamicsCompressor();
      this.gate.threshold.value = -48;
      this.gate.ratio.value = 20;
      this.gate.attack.value = 0.001;
      this.gate.release.value = 0.09;
      this.chain.push(this.gate, this.outputNode);
    
      this.presets = {
        'Total Silence': { params: { threshold: -65, attack: 0, release: 60, range: 95, hysteresis: 3 }},
        'Rhythmic Gate': { params: { threshold: -42, attack: 2, release: 90, range: 75, hysteresis: 8 }},
        'Lo-Cut Gate': { params: { threshold: -48, attack: 1, release: 120, range: 70, hysteresis: 5 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.gate.threshold.value = p.threshold.value;
      this.gate.attack.value = p.attack.value / 1000;
      this.gate.release.value = p.release.value / 1000;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.gate);
      }
    }
  }

  // ---- Put-Slant (Stereo Panner / Haas Effect) ----
  class PutSlant extends RackEffect {
    constructor() {
      super('put-slant', 'Put-Slant', 'Spatial');
      this.params = {
        pan:       { value: 50,  min: 0, max: 100, unit: '%' },
        width:     { value: 70,  min: 0, max: 100, unit: '%' },
        haasMs:    { value: 12,  min: 0, max: 50, unit: 'ms' },
        invert:    { value: 0,   min: 0, max: 1,  unit: '' },
        lowCut:    { value: 80,  min: 20, max: 500, unit: 'Hz' },
      };
      this.splitter = ctx.createChannelSplitter(2);
      this.merger = ctx.createChannelMerger(2);
      this.delayL = ctx.createDelay(0.1);
      this.delayL.delayTime.value = 0.012;
      this.delayR = ctx.createDelay(0.1);
      this.delayR.delayTime.value = 0;
      this.pannerL = ctx.createGain();
      this.pannerL.gain.value = 0.8;
      this.pannerR = ctx.createGain();
      this.pannerR.gain.value = 0.8;
      this.hp = ctx.createBiquadFilter();
      this.hp.type = 'highpass';
      this.hp.frequency.value = 80;
      this.inputNode.connect(this.splitter);
      this.inputNode.connect(this.hp);
      this.hp.connect(this.outputNode);
      this.chain.push(this.delayL, this.delayR, this.pannerL, this.pannerR, this.outputNode);
    
      this.presets = {
        'Hard Left': { params: { pan: 15, width: 70, haasMs: 18, invert: 0, lowCut: 80 }},
        'Center Wide': { params: { pan: 50, width: 95, haasMs: 8, invert: 0, lowCut: 120 }},
        'Haas Only': { params: { pan: 50, width: 50, haasMs: 35, invert: 1, lowCut: 60 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      const pan = p.pan.value / 100;
      const width = p.width.value / 100;
      this.pannerL.gain.value = Math.cos(pan * Math.PI / 2) * width;
      this.pannerR.gain.value = Math.sin(pan * Math.PI / 2) * width;
      this.delayL.delayTime.value = p.haasMs.value / 1000;
      this.delayR.delayTime.value = 0;
      this.hp.frequency.value = p.lowCut.value;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.splitter);
        this.inputNode.connect(this.hp);
      }
    }
  }

  // ---- Roby-Dub (Dub Delay w/ Saturation + Filter Sweep) ----
  class RobyDub extends RackEffect {
    constructor() {
      super('roby-dub', 'Roby-Dub', 'Time');
      this.params = {
        time:      { value: 320, min: 10, max: 3000, unit: 'ms' },
        feedback:  { value: 68, min: 0,  max: 92,  unit: '%' },
        filter:    { value: 1800, min: 200, max: 9000, unit: 'Hz' },
        drive:     { value: 45, min: 0, max: 100, unit: '%' },
        mix:       { value: 38, min: 0, max: 100, unit: '%' },
        width:     { value: 55, min: 0, max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.delayL = ctx.createDelay(6);
      this.delayL.delayTime.value = 0.32;
      this.delayR = ctx.createDelay(6);
      this.delayR.delayTime.value = 0.35;
      this.feedbackL = ctx.createGain();
      this.feedbackL.gain.value = 0.68;
      this.feedbackR = ctx.createGain();
      this.feedbackR.gain.value = 0.65;
      this.filterL = ctx.createBiquadFilter();
      this.filterL.type = 'lowpass';
      this.filterL.frequency.value = 3000;
      this.filterR = ctx.createBiquadFilter();
      this.filterR.type = 'lowpass';
      this.filterR.frequency.value = 3100;
      this.saturate = ctx.createWaveShaper();
      this.saturate.oversample = '4x';
      const satCurve = new Float32Array(44100);
      for (let i = 0; i < 44100; i++) {
        const x = (i * 2) / 44100 - 1;
        satCurve[i] = Math.tanh(x * 2);
      }
      this.saturate.curve = satCurve;
      this.chain.push(this.delayL, this.filterL, this.feedbackL, this.saturate);
      this.chain.push(this.delayR, this.filterR, this.feedbackR);
      this.chain.push(this.wetGain, this.dryGain, this.outputNode);
    
      this.presets = {
        'Dub Delay': { params: { time: 420, feedback: 72, filter: 1400, drive: 55, mix: 40, width: 60 }},
        'Space Echo': { params: { time: 320, feedback: 55, filter: 2800, drive: 30, mix: 35, width: 45 }},
        'Crush Dub': { params: { time: 520, feedback: 82, filter: 900, drive: 80, mix: 45, width: 70 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.delayL.delayTime.value = p.time.value / 1000;
      this.delayR.delayTime.value = (p.time.value + 18) / 1000;
      this.feedbackL.gain.value = p.feedback.value / 100 * 0.75;
      this.feedbackR.gain.value = p.feedback.value / 100 * 0.7;
      this.filterL.frequency.value = p.filter.value;
      this.filterR.frequency.value = p.filter.value * 1.05;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100 * 0.35;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.delayL);
      }
    }
  }

  // ---- Moran-Spring (Spring Reverb w/ Drip) ----
  class MoranSpring extends RackEffect {
    constructor() {
      super('moran-spring', 'Moran-Spring', 'Spatial');
      this.params = {
        length:   { value: 60,  min: 10, max: 100, unit: '%' },
        damping:  { value: 45,  min: 0,  max: 100, unit: '%' },
        drip:     { value: 40,  min: 0,  max: 100, unit: '%' },
        mix:      { value: 28,  min: 0,  max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.delay1 = ctx.createDelay(1);
      this.delay1.delayTime.value = 0.04;
      this.delay2 = ctx.createDelay(1);
      this.delay2.delayTime.value = 0.06;
      this.delay3 = ctx.createDelay(1);
      this.delay3.delayTime.value = 0.09;
      this.filter1 = ctx.createBiquadFilter();
      this.filter1.type = 'lowpass';
      this.filter1.frequency.value = 3000;
      this.filter2 = ctx.createBiquadFilter();
      this.filter2.type = 'lowpass';
      this.filter2.frequency.value = 5000;
      this.dripGain = ctx.createGain();
      this.dripGain.gain.value = 0.3;
      this.chain.push(this.delay1, this.filter1, this.delay2, this.filter2, this.delay3, this.dripGain, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Tight Spring': { params: { length: 40, damping: 60, drip: 20, mix: 24 }},
        'Long Drip': { params: { length: 85, damping: 35, drip: 80, mix: 32 }},
        'Muted Clang': { params: { length: 60, damping: 80, drip: 50, mix: 28 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.delay1.delayTime.value = 0.02 + (p.length.value / 100) * 0.06;
      this.delay2.delayTime.value = 0.04 + (p.length.value / 100) * 0.08;
      this.delay3.delayTime.value = 0.06 + (p.length.value / 100) * 0.1;
      this.filter2.frequency.value = 5000 - (p.damping.value / 100) * 4000;
      this.wetGain.gain.value = p.mix.value / 100 * 1.1;
      this.dryGain.gain.value = 1 - p.mix.value / 100 * 0.4;
      this.dripGain.gain.value = p.drip.value / 100 * 0.5;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.delay1);
      }
    }
  }

  // ---- Ham-Pomp (FET-Style Optical Compressor w/ Auto-Gain) ----
  class HamPomp extends RackEffect {
    constructor() {
      super('ham-pomp', 'Ham-Pomp', 'Dynamics');
      this.params = {
        threshold: { value: -14, min: -40, max: 0, unit: 'dB' },
        ratio:     { value: 4,  min: 1,  max: 20, unit: ':1' },
        attack:    { value: 2,  min: 0,  max: 100, unit: 'ms' },
        release:   { value: 90, min: 5,  max: 1000, unit: 'ms' },
        makeup:    { value: 5,  min: -24, max: 24, unit: 'dB' },
        mix:       { value: 100, min: 0, max: 100, unit: '%' },
      };
      this.comp = ctx.createDynamicsCompressor();
      this.comp.threshold.value = -14;
      this.comp.ratio.value = 4;
      this.comp.attack.value = 0.002;
      this.comp.release.value = 0.09;
      this.makeup = ctx.createGain();
      this.makeup.gain.value = 1.78;
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.chain.push(this.comp, this.makeup, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Leveler Auto': { params: { threshold: -18, ratio: 4, attack: 5, release: 120, makeup: 3, mix: 100 }},
        'Aggressive Limit': { params: { threshold: -8, ratio: 12, attack: 1, release: 50, makeup: 8, mix: 100 }},
        'Vintage Comp': { params: { threshold: -20, ratio: 3, attack: 8, release: 150, makeup: 2, mix: 100 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.comp.threshold.value = p.threshold.value;
      this.comp.ratio.value = p.ratio.value;
      this.comp.attack.value = p.attack.value / 1000;
      this.comp.release.value = p.release.value / 1000;
      this.makeup.gain.value = Math.pow(10, p.makeup.value / 20);
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.comp);
      }
    }
  }

  // ---- Winters-Res (Karplus-Strong Resonator / String Synth) ----
   // Note: Uses ScriptProcessorNode for compatibility. Consider migrating to AudioWorklet for better performance.
  class WintersRes extends RackEffect {
    constructor() {
      super('winters-res', 'Winters-Res', 'Synthesis');
      this.params = {
        pitch:   { value: 220, min: 40, max: 1200, unit: 'Hz' },
        decay:   { value: 2.5, min: 0.1, max: 10, unit: 's' },
        damping: { value: 30, min: 0, max: 100, unit: '%' },
        mix:     { value: 50, min: 0, max: 100, unit: '%' },
        brightness:{ value: 70, min: 0, max: 100, unit: '%' },
      };
      this.bufferSize = 44100;
      this.script = ctx.createScriptProcessor(this.bufferSize, 1, 1);
      this.script.onaudioprocess = (e) => {
        const input = e.inputBuffer.getChannelData(0);
        const output = e.outputBuffer.getChannelData(0);
        const baseFreq = Math.max(20, this.params.pitch.value);
        const period = Math.round(this.bufferSize / baseFreq);
        const decay = this.params.decay.value;
        const damp = this.params.damping.value / 100;
        const br = this.params.brightness.value / 100;
        for (let i = 0; i < input.length; i++) {
          const idx = i % period;
          let excite = input[i] * 0.8 + (Math.random() * 2 - 1) * 0.02 * br;
          this._buf[idx] = (this._buf[idx] || 0) * (1 - damp * 0.02) + excite + this._buf[(idx - 1 + period) % period] * 0.35 * br;
          output[i] = (this._buf[idx] || 0) * Math.pow(0.999, 1 / (decay * 3));
        }
      };
      this._buf = new Float32Array(this.bufferSize);
      this.mixGain = ctx.createGain();
      this.mixGain.gain.value = 0.5;
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
    
      this.presets = {
        'String Bass': { params: { pitch: 85, decay: 4.5, damping: 25, mix: 65, brightness: 60 }},
        'Kalimba': { params: { pitch: 980, decay: 1.2, damping: 50, mix: 50, brightness: 85 }},
        'Pluck Harmonics': { params: { pitch: 440, decay: 2, damping: 15, mix: 55, brightness: 90 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; }
    refresh() {
      const p = this.params;
      this.mixGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.script);
      }
    }
  }

  // ---- New-Tron (Envelope Filter / Auto-Wah w/ LFO) ----
  class NewTron extends RackEffect {
    constructor() {
      super('new-tron', 'New-Tron', 'Filter');
      this.params = {
        sensitivity: { value: 55, min: 0, max: 100, unit: '%' },
        peaks:      { value: 2,  min: 1, max: 8, unit: '' },
        resonance:  { value: 18, min: 1, max: 40, unit: 'Q' },
        lfoRate:    { value: 0.4, min: 0.05, max: 6, unit: 'Hz' },
        lfoDepth:   { value: 40, min: 0, max: 100, unit: '%' },
        mix:        { value: 60, min: 0, max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.stages = [];
      for (let i = 0; i < 8; i++) {
        const bp = ctx.createBiquadFilter();
        bp.type = 'bandpass';
        bp.frequency.value = 400;
        bp.Q.value = 18;
        const g = ctx.createGain();
        g.gain.value = 1;
        bp.connect(g);
        this.stages.push({ bp, g });
      }
      this.lfo = ctx.createOscillator();
      this.lfo.frequency.value = 0.4;
      this.lfo.type = 'sine';
      this.lfoGain = ctx.createGain();
      this.lfoGain.gain.value = 800;
      this.lfo.connect(this.lfoGain);
      this.lfo.start();
this.outputGain = ctx.createGain();
       // Connect wet/dry mix to output
       this.dryGain.connect(this.outputNode);
       this.wetGain.connect(this.outputNode);
       this.chain.push(this.wetGain);
    
      this.presets = {
        'Auto-Wah Funk': { params: { sensitivity: 65, peaks: 4, resonance: 22, lfoRate: 1.2, lfoDepth: 60, mix: 65 }},
        'Slow Sweep': { params: { sensitivity: 40, peaks: 2, resonance: 10, lfoRate: 0.2, lfoDepth: 45, mix: 50 }},
        'Phaser-ish': { params: { sensitivity: 55, peaks: 6, resonance: 30, lfoRate: 0.5, lfoDepth: 50, mix: 60 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      const active = Math.min(8, Math.round(p.peaks.value));
      const baseF = 200 + (p.lfoDepth.value / 100) * 3000;
      for (let i = 0; i < 8; i++) {
        if (i < active) {
          this.stages[i].bp.frequency.value = baseF + i * 200 + p.sensitivity.value * 3;
          this.stages[i].bp.Q.value = p.resonance.value;
          this.stages[i].g.gain.value = 1;
        } else {
          this.stages[i].bp.frequency.value = 0;
          this.stages[i].bp.Q.value = 0;
          this.stages[i].g.gain.value = 0;
        }
      }
      this.lfo.frequency.value = p.lfoRate.value;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.wetGain);
      }
    }
  }

  // ---- Law-Tape (Tape Stop / Variable Speed/Stop) ----
   // Note: Uses ScriptProcessorNode for compatibility. Consider migrating to AudioWorklet for better performance.
  class LawTape extends RackEffect {
    constructor() {
      super('law-tape', 'Law-Tape', 'LoFi');
      this.params = {
        speed:       { value: 100, min: 0, max: 120, unit: '%' },
        stop:        { value: 0,   min: 0, max: 100, unit: '%' },
        wow:         { value: 8,   min: 0, max: 100, unit: '%' },
        flutter:     { value: 12,  min: 0, max: 100, unit: '%' },
        noise:       { value: 15,  min: 0, max: 100, unit: '%' },
        mix:         { value: 60,  min: 0, max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.script = ctx.createScriptProcessor(2048, 1, 1);
      this._phase = 0;
      this._wowPhase = 0;
      this.script.onaudioprocess = (e) => {
        const input = e.inputBuffer.getChannelData(0);
        const output = e.outputBuffer.getChannelData(0);
        const stopAmt = this.params.stop.value / 100;
        const wowAmt = this.params.wow.value / 100 * 0.01;
        const flAmt = this.params.flutter.value / 100 * 0.005;
        const baseSpeed = this.params.speed.value / 100;
        for (let i = 0; i < input.length; i++) {
          this._wowPhase += 0.02;
          const wowMod = Math.sin(this._wowPhase) * wowAmt;
          const flutterMod = Math.sin(this._wowPhase * 12.7) * flAmt;
          let rate = baseSpeed - stopAmt * baseSpeed + wowMod + flutterMod;
          this._phase += rate;
          const idx = Math.abs(Math.floor(this._phase)) % input.length;
          output[i] = input[idx] + (Math.random() * 2 - 1) * (this.params.noise.value / 100 * 0.015);
        }
      };
      this.chain.push(this.script, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Stop Moment': { params: { speed: 100, stop: 90, wow: 20, flutter: 15, noise: 22, mix: 80 }},
        'Warm Warble': { params: { speed: 100, stop: 0, wow: 35, flutter: 25, noise: 18, mix: 45 }},
        'Dead Tape': { params: { speed: 85, stop: 40, wow: 45, flutter: 35, noise: 28, mix: 60 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; }
    refresh() {
      const p = this.params;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.script);
      }
    }
  }

  // ---- Potter-Pan (Rotary Speaker Sim / Leslie) ----
  class PotterPan extends RackEffect {
    constructor() {
      super('potter-pan', 'Potter-Pan', 'Modulation');
      this.params = {
        speed:    { value: 0.8, min: 0.1, max: 8, unit: 'Hz' },
        depth:    { value: 70,  min: 0,  max: 100, unit: '%' },
        doppler:  { value: 55,  min: 0,  max: 100, unit: '%' },
        mix:      { value: 55,  min: 0,  max: 100, unit: '%' },
      };
      this.splitter = ctx.createChannelSplitter(2);
      this.merger = ctx.createChannelMerger(2);
      this.lfo = ctx.createOscillator();
      this.lfo.frequency.value = 0.8;
      this.lfo.type = 'sine';
      this.lfoGain = ctx.createGain();
      this.lfoGain.gain.value = 0.5;
      this.dopplerL = ctx.createBiquadFilter();
      this.dopplerL.type = 'bandpass';
      this.dopplerL.frequency.value = 800;
      this.dopplerL.Q.value = 1.2;
      this.dopplerR = ctx.createBiquadFilter();
      this.dopplerR.type = 'bandpass';
      this.dopplerR.frequency.value = 900;
      this.dopplerR.Q.value = 1.2;
      this.lfo.connect(this.lfoGain);
      this.lfoGain.connect(this.dopplerL.frequency);
      this.lfoGain.connect(this.dopplerR.frequency);
      this.lfo.start();
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.chain.push(this.dopplerL, this.dopplerR, this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Slow Leslie': { params: { speed: 0.7, depth: 75, doppler: 50, mix: 58 }},
        'Fast Rotor': { params: { speed: 5.5, depth: 65, doppler: 60, mix: 55 }},
        'Subtle Movement': { params: { speed: 1.1, depth: 30, doppler: 35, mix: 40 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.lfo.frequency.value = p.speed.value;
      this.lfoGain.gain.value = (p.depth.value / 100) * p.doppler.value * 3;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.dopplerL);
      }
    }
  }

  // ---- Morgan-Gate (Rhythmic Gate / Sidechain Pump) ----
  class MorganGate extends RackEffect {
    constructor() {
      super('morgan-gate', 'Morgan-Gate', 'Dynamics');
      this.params = {
        threshold: { value: -30, min: -60, max: 0, unit: 'dB' },
        attack:    { value: 1,   min: 0, max: 100, unit: 'ms' },
        release:   { value: 120, min: 10, max: 1000, unit: 'ms' },
        range:     { value: 60,  min: 0, max: 100, unit: 'dB' },
        filter:    { value: 200, min: 20, max: 4000, unit: 'Hz' },
      };
      this.gate = ctx.createDynamicsCompressor();
      this.gate.threshold.value = -30;
      this.gate.ratio.value = 20;
      this.gate.attack.value = 0.001;
      this.gate.release.value = 0.12;
      this.hp = ctx.createBiquadFilter();
      this.hp.type = 'highpass';
      this.hp.frequency.value = 200;
      this.chain.push(this.hp, this.gate, this.outputNode);
    
      this.presets = {
        '4-on-Floor Pump': { params: { threshold: -28, attack: 1, release: 100, range: 65, filter: 180 }},
        'Aggressive Side': { params: { threshold: -22, attack: 0, release: 85, range: 80, filter: 120 }},
        'Ghost Pump': { params: { threshold: -38, attack: 2, release: 150, range: 50, filter: 250 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.gate.threshold.value = p.threshold.value;
      this.gate.attack.value = p.attack.value / 1000;
      this.gate.release.value = p.release.value / 1000;
      this.hp.frequency.value = p.filter.value;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.hp);
      }
    }
  }

  // ---- Buffalo-Gap (Wild West Saturation / Transformer Emulation) ----
  class BuffaloGap extends RackEffect {
    constructor() {
      super('buffalo-gap', 'Buffalo-Gap', 'Drive');
      this.params = {
        drive:    { value: 65, min: 0, max: 100, unit: '%' },
        output:   { value: 75, min: 0, max: 100, unit: '%' },
        bleed:    { value: 20, min: 0, max: 100, unit: '%' },
        mix:      { value: 80, min: 0, max: 100, unit: '%' },
        tone:     { value: 55, min: 0, max: 100, unit: '%' },
      };
      this.inputTrim = ctx.createGain();
      this.inputTrim.gain.value = 2.5;
      this.waveshaper = ctx.createWaveShaper();
      this.waveshaper.oversample = '4x';
      this.toneStack = ctx.createBiquadFilter();
      this.toneStack.type = 'lowpass';
      this.toneStack.frequency.value = 2800;
      this.toneStack.Q.value = 0.6;
      this.outputGain = ctx.createGain();
      this.outputGain.gain.value = 1;
      this.bleed = ctx.createBiquadFilter();
      this.bleed.type = 'highpass';
      this.bleed.frequency.value = 200;
      this.bleedGain = ctx.createGain();
      this.bleedGain.gain.value = 0.2;
      const satCurve = new Float32Array(44100);
      for (let i = 0; i < 44100; i++) {
        const x = (i * 2) / 44100 - 1;
        satCurve[i] = (1.5 * x) / (1 + Math.abs(x) * 1.2);
      }
      this.waveshaper.curve = satCurve;
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.chain.push(this.inputTrim, this.waveshaper, this.toneStack, this.outputGain);
      this.chain.push(this.wetGain);
      this.chain.push(this.dryGain, this.outputNode);
    
      this.presets = {
        'Crunch Tube': { params: { drive: 82, output: 78, bleed: 15, mix: 85, tone: 50 }},
        'Clean Boost': { params: { drive: 20, output: 60, bleed: 5, mix: 90, tone: 72 }},
        'Muffled Fuzz': { params: { drive: 95, output: 70, bleed: 35, mix: 80, tone: 28 }}
      };
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.inputTrim.gain.value = 1 + (p.drive.value / 100) * 6;
      this.waveshaper.curve = this.waveshaper.curve;
      this.toneStack.frequency.value = 500 + p.tone.value * 40;
      this.outputGain.gain.value = p.output.value / 100 * 1.5;
      this.wetGain.gain.value = p.mix.value / 100;
      this.dryGain.gain.value = 1 - p.mix.value / 100 * 0.4;
      this.bleedGain.gain.value = p.bleed.value / 100 * 0.3;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.inputTrim);
      }
    }
  }

  // ---- Jones-County-Creator (Hum-to-Music AI) ----
  // Takes hummed performance style and applies to existing song notes via SFZ/FLAC
  class JonesCountyCreator extends RackEffect {
    constructor() {
      super('jones-county', 'Jones County Creator', 'AI');
      this.params = {
        style:        { value: 70, min: 0,   max: 100, unit: '%' }, // Performance style intensity
        timing:       { value: 80, min: 0,   max: 100, unit: '%' }, // Timing quantization adherence
        velocityMap:  { value: 85, min: 0,   max: 100, unit: '%' }, // Dynamics mapping strength
        articulation: { value: 60, min: 0,   max: 100, unit: '%' }, // Articulation extraction
        blend:        { value: 90, min: 0,   max: 100, unit: '%' }, // Original/AI blend
        mode:         { value: 0,  min: 0,   max: 4,   unit: '' },  // SFZ instrument selector
        outputVol:    { value: 80, min: 0,   max: 100, unit: '%' },
      };
      this.dryGain = ctx.createGain();
      this.wetGain = ctx.createGain();
      this.styleBus = ctx.createGain();
      this.styleBus.gain.value = 0.7;
      this.presets = {
        'Piano Interpretation': { params: { style: 75, timing: 85, velocityMap: 80, articulation: 55, blend: 90, mode: 0, outputVol: 85 }},
        'String Quartet': { params: { style: 80, timing: 70, velocityMap: 75, articulation: 70, blend: 95, mode: 1, outputVol: 80 }},
        'Brass Section': { params: { style: 85, timing: 90, velocityMap: 90, articulation: 65, blend: 85, mode: 2, outputVol: 90 }},
        'Guitar Duo': { params: { style: 65, timing: 60, velocityMap: 70, articulation: 50, blend: 92, mode: 3, outputVol: 75 }},
        'Choir Pad': { params: { style: 90, timing: 50, velocityMap: 85, articulation: 80, blend: 98, mode: 4, outputVol: 70 }},
      };
      this.chain.push(this.styleBus, this.wetGain, this.dryGain, this.outputNode);
    }
    setParam(key, val) { this.params[key].value = val; this.refresh(); }
    refresh() {
      const p = this.params;
      this.styleBus.gain.value = p.style.value / 100;
      this.wetGain.gain.value = p.blend.value / 100 * p.outputVol.value / 100;
      this.dryGain.gain.value = 1 - p.blend.value / 100 * 0.7;
    }
    toggleBypass(state) {
      this.bypass = !state;
      if (this.bypass) {
        this.inputNode.disconnect();
        this.inputNode.connect(this.outputNode);
      } else {
        this.inputNode.disconnect();
        this.inputNode.connect(this.styleBus);
      }
    }
  }

  // ---- Registry ----
  const effects = {
    'clyde-verb':     () => new ClydeVerb(),
    'stamford-delay': () => new StamfordDelay(),
    'quanah-com':     () => new QuanahCom(),
    'electra-eq':     () => new ElectraEQ(),
    'albany-limit':   () => new AlbanyLimit(),
    'goltry-gate':    () => new GoltryGate(),
    'cisco-chorus':   () => new CiscoChorus(),
    'haskell-flange': () => new HaskellFlange(),
    'breck-phase':    () => new BreckPhase(),
    'funston-clip':   () => new FunstonClip(),
    'wichita-wah':    () => new WichitaWah(),
    'throck-trem':    () => new ThrockTrem(),
    'paducah-shift':  () => new PaducahShift(),
    'frede-filter':   () => new FredeFilter(),
    'munday-drive':   () => new MundayDrive(),
    'olney-amp':      () => new OlneyAmp(),
    'baird-expand':   () => new BairdExpander(),
    'gap-reverb':     () => new GapReverb(),
    'hawley-mod':     () => new HawleyMod(),
    'deleon-crush':   () => new DeLeonCrush(),
    'tusc-filter':    () => new TuscFilter(),
    'merkel-multi':   () => new MerkelMulti(),
    'eastland-ex':    () => new EastlandExciter(),
    'ranger-rev':     () => new RangerReverse(),
    'ball-ring':      () => new BallRing(),
    'cole-bit':       () => new ColeBit(),
    'win-tune':       () => new WinTune(),
    'anson-chomp':    () => new AnsonChomp(),
    'bro-gate':       () => new BroGate(),
    'put-slant':      () => new PutSlant(),
    'roby-dub':       () => new RobyDub(),
    'moran-spring':   () => new MoranSpring(),
    'ham-pomp':       () => new HamPomp(),
    'winters-res':    () => new WintersRes(),
    'new-tron':       () => new NewTron(),
    'law-tape':       () => new LawTape(),
    'potter-pan':     () => new PotterPan(),
    'morgan-gate':    () => new MorganGate(),
    'buffalo-gap':    () => new BuffaloGap(),
    'jones-county':   () => new JonesCountyCreator(),
  };

  return { effects, RackEffect };
    })();
