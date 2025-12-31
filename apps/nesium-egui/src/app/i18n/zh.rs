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

            MenuEmulation => "模拟",
            MenuEmulationPause => "暂停",
            MenuEmulationResume => "继续",

            MenuView => "视图",
            MenuViewScale => "比例",
            MenuViewScaleSquare => "1:1 (方形像素)",
            MenuViewScaleNtsc => "4:3 (NTSC)",
            MenuViewScaleStretch => "拉伸",

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
            InputTurboOnFramesLabel => "按下帧数",
            InputTurboOffFramesLabel => "抬起帧数",
            InputTurboLinkPressRelease => "联动按下/抬起",
            InputTurboHelp => "连发按 N 帧、抬 Z 帧循环（例如 1/1≈30Hz）。数值越小越快。",
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

            // Debugger viewport
            DebuggerNoRomTitle => "未加载 ROM",
            DebuggerNoRomSubtitle => "加载 ROM 后显示调试状态",
            DebuggerCpuRegisters => "CPU 寄存器",
            DebuggerPpuState => "PPU 状态",
            DebuggerCpuStatusTooltip => {
                "CPU 状态寄存器 (P)\nN: 负数标志 - 结果第7位为1时置位\nV: 溢出标志 - 有符号溢出时置位\nB: 中断标志 - BRK指令设置\nD: 十进制模式 - BCD模式（NES忽略）\nI: 中断禁用 - 阻止IRQ\nZ: 零标志 - 结果为零时置位\nC: 进位标志 - 无符号溢出时置位\n\n大写=置位, 小写=清除"
            }
            DebuggerPpuCtrlTooltip => {
                "PPU 控制寄存器 ($2000)\nV: NMI使能\nP: PPU主/从（未使用）\nH: 精灵高度（0=8x8, 1=8x16）\nB: 背景图案表地址\nS: 精灵图案表地址\nI: VRAM地址增量（0=1, 1=32）\nNN: 基础名称表地址\n\n大写=置位, 小写=清除"
            }
            DebuggerPpuMaskTooltip => {
                "PPU 掩码寄存器 ($2001)\nBGR: 颜色强调位\ns: 显示精灵\nb: 显示背景\nM: 左8像素显示精灵\nm: 左8像素显示背景\ng: 灰度模式\n\n大写=置位, 小写=清除"
            }
            DebuggerPpuStatusTooltip => {
                "PPU 状态寄存器 ($2002)\nV: VBlank已开始\nS: 精灵0命中\nO: 精灵溢出\n\n大写=置位, 小写=清除"
            }
            DebuggerScanlineTooltip => {
                "扫描线说明：\n0-239: 可见区域 (渲染)\n240: post-render (空闲)\n241-260: VBlank (垂直消隐)\n-1: pre-render (预渲染扫描线)"
            }
        }
    }
}
