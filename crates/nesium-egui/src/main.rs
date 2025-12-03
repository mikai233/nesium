mod app;
mod trace;

use std::{env, path::PathBuf};

use anyhow::{Result, anyhow};
use app::{AppConfig, NesiumApp};
use eframe::egui;
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use trace::{run_frame_report, run_trace};

fn main() -> Result<()> {
    let args = parse_args()?;

    if let Some(log_path) = args.trace_log {
        let rom_path = args
            .rom_path
            .clone()
            .ok_or_else(|| anyhow!("trace mode requires a ROM path"))?;
        return run_trace(rom_path, log_path);
    }

    if let Some(frames) = args.check_frames {
        let rom_path = args
            .rom_path
            .clone()
            .ok_or_else(|| anyhow!("frame report mode requires a ROM path"))?;
        return run_frame_report(rom_path, frames);
    }

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nesium")
            .with_inner_size([SCREEN_WIDTH as f32 * 3.0, SCREEN_HEIGHT as f32 * 3.0]),
        ..Default::default()
    };

    let config = AppConfig {
        rom_path: args.rom_path.clone(),
        start_pc: args.start_pc,
    };

    eframe::run_native(
        "Nesium",
        native_options,
        Box::new(|cc| Ok(Box::new(NesiumApp::new(cc, config)))),
    )
    .map_err(|e| anyhow!("eframe failed: {e}"))?;

    Ok(())
}

#[derive(Default)]
struct CliArgs {
    rom_path: Option<PathBuf>,
    trace_log: Option<PathBuf>,
    start_pc: Option<u16>,
    check_frames: Option<usize>,
}

fn parse_args() -> Result<CliArgs> {
    let mut args = env::args().skip(1);
    let mut cli = CliArgs::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--trace-log" => {
                let path = args
                    .next()
                    .ok_or_else(|| anyhow!("--trace-log requires a log file path"))?;
                cli.trace_log = Some(PathBuf::from(path));
            }
            "--start-pc" => {
                let pc_str = args
                    .next()
                    .ok_or_else(|| anyhow!("--start-pc requires a hex address (e.g. 0xC000)"))?;
                let pc = pc_str.trim_start_matches("0x");
                cli.start_pc = Some(
                    u16::from_str_radix(pc, 16)
                        .map_err(|_| anyhow!("--start-pc expects a hex address, got {pc_str}"))?,
                );
            }
            "--check-frame" => {
                let frames = args
                    .next()
                    .ok_or_else(|| anyhow!("--check-frame requires a frame count"))?;
                cli.check_frames = Some(frames.parse()?);
            }
            _ if cli.rom_path.is_none() => cli.rom_path = Some(PathBuf::from(arg)),
            _ => return Err(anyhow!("unexpected argument: {arg}")),
        }
    }

    if (cli.trace_log.is_some() || cli.check_frames.is_some()) && cli.rom_path.is_none() {
        return Err(anyhow!(
            "usage: nesium <path-to-rom.nes> [--trace-log <path-to-nestest.log>] [--start-pc <hex>] [--check-frame <n>]"
        ));
    }

    Ok(cli)
}
