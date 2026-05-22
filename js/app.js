// MICRO-1 on Web - UI glue.
//
// One simulator (the WASM m1sim), two assemblers (rm1asm for MM, rm1masm
// for CM).  Tabs separate the editors from the simulator view; both
// editors load their output into the same CPU instance.

(function () {
  'use strict';

  const $ = (id) => document.getElementById(id);
  const Mac = window.MICRO1Assembler;
  const Mas = window.MICRO1Microasm;
  const Mic = window.MICRO1Microsim;

  // ============================================================ Sample sets
  const MACRO_SAMPLES = [
    {
      name: 'hello.asm — print a string to LPT',
      body:
`TITLE Hello
; Print "Hello!" by walking a null-terminated string in MM with R1 as index.
        LA   1, MSG          ; R1 = address of first byte
LOOP:   LX   0, (1)          ; R0 = M[R1]; sets Z if zero terminator
        BZ   DONE
        WIO  LPT             ; write low byte of R0 to printer
        LEA  1, 1(1)         ; R1 = R1 + 1
        B    LOOP
DONE:   HLT
MSG:    DC   X"0048          ; 'H'
        DC   X"0065          ; 'e'
        DC   X"006C
        DC   X"006C
        DC   X"006F
        DC   X"0021          ; '!'
        DC   X"000A          ; newline
        DC   0
END
`,
    },
    {
      name: 'echo.asm — copy CR (input) to LPT (output)',
      body:
`TITLE Echo
LOOP:   RIO  CR
        BZ   DONE
        WIO  LPT
        B    LOOP
DONE:   HLT
END
`,
    },
    {
      name: 'sum.asm — sum 1..N via memory operand',
      body:
`TITLE Sum
        LC   2, 0
        L    0, ZERO
        L    1, NVAL
LOOP:   CMP  1, X"0B(2)
        BZ   DONE
        ST   1, TMP
        ADD  0, X"0D(2)
        SUB  1, X"0C(2)
        B    LOOP
DONE:   HLT
NVAL:   DC   10
ZERO:   DC   0
ONE:    DC   1
TMP:    DC   0
END
`,
    },
    {
      name: 'subroutine.asm — BSR / RET',
      body:
`TITLE Sub
        BSR  MYSUB
        HLT
MYSUB:  LC   0, X"AB
        RET
END
`,
    },
  ];

  const MICRO_SAMPLES = [
    {
      name: 'load.mas — set R0=42 then halt',
      body:
`.TITLE Load
; Two microinstructions: load a small literal then halt.
* START: 000
  R0 := ZERO + D"42
* HALT: 001
  SET HLT
.END
`,
    },
    {
      name: 'count.mas — count R0 up to 5 with a loop',
      body:
`.TITLE Count
* INIT: 000
  R0 := R0 XOR R0
* LOOP: 001
  R0 := R0 + 0
  WITH ONE
* CHECK: 002
  R1 := R0 - D"5
* BR: 003
  IF ZER = 0 THEN LOOP
* DONE: 004
  SET HLT
.END
`,
    },
    {
      name: 'flag.mas — observe flag updates',
      body:
`.TITLE Flag
* A: 000
  R0 := R0 XOR R0
* B: 001
  R0 := R0 - D"1
* C: 002
  R0 := R0 + D"1
* HALT: 003
  SET HLT
.END
`,
    },
  ];

  // ============================================================ Simulator state
  const sim = {
    cpu: null,
    runTimer: null,
    cmBase: 0,    followCMAR: true,
    mmBase: 0,    followPC: true,
    cmBreakpoints: new Set(),   // CMAR addresses that halt Run/Macrostep
    mmBreakpoints: new Set(),   // PC addresses, checked after each microstep
    microoneText: null,
  };

  // ============================================================ Helpers
  function flash(el) {
    if (!el) return;
    el.classList.remove('changed');
    void el.offsetWidth;
    el.classList.add('changed');
  }
  function setText(id, txt) { const e = $(id); if (e) e.textContent = txt; }
  function toHex(n, d) { return (n >>> 0).toString(16).toUpperCase().padStart(d, '0'); }

  function setBanner(msg, kind) {
    const el = $('sim-banner');
    if (!el) return;
    el.textContent = msg || '';
    el.classList.remove('error', 'ok');
    if (kind) el.classList.add(kind);
    el.style.display = msg ? '' : 'none';
  }

  function refreshBPStatus() {
    const el = $('bp-status');
    if (!el) return;
    el.textContent = `CM: ${sim.cmBreakpoints.size} / MM: ${sim.mmBreakpoints.size}`;
  }

  // Parse a `.b` or `.cm` hex dump into { title, words: [{addr, value}] }.
  function parseObjectMM(text) {
    const lines = text.split('\n');
    const words = [];
    let title = null, i = 0;
    if (lines.length && /^MM\b/.test(lines[0])) {
      const m = lines[0].match(/^MM\s+(.*)$/);
      if (m) title = m[1].trim();
      i = 1;
    }
    for (; i < lines.length; i++) {
      const m = lines[i].trim().match(/^([0-9A-Fa-f]+)\s+([0-9A-Fa-f]+)$/);
      if (!m) continue;
      words.push({ addr: parseInt(m[1], 16), value: parseInt(m[2], 16) });
    }
    return { title, words };
  }
  function parseObjectCM(text) {
    const lines = text.split('\n');
    const words = [];
    let title = null, i = 0;
    if (lines.length && /^CM\b/i.test(lines[0])) {
      const m = lines[0].match(/^CM\s+(.*)$/);
      if (m) title = m[1].trim();
      i = 1;
    }
    for (; i < lines.length; i++) {
      const m = lines[i].trim().match(/^([0-9A-Fa-f]+)\s+([0-9A-Fa-f]+)$/);
      if (!m) continue;
      const addr = parseInt(m[1], 16);
      const hex  = m[2];
      const lo   = parseInt(hex.slice(-8), 16);
      const hi   = parseInt(hex.slice(0, hex.length - 8) || '0', 16);
      words.push({ addr, value: hi * 0x100000000 + lo });
    }
    return { title, words };
  }

  // ============================================================ Macro / micro editors
  function populateSelect(selectId, samples, onPick) {
    const sel = $(selectId);
    sel.innerHTML = '';
    samples.forEach((s, i) => {
      const o = document.createElement('option');
      o.value = String(i); o.textContent = s.name;
      sel.appendChild(o);
    });
    sel.addEventListener('change', () => onPick(parseInt(sel.value, 10)));
    onPick(0);
  }

  function setLog(id, msg, isError) {
    const el = $(id);
    if (!msg) { el.textContent = ''; return; }
    el.textContent = msg;
    el.style.color = isError ? 'var(--err)' : '';
  }

  let macroAsm = null;
  let microAsm = null;

  function doMacroAssemble() {
    const src = $('macro-source').value;
    const r = Mac.assemble(src);
    macroAsm = r;
    $('macro-object-out').textContent = r.objectText || '';
    if (!r.ok) setLog('macro-log', r.errors.join('\n'), true);
    else       setLog('macro-log', `ok: ${r.words.length} word(s) assembled`, false);
    // Surface the assembled program's lowest address as the default entry.
    if (r.ok && r.words.length) {
      let lo = r.words[0].addr;
      for (const w of r.words) if (w.addr < lo) lo = w.addr;
      $('macro-entry').placeholder = toHex(lo, 4);
    }
    $('btn-macro-load').disabled = !(r.ok && r.words.length);
    return r;
  }
  function doMacroLoad() {
    const r = macroAsm || doMacroAssemble();
    if (!r || !r.ok) return;
    if (!sim.cpu) return;
    sim.cpu.loadMM(r.words);
    // Reset registers so the next microstep starts a fresh FETCH cycle.
    sim.cpu.resetRegs();
    // Resolve entry PC: explicit field overrides the assembler's lowest addr.
    const explicit = $('macro-entry').value.trim();
    let entry = null;
    if (explicit) {
      const v = parseInt(explicit, 16);
      if (Number.isFinite(v) && v >= 0 && v <= 0xffff) entry = v;
      else { setBanner(`Bad entry PC '${explicit}' (expected 4-digit hex)`, 'error'); return; }
    } else if (r.words.length) {
      entry = r.words[0].addr;
      for (const w of r.words) if (w.addr < entry) entry = w.addr;
    }
    if (entry !== null) sim.cpu.PC = entry;
    refreshSimUI();
    setBanner(`Loaded ${r.words.length} word(s) into MM; PC=${toHex(entry || 0, 4)}`, 'ok');
  }

  function doMicroAssemble() {
    const src = $('micro-source').value;
    const r = Mas.microAssemble(src);
    microAsm = r;
    $('micro-object-out').textContent = r.objectText || '';
    if (!r.ok) setLog('micro-log', r.errors.join('\n'), true);
    else       setLog('micro-log', `ok: ${r.words.length} microinstruction(s) assembled`, false);
    if (r.ok && r.words.length) {
      let lo = r.words[0].addr;
      for (const w of r.words) if (w.addr < lo) lo = w.addr;
      $('micro-entry').placeholder = toHex(lo, 3);
    }
    $('btn-micro-load').disabled = !(r.ok && r.words.length);
    return r;
  }
  function doMicroLoad() {
    const r = microAsm || doMicroAssemble();
    if (!r || !r.ok) return;
    if (!sim.cpu) return;
    sim.cpu.loadCM(r.words);
    sim.cpu.resetRegs();
    const explicit = $('micro-entry').value.trim();
    let entry = null;
    if (explicit) {
      const v = parseInt(explicit, 16);
      if (Number.isFinite(v) && v >= 0 && v <= 0xfff) entry = v;
      else { setBanner(`Bad entry CMAR '${explicit}' (expected 3-digit hex)`, 'error'); return; }
    } else if (r.words.length) {
      entry = r.words[0].addr;
      for (const w of r.words) if (w.addr < entry) entry = w.addr;
    }
    if (entry !== null) sim.cpu.CMAR = entry;
    refreshSimUI();
    setBanner(`Loaded ${r.words.length} microinstruction(s) into CM; CMAR=${toHex(entry || 0, 3)}`, 'ok');
  }

  // Load MICROONE (the textbook microprogram) into CM by fetching it from
  // the static samples/ folder.
  async function loadMicroone() {
    setBanner('Loading MICROONE…');
    try {
      if (!sim.microoneText) {
        const res = await fetch('samples/microone.cm');
        if (!res.ok) throw new Error('HTTP ' + res.status);
        sim.microoneText = await res.text();
      }
      const { words } = parseObjectCM(sim.microoneText);
      sim.cpu.loadCM(words);
      sim.cpu.CMAR = 0;
      refreshSimUI();
      setBanner(`MICROONE loaded into CM (${words.length} words)`, 'ok');
    } catch (e) {
      setBanner('failed to load MICROONE: ' + e.message, 'error');
    }
  }

  // ============================================================ Simulator controls
  function simStep() {
    if (!sim.cpu || sim.cpu.halt) return;
    sim.cpu.setInput($('io-input').value);
    sim.cpu.io.output = $('io-output').textContent;
    sim.cpu.step();
    refreshSimUI();
  }
  // Run microsteps until CMAR returns to 0 (next FETCH) — i.e. one macro
  // instruction worth of work has retired.  Capped at 5000 to keep an
  // infinite loop from freezing the page.
  // Read the user-supplied Max steps cap.  Empty / non-numeric => unlimited
  // (matches the upstream m1sim.c behaviour where pressing Enter at the
  // "MAX STEP=?" prompt disables the cap).
  function getMaxSteps() {
    const raw = $('max-steps').value.trim();
    if (!raw) return Infinity;
    const v = parseInt(raw, 10);
    return (Number.isFinite(v) && v > 0) ? v : Infinity;
  }

  function simMacrostep() {
    if (!sim.cpu || sim.cpu.halt) return;
    sim.cpu.setInput($('io-input').value);
    sim.cpu.io.output = $('io-output').textContent;
    const max = getMaxSteps();
    let hit = null;
    let r = stepOnceCheckingBps();
    if (r.bp) hit = r;
    let n = 1;
    while (!sim.cpu.halt && !hit && sim.cpu.CMAR !== 0 && n < max) {
      r = stepOnceCheckingBps();
      if (r.bp) hit = r;
      n++;
    }
    refreshSimUI();
    if (hit) setBanner(`Breakpoint hit: ${hit.bp.toUpperCase()}[${
      hit.bp === 'cm' ? toHex(hit.addr, 3) : toHex(hit.addr, 4)}]`, 'ok');
    else if (!sim.cpu.halt && n >= max && sim.cpu.CMAR !== 0)
      setBanner(`Max steps reached (${max}) without a FETCH return`, 'error');
  }
  // Step the CPU once and tell the caller whether a CM or MM breakpoint
  // was hit (so Run / Macrostep know to stop after the bp's microstep).
  function stepOnceCheckingBps() {
    if (sim.cpu.halt) return { halted: true };
    sim.cpu.step();
    if (sim.cpu.halt) return { halted: true };
    if (sim.cmBreakpoints.has(sim.cpu.CMAR)) return { bp: 'cm', addr: sim.cpu.CMAR };
    if (sim.mmBreakpoints.has(sim.cpu.PC))   return { bp: 'mm', addr: sim.cpu.PC };
    return {};
  }

  function simRun() {
    if (!sim.cpu) return;
    if (sim.runTimer) return;
    sim.cpu.setInput($('io-input').value);
    sim.cpu.io.output = $('io-output').textContent;
    $('btn-run').disabled = true;
    $('btn-step').disabled = true;
    $('btn-macrostep').disabled = true;
    $('btn-stop').disabled = false;
    const burstSize = 2000;
    const max = getMaxSteps();
    const startSteps = sim.cpu.steps;
    const tick = () => {
      if (sim.cpu.halt) { simStop(); refreshSimUI(); return; }
      let hit = null;
      let capped = false;
      for (let i = 0; i < burstSize; i++) {
        if ((sim.cpu.steps - startSteps) >= max) { capped = true; break; }
        const r = stepOnceCheckingBps();
        if (r.halted) break;
        if (r.bp) { hit = r; break; }
      }
      refreshSimUI();
      if (sim.cpu.halt || hit || capped) {
        if (hit) setBanner(`Breakpoint hit: ${hit.bp.toUpperCase()}[${
          hit.bp === 'cm' ? toHex(hit.addr, 3) : toHex(hit.addr, 4)}]`, 'ok');
        else if (capped)
          setBanner(`Max steps reached (${max}). Increase the limit or clear it to keep running.`, 'error');
        simStop();
        return;
      }
      sim.runTimer = setTimeout(tick, 0);
    };
    sim.runTimer = setTimeout(tick, 0);
  }
  function simStop() {
    if (sim.runTimer) clearTimeout(sim.runTimer);
    sim.runTimer = null;
    $('btn-run').disabled = false;
    $('btn-step').disabled = false;
    $('btn-macrostep').disabled = false;
    $('btn-stop').disabled = true;
  }
  function simReset() {
    simStop();
    sim.cpu.reset();
    $('io-output').textContent = '';
    refreshSimUI();
    setBanner('CPU reset (CM and MM cleared)', 'ok');
  }

  // ============================================================ Simulator render
  function refreshSimUI() {
    const cpu = sim.cpu;
    if (!cpu) return;
    const lw = cpu.lastWritten;
    for (let i = 0; i < 8; i++) setText('reg-r' + i, toHex(cpu.R[i], 4));
    if (lw && lw.regs) for (const r of lw.regs) {
      if (typeof r === 'number') flash($('reg-r' + r));
    }
    setText('sys-cmar', toHex(cpu.CMAR, 3));
    setText('sys-pc',   toHex(cpu.PC, 4));
    setText('sys-ir',   toHex(cpu.IR, 4));
    setText('sys-mar',  toHex(cpu.MAR, 4));
    setText('sys-c',    toHex(cpu.C, 2));
    setText('sys-fsr',  toHex(cpu.FSR, 1));
    setText('sys-cmdr', Mic.ctrlHex(cpu.CMDR));
    setText('bus-l',  toHex(cpu.LBUS, 4));
    setText('bus-r',  toHex(cpu.RBUS, 4));
    setText('bus-a',  toHex(cpu.ABUS, 4));
    setText('bus-s',  toHex(cpu.SBUS, 4));
    setText('bus-io', toHex(cpu.IOBUS & 0xff, 2));
    setText('flag-zer', cpu.ZER);
    setText('flag-neg', cpu.NEG);
    setText('flag-cry', cpu.CRY);
    setText('flag-ov',  cpu.OV);
    setText('flag-cz',  cpu.CZ);
    setText('flag-t',   cpu.T);
    const haltLabel = cpu.halt
      ? ('YES' + (cpu.fault_message ? ' (' + cpu.fault_message + ')' : ''))
      : 'no';
    setText('status-halt', haltLabel);
    setText('status-steps', cpu.steps);
    const next = cpu.CM[cpu.CMAR] || 0;
    setText('status-next', Mic.ctrlHex(next));
    setText('status-fields', Mic.disasmCtrl(next));
    const mmNext = cpu.MM[cpu.PC] || 0;
    setText('status-mm-next', toHex(mmNext, 4));
    setText('status-mm-disasm', macroDisasm(mmNext));

    $('io-output').textContent = cpu.io.output;

    if (sim.followCMAR) sim.cmBase = Math.max(0, cpu.CMAR - 2);
    if (sim.followPC)   sim.mmBase = Math.max(0, cpu.PC - 4);
    renderCM();
    renderMM();
  }

  // Minimal disassembler for the macro-level 16-bit instruction at MM[PC].
  function macroDisasm(w) {
    const op0 = (w >> 12) & 0xf;
    const ra  = (w >> 10) & 0x3;
    const rb  = (w >>  8) & 0x3;
    const ub  = w & 0xff;
    const sb  = (ub & 0x80) ? ub - 0x100 : ub;
    const A = { 0:'ADD',1:'SUB',2:'AND',3:'OR',4:'XOR',6:'MULT',7:'DIV',8:'CMP',0xF:'EX' };
    if (op0 in A) return `${A[op0]} ${rb}, ${sb}(${ra})`;
    if (op0 === 5) return `${['SL','SA','SC'][ra]||'?'} ${rb}, ${sb}`;
    if (op0 === 9) {
      if (ra === 3) return `LC ${rb}, X"${toHex(ub, 2)}`;
      return `${['L','ST','LA'][ra]} ${rb}, *${sb>=0?'+':''}${sb}`;
    }
    if (op0 === 0xA) return `LEA ${rb}, ${sb}(${ra})`;
    if (op0 === 0xB) return `LX ${rb}, ${sb}(${ra})`;
    if (op0 === 0xC) return `STX ${rb}, ${sb}(${ra})`;
    if (op0 === 0xD) {
      if (ra === 0) return `PUSH ${rb}, X"${toHex(ub, 2)}`;
      if (ra === 1) return `POP ${rb}, X"${toHex(ub, 2)}`;
      if (ra === 2) return `BIX ${rb}, ${sb}`;
      if (ra === 3) return `BDIS *${sb>=0?'+':''}${sb}`;
    }
    if (op0 === 0xE) {
      const n = (w >> 8) & 0xf;
      const names = ['BP','BZ','BM','BC','BNP','BNZ','BNM','BNC','B','BI','BSR'];
      if (n < names.length) return `${names[n]} *${sb>=0?'+':''}${sb}`;
      if (n === 0xB) return 'RET';
      if (n === 0xC) return `RIO ${ub}`;
      if (n === 0xD) return `WIO ${ub}`;
      if (n === 0xE) return 'NOP';
      if (n === 0xF) return 'HLT';
    }
    return `.WORD X"${toHex(w, 4)}`;
  }

  // Narrow viewports (phones) get a tighter memory display so the hex
  // dump doesn't force horizontal scrolling.
  function isNarrow() {
    return window.matchMedia && window.matchMedia('(max-width: 640px)').matches;
  }

  function renderCM() {
    const cpu = sim.cpu;
    const narrow = isNarrow();
    const linesPerView = narrow ? 18 : 14;
    const start = sim.cmBase & 0xfff;
    const lines = [];
    for (let i = 0; i < linesPerView; i++) {
      const a = (start + i) & 0xfff;
      const cw = cpu.CM[a];
      const isBP = sim.cmBreakpoints.has(a);
      const isPC = (a === cpu.CMAR);
      // Two-char prefix: '●' for BP, '▶' for CMAR.  Both can appear together.
      const prefix = (isBP ? '●' : ' ') + (isPC ? '▶' : ' ');
      const head = prefix + toHex(a, 3) + ': ' + Mic.ctrlHex(cw);
      const dis  = Mic.disasmCtrl(cw);
      if (narrow) {
        lines.push(head);
        lines.push('       ' + dis);
      } else {
        lines.push(head + '  ' + dis);
      }
    }
    $('cm-view').textContent = lines.join('\n');
    $('cm-base').value = toHex(start, 3);
  }

  function renderMM() {
    const cpu = sim.cpu;
    const narrow = isNarrow();
    // One macro instruction per line + its disassembly, matching the CM
    // pane layout so users can read the program directly.
    const linesPerView = narrow ? 16 : 20;
    const start = sim.mmBase & 0xffff;
    const lines = [];
    for (let i = 0; i < linesPerView; i++) {
      const a = (start + i) & 0xffff;
      const word = cpu.MM[a];
      const isBP = sim.mmBreakpoints.has(a);
      const isPC = (a === cpu.PC);
      const prefix = (isBP ? '●' : ' ') + (isPC ? '▶' : ' ');
      lines.push(prefix + toHex(a, 4) + ':  ' + toHex(word, 4) + '  ' + macroDisasm(word));
      if (a === 0xffff) break;
    }
    $('mm-view').textContent = lines.join('\n');
    $('mm-base').value = toHex(start, 4);
  }

  // ============================================================ Tab switching
  function setMode(mode) {
    document.querySelectorAll('.tab-button').forEach(b => {
      b.classList.toggle('active', b.dataset.mode === mode);
    });
    $('sim-view').classList.toggle('active', mode === 'sim');
    $('macro-view').classList.toggle('active', mode === 'macro');
    $('micro-view').classList.toggle('active', mode === 'micro');
  }

  // ============================================================ Init
  async function init() {
    document.querySelectorAll('.tab-button').forEach(b => {
      b.addEventListener('click', () => setMode(b.dataset.mode));
    });

    setBanner('Loading WASM modules…');
    try {
      await Promise.all([Mac.ready, Mas.ready, Mic.ready]);
    } catch (e) {
      setBanner('Failed to load WASM modules: ' + e.message, 'error');
      console.error(e);
      return;
    }

    sim.cpu = new Mic.MicroCPU();

    // Preload MICROONE so macro programs can run out of the box.
    await loadMicroone();
    refreshBPStatus();
    refreshSimUI();

    populateSelect('macro-sample-select', MACRO_SAMPLES, i => {
      $('macro-source').value = MACRO_SAMPLES[i].body;
      setLog('macro-log', '');
    });
    populateSelect('micro-sample-select', MICRO_SAMPLES, i => {
      $('micro-source').value = MICRO_SAMPLES[i].body;
      setLog('micro-log', '');
    });

    // Macro editor wiring.
    $('btn-macro-assemble').addEventListener('click', doMacroAssemble);
    $('btn-macro-load').addEventListener('click', () => {
      doMacroAssemble();
      doMacroLoad();
    });
    $('macro-source').addEventListener('keydown', e => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault(); doMacroAssemble(); doMacroLoad();
      }
    });

    // Micro editor wiring.
    $('btn-micro-assemble').addEventListener('click', doMicroAssemble);
    $('btn-micro-load').addEventListener('click', () => {
      doMicroAssemble();
      doMicroLoad();
    });
    $('micro-source').addEventListener('keydown', e => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
        e.preventDefault(); doMicroAssemble(); doMicroLoad();
      }
    });

    // Simulator controls.
    $('btn-step').addEventListener('click', simStep);
    $('btn-macrostep').addEventListener('click', simMacrostep);
    $('btn-run').addEventListener('click', simRun);
    $('btn-stop').addEventListener('click', simStop);
    $('btn-reset').addEventListener('click', simReset);
    $('btn-load-microone').addEventListener('click', () => loadMicroone().then(refreshSimUI));

    // Memory navigation.
    $('btn-cm-prev').addEventListener('click', () => {
      sim.followCMAR = false;
      sim.cmBase = Math.max(0, sim.cmBase - 14);
      renderCM();
    });
    $('btn-cm-next').addEventListener('click', () => {
      sim.followCMAR = false;
      sim.cmBase = Math.min(0xfff, sim.cmBase + 14);
      renderCM();
    });
    $('btn-cm-follow').addEventListener('click', () => {
      sim.followCMAR = true; refreshSimUI();
    });
    $('cm-base').addEventListener('change', e => {
      const v = parseInt(e.target.value, 16);
      if (!isNaN(v)) { sim.followCMAR = false; sim.cmBase = v & 0xfff; renderCM(); }
    });
    $('btn-mm-prev').addEventListener('click', () => {
      sim.followPC = false;
      sim.mmBase = Math.max(0, sim.mmBase - 0x10);
      renderMM();
    });
    $('btn-mm-next').addEventListener('click', () => {
      sim.followPC = false;
      sim.mmBase = Math.min(0xffff, sim.mmBase + 0x10);
      renderMM();
    });
    $('btn-mm-follow').addEventListener('click', () => {
      sim.followPC = true; refreshSimUI();
    });
    $('mm-base').addEventListener('change', e => {
      const v = parseInt(e.target.value, 16);
      if (!isNaN(v)) { sim.followPC = false; sim.mmBase = v & 0xffff; renderMM(); }
    });

    // ------- Direct memory editing ----------------------------------
    // CM[addr] := 40-bit hex value.  Either field accepts plain hex
    // digits (no `0x` / `X"` prefix).  Empty value is treated as 0.
    function commitCMEdit() {
      const aStr = $('cm-edit-addr').value.trim();
      const vStr = $('cm-edit-value').value.trim();
      if (!aStr) { setBanner('CM 編集: アドレスを入れてください', 'error'); return; }
      const addr = parseInt(aStr, 16);
      if (!Number.isFinite(addr) || addr < 0 || addr > 0xfff) {
        setBanner(`CM 編集: アドレス '${aStr}' が範囲外 (0..FFF)`, 'error'); return;
      }
      const hex = (vStr || '0').padStart(10, '0');
      if (!/^[0-9A-Fa-f]+$/.test(hex)) {
        setBanner(`CM 編集: 値 '${vStr}' が hex として不正`, 'error'); return;
      }
      const lo = parseInt(hex.slice(-8), 16);
      const hi = parseInt(hex.slice(0, hex.length - 8) || '0', 16);
      sim.cpu._loadCM(addr & 0xfff, hi >>> 0, lo >>> 0);
      sim.cpu._refresh();
      refreshSimUI();
      setBanner(`CM[${toHex(addr,3)}] := ${hex.toUpperCase().padStart(10,'0')}`, 'ok');
    }
    $('btn-cm-edit').addEventListener('click', commitCMEdit);
    $('cm-edit-value').addEventListener('keydown', e => {
      if (e.key === 'Enter') { e.preventDefault(); commitCMEdit(); }
    });
    $('cm-edit-addr').addEventListener('keydown', e => {
      if (e.key === 'Enter') { e.preventDefault(); commitCMEdit(); }
    });

    function commitMMEdit() {
      const aStr = $('mm-edit-addr').value.trim();
      const vStr = $('mm-edit-value').value.trim();
      if (!aStr) { setBanner('MM 編集: アドレスを入れてください', 'error'); return; }
      const addr = parseInt(aStr, 16);
      if (!Number.isFinite(addr) || addr < 0 || addr > 0xffff) {
        setBanner(`MM 編集: アドレス '${aStr}' が範囲外 (0..FFFF)`, 'error'); return;
      }
      const val = parseInt(vStr || '0', 16);
      if (!Number.isFinite(val) || val < 0 || val > 0xffff) {
        setBanner(`MM 編集: 値 '${vStr}' が範囲外 (0..FFFF)`, 'error'); return;
      }
      sim.cpu._loadMM(addr & 0xffff, val & 0xffff);
      sim.cpu._refresh();
      refreshSimUI();
      setBanner(`MM[${toHex(addr,4)}] := ${toHex(val,4)}`, 'ok');
    }
    $('btn-mm-edit').addEventListener('click', commitMMEdit);
    $('mm-edit-value').addEventListener('keydown', e => {
      if (e.key === 'Enter') { e.preventDefault(); commitMMEdit(); }
    });
    $('mm-edit-addr').addEventListener('keydown', e => {
      if (e.key === 'Enter') { e.preventDefault(); commitMMEdit(); }
    });

    // Clicking inside the memory pre views auto-fills the edit address
    // with the line the click landed on, so the user doesn't have to
    // retype it.
    function lineAtClick(preEl, evt) {
      const rect = preEl.getBoundingClientRect();
      const text = preEl.textContent;
      const lh = parseFloat(getComputedStyle(preEl).lineHeight) || 16;
      const rowIndex = Math.floor((evt.clientY - rect.top - 8) / lh);
      const lines = text.split('\n');
      return lines[rowIndex] || '';
    }
    $('cm-view').addEventListener('click', e => {
      const line = lineAtClick($('cm-view'), e);
      const m = line.match(/[●▶\s]*([0-9A-Fa-f]{3}):/);
      if (m) $('cm-edit-addr').value = m[1].toUpperCase();
    });
    $('mm-view').addEventListener('click', e => {
      const line = lineAtClick($('mm-view'), e);
      const m = line.match(/([0-9A-Fa-f]{4}):/);
      if (m) $('mm-edit-addr').value = m[1].toUpperCase();
    });

    // ------- Breakpoints ------------------------------------------------
    function toggleCmBP(addr) {
      if (sim.cmBreakpoints.has(addr)) sim.cmBreakpoints.delete(addr);
      else sim.cmBreakpoints.add(addr);
      refreshBPStatus();
      renderCM();
    }
    function toggleMmBP(addr) {
      if (sim.mmBreakpoints.has(addr)) sim.mmBreakpoints.delete(addr);
      else sim.mmBreakpoints.add(addr);
      refreshBPStatus();
      renderMM();
    }
    $('btn-cm-bp').addEventListener('click', () => {
      const s = $('cm-edit-addr').value.trim();
      if (!s) { setBanner('CM BP: アドレスを入れてください', 'error'); return; }
      const v = parseInt(s, 16);
      if (!Number.isFinite(v) || v < 0 || v > 0xfff) {
        setBanner(`CM BP: アドレス '${s}' が範囲外`, 'error'); return;
      }
      toggleCmBP(v);
      setBanner(`CM BP ${sim.cmBreakpoints.has(v) ? 'set' : 'cleared'} @ ${toHex(v, 3)}`, 'ok');
    });
    $('btn-mm-bp').addEventListener('click', () => {
      const s = $('mm-edit-addr').value.trim();
      if (!s) { setBanner('MM BP: アドレスを入れてください', 'error'); return; }
      const v = parseInt(s, 16);
      if (!Number.isFinite(v) || v < 0 || v > 0xffff) {
        setBanner(`MM BP: アドレス '${s}' が範囲外`, 'error'); return;
      }
      toggleMmBP(v);
      setBanner(`MM BP ${sim.mmBreakpoints.has(v) ? 'set' : 'cleared'} @ ${toHex(v, 4)}`, 'ok');
    });
    $('btn-bp-clear').addEventListener('click', () => {
      sim.cmBreakpoints.clear();
      sim.mmBreakpoints.clear();
      refreshBPStatus();
      refreshSimUI();
      setBanner('全 BP を解除しました', 'ok');
    });

    // Re-render memory views when crossing the narrow / wide breakpoint so
    // the word-per-line and line-count adjust to the new viewport.
    if (window.matchMedia) {
      const mq = window.matchMedia('(max-width: 640px)');
      mq.addEventListener('change', () => refreshSimUI());
    }
    window.addEventListener('resize', () => {
      clearTimeout(window.__resizeTimer);
      window.__resizeTimer = setTimeout(() => refreshSimUI(), 100);
    });

    // Click-to-edit for register / control / flag cells.  Delegation
    // means freshly rendered cells inherit the behaviour automatically.
    document.querySelector('.cpu-state').addEventListener('click', e => {
      const td = e.target.closest('td.editable');
      if (!td) return;
      beginCellEdit(td);
    });
  }

  function beginCellEdit(td) {
    if (td.querySelector('input')) return;       // already editing
    const original = td.textContent;
    const width = parseInt(td.dataset.width, 10) || 4;
    const input = document.createElement('input');
    input.type = 'text';
    input.className = 'cell-edit-input';
    input.value = original;
    input.maxLength = Math.max(width, 4);
    input.style.width = (Math.max(width, 4) + 2) + 'ch';
    td.textContent = '';
    td.appendChild(input);
    input.focus();
    input.select();

    let finished = false;
    const cancel = () => { if (finished) return; finished = true; td.textContent = original; };
    const commit = () => {
      if (finished) return;
      const raw = input.value.trim();
      try { applyCellEdit(td, raw); }
      catch (err) {
        setBanner('編集エラー: ' + err.message, 'error');
      }
      finished = true;
      refreshSimUI();
    };
    input.addEventListener('keydown', e => {
      if (e.key === 'Enter')  { e.preventDefault(); commit(); }
      if (e.key === 'Escape') { e.preventDefault(); cancel(); }
    });
    input.addEventListener('blur', commit);
  }

  function applyCellEdit(td, raw) {
    const kind = td.dataset.edit;
    const cpu = sim.cpu;
    if (kind === 'reg') {
      const idx = parseInt(td.dataset.idx, 10);
      const v = parseInt(raw || '0', 16);
      if (!Number.isFinite(v) || v < 0 || v > 0xffff) throw new Error('value out of range');
      cpu.setReg(idx, v);
    } else if (kind === 'pc' || kind === 'ir' || kind === 'mar') {
      const v = parseInt(raw || '0', 16);
      if (!Number.isFinite(v) || v < 0 || v > 0xffff) throw new Error('value out of range');
      if (kind === 'pc')  cpu.PC  = v;
      if (kind === 'ir')  cpu.IR  = v;
      if (kind === 'mar') cpu.MAR = v;
    } else if (kind === 'cmar') {
      const v = parseInt(raw || '0', 16);
      if (!Number.isFinite(v) || v < 0 || v > 0xfff) throw new Error('CMAR out of range (0..FFF)');
      cpu.CMAR = v;
    } else if (kind === 'c') {
      const v = parseInt(raw || '0', 16);
      if (!Number.isFinite(v) || v < 0 || v > 0xff) throw new Error('C out of range (0..FF)');
      cpu.C = v;
    } else if (kind === 'fsr') {
      const v = parseInt(raw || '0', 16);
      if (!Number.isFinite(v) || v < 0 || v > 0xf) throw new Error('FSR out of range (0..F)');
      cpu.FSR = v;
    } else if (kind === 'flag') {
      const v = parseInt(raw || '0', 10);
      if (v !== 0 && v !== 1) throw new Error('flag must be 0 or 1');
      const which = td.dataset.which;
      if (which === 'zer') cpu.ZER = v;
      else if (which === 'neg') cpu.NEG = v;
      else if (which === 'cry') cpu.CRY = v;
      else if (which === 'ov')  cpu.OV  = v;
      else if (which === 't')   cpu.T   = v;
    }
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
  } else {
    init();
  }
})();
