// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for English (`en`).
class AppLocalizationsEn extends AppLocalizations {
  AppLocalizationsEn([String locale = 'en']) : super(locale);

  @override
  String get settingsTitle => 'Settings';

  @override
  String get generalTitle => 'General';

  @override
  String get languageLabel => 'Language';

  @override
  String get languageSystem => 'System';

  @override
  String get languageEnglish => 'English';

  @override
  String get languageChineseSimplified => 'Simplified Chinese';

  @override
  String get inputTitle => 'Input';

  @override
  String get turboTitle => 'Turbo';

  @override
  String get turboLinkPressRelease => 'Link press/release';

  @override
  String get inputDeviceLabel => 'Input device';

  @override
  String get inputDeviceKeyboard => 'Keyboard';

  @override
  String get inputDeviceVirtualController => 'Virtual controller';

  @override
  String get keyboardPresetLabel => 'Keyboard preset';

  @override
  String get keyboardPresetNesStandard => 'NES standard';

  @override
  String get keyboardPresetFightStick => 'Fight stick';

  @override
  String get keyboardPresetArcadeLayout => 'Arcade layout';

  @override
  String get keyboardPresetCustom => 'Custom';

  @override
  String get customKeyBindingsTitle => 'Custom key bindings';

  @override
  String bindKeyTitle(String action) {
    return 'Bind $action';
  }

  @override
  String get unassignedKey => 'Unassigned';

  @override
  String get tipPressEscapeToClearBinding =>
      'Tip: press Escape to clear a binding.';

  @override
  String get keyboardActionUp => 'Up';

  @override
  String get keyboardActionDown => 'Down';

  @override
  String get keyboardActionLeft => 'Left';

  @override
  String get keyboardActionRight => 'Right';

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
  String get emulationTitle => 'Emulation';

  @override
  String get integerFpsTitle => 'Integer FPS mode (60Hz, NTSC)';

  @override
  String get integerFpsSubtitle =>
      'Reduces scrolling judder on 60Hz displays. PAL will be added later.';

  @override
  String get pauseInBackgroundTitle => 'Pause in background';

  @override
  String get pauseInBackgroundSubtitle =>
      'Automatically pauses the emulator when the app is not active.';

  @override
  String get virtualControlsTitle => 'Virtual Controls';

  @override
  String get virtualControlsSwitchInputTip =>
      'Switch input to \"Virtual controller\" to use these settings.';

  @override
  String get virtualControlsButtonSize => 'Button size';

  @override
  String get virtualControlsGap => 'Gap';

  @override
  String get virtualControlsOpacity => 'Opacity';

  @override
  String get virtualControlsHitboxScale => 'Hitbox scale';

  @override
  String get virtualControlsHapticFeedback => 'Haptic feedback';

  @override
  String get virtualControlsDpadDeadzone => 'D-pad deadzone';

  @override
  String get virtualControlsTurboFramesPerToggle => 'Turbo frames per toggle';

  @override
  String get virtualControlsTurboOnFrames => 'Turbo press frames';

  @override
  String get virtualControlsTurboOffFrames => 'Turbo release frames';

  @override
  String framesValue(int frames) {
    return '$frames frames';
  }

  @override
  String get tipAdjustButtonsInDrawer =>
      'Tip: adjust button position/size from the in-game drawer.';

  @override
  String get keyCapturePressKeyToBind => 'Press a key to bind.';

  @override
  String keyCaptureCurrent(String key) {
    return 'Current: $key';
  }

  @override
  String keyCaptureCaptured(String key) {
    return 'Captured: $key';
  }

  @override
  String get keyCapturePressEscToClear => 'Press Escape to clear.';

  @override
  String get keyBindingsTitle => 'Key bindings';

  @override
  String get cancel => 'Cancel';

  @override
  String get appName => 'Nesium';

  @override
  String get menuTooltip => 'Menu';

  @override
  String get menuSectionFile => 'File';

  @override
  String get menuSectionEmulation => 'Emulation';

  @override
  String get menuSectionSettings => 'Settings';

  @override
  String get menuSectionWindows => 'Windows';

  @override
  String get menuOpenRom => 'Open ROM...';

  @override
  String get menuReset => 'Reset';

  @override
  String get menuPowerReset => 'Power Reset';

  @override
  String get menuEject => 'Eject';

  @override
  String get menuPauseResume => 'Pause / Resume';

  @override
  String get menuPreferences => 'Preferences...';

  @override
  String get menuDebugger => 'Debugger';

  @override
  String get menuTools => 'Tools';

  @override
  String get menuOpenDebuggerWindow => 'Open Debugger Window';

  @override
  String get menuOpenToolsWindow => 'Open Tools Window';

  @override
  String get menuInputMappingComingSoon => 'Input Mapping (coming soon)';

  @override
  String get windowDebuggerTitle => 'Nesium Debugger';

  @override
  String get windowToolsTitle => 'Nesium Tools';

  @override
  String get virtualControlsEditTitle => 'Edit virtual controls';

  @override
  String get virtualControlsEditSubtitleEnabled =>
      'Drag to move, pinch or drag corner to resize';

  @override
  String get virtualControlsEditSubtitleDisabled =>
      'Enable interactive adjustment';

  @override
  String get gridSnappingTitle => 'Grid snapping';

  @override
  String get gridSpacingLabel => 'Grid spacing';

  @override
  String get debuggerPlaceholderBody =>
      'Space reserved for CPU/PPU monitors, memory viewers, and OAM inspectors. The same widgets can live in a desktop side panel or a mobile sheet.';

  @override
  String get toolsPlaceholderBody =>
      'Recording/playback, input mapping, and cheats can share these widgets between desktop side panes and mobile bottom sheets.';

  @override
  String get actionLoadRom => 'Load ROM';

  @override
  String get actionResetNes => 'Reset NES';

  @override
  String get actionPowerResetNes => 'Power Reset NES';

  @override
  String get actionEjectNes => 'Eject';

  @override
  String get actionLoadPalette => 'Load palette';

  @override
  String get videoTitle => 'Video';

  @override
  String get videoIntegerScalingTitle => 'Integer scaling';

  @override
  String get videoIntegerScalingSubtitle =>
      'Pixel-perfect scaling (reduces shimmer on scrolling).';

  @override
  String get paletteModeLabel => 'Palette';

  @override
  String get paletteModeBuiltin => 'Built-in';

  @override
  String get paletteModeCustom => 'Custom…';

  @override
  String paletteModeCustomActive(String name) {
    return 'Custom ($name)';
  }

  @override
  String get builtinPaletteLabel => 'Built-in palette';

  @override
  String get customPaletteLoadTitle => 'Load palette file (.pal)…';

  @override
  String get customPaletteLoadSubtitle => '192 bytes (RGB) or 256 bytes (RGBA)';

  @override
  String commandSucceeded(String label) {
    return '$label succeeded';
  }

  @override
  String commandFailed(String label) {
    return '$label failed';
  }

  @override
  String get snackPaused => 'Paused';

  @override
  String get snackResumed => 'Resumed';

  @override
  String snackPauseFailed(String error) {
    return 'Pause failed: $error';
  }

  @override
  String get dialogOk => 'OK';
}
