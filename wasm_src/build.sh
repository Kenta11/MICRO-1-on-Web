#!/usr/bin/env bash
#
# Build the MICRO-1 reference simulator and assemblers as WebAssembly
# modules.  Outputs land in ../js/wasm/.  Assumes EMSDK is sourced
# (emsdk_env.sh) and the Rust wasm32-unknown-unknown target is installed
# along with wasm-pack.

set -euo pipefail

HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$HERE/../js/wasm"
mkdir -p "$OUT"

echo ">> Building m1sim (Emscripten)"

# m1sim_core.c contains the upstream simulator with the I/O sites patched
# to call out to m1_io_read_byte / m1_io_write_byte and the interactive
# main() renamed to a static so it never runs.  emcc inlines the entire
# CISC simulator, including the command UI scaffolding (printf, etc.),
# but since none of it is reachable it gets DCE'd.
emcc \
  "$HERE/m1sim_core.c" "$HERE/m1sim_api.c" \
  -O2 \
  -s WASM=1 \
  -s MODULARIZE=1 \
  -s EXPORT_ES6=0 \
  -s EXPORT_NAME=loadM1Sim \
  -s ENVIRONMENT=web,worker,node \
  -s ALLOW_MEMORY_GROWTH=1 \
  -s INVOKE_RUN=0 \
  -s EXPORTED_RUNTIME_METHODS='["HEAPU8","HEAPU16","HEAPU32","ccall","cwrap"]' \
  -s EXPORTED_FUNCTIONS='["_m1_reset","_m1_reset_regs","_m1_load_cm","_m1_load_mm","_m1_step","_m1_run","_m1_is_halted","_m1_clear_halt","_m1_set_cmar","_m1_set_pc","_m1_set_reg","_m1_set_ir","_m1_set_mar","_m1_set_c","_m1_set_fsr","_m1_set_flag","_m1_io_input_push","_m1_io_input_clear","_m1_io_output_pop","_m1_io_output_clear","_m1_get_state","_m1_read_mm","_m1_read_mm_slab","_m1_read_cm_slab","_m1_cmar_stack_at","_malloc","_free"]' \
  -Wno-everything \
  -o "$OUT/m1sim.js"

echo ">> Building rm1asm (Rust)"
(
  cd "$HERE/rm1asm_wasm"
  wasm-pack build --release --target web --out-dir "$OUT/rm1asm"
)

echo ">> Building rm1masm (Rust)"
(
  cd "$HERE/rm1masm_wasm"
  wasm-pack build --release --target web --out-dir "$OUT/rm1masm"
)

echo "done -> $OUT"
ls -lah "$OUT"
