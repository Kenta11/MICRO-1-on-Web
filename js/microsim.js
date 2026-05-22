// WASM-backed MICRO-1 microsimulator.
//
// Wraps the Emscripten build of github.com/Kenta11/micro1/m1sim.c so the
// browser side gets bit-for-bit identical behaviour to the upstream
// reference simulator.  The class implements the same surface as the
// prior pure-JS MicroCPU it replaces — registers, buses, flags, CM/MM,
// lastWritten, step/run — so the UI in app.js does not have to know
// which backend is in use.

(function (global) {
  'use strict';

  // -------------------------------------------------------- field name maps
  // Identical to the pure-JS port, retained here because the renderer needs
  // them to disassemble individual ctrl words for display.
  const TSF_NAMES = ['ZER','NEG','CRY','OV','T','CZ','-','NTS'];
  const SQF_NAMES = ['B','BP','BRT','BT','BF','IOP','IRA','IAB','EI',
                     '-','-','-','-','-','-','NSQ'];
  const ALF_NAMES = ['ADD','SUB','AND','OR','XOR','IAL','-','NAL'];
  const SHF_NAMES = ['SLL','SRL','SLA','SRA','SNX','SWP','-','NSH'];
  const MMF_NAMES = ['RM','WM','-','NMM'];
  const EXF_NAMES = ['CM1','FLS','ASC','AS1','LIR','LIO','SC','EIO',
                     'ST','RT','INA','INB','DCB','HLT','OV','NEX'];
  const LBF_NAMES = ['R0','R1','R2','R3','R4','R5','R6','R7',
                     'RB','RBP','PC','IO','MM','IR','FSR','NLB'];
  const RBF_NAMES = ['R0','R1','R2','R3','R4','R5','R6','R7',
                     'RA','RAP','SLT','LLT','-','-','-','NRB'];
  const SBF_NAMES = ['R0','R1','R2','R3','R4','R5','R6','R7',
                     'SA','SAP','SB','SBP','PCS','-','-','NSB'];

  function field(cw, lo, width) {
    return Math.floor(cw / Math.pow(2, lo)) % Math.pow(2, width);
  }
  function toHex(n, digits) {
    return (n >>> 0).toString(16).toUpperCase().padStart(digits, '0');
  }
  function ctrlHex(cw) {
    const lo = cw % 0x100000000;
    const hi = Math.floor(cw / 0x100000000);
    return toHex(hi, 2) + toHex(lo, 8);
  }
  function decodeCtrlWord(cw) {
    return {
      LBF: field(cw, 36, 4), RBF: field(cw, 32, 4),
      ALF: field(cw, 29, 3), SHF: field(cw, 26, 3),
      SBF: field(cw, 22, 4), MMF: field(cw, 20, 2),
      SQF: field(cw, 16, 4), TSF: field(cw, 13, 3),
      EXF: field(cw,  9, 4), LTF: field(cw,  0, 9),
    };
  }
  function disasmCtrl(cw) {
    const f = decodeCtrlWord(cw);
    const parts = [];
    if (f.LBF !== 0xf) parts.push('LB=' + LBF_NAMES[f.LBF]);
    if (f.RBF !== 0xf) parts.push('RB=' + RBF_NAMES[f.RBF]);
    if (f.ALF !== 0x7) parts.push('AL=' + ALF_NAMES[f.ALF]);
    if (f.SHF !== 0x7) parts.push('SH=' + SHF_NAMES[f.SHF]);
    if (f.SBF !== 0xf) parts.push('SB=' + SBF_NAMES[f.SBF]);
    if (f.MMF !== 0x3) parts.push('MM=' + MMF_NAMES[f.MMF]);
    if (f.SQF !== 0xf) parts.push('SQ=' + SQF_NAMES[f.SQF]);
    if (f.TSF !== 0x7) parts.push('TS=' + TSF_NAMES[f.TSF]);
    if (f.EXF !== 0xf) parts.push('EX=' + EXF_NAMES[f.EXF]);
    if (f.LTF !== 0)   parts.push('LT=' + toHex(f.LTF, 3));
    return parts.length ? parts.join(' ') : 'NOP';
  }

  // -------------------------------------------------------------- WASM loader
  // The emcc factory pollutes the global with `loadM1Sim`.  Inject the
  // script tag once and resolve when the module reports ready.
  let modulePromise = null;
  function loadModule() {
    if (modulePromise) return modulePromise;
    modulePromise = new Promise((resolve, reject) => {
      if (typeof global.loadM1Sim === 'function') {
        global.loadM1Sim().then(resolve).catch(reject);
        return;
      }
      const script = document.createElement('script');
      script.src = 'js/wasm/m1sim.js';
      script.onload = () => {
        if (typeof global.loadM1Sim !== 'function') {
          reject(new Error('m1sim.js did not expose loadM1Sim')); return;
        }
        global.loadM1Sim().then(resolve).catch(reject);
      };
      script.onerror = () => reject(new Error('failed to load js/wasm/m1sim.js'));
      document.head.appendChild(script);
    });
    return modulePromise;
  }

  // ----------------------------------------------------------------- MicroCPU
  // Synchronous after init.  Wait on MICRO1Microsim.ready before constructing.
  class MicroCPU {
    constructor() {
      if (!MICRO1Microsim._mod) {
        throw new Error('MICRO1Microsim.ready must resolve before constructing MicroCPU');
      }
      this.mod = MICRO1Microsim._mod;
      const M = this.mod;
      this._reset      = M.cwrap('m1_reset', null, []);
      this._resetRegs  = M.cwrap('m1_reset_regs', null, []);
      this._loadCM   = M.cwrap('m1_load_cm', null, ['number','number','number']);
      this._loadMM   = M.cwrap('m1_load_mm', null, ['number','number']);
      this._step     = M.cwrap('m1_step', 'number', []);
      this._setCMAR  = M.cwrap('m1_set_cmar', null, ['number']);
      this._setPC    = M.cwrap('m1_set_pc', null, ['number']);
      this._setReg   = M.cwrap('m1_set_reg', null, ['number','number']);
      this._setIR    = M.cwrap('m1_set_ir', null, ['number']);
      this._setMAR   = M.cwrap('m1_set_mar', null, ['number']);
      this._setC     = M.cwrap('m1_set_c', null, ['number']);
      this._setFSR   = M.cwrap('m1_set_fsr', null, ['number']);
      this._setFlag  = M.cwrap('m1_set_flag', null, ['number','number']);
      this._inPush   = M.cwrap('m1_io_input_push', null, ['number']);
      this._inClear  = M.cwrap('m1_io_input_clear', null, []);
      this._outPop   = M.cwrap('m1_io_output_pop', 'number', []);
      this._getState = M.cwrap('m1_get_state', null, ['number']);
      this._readMM   = M.cwrap('m1_read_mm', 'number', ['number']);
      this._readCMSlab = M.cwrap('m1_read_cm_slab', null,
                                 ['number','number','number','number']);
      this._stackAt  = M.cwrap('m1_cmar_stack_at', 'number', ['number']);

      this._stateBuf = M._malloc(14 * 4);
      this._cmLoSlab = M._malloc(4);
      this._cmHiSlab = M._malloc(4);
      this._snapshot = makeEmptySnapshot();
      this._prev     = makeEmptySnapshot();
      this.io = { output: '' };
      this.steps = 0;
      this.lastWritten = { regs: [], addrs: [], buses: [], flags: false };
      this.cmarStack = [];
      this.fault_message = null;
      this.reset();
    }

    // Clear regs/PC/CMAR/buses/flags/stack but keep CM and MM as they are.
    resetRegs() {
      this._resetRegs();
      this.io = { output: '' };
      this.steps = 0;
      this.lastWritten = { regs: [], addrs: [], buses: [], flags: false };
      this.fault_message = null;
      this._refresh();
      this._prev = cloneSnapshot(this._snapshot);
    }

    reset() {
      this._reset();
      this.io = { output: '' };
      this.steps = 0;
      this._inputBuf = '';
      this._inputPos = 0;
      this.lastWritten = { regs: [], addrs: [], buses: [], flags: false };
      this.fault_message = null;
      this._refresh();
      this._prev = cloneSnapshot(this._snapshot);
    }

    setInput(s) {
      this._inClear();
      this._inputBuf = s || '';
      this._inputPos = 0;
      for (let i = 0; i < this._inputBuf.length; i++) {
        this._inPush(this._inputBuf.charCodeAt(i) & 0xff);
      }
    }

    loadCM(words) {
      for (const w of words) {
        const v = w.value;
        const lo = (v % 0x100000000) >>> 0;
        const hi = Math.floor(v / 0x100000000) >>> 0;
        this._loadCM(w.addr & 0xfff, hi, lo);
      }
      this._refresh();
    }
    loadMM(words) {
      for (const w of words) this._loadMM(w.addr & 0xffff, w.value & 0xffff);
      this._refresh();
    }

    step() {
      if (this._snapshot.halt) return { halted: true };
      this._prev = cloneSnapshot(this._snapshot);
      this._step();
      this.steps++;
      this._refresh();
      this._drainOutput();
      this._diffLastWritten();
      return { halted: this._snapshot.halt };
    }

    runBurst(maxSteps, breakpointSet) {
      let executed = 0;
      while (executed < maxSteps && !this._snapshot.halt) {
        if (breakpointSet && breakpointSet.has(this._snapshot.CMAR)) {
          return { hitBreakpoint: this._snapshot.CMAR, executed };
        }
        this._prev = cloneSnapshot(this._snapshot);
        this._step();
        executed++;
        this._refresh();
        if (this._snapshot.halt) break;
      }
      this.steps += executed;
      this._drainOutput();
      this._diffLastWritten();
      return { executed, halted: this._snapshot.halt };
    }

    _refresh() {
      this._getState(this._stateBuf);
      const u32 = new Uint32Array(this.mod.HEAPU32.buffer, this._stateBuf, 14);
      const s = this._snapshot;
      s.CMAR = u32[0] & 0xfff;
      s.PC   = (u32[0] >>> 16) & 0xffff;
      s.MAR  = u32[1] & 0xffff;
      s.IR   = (u32[1] >>> 16) & 0xffff;
      const hi = u32[3] & 0xff;
      s.CMDR = hi * 0x100000000 + (u32[2] >>> 0);
      s.R[0] = u32[4] & 0xffff;        s.R[1] = (u32[4] >>> 16) & 0xffff;
      s.R[2] = u32[5] & 0xffff;        s.R[3] = (u32[5] >>> 16) & 0xffff;
      s.R[4] = u32[6] & 0xffff;        s.R[5] = (u32[6] >>> 16) & 0xffff;
      s.R[6] = u32[7] & 0xffff;        s.R[7] = (u32[7] >>> 16) & 0xffff;
      s.LBUS = u32[8] & 0xffff;        s.RBUS = (u32[8] >>> 16) & 0xffff;
      s.ABUS = u32[9] & 0xffff;        s.SBUS = (u32[9] >>> 16) & 0xffff;
      s.IOBUS = u32[10] & 0xff;        s.FSR  = (u32[10] >>> 16) & 0xff;
      s.C    = u32[11] & 0xff;
      const flg = (u32[11] >>> 8) & 0xff;
      s.ZER = (flg & 0x01) ? 1 : 0;
      s.NEG = (flg & 0x02) ? 1 : 0;
      s.CRY = (flg & 0x04) ? 1 : 0;
      s.OV  = (flg & 0x08) ? 1 : 0;
      s.CZ  = (flg & 0x10) ? 1 : 0;
      s.T   = (flg & 0x20) ? 1 : 0;
      s.halt    = (flg & 0x40) !== 0;
      s.ov_lamp = (flg & 0x80) !== 0;
      const stackDepth = (u32[12] >>> 16) & 0xff;
      this.cmarStack.length = stackDepth;
      for (let i = 0; i < stackDepth; i++) this.cmarStack[i] = this._stackAt(i);
      this.fault_message = s.ov_lamp ? 'OV lamp on' : null;
    }

    _drainOutput() {
      let b;
      while ((b = this._outPop()) >= 0) {
        this.io.output += String.fromCharCode(b);
      }
    }

    _diffLastWritten() {
      const prev = this._prev, cur = this._snapshot;
      const regs = [];
      for (let i = 0; i < 8; i++) if (prev.R[i] !== cur.R[i]) regs.push(i);
      if (prev.PC !== cur.PC) regs.push('PC');
      const flags = prev.ZER !== cur.ZER || prev.NEG !== cur.NEG
                 || prev.CRY !== cur.CRY || prev.OV  !== cur.OV
                 || prev.CZ  !== cur.CZ  || prev.T   !== cur.T;
      this.lastWritten = { regs, addrs: [], buses: ['LBUS','RBUS','ABUS','SBUS'], flags };
    }

    // --------- Property proxies expected by the UI -----------------------
    get R() { return this._snapshot.R; }
    get PC()   { return this._snapshot.PC; }   set PC(v) { this._setPC(v); this._refresh(); }
    get IR()   { return this._snapshot.IR; }   set IR(v) { this._setIR(v); this._refresh(); }
    get MAR()  { return this._snapshot.MAR; }  set MAR(v) { this._setMAR(v); this._refresh(); }
    get CMAR() { return this._snapshot.CMAR; } set CMAR(v) { this._setCMAR(v); this._refresh(); }
    setReg(i, v) { this._setReg(i, v); this._refresh(); }
    get CMDR() { return this._snapshot.CMDR; }
    get LBUS() { return this._snapshot.LBUS; }
    get RBUS() { return this._snapshot.RBUS; }
    get ABUS() { return this._snapshot.ABUS; }
    get SBUS() { return this._snapshot.SBUS; }
    get IOBUS(){ return this._snapshot.IOBUS; }
    get C()    { return this._snapshot.C; }    set C(v) { this._setC(v); this._refresh(); }
    get FSR()  { return this._snapshot.FSR; }  set FSR(v) { this._setFSR(v); this._refresh(); }
    get ZER()  { return this._snapshot.ZER; }  set ZER(v) { this._setFlag(0, v); this._refresh(); }
    get NEG()  { return this._snapshot.NEG; }  set NEG(v) { this._setFlag(1, v); this._refresh(); }
    get CRY()  { return this._snapshot.CRY; }  set CRY(v) { this._setFlag(2, v); this._refresh(); }
    get OV()   { return this._snapshot.OV; }   set OV(v)  { this._setFlag(3, v); this._refresh(); }
    get CZ()   { return this._snapshot.CZ; }
    get T()    { return this._snapshot.T; }    set T(v)   { this._setFlag(4, v); this._refresh(); }
    get halt() { return this._snapshot.halt; }

    // MM / CM live in WASM linear memory; expose array-like proxies that
    // call into the wasm getters lazily.  Index reads only.
    get MM() {
      const f = this._readMM;
      return new Proxy({}, { get(_, k) {
        const i = Number(k); if (!Number.isFinite(i)) return undefined;
        return f(i & 0xffff);
      }});
    }
    get CM() {
      const slab = this._readCMSlab, lo = this._cmLoSlab, hi = this._cmHiSlab, buf = this.mod.HEAPU32.buffer;
      return new Proxy({}, { get(_, k) {
        const i = Number(k); if (!Number.isFinite(i)) return undefined;
        slab(i & 0xfff, 1, lo, hi);
        return (new Uint32Array(buf, hi, 1))[0] * 0x100000000
             + (new Uint32Array(buf, lo, 1))[0];
      }});
    }
  }

  function makeEmptySnapshot() {
    return {
      CMAR: 0, PC: 0, MAR: 0, IR: 0, CMDR: 0,
      R: [0,0,0,0,0,0,0,0],
      LBUS: 0, RBUS: 0, ABUS: 0, SBUS: 0, IOBUS: 0, FSR: 0,
      C: 0, ZER: 0, NEG: 0, CRY: 0, OV: 0, CZ: 1, T: 0,
      halt: false, ov_lamp: false,
    };
  }
  function cloneSnapshot(s) {
    return { ...s, R: [...s.R] };
  }

  const MICRO1Microsim = {
    _mod: null,
    MicroCPU,
    disasmCtrl, ctrlHex,
    LBF_NAMES, RBF_NAMES, ALF_NAMES, SHF_NAMES, SBF_NAMES,
    MMF_NAMES, SQF_NAMES, TSF_NAMES, EXF_NAMES,
    CM_WORDS: 0x1000,
    ready: null,
  };
  MICRO1Microsim.ready = loadModule().then(mod => { MICRO1Microsim._mod = mod; });
  global.MICRO1Microsim = MICRO1Microsim;
})(window);
