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
  String get settingsTabGeneral => 'General';

  @override
  String get settingsTabInput => 'Input';

  @override
  String get settingsTabVideo => 'Video';

  @override
  String get settingsTabEmulation => 'Emulation';

  @override
  String get generalTitle => 'General';

  @override
  String get themeLabel => 'Theme';

  @override
  String get themeSystem => 'System';

  @override
  String get themeLight => 'Light';

  @override
  String get themeDark => 'Dark';

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
  String get autoSaveEnabledTitle => 'Auto Save';

  @override
  String get autoSaveEnabledSubtitle =>
      'Periodically save game state to a dedicated slot.';

  @override
  String get autoSaveIntervalTitle => 'Auto Save Interval';

  @override
  String autoSaveIntervalValue(int minutes) {
    return '$minutes minutes';
  }

  @override
  String get rewindEnabledTitle => 'Rewind';

  @override
  String get rewindEnabledSubtitle =>
      'Enable real-time rewind support (Long-press Backspace on Desktop).';

  @override
  String get rewindSecondsTitle => 'Rewind Duration';

  @override
  String rewindSecondsValue(int seconds) {
    return '$seconds seconds';
  }

  @override
  String get autoSlotLabel => 'Auto Slot';

  @override
  String get menuAutoSave => 'Auto Save...';

  @override
  String get stateAutoSaved => 'Auto save created';

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
  String get virtualControlsDpadDeadzoneHelp =>
      'Center deadzone: touching near the center won’t trigger any direction.';

  @override
  String get virtualControlsDpadBoundaryDeadzone => 'D-pad boundary deadzone';

  @override
  String get virtualControlsDpadBoundaryDeadzoneHelp =>
      'Boundary deadzone: higher values make diagonals harder to trigger, reducing accidental neighbor presses.';

  @override
  String get virtualControlsReset => 'Reset layout';

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
  String get menuSectionHelp => 'Help';

  @override
  String get menuOpenRom => 'Open ROM...';

  @override
  String get menuReset => 'Reset';

  @override
  String get menuPowerReset => 'Power Reset';

  @override
  String get menuEject => 'Power Off';

  @override
  String get menuSaveState => 'Save State...';

  @override
  String get menuLoadState => 'Load State...';

  @override
  String get menuPauseResume => 'Pause / Resume';

  @override
  String get menuNetplay => 'Netplay';

  @override
  String get netplayStatusDisconnected => 'Disconnected';

  @override
  String get netplayStatusConnecting => 'Connecting...';

  @override
  String get netplayStatusConnected => 'Connected (Waiting for Room)';

  @override
  String get netplayStatusInRoom => 'In Room';

  @override
  String get netplayDisconnect => 'Disconnect';

  @override
  String get netplayServerAddress => 'Server Address';

  @override
  String get netplayPlayerName => 'Player Name';

  @override
  String get netplayConnect => 'Connect';

  @override
  String get netplayCreateRoom => 'Create Room';

  @override
  String get netplayJoinRoom => 'Join Room';

  @override
  String get netplayRoomCode => 'Room Code';

  @override
  String get menuLoadTasMovie => 'Load TAS Movie...';

  @override
  String get menuPreferences => 'Preferences...';

  @override
  String get saveToExternalFile => 'Save to file...';

  @override
  String get loadFromExternalFile => 'Load from file...';

  @override
  String get slotLabel => 'Slot';

  @override
  String get slotEmpty => 'Empty';

  @override
  String get slotHasData => 'Saved';

  @override
  String stateSavedToSlot(int index) {
    return 'State saved to slot $index';
  }

  @override
  String stateLoadedFromSlot(int index) {
    return 'State loaded from slot $index';
  }

  @override
  String slotCleared(int index) {
    return 'Slot $index cleared';
  }

  @override
  String get menuAbout => 'About';

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
  String get menuLastError => 'Last error';

  @override
  String get lastErrorDetailsAction => 'Details';

  @override
  String get lastErrorDialogTitle => 'Last error';

  @override
  String get lastErrorCopied => 'Copied';

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
  String get actionEjectNes => 'Power Off';

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
  String get videoScreenVerticalOffset => 'Screen vertical offset';

  @override
  String get videoAspectRatio => 'Aspect ratio';

  @override
  String get videoAspectRatioSquare => '1:1 (Square pixels)';

  @override
  String get videoAspectRatioNtsc => '4:3 (NTSC)';

  @override
  String get videoAspectRatioStretch => 'Stretch';

  @override
  String get aboutTitle => 'About Nesium';

  @override
  String get aboutLead =>
      'Nesium: Rust NES/FC emulator frontend built on nesium-core.';

  @override
  String get aboutIntro =>
      'This Flutter frontend reuses the Rust core for emulation. The web build runs in the browser via Flutter Web + Web Worker + WASM.';

  @override
  String get aboutLinksHeading => 'Links';

  @override
  String get aboutGitHubLabel => 'GitHub';

  @override
  String get aboutWebDemoLabel => 'Web demo';

  @override
  String get aboutComponentsHeading => 'Open-source components';

  @override
  String get aboutComponentsHint => 'Tap an item to copy the link.';

  @override
  String get aboutLicenseHeading => 'License';

  @override
  String get aboutLicenseBody =>
      'Nesium is licensed under GPL-3.0-or-later. See LICENSE.md in the repository root.';

  @override
  String get videoBackendLabel => 'Android renderer backend';

  @override
  String get videoBackendHardware => 'Hardware (AHardwareBuffer)';

  @override
  String get videoBackendUpload => 'Compatibility (CPU upload)';

  @override
  String get videoBackendRestartHint => 'Takes effect after app restart.';

  @override
  String get videoLowLatencyTitle => 'Low latency video';

  @override
  String get videoLowLatencySubtitle =>
      'Synchronize emulation and renderer to reduce jitter. Takes effect after app restart.';

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

  @override
  String get debuggerNoRomTitle => 'No ROM Running';

  @override
  String get debuggerNoRomSubtitle => 'Load a ROM to see debug state';

  @override
  String get debuggerCpuRegisters => 'CPU Registers';

  @override
  String get debuggerPpuState => 'PPU State';

  @override
  String get debuggerCpuStatusTooltip =>
      'CPU Status Register (P)\nN: Negative - set if result bit 7 is set\nV: Overflow - set on signed overflow\nB: Break - set by BRK instruction\nD: Decimal - BCD mode (ignored on NES)\nI: Interrupt Disable - blocks IRQ\nZ: Zero - set if result is zero\nC: Carry - set on unsigned overflow\n\nUppercase = set, lowercase = clear';

  @override
  String get debuggerPpuCtrlTooltip =>
      'PPU Control Register (\$2000)\nV: NMI enable\nP: PPU master/slave (unused)\nH: Sprite height (0=8x8, 1=8x16)\nB: Background pattern table address\nS: Sprite pattern table address\nI: VRAM address increment (0=1, 1=32)\nNN: Base nametable address\n\nUppercase = set, lowercase = clear';

  @override
  String get debuggerPpuMaskTooltip =>
      'PPU Mask Register (\$2001)\nBGR: Color emphasis bits\ns: Show sprites\nb: Show background\nM: Show sprites in leftmost 8 pixels\nm: Show background in leftmost 8 pixels\ng: Greyscale\n\nUppercase = set, lowercase = clear';

  @override
  String get debuggerPpuStatusTooltip =>
      'PPU Status Register (\$2002)\nV: VBlank has started\nS: Sprite 0 hit\nO: Sprite overflow\n\nUppercase = set, lowercase = clear';

  @override
  String get debuggerScanlineTooltip =>
      'Scanline Numbers:\n0-239: Visible (Render)\n240: Post-render (Idle)\n241-260: VBlank (Vertical Blanking)\n-1: Pre-render (Dummy scanline)';

  @override
  String get tilemapSettings => 'Settings';

  @override
  String get tilemapOverlay => 'Overlay';

  @override
  String get tilemapDisplayMode => 'Display mode';

  @override
  String get tilemapDisplayModeDefault => 'Default';

  @override
  String get tilemapDisplayModeGrayscale => 'Grayscale';

  @override
  String get tilemapDisplayModeAttributeView => 'Attribute view';

  @override
  String get tilemapTileGrid => 'Tile Grid (8×8)';

  @override
  String get tilemapAttrGrid => 'Attr Grid (16×16)';

  @override
  String get tilemapAttrGrid32 => 'Attr Grid (32×32)';

  @override
  String get tilemapNtBounds => 'NT Bounds';

  @override
  String get tilemapScrollOverlay => 'Scroll Overlay';

  @override
  String get tilemapPanelDisplay => 'Display';

  @override
  String get tilemapPanelTilemap => 'Tilemap';

  @override
  String get tilemapPanelSelectedTile => 'Selected Tile';

  @override
  String get tilemapHidePanel => 'Hide panel';

  @override
  String get tilemapShowPanel => 'Show panel';

  @override
  String get tilemapInfoSize => 'Size';

  @override
  String get tilemapInfoSizePx => 'Size (px)';

  @override
  String get tilemapInfoTilemapAddress => 'Tilemap Address';

  @override
  String get tilemapInfoTilesetAddress => 'Tileset Address';

  @override
  String get tilemapInfoMirroring => 'Mirroring';

  @override
  String get tilemapInfoTileFormat => 'Tile Format';

  @override
  String get tilemapInfoTileFormat2bpp => '2 bpp';

  @override
  String get tilemapMirroringHorizontal => 'Horizontal';

  @override
  String get tilemapMirroringVertical => 'Vertical';

  @override
  String get tilemapMirroringFourScreen => 'Four-screen';

  @override
  String get tilemapMirroringSingleScreenLower => 'Single-screen (Lower)';

  @override
  String get tilemapMirroringSingleScreenUpper => 'Single-screen (Upper)';

  @override
  String get tilemapMirroringMapperControlled => 'Mapper-controlled';

  @override
  String get tilemapLabelColumnRow => 'Column, Row';

  @override
  String get tilemapLabelXY => 'X, Y';

  @override
  String get tilemapLabelSize => 'Size';

  @override
  String get tilemapLabelTilemapAddress => 'Tilemap address';

  @override
  String get tilemapLabelTileIndex => 'Tile index';

  @override
  String get tilemapLabelTileAddressPpu => 'Tile address (PPU)';

  @override
  String get tilemapLabelPaletteIndex => 'Palette index';

  @override
  String get tilemapLabelPaletteAddress => 'Palette address';

  @override
  String get tilemapLabelAttributeAddress => 'Attribute address';

  @override
  String get tilemapLabelAttributeData => 'Attribute data';

  @override
  String get tilemapSelectedTileTilemap => 'Tilemap';

  @override
  String get tilemapSelectedTileTileIdx => 'Tile idx';

  @override
  String get tilemapSelectedTileTilePpu => 'Tile (PPU)';

  @override
  String get tilemapSelectedTilePalette => 'Palette';

  @override
  String get tilemapSelectedTileAttr => 'Attr';

  @override
  String get tilemapCapture => 'Capture';

  @override
  String get tilemapCaptureFrameStart => 'Frame Start';

  @override
  String get tilemapCaptureVblankStart => 'VBlank Start';

  @override
  String get tilemapCaptureManual => 'Manual';

  @override
  String get tilemapScanline => 'Scanline';

  @override
  String get tilemapDot => 'Dot';

  @override
  String tilemapError(String error) {
    return 'Error: $error';
  }

  @override
  String get tilemapRetry => 'Retry';

  @override
  String get tilemapResetZoom => 'Reset Zoom';

  @override
  String get menuTilemapViewer => 'Tilemap Viewer';

  @override
  String get menuTileViewer => 'Tile Viewer';

  @override
  String tileViewerError(String error) {
    return 'Error: $error';
  }

  @override
  String get tileViewerRetry => 'Retry';

  @override
  String get tileViewerSettings => 'Tile Viewer Settings';

  @override
  String get tileViewerOverlays => 'Overlays';

  @override
  String get tileViewerShowGrid => 'Show tile grid';

  @override
  String get tileViewerPalette => 'Palette';

  @override
  String tileViewerPaletteBg(int index) {
    return 'BG $index';
  }

  @override
  String tileViewerPaletteSprite(int index) {
    return 'Sprite $index';
  }

  @override
  String get tileViewerGrayscale => 'Use grayscale palette';

  @override
  String get tileViewerSelectedTile => 'Selected Tile';

  @override
  String get tileViewerPatternTable => 'Pattern Table';

  @override
  String get tileViewerTileIndex => 'Tile Index';

  @override
  String get tileViewerChrAddress => 'CHR Address';

  @override
  String get tileViewerClose => 'Close';

  @override
  String get tileViewerSource => 'Source';

  @override
  String get tileViewerSourcePpu => 'PPU Memory';

  @override
  String get tileViewerSourceChrRom => 'CHR ROM';

  @override
  String get tileViewerSourceChrRam => 'CHR RAM';

  @override
  String get tileViewerSourcePrgRom => 'PRG ROM';

  @override
  String get tileViewerAddress => 'Address';

  @override
  String get tileViewerSize => 'Size';

  @override
  String get tileViewerColumns => 'Cols';

  @override
  String get tileViewerRows => 'Rows';

  @override
  String get tileViewerLayout => 'Layout';

  @override
  String get tileViewerLayoutNormal => 'Normal';

  @override
  String get tileViewerLayout8x16 => '8×16 Sprites';

  @override
  String get tileViewerLayout16x16 => '16×16 Sprites';

  @override
  String get tileViewerBackground => 'Background';

  @override
  String get tileViewerBgDefault => 'Default';

  @override
  String get tileViewerBgTransparent => 'Transparent';

  @override
  String get tileViewerBgPalette => 'Palette Color';

  @override
  String get tileViewerBgBlack => 'Black';

  @override
  String get tileViewerBgWhite => 'White';

  @override
  String get tileViewerBgMagenta => 'Magenta';

  @override
  String get tileViewerPresets => 'Presets';

  @override
  String get tileViewerPresetPpu => 'PPU';

  @override
  String get tileViewerPresetChr => 'CHR';

  @override
  String get tileViewerPresetRom => 'ROM';

  @override
  String get tileViewerPresetBg => 'BG';

  @override
  String get tileViewerPresetOam => 'OAM';

  @override
  String get menuSpriteViewer => 'Sprite Viewer';

  @override
  String get menuPaletteViewer => 'Palette Viewer';

  @override
  String get paletteViewerPaletteRamTitle => 'Palette RAM (32)';

  @override
  String get paletteViewerSystemPaletteTitle => 'System Palette (64)';

  @override
  String get paletteViewerSettingsTooltip => 'Palette Viewer Settings';

  @override
  String paletteViewerTooltipPaletteRam(String addr, String value) {
    return '$addr = 0x$value';
  }

  @override
  String paletteViewerTooltipSystemIndex(int index) {
    return 'Index $index';
  }

  @override
  String spriteViewerError(String error) {
    return 'Sprite viewer error: $error';
  }

  @override
  String get spriteViewerSettingsTooltip => 'Sprite Viewer Settings';

  @override
  String get spriteViewerShowGrid => 'Show grid';

  @override
  String get spriteViewerShowOutline => 'Show outline around sprites';

  @override
  String get spriteViewerShowOffscreenRegions => 'Show offscreen regions';

  @override
  String get spriteViewerDimOffscreenSpritesGrid =>
      'Dim offscreen sprites (grid)';

  @override
  String get spriteViewerShowListView => 'Show list view';

  @override
  String get spriteViewerPanelSprites => 'Sprites';

  @override
  String get spriteViewerPanelDataSource => 'Data Source';

  @override
  String get spriteViewerPanelSprite => 'Sprite';

  @override
  String get spriteViewerPanelSelectedSprite => 'Selected sprite';

  @override
  String get spriteViewerLabelMode => 'Mode';

  @override
  String get spriteViewerLabelPatternBase => 'Pattern base';

  @override
  String get spriteViewerLabelThumbnailSize => 'Thumbnail size';

  @override
  String get spriteViewerBgGray => 'Gray';

  @override
  String get spriteViewerDataSourceSpriteRam => 'Sprite RAM';

  @override
  String get spriteViewerDataSourceCpuMemory => 'CPU Memory';

  @override
  String spriteViewerTooltipTitle(int index) {
    return 'Sprite #$index';
  }

  @override
  String get spriteViewerLabelIndex => 'Index';

  @override
  String get spriteViewerLabelPos => 'Pos';

  @override
  String get spriteViewerLabelSize => 'Size';

  @override
  String get spriteViewerLabelTile => 'Tile';

  @override
  String get spriteViewerLabelTileAddr => 'Tile addr';

  @override
  String get spriteViewerLabelPalette => 'Palette';

  @override
  String get spriteViewerLabelPaletteAddr => 'Palette addr';

  @override
  String get spriteViewerLabelFlip => 'Flip';

  @override
  String get spriteViewerLabelPriority => 'Priority';

  @override
  String get spriteViewerPriorityBehindBg => 'Behind BG';

  @override
  String get spriteViewerPriorityInFront => 'In front';

  @override
  String get spriteViewerLabelVisible => 'Visible';

  @override
  String get spriteViewerValueYes => 'Yes';

  @override
  String get spriteViewerValueNoOffscreen => 'No (offscreen)';

  @override
  String get spriteViewerVisibleStatusVisible => 'Visible';

  @override
  String get spriteViewerVisibleStatusOffscreen => 'Offscreen';
}
