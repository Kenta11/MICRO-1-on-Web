// WASM-backed MICRO-1 micro-assembler.
//
// Drives github.com/Kenta11/rm1masm (Rust) compiled with wasm-pack.

(function (global) {
  'use strict';

  function toHex3(n) {
    return (n & 0xfff).toString(16).toUpperCase().padStart(3, '0');
  }

  function parseCmDump(text) {
    const lines = text.replace(/\r\n?/g, '\n').split('\n');
    const words = [];
    let title = null;
    let i = 0;
    if (lines.length && /^CM\b/i.test(lines[0])) {
      const m = lines[0].match(/^CM\s+(.*)$/);
      if (m) title = m[1].trim();
      i = 1;
    }
    for (; i < lines.length; i++) {
      const line = lines[i].trim();
      if (!line) continue;
      const m = line.match(/^([0-9A-Fa-f]+)\s+([0-9A-Fa-f]+)$/);
      if (!m) continue;
      const addr = parseInt(m[1], 16);
      const hex  = m[2];
      const hi   = parseInt(hex.slice(0, Math.max(0, hex.length - 8)) || '0', 16);
      const lo   = parseInt(hex.slice(-8), 16);
      words.push({ addr, value: hi * 0x100000000 + lo });
    }
    return { title, words };
  }

  let wasmMod = null;
  const ready = import('./wasm/rm1masm/rm1masm_wasm.js').then(async (m) => {
    await m.default();
    wasmMod = m;
  });

  function microAssemble(source) {
    if (!wasmMod) {
      return { ok: false, errors: ['micro-assembler not yet initialised'],
               words: [], objectText: '', title: null, hasEnd: true };
    }
    // rm1masm's tokeniser only accepts CRLF for lines that carry a `;`
    // comment (its Eol regex is `(;[^\r]*\r)?\n`).  Normalise here so the
    // user can write either Unix or Windows line endings.
    const normalised = source.replace(/\r\n?|\n/g, '\r\n');
    const r = wasmMod.microassemble(normalised);
    if (!r.ok) {
      return {
        ok: false,
        errors: r.errors || ['unknown error'],
        title: r.title || null,
        words: [],
        objectText: r.object_text || '',
        hasEnd: true,
      };
    }
    const parsed = parseCmDump(r.object_text);
    return {
      ok: true,
      errors: [],
      title: r.title || parsed.title,
      words: parsed.words,
      objectText: r.object_text,
      hasEnd: true,
    };
  }

  global.MICRO1Microasm = { microAssemble, parseCmDump, toHex3, ready };
})(window);
