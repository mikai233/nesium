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
  String get generalTitle => '通用';

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
  String get inputDeviceVirtualController => '虚拟手柄';

  @override
  String get keyboardPresetLabel => '键盘预设';

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
  String get menuOpenRom => '打开 ROM…';

  @override
  String get menuReset => '重置';

  @override
  String get menuPowerReset => '电源重置';

  @override
  String get menuEject => '弹出';

  @override
  String get menuPauseResume => '暂停 / 继续';

  @override
  String get menuPreferences => '偏好设置…';

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
  String get actionEjectNes => '弹出';

  @override
  String get actionLoadPalette => '加载调色板';

  @override
  String get videoTitle => '画面';

  @override
  String get videoIntegerScalingTitle => '整数缩放';

  @override
  String get videoIntegerScalingSubtitle => '像素级整数缩放（减少滚动抖动）。';

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
}
