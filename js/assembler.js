// WASM-backed MICRO-1 macro assembler.
//
// Drives github.com/Kenta11/rm1asm (Rust) compiled with wasm-pack.  Keeps
// the same JS-side surface as the prior pure-JS port so app.js does not
// need to change: an `assemble(src)` call returns an object with
// `ok`, `errors`, `objectText`, `words`, `title`, `hasEnd`.
//
// The wasm module needs to be initialised before `assemble` works.  Wait
// on `MICRO1Assembler.ready` before calling.

(function (global) {
  'use strict';

  function parseObject(text) {
    // ".b" format: header line "MM <title>", then "<addr_4hex>  <word_4hex>".
    const lines = text.split('\n');
    let title = null;
    const words = [];
    let i = 0;
    if (lines.length && /^MM\b/.test(lines[0])) {
      const m = lines[0].match(/^MM\s+(.*)$/);
      if (m) title = m[1].trim();
      i = 1;
    }
    for (; i < lines.length; i++) {
      const line = lines[i].trim();
      if (!line) continue;
      const m = line.match(/^([0-9A-Fa-f]+)\s+([0-9A-Fa-f]+)$/);
      if (!m) continue;
      words.push({ addr: parseInt(m[1], 16), value: parseInt(m[2], 16) });
    }
    return { title, words };
  }

  function toHex4(n) {
    return (n & 0xffff).toString(16).toUpperCase().padStart(4, '0');
  }

  let wasmMod = null;
  // Paths resolve relative to this script (`/js/assembler.js`), so the WASM
  // bundle directory is one level down.
  const ready = import('./wasm/rm1asm/rm1asm_wasm.js').then(async (m) => {
    await m.default();
    wasmMod = m;
  });

  function assemble(source) {
    if (!wasmMod) {
      return { ok: false, errors: ['assembler not yet initialised'], words: [],
               objectText: '', title: null, hasEnd: true };
    }
    const r = wasmMod.assemble(source);
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
    const parsed = parseObject(r.object_text);
    return {
      ok: true,
      errors: [],
      title: r.title || parsed.title,
      words: parsed.words,
      objectText: r.object_text,
      hasEnd: true,
    };
  }

  global.MICRO1Assembler = { assemble, toHex4, ready };
})(window);
