mod app;
mod trace;

use std::{env, fs, path::PathBuf};

use anyhow::{Result, anyhow};
use app::{AppConfig, NesiumApp};
use eframe::egui;
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use trace::{run_frame_report, run_trace};
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::FmtSubscriber;

fn init_tracing() -> WorkerGuard {
    // 确保每次运行都从一个新的日志文件开始（覆盖旧内容）
    let _ = fs::remove_file("nesium_ppu_boot.log");

    // non-blocking 的文件 appender
    let file_appender = tracing_appender::rolling::never(".", "nesium_ppu_boot.log");
    let (non_blocking_writer, guard) =
        tracing_appender::non_blocking::NonBlockingBuilder::default()
            .lossy(false) // 关闭丢弃，缓冲区满时阻塞
            .buffered_lines_limit(1024 * 10) // 增大缓冲区
            .finish(file_appender);

    // 只要“消息本身”：不要时间、不要 level、不要 target
    let format = tracing_subscriber::fmt::format()
        .without_time()
        .with_level(false)
        .with_target(false);

    // 只输出到文件，不输出到控制台
    let subscriber = FmtSubscriber::builder()
        .event_format(format)
        .with_max_level(Level::DEBUG)
        .with_ansi(false) // 禁止颜色
        .with_file(false) // 不输出文件名
        .with_line_number(false) // 不输出行号
        .with_env_filter("nesium_core=debug")
        .with_thread_ids(false) // 不输出线程 id
        .with_thread_names(false) // 不输出线程名
        .with_writer(non_blocking_writer) // 写入文件（non-blocking）
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    guard
}

fn main() -> Result<()> {
    // let _guard = init_tracing();
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
