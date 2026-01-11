// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Chinese (`zh`).
class AppLocalizationsZh extends AppLocalizations {
  AppLocalizationsZh([String locale = 'zh']) : super(locale);

  @override
  String get settingsTitle => '设置';

  @override
  String get settingsTabGeneral => '通用';

  @override
  String get settingsTabInput => '输入';

  @override
  String get settingsTabVideo => '画面';

  @override
  String get settingsTabEmulation => '模拟器';

  @override
  String get settingsTabServer => '服务器';

  @override
  String get serverTitle => '联机服务器';

  @override
  String get serverPortLabel => '端口';

  @override
  String get serverStartButton => '启动服务器';

  @override
  String get serverStopButton => '停止服务器';

  @override
  String get serverStatusRunning => '运行中';

  @override
  String get serverStatusStopped => '已停止';

  @override
  String serverClientCount(int count) {
    return '已连接客户端: $count';
  }

  @override
  String serverStartFailed(String error) {
    return '服务器启动失败: $error';
  }

  @override
  String serverStopFailed(String error) {
    return '服务器停止失败: $error';
  }

  @override
  String serverBindAddress(String address) {
    return '绑定地址: $address';
  }

  @override
  String get generalTitle => '通用';

  @override
  String get themeLabel => '主题';

  @override
  String get themeSystem => '跟随系统';

  @override
  String get themeLight => '浅色';

  @override
  String get themeDark => '深色';

  @override
  String get languageLabel => '语言';

  @override
  String get languageSystem => '跟随系统';

  @override
  String get languageEnglish => '英语';

  @override
  String get languageChineseSimplified => '简体中文';

  @override
  String get inputTitle => '输入';

  @override
  String get turboTitle => '连发';

  @override
  String get turboLinkPressRelease => '联动按下/抬起';

  @override
  String get inputDeviceLabel => '输入设备';

  @override
  String get inputDeviceKeyboard => '键盘';

  @override
  String get inputDeviceGamepad => '手柄';

  @override
  String get connectedGamepadsTitle => '已连接手柄';

  @override
  String get connectedGamepadsNone => '未连接手柄';

  @override
  String get webGamepadActivationHint => 'Web 限制：请按下手柄上的任意按键以激活。';

  @override
  String connectedGamepadsPort(int port) {
    return '玩家 $port';
  }

  @override
  String get connectedGamepadsUnassigned => '未分配';

  @override
  String get inputDeviceVirtualController => '虚拟手柄';

  @override
  String get inputGamepadAssignmentLabel => '手柄分配';

  @override
  String get inputGamepadNone => '无/未分配';

  @override
  String get inputListening => '监听中...';

  @override
  String inputDetected(String buttons) {
    return '检测到输入: $buttons';
  }

  @override
  String get inputGamepadMappingLabel => '按键映射';

  @override
  String get inputResetToDefault => '恢复默认设置';

  @override
  String get inputButtonA => 'A';

  @override
  String get inputButtonB => 'B';

  @override
  String get inputButtonTurboA => '连发 A';

  @override
  String get inputButtonTurboB => '连发 B';

  @override
  String get inputButtonSelect => '选择';

  @override
  String get inputButtonStart => '开始';

  @override
  String get inputButtonUp => '上';

  @override
  String get inputButtonDown => '下';

  @override
  String get inputButtonLeft => '左';

  @override
  String get inputButtonRight => '右';

  @override
  String get inputButtonRewind => '倒带';

  @override
  String get inputButtonFastForward => '快进';

  @override
  String get inputButtonSaveState => '即时存档';

  @override
  String get inputButtonLoadState => '读取存档';

  @override
  String get inputButtonPause => '暂停';

  @override
  String get globalHotkeysTitle => '模拟器热键';

  @override
  String get gamepadHotkeysTitle => '手柄热键 (玩家 1)';

  @override
  String get inputPortLabel => '配置玩家';

  @override
  String get player1 => '玩家 1';

  @override
  String get player2 => '玩家 2';

  @override
  String get player3 => '玩家 3';

  @override
  String get player4 => '玩家 4';

  @override
  String get keyboardPresetLabel => '键盘预设';

  @override
  String get keyboardPresetNone => '无';

  @override
  String get keyboardPresetNesStandard => 'NES 标准';

  @override
  String get keyboardPresetFightStick => '摇杆';

  @override
  String get keyboardPresetArcadeLayout => '街机布局';

  @override
  String get keyboardPresetCustom => '自定义';

  @override
  String get customKeyBindingsTitle => '自定义按键绑定';

  @override
  String bindKeyTitle(String action) {
    return '绑定 $action';
  }

  @override
  String get unassignedKey => '未设置';

  @override
  String get tipPressEscapeToClearBinding => '提示：按 Esc 清除绑定。';

  @override
  String get keyboardActionUp => '上';

  @override
  String get keyboardActionDown => '下';

  @override
  String get keyboardActionLeft => '左';

  @override
  String get keyboardActionRight => '右';

  @override
  String get keyboardActionA => 'A';

  @override
  String get keyboardActionB => 'B';

  @override
  String get keyboardActionSelect => 'Select';

  @override
  String get keyboardActionStart => 'Start';

  @override
  String get keyboardActionTurboA => 'Turbo A';

  @override
  String get keyboardActionTurboB => 'Turbo B';

  @override
  String get keyboardActionRewind => '倒带';

  @override
  String get keyboardActionFastForward => '快进';

  @override
  String get keyboardActionSaveState => '即时存档';

  @override
  String get keyboardActionLoadState => '读取存档';

  @override
  String get keyboardActionPause => '暂停';

  @override
  String inputBindingConflictCleared(String player, String action) {
    return '$player 的 $action 绑定已清除。';
  }

  @override
  String inputBindingConflictHint(String player, String action) {
    return '($player - $action)';
  }

  @override
  String inputBindingCapturedConflictHint(String player, String action) {
    return '由 $player - $action 占用';
  }

  @override
  String get emulationTitle => '模拟器';

  @override
  String get integerFpsTitle => '整数 FPS 模式（60Hz，NTSC）';

  @override
  String get integerFpsSubtitle => '在 60Hz 屏幕上减少滚动抖动。PAL 之后支持。';

  @override
  String get pauseInBackgroundTitle => '后台暂停';

  @override
  String get pauseInBackgroundSubtitle => '应用不在前台时自动暂停模拟器。';

  @override
  String get autoSaveEnabledTitle => '自动存档';

  @override
  String get autoSaveEnabledSubtitle => '定期自动将游戏状态保存到专用槽位。';

  @override
  String get autoSaveIntervalTitle => '自动存档间隔';

  @override
  String autoSaveIntervalValue(int minutes) {
    return '$minutes 分钟';
  }

  @override
  String get rewindEnabledTitle => '倒带';

  @override
  String get rewindEnabledSubtitle => '开启实时倒带功能（桌面端长按退格键 Backspace 触发）。';

  @override
  String get rewindSecondsTitle => '倒带时长';

  @override
  String rewindSecondsValue(int seconds) {
    return '$seconds 秒';
  }

  @override
  String get autoSlotLabel => '自动存档位';

  @override
  String get menuAutoSave => '自动存档…';

  @override
  String get stateAutoSaved => '自动存档已创建';

  @override
  String get virtualControlsTitle => '虚拟按键';

  @override
  String get virtualControlsSwitchInputTip => '切换输入设备为“虚拟手柄”后才能调整。';

  @override
  String get virtualControlsButtonSize => '按键大小';

  @override
  String get virtualControlsGap => '间距';

  @override
  String get virtualControlsOpacity => '透明度';

  @override
  String get virtualControlsHitboxScale => '点击范围倍率';

  @override
  String get virtualControlsHapticFeedback => '触觉反馈';

  @override
  String get virtualControlsDpadDeadzone => '方向键死区';

  @override
  String get virtualControlsDpadDeadzoneHelp => '中心死区：触点靠近中心时不会触发任何方向。';

  @override
  String get virtualControlsDpadBoundaryDeadzone => '方向键边界死区';

  @override
  String get virtualControlsDpadBoundaryDeadzoneHelp =>
      '边界死区：数值越大越不容易触发斜方向，减少误触相邻方向。';

  @override
  String get virtualControlsReset => '重置布局';

  @override
  String get virtualControlsTurboFramesPerToggle => '连发切换帧数';

  @override
  String get virtualControlsTurboOnFrames => '连发按下帧数';

  @override
  String get virtualControlsTurboOffFrames => '连发抬起帧数';

  @override
  String framesValue(int frames) {
    return '$frames 帧';
  }

  @override
  String get tipAdjustButtonsInDrawer => '提示：在游戏内抽屉中调整按键位置/大小。';

  @override
  String get keyCapturePressKeyToBind => '按下按键进行绑定。';

  @override
  String keyCaptureCurrent(String key) {
    return '当前：$key';
  }

  @override
  String keyCaptureCaptured(String key) {
    return '捕获：$key';
  }

  @override
  String get keyCapturePressEscToClear => '按 Esc 清除。';

  @override
  String get keyBindingsTitle => '按键绑定';

  @override
  String get cancel => '取消';

  @override
  String get appName => 'Nesium';

  @override
  String get menuTooltip => '菜单';

  @override
  String get menuSectionFile => '文件';

  @override
  String get menuSectionEmulation => '模拟';

  @override
  String get menuSectionSettings => '设置';

  @override
  String get menuSectionWindows => '窗口';

  @override
  String get menuSectionHelp => '帮助';

  @override
  String get menuOpenRom => '打开 ROM…';

  @override
  String get menuReset => '重置';

  @override
  String get menuPowerReset => '电源重置';

  @override
  String get menuEject => '关机';

  @override
  String get menuSaveState => '保存存档…';

  @override
  String get menuLoadState => '读取存档…';

  @override
  String get menuPauseResume => '暂停 / 继续';

  @override
  String get menuNetplay => '联机游戏';

  @override
  String get netplayStatusDisconnected => '未连接';

  @override
  String get netplayStatusConnecting => '正在连接…';

  @override
  String get netplayStatusConnected => '已连接 (等待房间)';

  @override
  String get netplayStatusInRoom => '已进入房间';

  @override
  String get netplayDisconnect => '断开连接';

  @override
  String get netplayServerAddress => '服务器地址';

  @override
  String get netplayPlayerName => '玩家名称';

  @override
  String get netplayConnect => '连接';

  @override
  String get netplayCreateRoom => '创建房间';

  @override
  String get netplayJoinRoom => '加入房间';

  @override
  String get netplayRoomCode => '房间代码';

  @override
  String get netplayRoleLabel => '角色';

  @override
  String netplayPlayerIndex(int index) {
    return '玩家 $index';
  }

  @override
  String get netplaySpectator => '旁观者';

  @override
  String get netplayClientId => '客户端 ID';

  @override
  String get netplayPlayerListHeader => '房间玩家列表';

  @override
  String get netplayYouIndicator => ' (你)';

  @override
  String get netplayOrSeparator => '或';

  @override
  String netplayConnectFailed(String error) {
    return '连接失败: $error';
  }

  @override
  String netplayDisconnectFailed(String error) {
    return '断开连接失败: $error';
  }

  @override
  String netplayCreateRoomFailed(String error) {
    return '创建房间失败: $error';
  }

  @override
  String netplayJoinRoomFailed(String error) {
    return '加入房间失败: $error';
  }

  @override
  String netplaySwitchRoleFailed(String error) {
    return '切换角色失败: $error';
  }

  @override
  String get netplayInvalidRoomCode => '房间代码无效';

  @override
  String get netplayRomBroadcasted => '联机游戏: ROM 已广播至房间';

  @override
  String get menuLoadTasMovie => '加载 TAS 录像…';

  @override
  String get menuPreferences => '偏好设置…';

  @override
  String get saveToExternalFile => '保存到外部文件…';

  @override
  String get loadFromExternalFile => '从外部文件加载…';

  @override
  String get slotLabel => '槽位';

  @override
  String get slotEmpty => '空';

  @override
  String get slotHasData => '已保存';

  @override
  String stateSavedToSlot(int index) {
    return '存档已保存至槽位 $index';
  }

  @override
  String stateLoadedFromSlot(int index) {
    return '已从槽位 $index 读取存档';
  }

  @override
  String slotCleared(int index) {
    return '槽位 $index 已清除';
  }

  @override
  String get menuAbout => '关于';

  @override
  String get menuDebugger => '调试器';

  @override
  String get menuTools => '工具';

  @override
  String get menuOpenDebuggerWindow => '打开调试器窗口';

  @override
  String get menuOpenToolsWindow => '打开工具窗口';

  @override
  String get menuInputMappingComingSoon => '按键映射（敬请期待）';

  @override
  String get menuLastError => '最近错误';

  @override
  String get lastErrorDetailsAction => '详情';

  @override
  String get lastErrorDialogTitle => '最近错误';

  @override
  String get lastErrorCopied => '已复制';

  @override
  String get windowDebuggerTitle => 'Nesium 调试器';

  @override
  String get windowToolsTitle => 'Nesium 工具';

  @override
  String get virtualControlsEditTitle => '调整虚拟按键';

  @override
  String get virtualControlsEditSubtitleEnabled => '拖动移动，双指缩放或拖动角落缩放';

  @override
  String get virtualControlsEditSubtitleDisabled => '开启交互式调整';

  @override
  String get gridSnappingTitle => '网格吸附';

  @override
  String get gridSpacingLabel => '网格间距';

  @override
  String get debuggerPlaceholderBody =>
      '这里预留给 CPU/PPU 监视器、内存查看器、OAM 检视器等。相同的组件可以复用到桌面侧边栏或移动端面板中。';

  @override
  String get toolsPlaceholderBody =>
      '录制/回放、按键映射、金手指等功能可以在桌面侧边栏和移动端底部面板中共享这套组件。';

  @override
  String get actionLoadRom => '加载 ROM';

  @override
  String get actionResetNes => '重置 NES';

  @override
  String get actionPowerResetNes => '电源重置 NES';

  @override
  String get actionEjectNes => '关机';

  @override
  String get actionLoadPalette => '加载调色板';

  @override
  String get videoTitle => '画面';

  @override
  String get videoIntegerScalingTitle => '整数缩放';

  @override
  String get videoIntegerScalingSubtitle => '像素级整数缩放（减少滚动抖动）。';

  @override
  String get videoScreenVerticalOffset => '画面垂直偏移';

  @override
  String get videoAspectRatio => '画面比例';

  @override
  String get videoAspectRatioSquare => '1:1（方形像素）';

  @override
  String get videoAspectRatioNtsc => '4:3（NTSC）';

  @override
  String get videoAspectRatioStretch => '拉伸';

  @override
  String get aboutTitle => '关于 Nesium';

  @override
  String get aboutLead => 'Nesium：基于 nesium-core 的 Rust NES/FC 前端。';

  @override
  String get aboutIntro =>
      'Flutter 前端复用 Rust 核心进行模拟；Web 版本通过 Flutter Web + Web Worker + WASM 在浏览器中运行。';

  @override
  String get aboutLinksHeading => '链接';

  @override
  String get aboutGitHubLabel => 'GitHub';

  @override
  String get aboutWebDemoLabel => '在线游玩';

  @override
  String get aboutComponentsHeading => '开源组件';

  @override
  String get aboutComponentsHint => '点击条目可复制链接。';

  @override
  String get aboutLicenseHeading => '许可证';

  @override
  String get aboutLicenseBody =>
      'Nesium 使用 GPL-3.0-or-later 授权。详见仓库根目录的 LICENSE.md。';

  @override
  String get videoBackendLabel => 'Android 渲染后端';

  @override
  String get videoBackendHardware => '硬件（AHardwareBuffer）';

  @override
  String get videoBackendUpload => '兼容（CPU 上传）';

  @override
  String get videoBackendRestartHint => '重启后生效。';

  @override
  String get videoLowLatencyTitle => '低延迟视频';

  @override
  String get videoLowLatencySubtitle => '同步模拟与渲染以减少抖动。重启后生效。';

  @override
  String get paletteModeLabel => '调色板';

  @override
  String get paletteModeBuiltin => '内置';

  @override
  String get paletteModeCustom => '自定义…';

  @override
  String paletteModeCustomActive(String name) {
    return '自定义（$name）';
  }

  @override
  String get builtinPaletteLabel => '内置调色板';

  @override
  String get customPaletteLoadTitle => '加载调色板文件（.pal）…';

  @override
  String get customPaletteLoadSubtitle => '192 字节（RGB）或 256 字节（RGBA）';

  @override
  String commandSucceeded(String label) {
    return '$label 成功';
  }

  @override
  String commandFailed(String label) {
    return '$label 失败';
  }

  @override
  String get snackPaused => '已暂停';

  @override
  String get snackResumed => '已继续';

  @override
  String snackPauseFailed(String error) {
    return '暂停失败：$error';
  }

  @override
  String get dialogOk => '确定';

  @override
  String get debuggerNoRomTitle => '未加载ROM';

  @override
  String get debuggerNoRomSubtitle => '加载ROM后显示调试状态';

  @override
  String get debuggerCpuRegisters => 'CPU 寄存器';

  @override
  String get debuggerPpuState => 'PPU 状态';

  @override
  String get debuggerCpuStatusTooltip =>
      'CPU 状态寄存器 (P)\nN: 负数标志 - 结果第7位为1时置位\nV: 溢出标志 - 有符号溢出时置位\nB: 中断标志 - BRK指令设置\nD: 十进制模式 - BCD模式（NES忽略）\nI: 中断禁用 - 阻止IRQ\nZ: 零标志 - 结果为零时置位\nC: 进位标志 - 无符号溢出时置位\n\n大写=置位, 小写=清除';

  @override
  String get debuggerPpuCtrlTooltip =>
      'PPU 控制寄存器 (\$2000)\nV: NMI使能\nP: PPU主/从（未使用）\nH: 精灵高度（0=8x8, 1=8x16）\nB: 背景图案表地址\nS: 精灵图案表地址\nI: VRAM地址增量（0=1, 1=32）\nNN: 基础名称表地址\n\n大写=置位, 小写=清除';

  @override
  String get debuggerPpuMaskTooltip =>
      'PPU 掩码寄存器 (\$2001)\nBGR: 颜色强调位\ns: 显示精灵\nb: 显示背景\nM: 左8像素显示精灵\nm: 左8像素显示背景\ng: 灰度模式\n\n大写=置位, 小写=清除';

  @override
  String get debuggerPpuStatusTooltip =>
      'PPU 状态寄存器 (\$2002)\nV: VBlank已开始\nS: 精灵0命中\nO: 精灵溢出\n\n大写=置位, 小写=清除';

  @override
  String get debuggerScanlineTooltip =>
      '扫描线说明：\n0-239: 可见区域 (渲染)\n240: post-render (空闲)\n241-260: VBlank (垂直消隐)\n-1: pre-render (预渲染扫描线)';

  @override
  String get tilemapSettings => '设置';

  @override
  String get tilemapOverlay => '叠加层';

  @override
  String get tilemapDisplayMode => '显示模式';

  @override
  String get tilemapDisplayModeDefault => '默认';

  @override
  String get tilemapDisplayModeGrayscale => '灰度';

  @override
  String get tilemapDisplayModeAttributeView => '属性视图';

  @override
  String get tilemapTileGrid => 'Tile 网格 (8×8)';

  @override
  String get tilemapAttrGrid => '属性网格 (16×16)';

  @override
  String get tilemapAttrGrid32 => '属性网格 (32×32)';

  @override
  String get tilemapNtBounds => '名称表边界';

  @override
  String get tilemapScrollOverlay => '滚动叠加';

  @override
  String get tilemapPanelDisplay => '显示';

  @override
  String get tilemapPanelTilemap => 'Tilemap';

  @override
  String get tilemapPanelSelectedTile => '选中 Tile';

  @override
  String get tilemapHidePanel => '隐藏侧边栏';

  @override
  String get tilemapShowPanel => '显示侧边栏';

  @override
  String get tilemapInfoSize => '尺寸';

  @override
  String get tilemapInfoSizePx => '尺寸 (px)';

  @override
  String get tilemapInfoTilemapAddress => 'Tilemap 地址';

  @override
  String get tilemapInfoTilesetAddress => '图案表地址';

  @override
  String get tilemapInfoMirroring => '镜像';

  @override
  String get tilemapInfoTileFormat => 'Tile 格式';

  @override
  String get tilemapInfoTileFormat2bpp => '2 bpp';

  @override
  String get tilemapMirroringHorizontal => '水平';

  @override
  String get tilemapMirroringVertical => '垂直';

  @override
  String get tilemapMirroringFourScreen => '四屏';

  @override
  String get tilemapMirroringSingleScreenLower => '单屏（下）';

  @override
  String get tilemapMirroringSingleScreenUpper => '单屏（上）';

  @override
  String get tilemapMirroringMapperControlled => '由 Mapper 控制';

  @override
  String get tilemapLabelColumnRow => '列, 行';

  @override
  String get tilemapLabelXY => 'X, Y';

  @override
  String get tilemapLabelSize => '尺寸';

  @override
  String get tilemapLabelTilemapAddress => 'Tilemap 地址';

  @override
  String get tilemapLabelTileIndex => 'Tile 索引';

  @override
  String get tilemapLabelTileAddressPpu => 'Tile 地址 (PPU)';

  @override
  String get tilemapLabelPaletteIndex => '调色板索引';

  @override
  String get tilemapLabelPaletteAddress => '调色板地址';

  @override
  String get tilemapLabelAttributeAddress => '属性地址';

  @override
  String get tilemapLabelAttributeData => '属性数据';

  @override
  String get tilemapSelectedTileTilemap => 'Tilemap';

  @override
  String get tilemapSelectedTileTileIdx => 'Tile 索引';

  @override
  String get tilemapSelectedTileTilePpu => 'Tile (PPU)';

  @override
  String get tilemapSelectedTilePalette => '调色板';

  @override
  String get tilemapSelectedTileAttr => '属性';

  @override
  String get tilemapCapture => '捕获';

  @override
  String get tilemapCaptureFrameStart => '帧起始';

  @override
  String get tilemapCaptureVblankStart => 'VBlank 起始';

  @override
  String get tilemapCaptureManual => '手动';

  @override
  String get tilemapScanline => '扫描线';

  @override
  String get tilemapDot => '点';

  @override
  String tilemapError(String error) {
    return '错误：$error';
  }

  @override
  String get tilemapRetry => '重试';

  @override
  String get tilemapResetZoom => '重置缩放';

  @override
  String get menuTilemapViewer => 'Tilemap 查看器';

  @override
  String get menuTileViewer => 'Tile 查看器';

  @override
  String tileViewerError(String error) {
    return '错误：$error';
  }

  @override
  String get tileViewerRetry => '重试';

  @override
  String get tileViewerSettings => 'Tile 查看器设置';

  @override
  String get tileViewerOverlays => '覆盖层';

  @override
  String get tileViewerShowGrid => '显示图块网格';

  @override
  String get tileViewerPalette => '调色板';

  @override
  String tileViewerPaletteBg(int index) {
    return '背景 $index';
  }

  @override
  String tileViewerPaletteSprite(int index) {
    return '精灵 $index';
  }

  @override
  String get tileViewerGrayscale => '使用灰度调色板';

  @override
  String get tileViewerSelectedTile => '选中的图块';

  @override
  String get tileViewerPatternTable => '图案表';

  @override
  String get tileViewerTileIndex => '图块索引';

  @override
  String get tileViewerChrAddress => 'CHR 地址';

  @override
  String get tileViewerClose => '关闭';

  @override
  String get tileViewerSource => '数据源';

  @override
  String get tileViewerSourcePpu => 'PPU 内存';

  @override
  String get tileViewerSourceChrRom => 'CHR ROM';

  @override
  String get tileViewerSourceChrRam => 'CHR RAM';

  @override
  String get tileViewerSourcePrgRom => 'PRG ROM';

  @override
  String get tileViewerAddress => '地址';

  @override
  String get tileViewerSize => '尺寸';

  @override
  String get tileViewerColumns => '列';

  @override
  String get tileViewerRows => '行';

  @override
  String get tileViewerLayout => '布局';

  @override
  String get tileViewerLayoutNormal => '正常';

  @override
  String get tileViewerLayout8x16 => '8×16 精灵';

  @override
  String get tileViewerLayout16x16 => '16×16 精灵';

  @override
  String get tileViewerBackground => '背景色';

  @override
  String get tileViewerBgDefault => '默认';

  @override
  String get tileViewerBgTransparent => '透明';

  @override
  String get tileViewerBgPalette => '调色板颜色';

  @override
  String get tileViewerBgBlack => '黑色';

  @override
  String get tileViewerBgWhite => '白色';

  @override
  String get tileViewerBgMagenta => '品红';

  @override
  String get tileViewerPresets => '预设';

  @override
  String get tileViewerPresetPpu => 'PPU';

  @override
  String get tileViewerPresetChr => 'CHR';

  @override
  String get tileViewerPresetRom => 'ROM';

  @override
  String get tileViewerPresetBg => '背景';

  @override
  String get tileViewerPresetOam => '精灵';

  @override
  String get menuSpriteViewer => '精灵查看器';

  @override
  String get menuPaletteViewer => '调色板查看器';

  @override
  String get paletteViewerPaletteRamTitle => '调色板 RAM (32)';

  @override
  String get paletteViewerSystemPaletteTitle => '系统调色板 (64)';

  @override
  String get paletteViewerSettingsTooltip => '调色板查看器设置';

  @override
  String paletteViewerTooltipPaletteRam(String addr, String value) {
    return '$addr = 0x$value';
  }

  @override
  String paletteViewerTooltipSystemIndex(int index) {
    return '索引 $index';
  }

  @override
  String spriteViewerError(String error) {
    return '精灵查看器错误：$error';
  }

  @override
  String get spriteViewerSettingsTooltip => '精灵查看器设置';

  @override
  String get spriteViewerShowGrid => '显示网格';

  @override
  String get spriteViewerShowOutline => '显示精灵轮廓';

  @override
  String get spriteViewerShowOffscreenRegions => '显示屏幕外区域';

  @override
  String get spriteViewerDimOffscreenSpritesGrid => '淡化屏幕外精灵（网格）';

  @override
  String get spriteViewerShowListView => '显示列表视图';

  @override
  String get spriteViewerPanelSprites => '精灵';

  @override
  String get spriteViewerPanelDataSource => '数据源';

  @override
  String get spriteViewerPanelSprite => '精灵';

  @override
  String get spriteViewerPanelSelectedSprite => '选中精灵';

  @override
  String get spriteViewerLabelMode => '模式';

  @override
  String get spriteViewerLabelPatternBase => '图案表基址';

  @override
  String get spriteViewerLabelThumbnailSize => '缩略图尺寸';

  @override
  String get spriteViewerBgGray => '灰色';

  @override
  String get spriteViewerDataSourceSpriteRam => '精灵 RAM';

  @override
  String get spriteViewerDataSourceCpuMemory => 'CPU 内存';

  @override
  String spriteViewerTooltipTitle(int index) {
    return '精灵 #$index';
  }

  @override
  String get spriteViewerLabelIndex => '索引';

  @override
  String get spriteViewerLabelPos => '位置';

  @override
  String get spriteViewerLabelSize => '尺寸';

  @override
  String get spriteViewerLabelTile => '图块';

  @override
  String get spriteViewerLabelTileAddr => '图块地址';

  @override
  String get spriteViewerLabelPalette => '调色板';

  @override
  String get spriteViewerLabelPaletteAddr => '调色板地址';

  @override
  String get spriteViewerLabelFlip => '翻转';

  @override
  String get spriteViewerLabelPriority => '优先级';

  @override
  String get spriteViewerPriorityBehindBg => '在背景后';

  @override
  String get spriteViewerPriorityInFront => '在前景';

  @override
  String get spriteViewerLabelVisible => '可见';

  @override
  String get spriteViewerValueYes => '是';

  @override
  String get spriteViewerValueNoOffscreen => '否（屏幕外）';

  @override
  String get spriteViewerVisibleStatusVisible => '可见';

  @override
  String get spriteViewerVisibleStatusOffscreen => '屏幕外';

  @override
  String get longPressToClear => '长按清除';
}
