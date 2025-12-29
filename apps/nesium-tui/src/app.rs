use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyboardEnhancementFlags, PushKeyboardEnhancementFlags, PopKeyboardEnhancementFlags},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use nesium_core::ppu::{buffer::ColorFormat, SCREEN_HEIGHT, SCREEN_WIDTH};
use nesium_runtime::{
    AudioMode, Runtime, RuntimeConfig, VideoConfig, VideoExternalConfig,
};

use crate::{args::Args, input::{AppAction, InputManager}, ui};

pub struct App {
    runtime: Option<Runtime>,
    input_manager: InputManager,
    /// Backing memory for the NES framebuffer (Plane 0)
    #[allow(dead_code)]
    plane0: Box<[u8]>,
    /// Backing memory for the NES framebuffer (Plane 1)
    #[allow(dead_code)]
    plane1: Box<[u8]>,
    rom_name: String,
    should_quit: bool,
    fps_counter: u32,
    last_fps_time: Instant,
    current_fps: u32,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let rom_path = args.rom;
        let rom_name = rom_path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown ROM")
            .to_string();

        let format = ColorFormat::Rgba8888;
        let bpp = format.bytes_per_pixel();
        let len = SCREEN_WIDTH * SCREEN_HEIGHT * bpp;

        // Allocate pinned buffers for the runtime to write into.
        let mut plane0 = vec![0u8; len].into_boxed_slice();
        let mut plane1 = vec![0u8; len].into_boxed_slice();

        let video_config = VideoConfig::External(VideoExternalConfig {
            color_format: format,
            pitch_bytes: SCREEN_WIDTH * bpp,
            plane0: plane0.as_mut_ptr(),
            plane1: plane1.as_mut_ptr(),
        });

        let audio_mode = if args.no_audio {
            AudioMode::Disabled
        } else {
            AudioMode::Auto
        };

        let config = RuntimeConfig {
            video: video_config,
            audio: audio_mode,
        };

        let runtime = Runtime::start(config).context("Failed to start NES runtime")?;
        let handle = runtime.handle();

        if args.integer_fps {
            handle.set_integer_fps_target(Some(60)).ok();
        }

        handle.load_rom(&rom_path).context("Failed to load ROM")?;

        Ok(Self {
            runtime: Some(runtime),
            input_manager: InputManager::new(),
            plane0,
            plane1,
            rom_name,
            should_quit: false,
            fps_counter: 0,
            last_fps_time: Instant::now(),
            current_fps: 0,
        })
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = std::io::stdout();

        let supports_enhancement = crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);

        if supports_enhancement {
            execute!(
                stdout,
                EnterAlternateScreen,
                PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES),
                Hide
            )?;
        } else {
            execute!(
                stdout,
                EnterAlternateScreen,
                Hide
            )?;
        }

        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let res = self.run_loop(&mut terminal);

        disable_raw_mode()?;
        
        if supports_enhancement {
            execute!(
                terminal.backend_mut(),
                PopKeyboardEnhancementFlags,
                LeaveAlternateScreen,
                Show
            )?;
        } else {
            execute!(
                terminal.backend_mut(),
                LeaveAlternateScreen,
                Show
            )?;
        }
        terminal.show_cursor()?;

        res
    }

    fn run_loop(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    ) -> Result<()> {
        // Target ~60 FPS update loop for the UI
        let frame_duration = Duration::from_micros(16667);

        while !self.should_quit {
            let start = Instant::now();

            // 1. Input
            // Poll for a bit to gather multiple events if they queued up
            while event::poll(Duration::from_millis(0))? {
                if let Event::Key(key) = event::read()? {
                    match self.input_manager.handle_event(key, &self.runtime) {
                        AppAction::Quit => self.should_quit = true,
                        AppAction::Reset => { /* Reset handled in InputManager */ }
                        AppAction::None => {}
                    }
                }
            }
            
            // Handle key decay (autoreset for keys that stopped repeating)
            self.input_manager.update(&self.runtime);

            // 2. Update FPS
            self.fps_counter += 1;
            if self.last_fps_time.elapsed() >= Duration::from_secs(1) {
                self.current_fps = self.fps_counter;
                self.fps_counter = 0;
                self.last_fps_time = Instant::now();
            }

            // 3. Render
            terminal.draw(|f| ui::draw(f, self))?;

            // 4. Sleep to cap UI FPS
            let elapsed = start.elapsed();
            if elapsed < frame_duration {
                std::thread::sleep(frame_duration - elapsed);
            }
        }

        Ok(())
    }

    pub fn rom_name(&self) -> &str {
        &self.rom_name
    }

    pub fn current_fps(&self) -> u32 {
        self.current_fps
    }

    pub fn runtime(&self) -> &Option<Runtime> {
        &self.runtime
    }
}