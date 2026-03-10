#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayMode {
    #[default]
    Square,
    Ntsc,
    Stretch,
}

impl DisplayMode {
    pub const fn as_index(self) -> i32 {
        match self {
            Self::Square => 0,
            Self::Ntsc => 1,
            Self::Stretch => 2,
        }
    }

    pub const fn from_index(value: i32) -> Self {
        match value {
            1 => Self::Ntsc,
            2 => Self::Stretch,
            _ => Self::Square,
        }
    }

    pub const fn label(self) -> &'static str {
        match self {
            Self::Square => "Square Pixels",
            Self::Ntsc => "NTSC 4:3",
            Self::Stretch => "Stretch",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuxWindowKind {
    Debugger,
    Tools,
    Palette,
    Input,
    Audio,
    About,
}

impl AuxWindowKind {
    pub const fn as_index(self) -> i32 {
        match self {
            Self::Debugger => 0,
            Self::Tools => 1,
            Self::Palette => 2,
            Self::Input => 3,
            Self::Audio => 4,
            Self::About => 5,
        }
    }

    pub const fn from_index(value: i32) -> Option<Self> {
        match value {
            0 => Some(Self::Debugger),
            1 => Some(Self::Tools),
            2 => Some(Self::Palette),
            3 => Some(Self::Input),
            4 => Some(Self::Audio),
            5 => Some(Self::About),
            _ => None,
        }
    }
}
