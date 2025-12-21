use super::{LanguagePack, TextId};

pub struct En;

impl LanguagePack for En {
    fn text(id: TextId) -> &'static str {
        use TextId::*;

        match id {
            // Menu bar
            MenuFile => "File",
            MenuFileLoadRom => "Load ROM…",
            MenuFileReset => "Reset",
            MenuFileEject => "Eject",
            MenuFileStartRecording => "Start recording WAV…",
            MenuFileStopRecording => "Stop recording WAV",
            MenuFileQuit => "Quit",

            MenuEmulation => "Emulation",
            MenuEmulationPause => "Pause",
            MenuEmulationResume => "Resume",

            MenuWindow => "Window",
            MenuWindowDebugger => "Debugger",
            MenuWindowTools => "Tools",
            MenuWindowPalette => "Palette",
            MenuWindowInput => "Input",
            MenuWindowAudio => "Audio",

            MenuHelp => "Help",
            MenuHelpAbout => "About",
            MenuHelpLine1 => "Desktop frontend built with eframe + egui",
            MenuHelpLine2 => "Drag a .nes/.fds file here, or use File → Load ROM",
            MenuLanguage => "Language",
            AboutWindowTitle => "About Nesium",
            AboutLead => "Nesium: Rust NES/FC emulator frontend built on nesium-core.",
            AboutIntro => {
                "Desktop UI uses eframe + egui, with nesium-audio/cpal for sound and gilrs for controllers."
            }
            AboutComponentsHeading => "Open-source components",
            AboutComponentsHint => "Click a component to open GitHub or crates.io.",

            // Tools viewport
            ToolsHeading => "Tools",
            ToolsPlaceholder => "Add save states, breakpoints and helpers here.",

            // Palette viewport
            PaletteHeading => "Current palette (first 16 entries)",

            // Input viewport
            InputHeading => "Input configuration",
            InputControllerPortsLabel => "Controller ports:",
            InputDeviceKeyboard => "Keyboard",
            InputDeviceDisabled => "Disabled",
            InputNoGamepads => "No gamepad connected",
            InputGamepadUnavailable => "Gamepad unavailable",
            InputPort34Notice => {
                "Note: Ports 3 and 4 are not wired to the NES core yet; they are only for pre-configuring mappings."
            }
            InputPresetLabel => "Preset:",
            InputPresetNesStandard => "NES Standard Gamepad",
            InputPresetFightStick => "Fight Stick",
            InputPresetArcadeLayout => "Arcade Layout",
            InputKeyboardMappingTitle => "Keyboard mapping → NES pad",
            InputKeyboardMappingHelp => {
                "Click “Bind” then press a key; Esc clears binding. “Reset to defaults” restores factory mapping."
            }
            InputGridHeaderCategory => "Category",
            InputGridHeaderButton => "Button",
            InputGridHeaderCurrentKey => "Current key",
            InputGridHeaderAction => "Action",
            InputCategoryDirection => "Direction",
            InputCategoryAction => "Action",
            InputCategorySystem => "System",
            InputPromptPressAnyKey => "Press any key...",
            InputNotBound => "Unbound",
            InputBindButton => "Bind",
            InputCancelButton => "Cancel",
            InputCurrentlyPressedLabel => "Buttons currently pressed:",
            InputGamepadMappingSection => "Gamepad mapping",
            InputGamepadMappingTitle => "NES button → Gamepad button",
            InputGamepadGridHeaderCategory => "Category",
            InputGamepadGridHeaderButton => "Button",
            InputGamepadGridHeaderGamepadButton => "Gamepad button",
            InputRestoreDefaults => "Reset to defaults",

            // Audio viewport
            AudioHeading => "Audio settings",
            AudioMasterVolumeLabel => "Master volume",
            AudioBgFastBehaviorLabel => "Background / fast-forward behavior",
            AudioMuteInBackground => "Mute in background",
            AudioReduceInBackground => "Reduce volume in background",
            AudioReduceInFastForward => "Reduce volume during fast-forward",
            AudioReduceAmount => "Reduction amount",
            AudioReverbSection => "Reverb",
            AudioEnableReverb => "Enable reverb",
            AudioReverbStrength => "Strength",
            AudioReverbDelayMs => "Delay (ms)",
            AudioCrossfeedSection => "Crossfeed",
            AudioEnableCrossfeed => "Enable crossfeed",
            AudioCrossfeedRatio => "Ratio",
            AudioEqSection => "Equalizer (EQ)",
            AudioEnableEq => "Enable EQ",
            AudioEqGlobalGain => "Global gain (dB)",
        }
    }
}
