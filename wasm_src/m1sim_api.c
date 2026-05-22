/*
 *  WASM-facing API for the MICRO-1 simulator.
 *
 *  This file is compiled together with m1sim_core.c (which is the
 *  upstream Kenta11/micro1/m1sim.c with the interactive command UI
 *  bypassed and the I/O sites patched to go through m1_io_read_byte /
 *  m1_io_write_byte instead of FILE *).
 *
 *  Everything user-visible (state, step, I/O queues) lives here.  The
 *  goal is to expose just enough surface that the JS side can drive
 *  the C simulator bit-for-bit identically to the upstream binary.
 */

#include <stdint.h>
#include <string.h>
#include <emscripten.h>

/* Forward declarations from the simulator core ------------------------------ */
typedef unsigned long long MCtrlWord;
typedef unsigned short     MInstWord;
typedef unsigned short     MWord;
typedef unsigned int       MAddr;
typedef int                MInt;
typedef enum MBool { MFALSE = 0, MTRUE, } MBool;
typedef enum MemCycle      { MemNop = 0, MemRead, MemWrite, } MemCycle;
typedef enum IoCycle       { IoNop = 0,  IoRead,  IoWrite,  } IoCycle;

#define MAX_CM        0x1000
#define MAX_MM        0x10000
#define MAX_REGS      8
#define MAX_CMARSTACK 8

extern MInstWord IR;
extern MAddr     PC;
extern MWord     LBUS, RBUS, SBUS, ABUS, IOBUS;
extern MWord     R[MAX_REGS];
extern MCtrlWord CMDR;
extern MCtrlWord CM[MAX_CM];
extern MWord     MM[MAX_MM];
extern MWord     CMAR_STACK[MAX_CMARSTACK];
extern MInt      CMAR_STACK_POS;
extern MAddr     MAR;
extern MAddr     CMAR;
extern MInt      C;
extern MBool     CRY, NEG, OV, ZER, CZ, T;
extern MWord     FSR;
extern MBool     HALT_ON, OV_LAMP, BREAK_ON, TRACE_ON;
extern MInt      STEP, MAXSTEP;
extern MemCycle  MEM_CYCLE;
extern IoCycle   IO_CYCLE;

void init_REG(void);
void init_MM(void);
void init_CM(void);
void execute(void);

/* I/O byte queues ----------------------------------------------------------- */
#define M1_IO_BUF 65536
static unsigned char m1_in_buf[M1_IO_BUF];
static int           m1_in_head = 0;
static int           m1_in_tail = 0;
static unsigned char m1_out_buf[M1_IO_BUF];
static int           m1_out_head = 0;
static int           m1_out_tail = 0;

int m1_io_read_byte(void) {
    if (m1_in_head == m1_in_tail) return -1;
    int b = m1_in_buf[m1_in_head];
    m1_in_head = (m1_in_head + 1) % M1_IO_BUF;
    return b;
}

void m1_io_write_byte(int b) {
    int next = (m1_out_tail + 1) % M1_IO_BUF;
    if (next == m1_out_head) return;       /* drop on overflow */
    m1_out_buf[m1_out_tail] = (unsigned char)(b & 0xff);
    m1_out_tail = next;
}

EMSCRIPTEN_KEEPALIVE
void m1_io_input_push(int b) {
    int next = (m1_in_tail + 1) % M1_IO_BUF;
    if (next == m1_in_head) return;
    m1_in_buf[m1_in_tail] = (unsigned char)(b & 0xff);
    m1_in_tail = next;
}

EMSCRIPTEN_KEEPALIVE
void m1_io_input_clear(void) { m1_in_head = m1_in_tail = 0; }

EMSCRIPTEN_KEEPALIVE
int  m1_io_output_pop(void) {
    if (m1_out_head == m1_out_tail) return -1;
    int b = m1_out_buf[m1_out_head];
    m1_out_head = (m1_out_head + 1) % M1_IO_BUF;
    return b;
}

EMSCRIPTEN_KEEPALIVE
void m1_io_output_clear(void) { m1_out_head = m1_out_tail = 0; }

/* Core control -------------------------------------------------------------- */
EMSCRIPTEN_KEEPALIVE
void m1_reset(void) {
    init_REG();
    init_CM();
    init_MM();
    m1_io_input_clear();
    m1_io_output_clear();
    HALT_ON = MFALSE;
    OV_LAMP = MFALSE;
}

EMSCRIPTEN_KEEPALIVE
void m1_reset_regs(void) {
    init_REG();
    HALT_ON = MFALSE;
    OV_LAMP = MFALSE;
}

EMSCRIPTEN_KEEPALIVE
void m1_load_cm(unsigned int addr, unsigned int hi, unsigned int lo) {
    if (addr < MAX_CM) {
        CM[addr] = ((MCtrlWord)hi << 32) | (MCtrlWord)lo;
    }
}

EMSCRIPTEN_KEEPALIVE
void m1_load_mm(unsigned int addr, unsigned int word) {
    if (addr < MAX_MM) MM[addr] = (MWord)(word & 0xffff);
}

EMSCRIPTEN_KEEPALIVE
unsigned int m1_step(void) {
    if (HALT_ON) return 1;
    execute();
    return HALT_ON ? 1 : 0;
}

EMSCRIPTEN_KEEPALIVE
int m1_run(unsigned int max_steps) {
    unsigned int i;
    for (i = 0; i < max_steps && !HALT_ON; i++) {
        execute();
    }
    return (int)i;
}

EMSCRIPTEN_KEEPALIVE
int m1_is_halted(void) { return HALT_ON ? 1 : 0; }

EMSCRIPTEN_KEEPALIVE
void m1_clear_halt(void) { HALT_ON = MFALSE; }

EMSCRIPTEN_KEEPALIVE
void m1_set_cmar(unsigned int v) { CMAR = v & 0xfff; }

EMSCRIPTEN_KEEPALIVE
void m1_set_pc(unsigned int v) { PC = v & 0xffff; }

EMSCRIPTEN_KEEPALIVE
void m1_set_reg(unsigned int i, unsigned int v) {
    if (i < MAX_REGS) R[i] = (MWord)(v & 0xffff);
}

EMSCRIPTEN_KEEPALIVE
void m1_set_ir(unsigned int v)   { IR  = (MInstWord)(v & 0xffff); }
EMSCRIPTEN_KEEPALIVE
void m1_set_mar(unsigned int v)  { MAR = v & 0xffff; }
EMSCRIPTEN_KEEPALIVE
void m1_set_c(unsigned int v)    { C   = (MInt)(v & 0xff); }
EMSCRIPTEN_KEEPALIVE
void m1_set_fsr(unsigned int v)  { FSR = (MWord)(v & 0xf); }

/* Each flag bit gets its own setter so the JS side stays explicit. */
EMSCRIPTEN_KEEPALIVE
void m1_set_flag(unsigned int which, unsigned int bit) {
    bit = bit ? 1 : 0;
    switch (which) {
        case 0: ZER = bit; break;
        case 1: NEG = bit; break;
        case 2: CRY = bit; break;
        case 3: OV  = bit; break;
        case 4: T   = bit; break;
        /* CZ is derived from C == 0; don't expose a setter. */
    }
}

/* Bulk state accessor ------------------------------------------------------- */
/* Layout (32-bit words, little-endian) -- keep in sync with JS reader:
 *   [0] cmar(12) | pc<<16
 *   [1] mar(16)  | ir<<16
 *   [2] cmdr_lo (32 lsb of 40-bit CMDR)
 *   [3] cmdr_hi (8 msb)
 *   [4] r0      | r1<<16
 *   [5] r2      | r3<<16
 *   [6] r4      | r5<<16
 *   [7] r6      | r7<<16
 *   [8] lbus    | rbus<<16
 *   [9] abus    | sbus<<16
 *   [10] iobus  | fsr<<16
 *   [11] c(8)   | flags<<8  (bit0=Z,1=N,2=C,3=OV,4=CZ,5=T,6=HALT,7=OV_LAMP)
 *   [12] mem_cycle | io_cycle<<8 | cmar_stack_pos<<16 | step<<24
 *   [13] reserved
 */
EMSCRIPTEN_KEEPALIVE
void m1_get_state(unsigned int *out) {
    out[0]  = ((unsigned)CMAR & 0xfff) | ((unsigned)PC << 16);
    out[1]  = ((unsigned)MAR & 0xffff) | ((unsigned)IR << 16);
    out[2]  = (unsigned)(CMDR & 0xffffffffull);
    out[3]  = (unsigned)((CMDR >> 32) & 0xffull);
    out[4]  = (unsigned)R[0] | ((unsigned)R[1] << 16);
    out[5]  = (unsigned)R[2] | ((unsigned)R[3] << 16);
    out[6]  = (unsigned)R[4] | ((unsigned)R[5] << 16);
    out[7]  = (unsigned)R[6] | ((unsigned)R[7] << 16);
    out[8]  = (unsigned)LBUS | ((unsigned)RBUS << 16);
    out[9]  = (unsigned)ABUS | ((unsigned)SBUS << 16);
    out[10] = (unsigned)IOBUS | ((unsigned)FSR << 16);
    unsigned flags =
        (ZER ? 0x01 : 0) | (NEG ? 0x02 : 0) |
        (CRY ? 0x04 : 0) | (OV  ? 0x08 : 0) |
        (CZ  ? 0x10 : 0) | (T   ? 0x20 : 0) |
        (HALT_ON ? 0x40 : 0) | (OV_LAMP ? 0x80 : 0);
    out[11] = ((unsigned)C & 0xff) | (flags << 8);
    out[12] = ((unsigned)MEM_CYCLE & 0xff)
            | (((unsigned)IO_CYCLE & 0xff) << 8)
            | (((unsigned)CMAR_STACK_POS & 0xff) << 16);
    out[13] = 0;
}

EMSCRIPTEN_KEEPALIVE
unsigned int m1_read_mm(unsigned int addr) {
    if (addr < MAX_MM) return MM[addr];
    return 0;
}

/* Reads a contiguous slab of MM into a JS-supplied buffer. */
EMSCRIPTEN_KEEPALIVE
void m1_read_mm_slab(unsigned int start, unsigned int count, unsigned short *out) {
    unsigned i;
    for (i = 0; i < count && (start + i) < MAX_MM; i++) {
        out[i] = MM[start + i];
    }
}

/* Reads CM as 32-bit pairs (lo, hi).  out_lo[i] and out_hi[i] form CM[start+i]. */
EMSCRIPTEN_KEEPALIVE
void m1_read_cm_slab(unsigned int start, unsigned int count,
                     unsigned int *out_lo, unsigned int *out_hi) {
    unsigned i;
    for (i = 0; i < count && (start + i) < MAX_CM; i++) {
        MCtrlWord cw = CM[start + i];
        out_lo[i] = (unsigned)(cw & 0xffffffffull);
        out_hi[i] = (unsigned)((cw >> 32) & 0xffull);
    }
}

EMSCRIPTEN_KEEPALIVE
unsigned int m1_cmar_stack_at(unsigned int i) {
    return (i < MAX_CMARSTACK) ? CMAR_STACK[i] : 0;
}
