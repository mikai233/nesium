use super::{LanguagePack, TextId};

pub struct Zh;

impl LanguagePack for Zh {
    fn text(id: TextId) -> &'static str {
        use TextId::*;

        match id {
            // Menu bar
            MenuFile => "文件",
            MenuFileLoadRom => "加载 ROM…",
            MenuFileReset => "重置",
            MenuFilePowerReset => "电源重置",
            MenuFileEject => "弹出",
            MenuFileStartRecording => "开始录制 WAV…",
            MenuFileStopRecording => "停止录制 WAV",
            MenuFileQuit => "退出",

            MenuEmulation => "仿真",
            MenuEmulationPause => "暂停",
            MenuEmulationResume => "继续",

            MenuWindow => "窗口",
            MenuWindowDebugger => "调试器",
            MenuWindowTools => "工具箱",
            MenuWindowPalette => "调色板",
            MenuWindowInput => "输入",
            MenuWindowAudio => "音频",

            MenuHelp => "帮助",
            MenuHelpAbout => "关于",
            MenuHelpLine1 => "基于 eframe + egui 的桌面前端",
            MenuHelpLine2 => "将 .nes/.fds 拖拽到窗口，或使用「文件 → 加载 ROM」",
            MenuLanguage => "语言",
            AboutWindowTitle => "关于 Nesium",
            AboutLead => "Nesium：基于 nesium-core 的 Rust NES/FC 前端。",
            AboutIntro => {
                "桌面界面使用 eframe + egui，nesium-audio/cpal 负责音频，gilrs 负责手柄。"
            }
            AboutComponentsHeading => "开源组件",
            AboutComponentsHint => "点击名称跳转到 GitHub 或 crates.io。",

            // Tools viewport
            ToolsHeading => "工具箱",
            ToolsPlaceholder => "在此添加保存状态、断点等工具逻辑。",

            // Palette viewport
            PaletteHeading => "当前调色板 (前 16 项)",
            PaletteModeLabel => "调色板来源",
            PaletteModeBuiltin => "内置",
            PaletteModeCustom => "外部文件（.pal）",
            PaletteBuiltinLabel => "内置调色板",
            PaletteCustomLoad => "加载 .pal 文件…",
            PaletteCustomActive => "外部调色板：",
            PaletteUseBuiltin => "使用内置调色板",
            PaletteError => "错误：",

            // Input viewport
            InputHeading => "输入配置",
            InputControllerPortsLabel => "控制器端口:",
            InputDeviceKeyboard => "键盘",
            InputDeviceDisabled => "禁用",
            InputNoGamepads => "无手柄连接",
            InputGamepadUnavailable => "手柄不可用",
            InputPort34Notice => "注意：端口 3 和 4 当前尚未接入 NES 核心，仅用于预配置映射。",
            InputPresetLabel => "预设:",
            InputPresetNesStandard => "NES 标准手柄",
            InputPresetFightStick => "Fight Stick",
            InputPresetArcadeLayout => "Arcade Layout",
            InputKeyboardMappingTitle => "键盘映射 → NES 手柄",
            InputKeyboardMappingHelp => {
                "点击“绑定”后按一个键，Esc 清除绑定；右下角“恢复默认”可还原出厂配置。"
            }
            InputGridHeaderCategory => "类别",
            InputGridHeaderButton => "按钮",
            InputGridHeaderCurrentKey => "当前键位",
            InputGridHeaderAction => "操作",
            InputCategoryDirection => "方向",
            InputCategoryAction => "动作",
            InputCategorySystem => "系统",
            InputButtonTurboA => "连发 A",
            InputButtonTurboB => "连发 B",
            InputTurboSection => "连发 (Turbo)",
            InputTurboRateLabel => "切换帧数",
            InputTurboHelp => {
                "数值越小越快。连发会每 N 帧在“按下/松开”之间切换（例如 N=1≈30Hz，N=2≈15Hz）。"
            }
            InputPromptPressAnyKey => "按任意键...",
            InputNotBound => "未绑定",
            InputBindButton => "绑定",
            InputCancelButton => "取消",
            InputCurrentlyPressedLabel => "当前按下的按钮:",
            InputGamepadMappingSection => "手柄映射",
            InputGamepadMappingTitle => "NES 按钮 → 手柄按键",
            InputGamepadGridHeaderCategory => "类别",
            InputGamepadGridHeaderButton => "按钮",
            InputGamepadGridHeaderGamepadButton => "手柄按键",
            InputRestoreDefaults => "恢复默认",

            // Audio viewport
            AudioHeading => "音频设置",
            AudioMasterVolumeLabel => "主音量",
            AudioBgFastBehaviorLabel => "后台 / 快进行为",
            AudioMuteInBackground => "后台静音",
            AudioReduceInBackground => "后台降低音量",
            AudioReduceInFastForward => "快进时降低音量",
            AudioReduceAmount => "降低幅度",
            AudioReverbSection => "混响 (Reverb)",
            AudioEnableReverb => "启用混响",
            AudioReverbStrength => "强度",
            AudioReverbDelayMs => "延迟 (ms)",
            AudioCrossfeedSection => "串音 (Crossfeed)",
            AudioEnableCrossfeed => "启用串音",
            AudioCrossfeedRatio => "比率",
            AudioEqSection => "均衡器 (EQ)",
            AudioEnableEq => "启用 EQ",
            AudioEqGlobalGain => "全局增益 (dB)",
        }
    }
}
