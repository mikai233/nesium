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
            MenuFilePowerReset => "Power Reset",
            MenuFileEject => "Eject",
            MenuFileStartRecording => "Start recording WAV…",
            MenuFileStopRecording => "Stop recording WAV",
            MenuFileQuit => "Quit",

            MenuEmulation => "Emulation",
            MenuEmulationPause => "Pause",
            MenuEmulationResume => "Resume",

            MenuView => "View",
            MenuViewScale => "Scale",
            MenuViewScaleSquare => "1:1 (Square pixels)",
            MenuViewScaleNtsc => "4:3 (NTSC)",
            MenuViewScaleStretch => "Stretch",

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
            PaletteModeLabel => "Palette source",
            PaletteModeBuiltin => "Built-in",
            PaletteModeCustom => "External (.pal)",
            PaletteBuiltinLabel => "Built-in palette",
            PaletteCustomLoad => "Load .pal file…",
            PaletteCustomActive => "External palette:",
            PaletteUseBuiltin => "Use built-in palette",
            PaletteError => "Error:",

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
            InputButtonTurboA => "Turbo A",
            InputButtonTurboB => "Turbo B",
            InputTurboSection => "Turbo",
            InputTurboOnFramesLabel => "Press frames",
            InputTurboOffFramesLabel => "Release frames",
            InputTurboLinkPressRelease => "Link press/release",
            InputTurboHelp => {
                "Turbo repeats: press for N frames, then release for Z frames (e.g. 1/1≈30Hz on NTSC)."
            }
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

            // Debugger viewport
            DebuggerNoRomTitle => "No ROM Loaded",
            DebuggerNoRomSubtitle => "Load a ROM to see debug status",
            DebuggerCpuRegisters => "CPU Registers",
            DebuggerPpuState => "PPU State",
            DebuggerCpuStatusTooltip => {
                "CPU Status Register (P)\nN: Negative - Set if bit 7 of result is set\nV: Overflow - Set if signed overflow occurred\nB: Break - Set by BRK instruction\nD: Decimal - Binary Coded Decimal (Ignored on NES)\nI: Interrupt Disable - Prevents IRQs\nZ: Zero - Set if result is zero\nC: Carry - Set if unsigned overflow occurred\n\nUppercase=Set, Lowercase=Clear"
            }
            DebuggerPpuCtrlTooltip => {
                "PPU Control Register ($2000)\nV: NMI Enable\nP: PPU Master/Slave (Unused)\nH: Sprite Height (0=8x8, 1=8x16)\nB: Background Pattern Table Address\nS: Sprite Pattern Table Address\nI: VRAM Address Increment (0=1, 1=32)\nNN: Base Nametable Address\n\nUppercase=Set, Lowercase=Clear"
            }
            DebuggerPpuMaskTooltip => {
                "PPU Mask Register ($2001)\nBGR: Color Emphasis Bits\ns: Show Sprites\nb: Show Background\nM: Show Sprites in Leftmost 8 Pixels\nm: Show Background in Leftmost 8 Pixels\ng: Grayscale\n\nUppercase=Set, Lowercase=Clear"
            }
            DebuggerPpuStatusTooltip => {
                "PPU Status Register ($2002)\nV: VBlank Started\nS: Sprite 0 Hit\nO: Sprite Overflow\n\nUppercase=Set, Lowercase=Clear"
            }
            DebuggerScanlineTooltip => {
                "Scanline Information:\n0-239: Visible (Rendering)\n240: Post-render (Idle)\n241-260: VBlank\n-1: Pre-render"
            }
        }
    }
}
