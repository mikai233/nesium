import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/widgets.dart';
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:intl/intl.dart' as intl;

import 'app_localizations_en.dart';
import 'app_localizations_zh.dart';

// ignore_for_file: type=lint

/// Callers can lookup localized strings with an instance of AppLocalizations
/// returned by `AppLocalizations.of(context)`.
///
/// Applications need to include `AppLocalizations.delegate()` in their app's
/// `localizationDelegates` list, and the locales they support in the app's
/// `supportedLocales` list. For example:
///
/// ```dart
/// import 'l10n/app_localizations.dart';
///
/// return MaterialApp(
///   localizationsDelegates: AppLocalizations.localizationsDelegates,
///   supportedLocales: AppLocalizations.supportedLocales,
///   home: MyApplicationHome(),
/// );
/// ```
///
/// ## Update pubspec.yaml
///
/// Please make sure to update your pubspec.yaml to include the following
/// packages:
///
/// ```yaml
/// dependencies:
///   # Internationalization support.
///   flutter_localizations:
///     sdk: flutter
///   intl: any # Use the pinned version from flutter_localizations
///
///   # Rest of dependencies
/// ```
///
/// ## iOS Applications
///
/// iOS applications define key application metadata, including supported
/// locales, in an Info.plist file that is built into the application bundle.
/// To configure the locales supported by your app, you’ll need to edit this
/// file.
///
/// First, open your project’s ios/Runner.xcworkspace Xcode workspace file.
/// Then, in the Project Navigator, open the Info.plist file under the Runner
/// project’s Runner folder.
///
/// Next, select the Information Property List item, select Add Item from the
/// Editor menu, then select Localizations from the pop-up menu.
///
/// Select and expand the newly-created Localizations item then, for each
/// locale your application supports, add a new item and select the locale
/// you wish to add from the pop-up menu in the Value field. This list should
/// be consistent with the languages listed in the AppLocalizations.supportedLocales
/// property.
abstract class AppLocalizations {
  AppLocalizations(String locale)
    : localeName = intl.Intl.canonicalizedLocale(locale.toString());

  final String localeName;

  static AppLocalizations? of(BuildContext context) {
    return Localizations.of<AppLocalizations>(context, AppLocalizations);
  }

  static const LocalizationsDelegate<AppLocalizations> delegate =
      _AppLocalizationsDelegate();

  /// A list of this localizations delegate along with the default localizations
  /// delegates.
  ///
  /// Returns a list of localizations delegates containing this delegate along with
  /// GlobalMaterialLocalizations.delegate, GlobalCupertinoLocalizations.delegate,
  /// and GlobalWidgetsLocalizations.delegate.
  ///
  /// Additional delegates can be added by appending to this list in
  /// MaterialApp. This list does not have to be used at all if a custom list
  /// of delegates is preferred or required.
  static const List<LocalizationsDelegate<dynamic>> localizationsDelegates =
      <LocalizationsDelegate<dynamic>>[
        delegate,
        GlobalMaterialLocalizations.delegate,
        GlobalCupertinoLocalizations.delegate,
        GlobalWidgetsLocalizations.delegate,
      ];

  /// A list of this localizations delegate's supported locales.
  static const List<Locale> supportedLocales = <Locale>[
    Locale('en'),
    Locale('zh'),
  ];

  /// No description provided for @settingsTitle.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get settingsTitle;

  /// No description provided for @generalTitle.
  ///
  /// In en, this message translates to:
  /// **'General'**
  String get generalTitle;

  /// No description provided for @languageLabel.
  ///
  /// In en, this message translates to:
  /// **'Language'**
  String get languageLabel;

  /// No description provided for @languageSystem.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get languageSystem;

  /// No description provided for @languageEnglish.
  ///
  /// In en, this message translates to:
  /// **'English'**
  String get languageEnglish;

  /// No description provided for @languageChineseSimplified.
  ///
  /// In en, this message translates to:
  /// **'Simplified Chinese'**
  String get languageChineseSimplified;

  /// No description provided for @inputTitle.
  ///
  /// In en, this message translates to:
  /// **'Input'**
  String get inputTitle;

  /// No description provided for @turboTitle.
  ///
  /// In en, this message translates to:
  /// **'Turbo'**
  String get turboTitle;

  /// No description provided for @turboLinkPressRelease.
  ///
  /// In en, this message translates to:
  /// **'Link press/release'**
  String get turboLinkPressRelease;

  /// No description provided for @inputDeviceLabel.
  ///
  /// In en, this message translates to:
  /// **'Input device'**
  String get inputDeviceLabel;

  /// No description provided for @inputDeviceKeyboard.
  ///
  /// In en, this message translates to:
  /// **'Keyboard'**
  String get inputDeviceKeyboard;

  /// No description provided for @inputDeviceVirtualController.
  ///
  /// In en, this message translates to:
  /// **'Virtual controller'**
  String get inputDeviceVirtualController;

  /// No description provided for @keyboardPresetLabel.
  ///
  /// In en, this message translates to:
  /// **'Keyboard preset'**
  String get keyboardPresetLabel;

  /// No description provided for @keyboardPresetNesStandard.
  ///
  /// In en, this message translates to:
  /// **'NES standard'**
  String get keyboardPresetNesStandard;

  /// No description provided for @keyboardPresetFightStick.
  ///
  /// In en, this message translates to:
  /// **'Fight stick'**
  String get keyboardPresetFightStick;

  /// No description provided for @keyboardPresetArcadeLayout.
  ///
  /// In en, this message translates to:
  /// **'Arcade layout'**
  String get keyboardPresetArcadeLayout;

  /// No description provided for @keyboardPresetCustom.
  ///
  /// In en, this message translates to:
  /// **'Custom'**
  String get keyboardPresetCustom;

  /// No description provided for @customKeyBindingsTitle.
  ///
  /// In en, this message translates to:
  /// **'Custom key bindings'**
  String get customKeyBindingsTitle;

  /// No description provided for @bindKeyTitle.
  ///
  /// In en, this message translates to:
  /// **'Bind {action}'**
  String bindKeyTitle(String action);

  /// No description provided for @unassignedKey.
  ///
  /// In en, this message translates to:
  /// **'Unassigned'**
  String get unassignedKey;

  /// No description provided for @tipPressEscapeToClearBinding.
  ///
  /// In en, this message translates to:
  /// **'Tip: press Escape to clear a binding.'**
  String get tipPressEscapeToClearBinding;

  /// No description provided for @keyboardActionUp.
  ///
  /// In en, this message translates to:
  /// **'Up'**
  String get keyboardActionUp;

  /// No description provided for @keyboardActionDown.
  ///
  /// In en, this message translates to:
  /// **'Down'**
  String get keyboardActionDown;

  /// No description provided for @keyboardActionLeft.
  ///
  /// In en, this message translates to:
  /// **'Left'**
  String get keyboardActionLeft;

  /// No description provided for @keyboardActionRight.
  ///
  /// In en, this message translates to:
  /// **'Right'**
  String get keyboardActionRight;

  /// No description provided for @keyboardActionA.
  ///
  /// In en, this message translates to:
  /// **'A'**
  String get keyboardActionA;

  /// No description provided for @keyboardActionB.
  ///
  /// In en, this message translates to:
  /// **'B'**
  String get keyboardActionB;

  /// No description provided for @keyboardActionSelect.
  ///
  /// In en, this message translates to:
  /// **'Select'**
  String get keyboardActionSelect;

  /// No description provided for @keyboardActionStart.
  ///
  /// In en, this message translates to:
  /// **'Start'**
  String get keyboardActionStart;

  /// No description provided for @keyboardActionTurboA.
  ///
  /// In en, this message translates to:
  /// **'Turbo A'**
  String get keyboardActionTurboA;

  /// No description provided for @keyboardActionTurboB.
  ///
  /// In en, this message translates to:
  /// **'Turbo B'**
  String get keyboardActionTurboB;

  /// No description provided for @emulationTitle.
  ///
  /// In en, this message translates to:
  /// **'Emulation'**
  String get emulationTitle;

  /// No description provided for @integerFpsTitle.
  ///
  /// In en, this message translates to:
  /// **'Integer FPS mode (60Hz, NTSC)'**
  String get integerFpsTitle;

  /// No description provided for @integerFpsSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Reduces scrolling judder on 60Hz displays. PAL will be added later.'**
  String get integerFpsSubtitle;

  /// No description provided for @pauseInBackgroundTitle.
  ///
  /// In en, this message translates to:
  /// **'Pause in background'**
  String get pauseInBackgroundTitle;

  /// No description provided for @pauseInBackgroundSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Automatically pauses the emulator when the app is not active.'**
  String get pauseInBackgroundSubtitle;

  /// No description provided for @autoSaveEnabledTitle.
  ///
  /// In en, this message translates to:
  /// **'Auto Save'**
  String get autoSaveEnabledTitle;

  /// No description provided for @autoSaveEnabledSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Periodically save game state to a dedicated slot.'**
  String get autoSaveEnabledSubtitle;

  /// No description provided for @autoSaveIntervalTitle.
  ///
  /// In en, this message translates to:
  /// **'Auto Save Interval'**
  String get autoSaveIntervalTitle;

  /// No description provided for @autoSaveIntervalValue.
  ///
  /// In en, this message translates to:
  /// **'{minutes} minutes'**
  String autoSaveIntervalValue(int minutes);

  /// No description provided for @rewindEnabledTitle.
  ///
  /// In en, this message translates to:
  /// **'Rewind'**
  String get rewindEnabledTitle;

  /// No description provided for @rewindEnabledSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Enable real-time rewind support (Long-press Backspace on Desktop).'**
  String get rewindEnabledSubtitle;

  /// No description provided for @rewindSecondsTitle.
  ///
  /// In en, this message translates to:
  /// **'Rewind Duration'**
  String get rewindSecondsTitle;

  /// No description provided for @rewindSecondsValue.
  ///
  /// In en, this message translates to:
  /// **'{seconds} seconds'**
  String rewindSecondsValue(int seconds);

  /// No description provided for @autoSlotLabel.
  ///
  /// In en, this message translates to:
  /// **'Auto Slot'**
  String get autoSlotLabel;

  /// No description provided for @menuAutoSave.
  ///
  /// In en, this message translates to:
  /// **'Auto Save...'**
  String get menuAutoSave;

  /// No description provided for @stateAutoSaved.
  ///
  /// In en, this message translates to:
  /// **'Auto save created'**
  String get stateAutoSaved;

  /// No description provided for @virtualControlsTitle.
  ///
  /// In en, this message translates to:
  /// **'Virtual Controls'**
  String get virtualControlsTitle;

  /// No description provided for @virtualControlsSwitchInputTip.
  ///
  /// In en, this message translates to:
  /// **'Switch input to \"Virtual controller\" to use these settings.'**
  String get virtualControlsSwitchInputTip;

  /// No description provided for @virtualControlsButtonSize.
  ///
  /// In en, this message translates to:
  /// **'Button size'**
  String get virtualControlsButtonSize;

  /// No description provided for @virtualControlsGap.
  ///
  /// In en, this message translates to:
  /// **'Gap'**
  String get virtualControlsGap;

  /// No description provided for @virtualControlsOpacity.
  ///
  /// In en, this message translates to:
  /// **'Opacity'**
  String get virtualControlsOpacity;

  /// No description provided for @virtualControlsHitboxScale.
  ///
  /// In en, this message translates to:
  /// **'Hitbox scale'**
  String get virtualControlsHitboxScale;

  /// No description provided for @virtualControlsHapticFeedback.
  ///
  /// In en, this message translates to:
  /// **'Haptic feedback'**
  String get virtualControlsHapticFeedback;

  /// No description provided for @virtualControlsDpadDeadzone.
  ///
  /// In en, this message translates to:
  /// **'D-pad deadzone'**
  String get virtualControlsDpadDeadzone;

  /// No description provided for @virtualControlsDpadDeadzoneHelp.
  ///
  /// In en, this message translates to:
  /// **'Center deadzone: touching near the center won’t trigger any direction.'**
  String get virtualControlsDpadDeadzoneHelp;

  /// No description provided for @virtualControlsDpadBoundaryDeadzone.
  ///
  /// In en, this message translates to:
  /// **'D-pad boundary deadzone'**
  String get virtualControlsDpadBoundaryDeadzone;

  /// No description provided for @virtualControlsDpadBoundaryDeadzoneHelp.
  ///
  /// In en, this message translates to:
  /// **'Boundary deadzone: higher values make diagonals harder to trigger, reducing accidental neighbor presses.'**
  String get virtualControlsDpadBoundaryDeadzoneHelp;

  /// No description provided for @virtualControlsReset.
  ///
  /// In en, this message translates to:
  /// **'Reset layout'**
  String get virtualControlsReset;

  /// No description provided for @virtualControlsTurboFramesPerToggle.
  ///
  /// In en, this message translates to:
  /// **'Turbo frames per toggle'**
  String get virtualControlsTurboFramesPerToggle;

  /// No description provided for @virtualControlsTurboOnFrames.
  ///
  /// In en, this message translates to:
  /// **'Turbo press frames'**
  String get virtualControlsTurboOnFrames;

  /// No description provided for @virtualControlsTurboOffFrames.
  ///
  /// In en, this message translates to:
  /// **'Turbo release frames'**
  String get virtualControlsTurboOffFrames;

  /// No description provided for @framesValue.
  ///
  /// In en, this message translates to:
  /// **'{frames} frames'**
  String framesValue(int frames);

  /// No description provided for @tipAdjustButtonsInDrawer.
  ///
  /// In en, this message translates to:
  /// **'Tip: adjust button position/size from the in-game drawer.'**
  String get tipAdjustButtonsInDrawer;

  /// No description provided for @keyCapturePressKeyToBind.
  ///
  /// In en, this message translates to:
  /// **'Press a key to bind.'**
  String get keyCapturePressKeyToBind;

  /// No description provided for @keyCaptureCurrent.
  ///
  /// In en, this message translates to:
  /// **'Current: {key}'**
  String keyCaptureCurrent(String key);

  /// No description provided for @keyCaptureCaptured.
  ///
  /// In en, this message translates to:
  /// **'Captured: {key}'**
  String keyCaptureCaptured(String key);

  /// No description provided for @keyCapturePressEscToClear.
  ///
  /// In en, this message translates to:
  /// **'Press Escape to clear.'**
  String get keyCapturePressEscToClear;

  /// No description provided for @keyBindingsTitle.
  ///
  /// In en, this message translates to:
  /// **'Key bindings'**
  String get keyBindingsTitle;

  /// No description provided for @cancel.
  ///
  /// In en, this message translates to:
  /// **'Cancel'**
  String get cancel;

  /// No description provided for @appName.
  ///
  /// In en, this message translates to:
  /// **'Nesium'**
  String get appName;

  /// No description provided for @menuTooltip.
  ///
  /// In en, this message translates to:
  /// **'Menu'**
  String get menuTooltip;

  /// No description provided for @menuSectionFile.
  ///
  /// In en, this message translates to:
  /// **'File'**
  String get menuSectionFile;

  /// No description provided for @menuSectionEmulation.
  ///
  /// In en, this message translates to:
  /// **'Emulation'**
  String get menuSectionEmulation;

  /// No description provided for @menuSectionSettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get menuSectionSettings;

  /// No description provided for @menuSectionWindows.
  ///
  /// In en, this message translates to:
  /// **'Windows'**
  String get menuSectionWindows;

  /// No description provided for @menuSectionHelp.
  ///
  /// In en, this message translates to:
  /// **'Help'**
  String get menuSectionHelp;

  /// No description provided for @menuOpenRom.
  ///
  /// In en, this message translates to:
  /// **'Open ROM...'**
  String get menuOpenRom;

  /// No description provided for @menuReset.
  ///
  /// In en, this message translates to:
  /// **'Reset'**
  String get menuReset;

  /// No description provided for @menuPowerReset.
  ///
  /// In en, this message translates to:
  /// **'Power Reset'**
  String get menuPowerReset;

  /// No description provided for @menuEject.
  ///
  /// In en, this message translates to:
  /// **'Eject'**
  String get menuEject;

  /// No description provided for @menuSaveState.
  ///
  /// In en, this message translates to:
  /// **'Save State...'**
  String get menuSaveState;

  /// No description provided for @menuLoadState.
  ///
  /// In en, this message translates to:
  /// **'Load State...'**
  String get menuLoadState;

  /// No description provided for @menuPauseResume.
  ///
  /// In en, this message translates to:
  /// **'Pause / Resume'**
  String get menuPauseResume;

  /// No description provided for @menuLoadTasMovie.
  ///
  /// In en, this message translates to:
  /// **'Load TAS Movie...'**
  String get menuLoadTasMovie;

  /// No description provided for @menuPreferences.
  ///
  /// In en, this message translates to:
  /// **'Preferences...'**
  String get menuPreferences;

  /// No description provided for @saveToExternalFile.
  ///
  /// In en, this message translates to:
  /// **'Save to file...'**
  String get saveToExternalFile;

  /// No description provided for @loadFromExternalFile.
  ///
  /// In en, this message translates to:
  /// **'Load from file...'**
  String get loadFromExternalFile;

  /// No description provided for @slotLabel.
  ///
  /// In en, this message translates to:
  /// **'Slot'**
  String get slotLabel;

  /// No description provided for @slotEmpty.
  ///
  /// In en, this message translates to:
  /// **'Empty'**
  String get slotEmpty;

  /// No description provided for @slotHasData.
  ///
  /// In en, this message translates to:
  /// **'Saved'**
  String get slotHasData;

  /// No description provided for @stateSavedToSlot.
  ///
  /// In en, this message translates to:
  /// **'State saved to slot {index}'**
  String stateSavedToSlot(int index);

  /// No description provided for @stateLoadedFromSlot.
  ///
  /// In en, this message translates to:
  /// **'State loaded from slot {index}'**
  String stateLoadedFromSlot(int index);

  /// No description provided for @slotCleared.
  ///
  /// In en, this message translates to:
  /// **'Slot {index} cleared'**
  String slotCleared(int index);

  /// No description provided for @menuAbout.
  ///
  /// In en, this message translates to:
  /// **'About'**
  String get menuAbout;

  /// No description provided for @menuDebugger.
  ///
  /// In en, this message translates to:
  /// **'Debugger'**
  String get menuDebugger;

  /// No description provided for @menuTools.
  ///
  /// In en, this message translates to:
  /// **'Tools'**
  String get menuTools;

  /// No description provided for @menuOpenDebuggerWindow.
  ///
  /// In en, this message translates to:
  /// **'Open Debugger Window'**
  String get menuOpenDebuggerWindow;

  /// No description provided for @menuOpenToolsWindow.
  ///
  /// In en, this message translates to:
  /// **'Open Tools Window'**
  String get menuOpenToolsWindow;

  /// No description provided for @menuInputMappingComingSoon.
  ///
  /// In en, this message translates to:
  /// **'Input Mapping (coming soon)'**
  String get menuInputMappingComingSoon;

  /// No description provided for @menuLastError.
  ///
  /// In en, this message translates to:
  /// **'Last error'**
  String get menuLastError;

  /// No description provided for @lastErrorDetailsAction.
  ///
  /// In en, this message translates to:
  /// **'Details'**
  String get lastErrorDetailsAction;

  /// No description provided for @lastErrorDialogTitle.
  ///
  /// In en, this message translates to:
  /// **'Last error'**
  String get lastErrorDialogTitle;

  /// No description provided for @lastErrorCopied.
  ///
  /// In en, this message translates to:
  /// **'Copied'**
  String get lastErrorCopied;

  /// No description provided for @windowDebuggerTitle.
  ///
  /// In en, this message translates to:
  /// **'Nesium Debugger'**
  String get windowDebuggerTitle;

  /// No description provided for @windowToolsTitle.
  ///
  /// In en, this message translates to:
  /// **'Nesium Tools'**
  String get windowToolsTitle;

  /// No description provided for @virtualControlsEditTitle.
  ///
  /// In en, this message translates to:
  /// **'Edit virtual controls'**
  String get virtualControlsEditTitle;

  /// No description provided for @virtualControlsEditSubtitleEnabled.
  ///
  /// In en, this message translates to:
  /// **'Drag to move, pinch or drag corner to resize'**
  String get virtualControlsEditSubtitleEnabled;

  /// No description provided for @virtualControlsEditSubtitleDisabled.
  ///
  /// In en, this message translates to:
  /// **'Enable interactive adjustment'**
  String get virtualControlsEditSubtitleDisabled;

  /// No description provided for @gridSnappingTitle.
  ///
  /// In en, this message translates to:
  /// **'Grid snapping'**
  String get gridSnappingTitle;

  /// No description provided for @gridSpacingLabel.
  ///
  /// In en, this message translates to:
  /// **'Grid spacing'**
  String get gridSpacingLabel;

  /// No description provided for @debuggerPlaceholderBody.
  ///
  /// In en, this message translates to:
  /// **'Space reserved for CPU/PPU monitors, memory viewers, and OAM inspectors. The same widgets can live in a desktop side panel or a mobile sheet.'**
  String get debuggerPlaceholderBody;

  /// No description provided for @toolsPlaceholderBody.
  ///
  /// In en, this message translates to:
  /// **'Recording/playback, input mapping, and cheats can share these widgets between desktop side panes and mobile bottom sheets.'**
  String get toolsPlaceholderBody;

  /// No description provided for @actionLoadRom.
  ///
  /// In en, this message translates to:
  /// **'Load ROM'**
  String get actionLoadRom;

  /// No description provided for @actionResetNes.
  ///
  /// In en, this message translates to:
  /// **'Reset NES'**
  String get actionResetNes;

  /// No description provided for @actionPowerResetNes.
  ///
  /// In en, this message translates to:
  /// **'Power Reset NES'**
  String get actionPowerResetNes;

  /// No description provided for @actionEjectNes.
  ///
  /// In en, this message translates to:
  /// **'Eject'**
  String get actionEjectNes;

  /// No description provided for @actionLoadPalette.
  ///
  /// In en, this message translates to:
  /// **'Load palette'**
  String get actionLoadPalette;

  /// No description provided for @videoTitle.
  ///
  /// In en, this message translates to:
  /// **'Video'**
  String get videoTitle;

  /// No description provided for @videoIntegerScalingTitle.
  ///
  /// In en, this message translates to:
  /// **'Integer scaling'**
  String get videoIntegerScalingTitle;

  /// No description provided for @videoIntegerScalingSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Pixel-perfect scaling (reduces shimmer on scrolling).'**
  String get videoIntegerScalingSubtitle;

  /// No description provided for @videoScreenVerticalOffset.
  ///
  /// In en, this message translates to:
  /// **'Screen vertical offset'**
  String get videoScreenVerticalOffset;

  /// No description provided for @videoAspectRatio.
  ///
  /// In en, this message translates to:
  /// **'Aspect ratio'**
  String get videoAspectRatio;

  /// No description provided for @videoAspectRatioSquare.
  ///
  /// In en, this message translates to:
  /// **'1:1 (Square pixels)'**
  String get videoAspectRatioSquare;

  /// No description provided for @videoAspectRatioNtsc.
  ///
  /// In en, this message translates to:
  /// **'4:3 (NTSC)'**
  String get videoAspectRatioNtsc;

  /// No description provided for @videoAspectRatioStretch.
  ///
  /// In en, this message translates to:
  /// **'Stretch'**
  String get videoAspectRatioStretch;

  /// No description provided for @aboutTitle.
  ///
  /// In en, this message translates to:
  /// **'About Nesium'**
  String get aboutTitle;

  /// No description provided for @aboutLead.
  ///
  /// In en, this message translates to:
  /// **'Nesium: Rust NES/FC emulator frontend built on nesium-core.'**
  String get aboutLead;

  /// No description provided for @aboutIntro.
  ///
  /// In en, this message translates to:
  /// **'This Flutter frontend reuses the Rust core for emulation. The web build runs in the browser via Flutter Web + Web Worker + WASM.'**
  String get aboutIntro;

  /// No description provided for @aboutLinksHeading.
  ///
  /// In en, this message translates to:
  /// **'Links'**
  String get aboutLinksHeading;

  /// No description provided for @aboutGitHubLabel.
  ///
  /// In en, this message translates to:
  /// **'GitHub'**
  String get aboutGitHubLabel;

  /// No description provided for @aboutWebDemoLabel.
  ///
  /// In en, this message translates to:
  /// **'Web demo'**
  String get aboutWebDemoLabel;

  /// No description provided for @aboutComponentsHeading.
  ///
  /// In en, this message translates to:
  /// **'Open-source components'**
  String get aboutComponentsHeading;

  /// No description provided for @aboutComponentsHint.
  ///
  /// In en, this message translates to:
  /// **'Tap an item to copy the link.'**
  String get aboutComponentsHint;

  /// No description provided for @aboutLicenseHeading.
  ///
  /// In en, this message translates to:
  /// **'License'**
  String get aboutLicenseHeading;

  /// No description provided for @aboutLicenseBody.
  ///
  /// In en, this message translates to:
  /// **'Nesium is licensed under GPL-3.0-or-later. See LICENSE.md in the repository root.'**
  String get aboutLicenseBody;

  /// No description provided for @videoBackendLabel.
  ///
  /// In en, this message translates to:
  /// **'Android renderer backend'**
  String get videoBackendLabel;

  /// No description provided for @videoBackendHardware.
  ///
  /// In en, this message translates to:
  /// **'Hardware (AHardwareBuffer)'**
  String get videoBackendHardware;

  /// No description provided for @videoBackendUpload.
  ///
  /// In en, this message translates to:
  /// **'Compatibility (CPU upload)'**
  String get videoBackendUpload;

  /// No description provided for @videoBackendRestartHint.
  ///
  /// In en, this message translates to:
  /// **'Takes effect after app restart.'**
  String get videoBackendRestartHint;

  /// No description provided for @videoLowLatencyTitle.
  ///
  /// In en, this message translates to:
  /// **'Low latency video'**
  String get videoLowLatencyTitle;

  /// No description provided for @videoLowLatencySubtitle.
  ///
  /// In en, this message translates to:
  /// **'Synchronize emulation and renderer to reduce jitter. Takes effect after app restart.'**
  String get videoLowLatencySubtitle;

  /// No description provided for @paletteModeLabel.
  ///
  /// In en, this message translates to:
  /// **'Palette'**
  String get paletteModeLabel;

  /// No description provided for @paletteModeBuiltin.
  ///
  /// In en, this message translates to:
  /// **'Built-in'**
  String get paletteModeBuiltin;

  /// No description provided for @paletteModeCustom.
  ///
  /// In en, this message translates to:
  /// **'Custom…'**
  String get paletteModeCustom;

  /// No description provided for @paletteModeCustomActive.
  ///
  /// In en, this message translates to:
  /// **'Custom ({name})'**
  String paletteModeCustomActive(String name);

  /// No description provided for @builtinPaletteLabel.
  ///
  /// In en, this message translates to:
  /// **'Built-in palette'**
  String get builtinPaletteLabel;

  /// No description provided for @customPaletteLoadTitle.
  ///
  /// In en, this message translates to:
  /// **'Load palette file (.pal)…'**
  String get customPaletteLoadTitle;

  /// No description provided for @customPaletteLoadSubtitle.
  ///
  /// In en, this message translates to:
  /// **'192 bytes (RGB) or 256 bytes (RGBA)'**
  String get customPaletteLoadSubtitle;

  /// No description provided for @commandSucceeded.
  ///
  /// In en, this message translates to:
  /// **'{label} succeeded'**
  String commandSucceeded(String label);

  /// No description provided for @commandFailed.
  ///
  /// In en, this message translates to:
  /// **'{label} failed'**
  String commandFailed(String label);

  /// No description provided for @snackPaused.
  ///
  /// In en, this message translates to:
  /// **'Paused'**
  String get snackPaused;

  /// No description provided for @snackResumed.
  ///
  /// In en, this message translates to:
  /// **'Resumed'**
  String get snackResumed;

  /// No description provided for @snackPauseFailed.
  ///
  /// In en, this message translates to:
  /// **'Pause failed: {error}'**
  String snackPauseFailed(String error);

  /// No description provided for @dialogOk.
  ///
  /// In en, this message translates to:
  /// **'OK'**
  String get dialogOk;

  /// No description provided for @debuggerNoRomTitle.
  ///
  /// In en, this message translates to:
  /// **'No ROM Running'**
  String get debuggerNoRomTitle;

  /// No description provided for @debuggerNoRomSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Load a ROM to see debug state'**
  String get debuggerNoRomSubtitle;

  /// No description provided for @debuggerCpuRegisters.
  ///
  /// In en, this message translates to:
  /// **'CPU Registers'**
  String get debuggerCpuRegisters;

  /// No description provided for @debuggerPpuState.
  ///
  /// In en, this message translates to:
  /// **'PPU State'**
  String get debuggerPpuState;

  /// No description provided for @debuggerCpuStatusTooltip.
  ///
  /// In en, this message translates to:
  /// **'CPU Status Register (P)\nN: Negative - set if result bit 7 is set\nV: Overflow - set on signed overflow\nB: Break - set by BRK instruction\nD: Decimal - BCD mode (ignored on NES)\nI: Interrupt Disable - blocks IRQ\nZ: Zero - set if result is zero\nC: Carry - set on unsigned overflow\n\nUppercase = set, lowercase = clear'**
  String get debuggerCpuStatusTooltip;

  /// No description provided for @debuggerPpuCtrlTooltip.
  ///
  /// In en, this message translates to:
  /// **'PPU Control Register (\$2000)\nV: NMI enable\nP: PPU master/slave (unused)\nH: Sprite height (0=8x8, 1=8x16)\nB: Background pattern table address\nS: Sprite pattern table address\nI: VRAM address increment (0=1, 1=32)\nNN: Base nametable address\n\nUppercase = set, lowercase = clear'**
  String get debuggerPpuCtrlTooltip;

  /// No description provided for @debuggerPpuMaskTooltip.
  ///
  /// In en, this message translates to:
  /// **'PPU Mask Register (\$2001)\nBGR: Color emphasis bits\ns: Show sprites\nb: Show background\nM: Show sprites in leftmost 8 pixels\nm: Show background in leftmost 8 pixels\ng: Greyscale\n\nUppercase = set, lowercase = clear'**
  String get debuggerPpuMaskTooltip;

  /// No description provided for @debuggerPpuStatusTooltip.
  ///
  /// In en, this message translates to:
  /// **'PPU Status Register (\$2002)\nV: VBlank has started\nS: Sprite 0 hit\nO: Sprite overflow\n\nUppercase = set, lowercase = clear'**
  String get debuggerPpuStatusTooltip;

  /// No description provided for @debuggerScanlineTooltip.
  ///
  /// In en, this message translates to:
  /// **'Scanline Numbers:\n0-239: Visible (Render)\n240: Post-render (Idle)\n241-260: VBlank (Vertical Blanking)\n-1: Pre-render (Dummy scanline)'**
  String get debuggerScanlineTooltip;

  /// No description provided for @tilemapSettings.
  ///
  /// In en, this message translates to:
  /// **'Settings'**
  String get tilemapSettings;

  /// No description provided for @tilemapOverlay.
  ///
  /// In en, this message translates to:
  /// **'Overlay'**
  String get tilemapOverlay;

  /// No description provided for @tilemapDisplayMode.
  ///
  /// In en, this message translates to:
  /// **'Display mode'**
  String get tilemapDisplayMode;

  /// No description provided for @tilemapDisplayModeDefault.
  ///
  /// In en, this message translates to:
  /// **'Default'**
  String get tilemapDisplayModeDefault;

  /// No description provided for @tilemapDisplayModeGrayscale.
  ///
  /// In en, this message translates to:
  /// **'Grayscale'**
  String get tilemapDisplayModeGrayscale;

  /// No description provided for @tilemapDisplayModeAttributeView.
  ///
  /// In en, this message translates to:
  /// **'Attribute view'**
  String get tilemapDisplayModeAttributeView;

  /// No description provided for @tilemapTileGrid.
  ///
  /// In en, this message translates to:
  /// **'Tile Grid (8×8)'**
  String get tilemapTileGrid;

  /// No description provided for @tilemapAttrGrid.
  ///
  /// In en, this message translates to:
  /// **'Attr Grid (16×16)'**
  String get tilemapAttrGrid;

  /// No description provided for @tilemapAttrGrid32.
  ///
  /// In en, this message translates to:
  /// **'Attr Grid (32×32)'**
  String get tilemapAttrGrid32;

  /// No description provided for @tilemapNtBounds.
  ///
  /// In en, this message translates to:
  /// **'NT Bounds'**
  String get tilemapNtBounds;

  /// No description provided for @tilemapScrollOverlay.
  ///
  /// In en, this message translates to:
  /// **'Scroll Overlay'**
  String get tilemapScrollOverlay;

  /// No description provided for @tilemapPanelDisplay.
  ///
  /// In en, this message translates to:
  /// **'Display'**
  String get tilemapPanelDisplay;

  /// No description provided for @tilemapPanelTilemap.
  ///
  /// In en, this message translates to:
  /// **'Tilemap'**
  String get tilemapPanelTilemap;

  /// No description provided for @tilemapPanelSelectedTile.
  ///
  /// In en, this message translates to:
  /// **'Selected Tile'**
  String get tilemapPanelSelectedTile;

  /// No description provided for @tilemapHidePanel.
  ///
  /// In en, this message translates to:
  /// **'Hide panel'**
  String get tilemapHidePanel;

  /// No description provided for @tilemapShowPanel.
  ///
  /// In en, this message translates to:
  /// **'Show panel'**
  String get tilemapShowPanel;

  /// No description provided for @tilemapInfoSize.
  ///
  /// In en, this message translates to:
  /// **'Size'**
  String get tilemapInfoSize;

  /// No description provided for @tilemapInfoSizePx.
  ///
  /// In en, this message translates to:
  /// **'Size (px)'**
  String get tilemapInfoSizePx;

  /// No description provided for @tilemapInfoTilemapAddress.
  ///
  /// In en, this message translates to:
  /// **'Tilemap Address'**
  String get tilemapInfoTilemapAddress;

  /// No description provided for @tilemapInfoTilesetAddress.
  ///
  /// In en, this message translates to:
  /// **'Tileset Address'**
  String get tilemapInfoTilesetAddress;

  /// No description provided for @tilemapInfoMirroring.
  ///
  /// In en, this message translates to:
  /// **'Mirroring'**
  String get tilemapInfoMirroring;

  /// No description provided for @tilemapInfoTileFormat.
  ///
  /// In en, this message translates to:
  /// **'Tile Format'**
  String get tilemapInfoTileFormat;

  /// No description provided for @tilemapInfoTileFormat2bpp.
  ///
  /// In en, this message translates to:
  /// **'2 bpp'**
  String get tilemapInfoTileFormat2bpp;

  /// No description provided for @tilemapMirroringHorizontal.
  ///
  /// In en, this message translates to:
  /// **'Horizontal'**
  String get tilemapMirroringHorizontal;

  /// No description provided for @tilemapMirroringVertical.
  ///
  /// In en, this message translates to:
  /// **'Vertical'**
  String get tilemapMirroringVertical;

  /// No description provided for @tilemapMirroringFourScreen.
  ///
  /// In en, this message translates to:
  /// **'Four-screen'**
  String get tilemapMirroringFourScreen;

  /// No description provided for @tilemapMirroringSingleScreenLower.
  ///
  /// In en, this message translates to:
  /// **'Single-screen (Lower)'**
  String get tilemapMirroringSingleScreenLower;

  /// No description provided for @tilemapMirroringSingleScreenUpper.
  ///
  /// In en, this message translates to:
  /// **'Single-screen (Upper)'**
  String get tilemapMirroringSingleScreenUpper;

  /// No description provided for @tilemapMirroringMapperControlled.
  ///
  /// In en, this message translates to:
  /// **'Mapper-controlled'**
  String get tilemapMirroringMapperControlled;

  /// No description provided for @tilemapLabelColumnRow.
  ///
  /// In en, this message translates to:
  /// **'Column, Row'**
  String get tilemapLabelColumnRow;

  /// No description provided for @tilemapLabelXY.
  ///
  /// In en, this message translates to:
  /// **'X, Y'**
  String get tilemapLabelXY;

  /// No description provided for @tilemapLabelSize.
  ///
  /// In en, this message translates to:
  /// **'Size'**
  String get tilemapLabelSize;

  /// No description provided for @tilemapLabelTilemapAddress.
  ///
  /// In en, this message translates to:
  /// **'Tilemap address'**
  String get tilemapLabelTilemapAddress;

  /// No description provided for @tilemapLabelTileIndex.
  ///
  /// In en, this message translates to:
  /// **'Tile index'**
  String get tilemapLabelTileIndex;

  /// No description provided for @tilemapLabelTileAddressPpu.
  ///
  /// In en, this message translates to:
  /// **'Tile address (PPU)'**
  String get tilemapLabelTileAddressPpu;

  /// No description provided for @tilemapLabelPaletteIndex.
  ///
  /// In en, this message translates to:
  /// **'Palette index'**
  String get tilemapLabelPaletteIndex;

  /// No description provided for @tilemapLabelPaletteAddress.
  ///
  /// In en, this message translates to:
  /// **'Palette address'**
  String get tilemapLabelPaletteAddress;

  /// No description provided for @tilemapLabelAttributeAddress.
  ///
  /// In en, this message translates to:
  /// **'Attribute address'**
  String get tilemapLabelAttributeAddress;

  /// No description provided for @tilemapLabelAttributeData.
  ///
  /// In en, this message translates to:
  /// **'Attribute data'**
  String get tilemapLabelAttributeData;

  /// No description provided for @tilemapSelectedTileTilemap.
  ///
  /// In en, this message translates to:
  /// **'Tilemap'**
  String get tilemapSelectedTileTilemap;

  /// No description provided for @tilemapSelectedTileTileIdx.
  ///
  /// In en, this message translates to:
  /// **'Tile idx'**
  String get tilemapSelectedTileTileIdx;

  /// No description provided for @tilemapSelectedTileTilePpu.
  ///
  /// In en, this message translates to:
  /// **'Tile (PPU)'**
  String get tilemapSelectedTileTilePpu;

  /// No description provided for @tilemapSelectedTilePalette.
  ///
  /// In en, this message translates to:
  /// **'Palette'**
  String get tilemapSelectedTilePalette;

  /// No description provided for @tilemapSelectedTileAttr.
  ///
  /// In en, this message translates to:
  /// **'Attr'**
  String get tilemapSelectedTileAttr;

  /// No description provided for @tilemapCapture.
  ///
  /// In en, this message translates to:
  /// **'Capture'**
  String get tilemapCapture;

  /// No description provided for @tilemapCaptureFrameStart.
  ///
  /// In en, this message translates to:
  /// **'Frame Start'**
  String get tilemapCaptureFrameStart;

  /// No description provided for @tilemapCaptureVblankStart.
  ///
  /// In en, this message translates to:
  /// **'VBlank Start'**
  String get tilemapCaptureVblankStart;

  /// No description provided for @tilemapCaptureManual.
  ///
  /// In en, this message translates to:
  /// **'Manual'**
  String get tilemapCaptureManual;

  /// No description provided for @tilemapScanline.
  ///
  /// In en, this message translates to:
  /// **'Scanline'**
  String get tilemapScanline;

  /// No description provided for @tilemapDot.
  ///
  /// In en, this message translates to:
  /// **'Dot'**
  String get tilemapDot;

  /// No description provided for @tilemapError.
  ///
  /// In en, this message translates to:
  /// **'Error: {error}'**
  String tilemapError(String error);

  /// No description provided for @tilemapRetry.
  ///
  /// In en, this message translates to:
  /// **'Retry'**
  String get tilemapRetry;

  /// No description provided for @tilemapResetZoom.
  ///
  /// In en, this message translates to:
  /// **'Reset Zoom'**
  String get tilemapResetZoom;

  /// No description provided for @menuTilemapViewer.
  ///
  /// In en, this message translates to:
  /// **'Tilemap Viewer'**
  String get menuTilemapViewer;
}

class _AppLocalizationsDelegate
    extends LocalizationsDelegate<AppLocalizations> {
  const _AppLocalizationsDelegate();

  @override
  Future<AppLocalizations> load(Locale locale) {
    return SynchronousFuture<AppLocalizations>(lookupAppLocalizations(locale));
  }

  @override
  bool isSupported(Locale locale) =>
      <String>['en', 'zh'].contains(locale.languageCode);

  @override
  bool shouldReload(_AppLocalizationsDelegate old) => false;
}

AppLocalizations lookupAppLocalizations(Locale locale) {
  // Lookup logic when only language code is specified.
  switch (locale.languageCode) {
    case 'en':
      return AppLocalizationsEn();
    case 'zh':
      return AppLocalizationsZh();
  }

  throw FlutterError(
    'AppLocalizations.delegate failed to load unsupported locale "$locale". This is likely '
    'an issue with the localizations generation tool. Please file an issue '
    'on GitHub with a reproducible sample app and the gen-l10n configuration '
    'that was used.',
  );
}
