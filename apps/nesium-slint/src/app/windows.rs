use anyhow::{Context, Result};
use slint::{CloseRequestResponse, ComponentHandle, SharedString};

use crate::{
    AboutWindow, AudioWindow, DebuggerWindow, InputWindow, MainWindow, PaletteWindow, ToolsWindow,
};

use super::state::AuxWindowKind;

pub struct AuxWindows {
    debugger: DebuggerWindow,
    tools: ToolsWindow,
    palette: PaletteWindow,
    input: InputWindow,
    audio: AudioWindow,
    about: AboutWindow,
}

impl AuxWindows {
    pub fn new(main_window: &MainWindow) -> Result<Self> {
        let windows = Self {
            debugger: DebuggerWindow::new().context("failed to create debugger window")?,
            tools: ToolsWindow::new().context("failed to create tools window")?,
            palette: PaletteWindow::new().context("failed to create palette window")?,
            input: InputWindow::new().context("failed to create input window")?,
            audio: AudioWindow::new().context("failed to create audio window")?,
            about: AboutWindow::new().context("failed to create about window")?,
        };

        windows.debugger.set_has_rom(false);
        windows.debugger.set_rom_name(SharedString::from("No ROM"));

        install_aux_close_handler(main_window, &windows.debugger, AuxWindowKind::Debugger);
        install_aux_close_handler(main_window, &windows.tools, AuxWindowKind::Tools);
        install_aux_close_handler(main_window, &windows.palette, AuxWindowKind::Palette);
        install_aux_close_handler(main_window, &windows.input, AuxWindowKind::Input);
        install_aux_close_handler(main_window, &windows.audio, AuxWindowKind::Audio);
        install_aux_close_handler(main_window, &windows.about, AuxWindowKind::About);

        Ok(windows)
    }

    pub fn debugger(&self) -> &DebuggerWindow {
        &self.debugger
    }

    pub fn palette(&self) -> &PaletteWindow {
        &self.palette
    }

    pub fn input(&self) -> &InputWindow {
        &self.input
    }

    pub fn audio(&self) -> &AudioWindow {
        &self.audio
    }

    pub fn set_rom_state(&self, rom_name: SharedString, has_rom: bool) {
        self.debugger.set_rom_name(rom_name);
        self.debugger.set_has_rom(has_rom);
    }

    pub fn open_or_focus(&self, kind: AuxWindowKind) -> Result<()> {
        match kind {
            AuxWindowKind::Debugger => show_component(&self.debugger),
            AuxWindowKind::Tools => show_component(&self.tools),
            AuxWindowKind::Palette => show_component(&self.palette),
            AuxWindowKind::Input => show_component(&self.input),
            AuxWindowKind::Audio => show_component(&self.audio),
            AuxWindowKind::About => show_component(&self.about),
        }
    }
}

fn install_aux_close_handler<T>(main_window: &MainWindow, component: &T, kind: AuxWindowKind)
where
    T: ComponentHandle,
{
    let main_window_weak = main_window.as_weak();
    component.window().on_close_requested(move || {
        let main_window_weak = main_window_weak.clone();
        let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
            main_window.invoke_aux_window_closed(kind.as_index());
        });
        CloseRequestResponse::HideWindow
    });
}

fn show_component<T>(component: &T) -> Result<()>
where
    T: ComponentHandle,
{
    component
        .show()
        .context("failed to show or focus auxiliary window")
}
