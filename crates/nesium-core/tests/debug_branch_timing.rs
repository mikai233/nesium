#![allow(dead_code)]

use anyhow::Result;
use nesium_core::NES;

const ROM: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/vendor/nes-test-roms/branch_timing_tests/1.Branch_Basics.nes"
);

/// Logs rising edges of NMI output (should be once per frame).
#[test]
#[ignore = "debug helper"]
fn log_nmi_edges() -> Result<()> {
    let mut nes = NES::new();
    nes.load_cartridge_from_file(ROM)?;

    let mut last_output = nes.ppu_nmi_debug().nmi_output;
    let mut prev_dot = 0;
    let mut edges = 0;
    let mut guard: u64 = 0;
    let mut last_vblank_set_dot: u64 = 0;
    let mut last_vblank_clear_dot: u64 = 0;
    let mut falls = 0;
    let mut last_edge_dot: u64 = 0;
    while edges < 4 {
        nes.clock_dot();
        guard += 1;
        if guard > 2_000_000 {
            panic!("timeout waiting for NMI output edges (got {edges})");
        }
        let state = nes.ppu_nmi_debug();
        // Heuristic VBlank boundary logs (helps detect stuck VBlank/scanline wrap).
        let now_dot = nes.dot_counter();
        // If the ROM disables NMI, we may stop seeing rising edges.
        // Heuristic: if we saw at least one edge but none for ~2 frames, stop early.
        const DOTS_PER_FRAME: u64 = 341 * 262;
        if edges > 0 && last_edge_dot > 0 && now_dot - last_edge_dot > 2 * DOTS_PER_FRAME {
            println!(
                "No NMI rising edge for >2 frames after last edge (last at dot {}). Assuming NMI disabled by ROM; stopping.",
                last_edge_dot
            );
            break;
        }
        if state.scanline == 241 && state.cycle == 1 && now_dot != last_vblank_set_dot {
            println!(
                "VBlank set at dot {}, frame {}, scanline {}, cycle {}, nmi_output {}, pending {}",
                now_dot,
                state.frame,
                state.scanline,
                state.cycle,
                state.nmi_output,
                state.nmi_pending
            );
            last_vblank_set_dot = now_dot;
        }
        if state.scanline == -1 && state.cycle == 1 && now_dot != last_vblank_clear_dot {
            println!(
                "VBlank clear at dot {}, frame {}, scanline {}, cycle {}, nmi_output {}, pending {}",
                now_dot,
                state.frame,
                state.scanline,
                state.cycle,
                state.nmi_output,
                state.nmi_pending
            );
            last_vblank_clear_dot = now_dot;
        }

        // Log NMI output falling edges (should happen once per frame too).
        if !state.nmi_output && last_output {
            falls += 1;
            println!(
                "NMI fall {} at dot {}, frame {}, scanline {}, cycle {}, pending {}",
                falls, now_dot, state.frame, state.scanline, state.cycle, state.nmi_pending
            );
        }
        if state.nmi_output && !last_output {
            let now = now_dot;
            let delta = if edges == 0 { 0 } else { now - prev_dot };
            println!(
                "NMI edge {} at dots {}, delta {} (~{:.2} CPU cycles), frame {}, scanline {}, cycle {}, pending {}",
                edges + 1,
                now,
                delta,
                delta as f64 / 3.0,
                state.frame,
                state.scanline,
                state.cycle,
                state.nmi_pending
            );
            prev_dot = now;
            last_edge_dot = now;
            edges += 1;
        }
        last_output = state.nmi_output;
    }
    Ok(())
}
