#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;
mod trace;

use std::{env, fs, path::PathBuf};

use anyhow::{Result, anyhow};
use app::{AppConfig, NesiumApp};
use eframe::egui;
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
use trace::run_frame_report;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::FmtSubscriber;

const APP_ID: &str = env!("NESIUM_APP_ID");

include!(concat!(env!("OUT_DIR"), "/egui_icon.rs"));

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

    if let Some(frames) = args.check_frames {
        let rom_path = args
            .rom_path
            .clone()
            .ok_or_else(|| anyhow!("frame report mode requires a ROM path"))?;
        return run_frame_report(rom_path, frames);
    }

    let icon = egui::IconData {
        rgba: ICON_RGBA.to_vec(),
        width: ICON_WIDTH,
        height: ICON_HEIGHT,
    };

    let renderer = choose_renderer();

    let wgpu_options = {
        #[allow(unused_mut)]
        let mut options = eframe::egui_wgpu::WgpuConfiguration {
            desired_maximum_frame_latency: Some(1),
            ..Default::default()
        };

        // On Windows ARM, the default DX12 backend can have driver/toolchain issues that
        // manifest as "everything except text renders" (text uses a textured pipeline).
        // Prefer Vulkan/OpenGL there, while still allowing `WGPU_BACKEND` to override.
        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        {
            let mut setup = eframe::egui_wgpu::WgpuSetupCreateNew::default();
            setup.instance_descriptor.backends = eframe::egui_wgpu::wgpu::Backends::from_env()
                .unwrap_or(
                    eframe::egui_wgpu::wgpu::Backends::VULKAN
                        | eframe::egui_wgpu::wgpu::Backends::GL,
                );
            options.wgpu_setup = eframe::egui_wgpu::WgpuSetup::CreateNew(setup);
        }

        options
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nesium")
            .with_app_id(APP_ID)
            .with_icon(icon)
            .with_inner_size([SCREEN_WIDTH as f32 * 3.0, SCREEN_HEIGHT as f32 * 3.0]),
        // On macOS, the default glow/OpenGL backend can have worse frame pacing than wgpu/Metal.
        renderer,
        wgpu_options,
        ..Default::default()
    };

    let config = AppConfig {
        rom_path: args.rom_path.clone(),
    };

    eframe::run_native(
        "Nesium",
        native_options,
        Box::new(|cc| Ok(Box::new(NesiumApp::new(cc, config)))),
    )
    .map_err(|e| anyhow!("eframe failed: {e}"))?;

    Ok(())
}

fn choose_renderer() -> eframe::Renderer {
    // Allow forcing renderer for troubleshooting:
    // - `NESIUM_EGUI_RENDERER=glow` (OpenGL)
    // - `NESIUM_EGUI_RENDERER=wgpu` (wgpu)
    if let Ok(v) = env::var("NESIUM_EGUI_RENDERER") {
        match v.to_ascii_lowercase().as_str() {
            "glow" | "gl" | "opengl" => return eframe::Renderer::Glow,
            "wgpu" => return eframe::Renderer::Wgpu,
            _ => {}
        }
    }

    // Default to Glow on Windows ARM to avoid known wgpu/DX12 driver issues impacting text.
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return eframe::Renderer::Glow;

    eframe::Renderer::Wgpu
}

#[derive(Default)]
struct CliArgs {
    rom_path: Option<PathBuf>,
    check_frames: Option<usize>,
}

fn parse_args() -> Result<CliArgs> {
    let mut args = env::args().skip(1);
    let mut cli = CliArgs::default();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--trace-log" => {
                // Trace replay no longer supported; keep flag for compatibility.
                let _ = args.next();
                return Err(anyhow!("trace replay mode is no longer supported"));
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

    if cli.check_frames.is_some() && cli.rom_path.is_none() {
        return Err(anyhow!(
            "usage: nesium <path-to-rom.nes> [--check-frame <n>]"
        ));
    }

    Ok(cli)
}
