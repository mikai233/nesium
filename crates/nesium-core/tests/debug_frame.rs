use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use nesium_core::{
    CpuSnapshot, Nes,
    cpu::{addressing::Addressing, opcode_meta},
    ppu::nmi_debug_state::NmiDebugState,
};

/// Instruction-level trace in a Mesen-like format:
///
/// PC  DISASM/OPBYTES  A:.. X:.. Y:.. S:.. P:nv--dIzC  V:scanline H:dot  Fr:frame  Cycle:cpu_cycles
///
/// Notes:
/// - This file intentionally focuses on CPU/PPU timing alignment.
/// - Disassembly and effective addresses are derived from the opcode table via `opcode_meta`.
/// - Frame/scanline/dot are taken from ppu_nmi_debug(); adjust field names if needed.
pub fn dump_instruction_trace<P: AsRef<Path>, Q: AsRef<Path>>(
    rom_path: P,
    out_path: Q,
    start_frame: u32,
    end_frame: u32,
    max_instructions: usize,
) {
    let mut nes = Nes::default();
    nes.load_cartridge_from_file(&rom_path).expect("load rom");

    let file = File::create(out_path).expect("create trace file");
    let mut w = BufWriter::new(file);

    // Run until we enter the requested frame window.
    while nes.ppu_nmi_debug().frame < start_frame {
        nes.run_frame(false);
    }

    // Advance PPU naturally until it reaches the same visible boundary Mesen logs (frame 1, V:0, H:27).
    // If we need to align to a visible boundary, run the machine normally until V:0/H:27/Fr:1.
    // This keeps CPU/PPU in sync (Mesen2 logs its first line there).
    if start_frame == 0 {
        while {
            let dbg = nes.ppu_nmi_debug();
            dbg.frame != 1 || dbg.scanline != 0 || dbg.cycle != 27
        } {
            nes.clock_cpu_cycle(false);
        }
        while nes.cpu_opcode_active() {
            nes.step_instruction();
        }
    }

    // Align CPU registers to the same power-on state Mesen2 logs use.
    // This makes the first few instructions comparable (A/Y/S/P match its reset dump).
    {
        let mut snap = nes.cpu_snapshot();
        snap.pc = 0xEB59; // Start at same reset vector address logged by Mesen2
        snap.a = 0x16;
        snap.x = 0x00;
        snap.y = 0x16;
        snap.s = 0x80;
        snap.p = 0x27; // I,Z,C set; N,V,D clear; bit5 reserved set
        nes.set_cpu_snapshot(snap);
    }

    let mut instr_count = 0usize;

    while instr_count < max_instructions {
        let snap = nes.cpu_snapshot();
        let nmi_dbg = nes.ppu_nmi_debug();

        let fr = nmi_dbg.frame;
        if fr > end_frame {
            break;
        }

        let cpu_cyc = cpu_cycle_in_frame(&nmi_dbg);

        // CPU registers
        let pc = snap.pc;
        let a = snap.a;
        let x = snap.x;
        let y = snap.y;
        let sp = snap.s;
        let p: u8 = snap.p;

        // PPU position (adjust names if your NmiDebugState differs)
        let v = nmi_dbg.scanline as i32;
        let h = nmi_dbg.cycle as i32;

        let disasm = format_instruction_line(&mut nes, &snap);

        writeln!(
            w,
            "{pc:04X}   {disasm:<25} A:{a:02X} X:{x:02X} Y:{y:02X} S:{sp:02X} P:{} V:{v:<3} H:{h:<3}  Fr:{fr} Cycle:{cpu_cyc}",
            fmt_p_flags(p),
        ).expect("write trace line");

        nes.step_instruction();
        instr_count += 1;
    }

    w.flush().ok();
}

fn fmt_p_flags(p: u8) -> String {
    let n = if p & 0x80 != 0 { 'N' } else { 'n' };
    let v = if p & 0x40 != 0 { 'V' } else { 'v' };
    let d = if p & 0x08 != 0 { 'D' } else { 'd' };
    let i = if p & 0x04 != 0 { 'I' } else { 'i' };
    let z = if p & 0x02 != 0 { 'Z' } else { 'z' };
    let c = if p & 0x01 != 0 { 'C' } else { 'c' };
    format!("{n}{v}--{d}{i}{z}{c}")
}

fn cpu_cycle_in_frame(nmi_dbg: &NmiDebugState) -> u64 {
    // Mesen reports CPU cycles within the current frame; it counts PPU dots from 1.
    const DOTS_PER_SCANLINE: i64 = 341;
    const SCANLINES_PER_FRAME: i64 = 262;

    let scanline = if nmi_dbg.scanline < 0 {
        SCANLINES_PER_FRAME - 1
    } else {
        nmi_dbg.scanline as i64
    };
    let dot_index = scanline * DOTS_PER_SCANLINE + nmi_dbg.cycle as i64;
    if dot_index <= 0 {
        0
    } else {
        ((dot_index - 1) / 3) as u64
    }
}

fn format_instruction_line(nes: &mut Nes, snap: &CpuSnapshot) -> String {
    let mut bytes = [0u8; 3];
    nes.peek_cpu_slice(snap.pc, &mut bytes);
    let opcode = bytes[0];
    let meta = opcode_meta(opcode);
    let operand_len = meta.addressing.operand_len();
    let operands = &bytes[1..1 + operand_len];
    format_operands(&meta.mnemonic, meta.addressing, operands, snap, nes)
}

fn format_operands(
    mnemonic: &str,
    addressing: Addressing,
    operands: &[u8],
    snap: &CpuSnapshot,
    _nes: &mut Nes,
) -> String {
    match addressing {
        Addressing::Implied => mnemonic.to_string(),
        Addressing::Accumulator => format!("{mnemonic} A"),
        Addressing::Immediate => format!("{mnemonic} #${:02X}", operands[0]),
        Addressing::Relative => {
            let offset = operands[0] as i8 as i16;
            let target = snap.pc.wrapping_add(2).wrapping_add(offset as u16);
            format!("{mnemonic} ${target:04X}")
        }
        Addressing::ZeroPage => {
            let addr = operands[0];
            format!("{mnemonic} ${addr:02X}")
        }
        Addressing::ZeroPageX => {
            let addr = operands[0];
            format!("{mnemonic} ${addr:02X},X")
        }
        Addressing::ZeroPageY => {
            let addr = operands[0];
            format!("{mnemonic} ${addr:02X},Y")
        }
        Addressing::Absolute => {
            let base = read_u16(operands);
            format!("{mnemonic} ${base:04X}")
        }
        Addressing::AbsoluteX => {
            let base = read_u16(operands);
            format!("{mnemonic} ${base:04X},X")
        }
        Addressing::AbsoluteY => {
            let base = read_u16(operands);
            format!("{mnemonic} ${base:04X},Y")
        }
        Addressing::Indirect => {
            let base = read_u16(operands);
            format!("{mnemonic} (${base:04X})")
        }
        Addressing::IndirectX => {
            let zp = operands[0];
            format!("{mnemonic} (${zp:02X},X)")
        }
        Addressing::IndirectY => {
            let zp = operands[0];
            format!("{mnemonic} (${zp:02X}),Y")
        }
    }
}

fn read_u16(operands: &[u8]) -> u16 {
    u16::from_le_bytes([operands[0], operands[1]])
}

#[test]
#[ignore = "manual test"]
fn debug_dump_instructions_example() {
    let rom = "/Users/mikai/RustroverProjects/nesium/crates/nesium-core/vendor/nes-test-roms/instr_test-v5/all_instrs.nes";
    let out = "/Users/mikai/RustroverProjects/nesium/nesium_instr.log";

    // Dump instructions from frame 0..10, up to 200k instructions.
    dump_instruction_trace(rom, out, 0, 10, 200_000);
}
