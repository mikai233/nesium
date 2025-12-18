mod en;
mod zh;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Language {
    English,
    ChineseSimplified,
}

impl Language {
    pub const ALL: [Language; 2] = [Language::English, Language::ChineseSimplified];

    pub fn label(self) -> &'static str {
        match self {
            Language::English => "English",
            Language::ChineseSimplified => "简体中文",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TextId {
    // Menu bar
    MenuFile,
    MenuFileLoadRom,
    MenuFileReset,
    MenuFileEject,
    MenuFileStartRecording,
    MenuFileStopRecording,
    MenuFileQuit,
    MenuEmulation,
    MenuEmulationPause,
    MenuEmulationResume,
    MenuWindow,
    MenuWindowDebugger,
    MenuWindowTools,
    MenuWindowPalette,
    MenuWindowInput,
    MenuWindowAudio,
    MenuHelp,
    MenuHelpAbout,
    MenuHelpLine1,
    MenuHelpLine2,
    MenuLanguage,
    AboutWindowTitle,
    AboutLead,
    AboutIntro,
    AboutComponentsHeading,
    AboutComponentsHint,

    // Status line / notifications
    StatusReset,
    StatusEject,
    StatusPaused,
    StatusResumed,

    // Main view
    MainNoRom,
    MainWaitingFirstFrame,

    // Tools viewport
    ToolsHeading,
    ToolsPlaceholder,

    // Palette viewport
    PaletteHeading,

    // Input viewport
    InputHeading,
    InputControllerPortsLabel,
    InputDeviceKeyboard,
    InputDeviceDisabled,
    InputNoGamepads,
    InputGamepadUnavailable,
    InputPort34Notice,
    InputPresetLabel,
    InputPresetNesStandard,
    InputPresetFightStick,
    InputPresetArcadeLayout,
    InputKeyboardMappingTitle,
    InputKeyboardMappingHelp,
    InputGridHeaderCategory,
    InputGridHeaderButton,
    InputGridHeaderCurrentKey,
    InputGridHeaderAction,
    InputCategoryDirection,
    InputCategoryAction,
    InputCategorySystem,
    InputPromptPressAnyKey,
    InputNotBound,
    InputBindButton,
    InputCancelButton,
    InputCurrentlyPressedLabel,
    InputGamepadMappingSection,
    InputGamepadMappingTitle,
    InputGamepadGridHeaderCategory,
    InputGamepadGridHeaderButton,
    InputGamepadGridHeaderGamepadButton,
    InputRestoreDefaults,

    // Audio viewport
    AudioHeading,
    AudioMasterVolumeLabel,
    AudioBgFastBehaviorLabel,
    AudioMuteInBackground,
    AudioReduceInBackground,
    AudioReduceInFastForward,
    AudioReduceAmount,
    AudioReverbSection,
    AudioEnableReverb,
    AudioReverbStrength,
    AudioReverbDelayMs,
    AudioCrossfeedSection,
    AudioEnableCrossfeed,
    AudioCrossfeedRatio,
    AudioEqSection,
    AudioEnableEq,
    AudioEqGlobalGain,
}

pub trait LanguagePack {
    fn text(id: TextId) -> &'static str;
}

pub struct I18n {
    language: Language,
}

impl I18n {
    pub fn new(language: Language) -> Self {
        Self { language }
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn set_language(&mut self, language: Language) {
        self.language = language;
    }

    pub fn text(&self, id: TextId) -> &'static str {
        match self.language {
            Language::English => en::En::text(id),
            Language::ChineseSimplified => zh::Zh::text(id),
        }
    }
}
