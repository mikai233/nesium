#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

mod app;

use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use app::{AppConfig, NesiumApp};
use clap::{CommandFactory, Parser, error::ErrorKind};
use eframe::egui;
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};
#[cfg(all(windows, not(debug_assertions)))]
use rfd::MessageDialog;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::FmtSubscriber;

const APP_ID: &str = env!("NESIUM_APP_ID");

include!(concat!(env!("OUT_DIR"), "/egui_icon.rs"));

#[cfg(all(windows, not(debug_assertions)))]
fn has_console_attached() -> bool {
    // Prefer checking whether we have an attached console, rather than whether stderr is a TTY.
    // This avoids popping a dialog when stderr is redirected from a real terminal.
    unsafe {
        type Handle = isize;
        type Hwnd = isize;

        const STD_ERROR_HANDLE: i32 = -12;
        const INVALID_HANDLE_VALUE: Handle = -1;

        #[link(name = "kernel32")]
        unsafe extern "system" {
            fn GetConsoleWindow() -> Hwnd;
            fn GetStdHandle(n_std_handle: i32) -> Handle;
            fn GetConsoleMode(handle: Handle, mode: *mut u32) -> i32;
        }

        if GetConsoleWindow() != 0 {
            return true;
        }

        let stderr_handle = GetStdHandle(STD_ERROR_HANDLE);
        if stderr_handle == 0 || stderr_handle == INVALID_HANDLE_VALUE {
            return false;
        }

        let mut mode = 0u32;
        GetConsoleMode(stderr_handle, &mut mode) != 0
    }
}

fn exit_with_cli_error(message: &str, exit_code: i32) -> ! {
    eprintln!("{message}");

    // On Windows release builds we use the GUI subsystem (no console window). If the app is
    // launched without an attached console, stderr output is invisible; show a dialog in that case.
    #[cfg(all(windows, not(debug_assertions)))]
    if !has_console_attached() {
        let _ = MessageDialog::new()
            .set_title("Nesium")
            .set_description(message)
            .show();
    }

    std::process::exit(exit_code);
}

fn exit_with_cli_info(message: &str) -> ! {
    println!("{message}");

    #[cfg(all(windows, not(debug_assertions)))]
    if !has_console_attached() {
        let _ = MessageDialog::new()
            .set_title("Nesium")
            .set_description(message)
            .show();
    }

    std::process::exit(0);
}

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

fn parse_cli_args_or_exit() -> CliArgs {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();
    CliArgs::try_parse().unwrap_or_else(|err| {
        // For help/version, render text ourselves so it can also be shown in a dialog when the
        // GUI-subsystem binary is launched without a console.
        match err.kind() {
            ErrorKind::DisplayHelp => {
                let help = CliArgs::command().render_long_help().to_string();
                exit_with_cli_info(&help);
            }
            ErrorKind::DisplayVersion => {
                let version = format!(
                    "{} {}",
                    CliArgs::command().get_name(),
                    env!("CARGO_PKG_VERSION")
                );
                exit_with_cli_info(&version);
            }
            _ => {}
        }

        let mut message = err.to_string();

        // A common mistake is using single-dash "long" options like `-rom`.
        if raw_args.iter().any(|arg| {
            arg == "-rom"
                || arg.starts_with("-rom=")
                || arg == "-lua"
                || arg.starts_with("-lua=")
                || arg == "-script"
                || arg.starts_with("-script=")
        }) {
            message.push_str(
                "\nHint: long options use `--`, not `-`. Try `--rom <PATH>` or `-r <PATH>`.\n",
            );
        } else if matches!(
            err.kind(),
            ErrorKind::UnknownArgument
                | ErrorKind::InvalidValue
                | ErrorKind::InvalidSubcommand
                | ErrorKind::ArgumentConflict
        ) {
            message.push_str("\nHint: run `--help` to see available options.\n");
        }

        exit_with_cli_error(&message, err.exit_code());
    })
}

fn main() -> Result<()> {
    // let _guard = init_tracing();
    let args = parse_cli_args_or_exit();

    let icon = egui::IconData {
        rgba: ICON_RGBA.to_vec(),
        width: ICON_WIDTH,
        height: ICON_HEIGHT,
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("Nesium")
            .with_app_id(APP_ID)
            .with_icon(icon)
            .with_inner_size([SCREEN_WIDTH as f32 * 3.0, SCREEN_HEIGHT as f32 * 3.0]),
        renderer: preferred_renderer(),
        wgpu_options: wgpu_options(),
        ..Default::default()
    };

    let config = AppConfig {
        rom_path: args.rom_path,
        lua_script_path: args.lua_script_path,
    };

    eframe::run_native(
        "Nesium",
        native_options,
        Box::new(|cc| Ok(Box::new(NesiumApp::new(cc, config)))),
    )
    .map_err(|e| anyhow!("eframe failed: {e}"))?;

    Ok(())
}

fn wgpu_options() -> eframe::egui_wgpu::WgpuConfiguration {
    eframe::egui_wgpu::WgpuConfiguration {
        desired_maximum_frame_latency: Some(1),
        ..Default::default()
    }
}

fn preferred_renderer() -> eframe::Renderer {
    // Windows ARM defaults to OpenGL (Glow) to avoid wgpu backend issues that can
    // manifest as "everything except text renders".
    #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
    return eframe::Renderer::Glow;

    eframe::Renderer::Wgpu
}

#[derive(Debug, Parser)]
#[command(
    name = "nesium_egui",
    author,
    version,
    color = clap::ColorChoice::Never,
    about = "Nesium (egui frontend)",
    long_about = "Nesium - a cross-platform NES emulator.\n\nExamples:\n  nesium_egui --rom path/to/game.nes\n  nesium_egui -r path/to/game.nes --lua path/to/script.lua\n\nNote: Lua script execution is not implemented yet; this flag is accepted for future support."
)]
struct CliArgs {
    /// ROM path to load at startup.
    #[arg(short = 'r', long = "rom", value_name = "PATH")]
    rom_path: Option<PathBuf>,

    /// Lua script path to execute when game starts (not implemented yet).
    #[arg(short = 'l', long = "lua", alias = "script", value_name = "PATH")]
    lua_script_path: Option<PathBuf>,
}
