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

  /// No description provided for @settingsTabGeneral.
  ///
  /// In en, this message translates to:
  /// **'General'**
  String get settingsTabGeneral;

  /// No description provided for @settingsTabInput.
  ///
  /// In en, this message translates to:
  /// **'Input'**
  String get settingsTabInput;

  /// No description provided for @settingsTabVideo.
  ///
  /// In en, this message translates to:
  /// **'Video'**
  String get settingsTabVideo;

  /// No description provided for @settingsTabEmulation.
  ///
  /// In en, this message translates to:
  /// **'Emulation'**
  String get settingsTabEmulation;

  /// No description provided for @settingsTabServer.
  ///
  /// In en, this message translates to:
  /// **'Server'**
  String get settingsTabServer;

  /// No description provided for @settingsFloatingPreviewToggle.
  ///
  /// In en, this message translates to:
  /// **'Floating Preview'**
  String get settingsFloatingPreviewToggle;

  /// No description provided for @settingsFloatingPreviewTooltip.
  ///
  /// In en, this message translates to:
  /// **'Show game preview'**
  String get settingsFloatingPreviewTooltip;

  /// No description provided for @serverTitle.
  ///
  /// In en, this message translates to:
  /// **'Netplay Server'**
  String get serverTitle;

  /// No description provided for @serverPortLabel.
  ///
  /// In en, this message translates to:
  /// **'Port'**
  String get serverPortLabel;

  /// No description provided for @serverStartButton.
  ///
  /// In en, this message translates to:
  /// **'Start Server'**
  String get serverStartButton;

  /// No description provided for @serverStopButton.
  ///
  /// In en, this message translates to:
  /// **'Stop Server'**
  String get serverStopButton;

  /// No description provided for @serverStatusRunning.
  ///
  /// In en, this message translates to:
  /// **'Running'**
  String get serverStatusRunning;

  /// No description provided for @serverStatusStopped.
  ///
  /// In en, this message translates to:
  /// **'Stopped'**
  String get serverStatusStopped;

  /// No description provided for @serverClientCount.
  ///
  /// In en, this message translates to:
  /// **'Connected clients: {count}'**
  String serverClientCount(int count);

  /// No description provided for @serverStartFailed.
  ///
  /// In en, this message translates to:
  /// **'Server start failed: {error}'**
  String serverStartFailed(String error);

  /// No description provided for @serverStopFailed.
  ///
  /// In en, this message translates to:
  /// **'Server stop failed: {error}'**
  String serverStopFailed(String error);

  /// No description provided for @serverBindAddress.
  ///
  /// In en, this message translates to:
  /// **'Bind address: {address}'**
  String serverBindAddress(String address);

  /// No description provided for @serverQuicFingerprint.
  ///
  /// In en, this message translates to:
  /// **'QUIC fingerprint: {fingerprint}'**
  String serverQuicFingerprint(String fingerprint);

  /// No description provided for @generalTitle.
  ///
  /// In en, this message translates to:
  /// **'General'**
  String get generalTitle;

  /// No description provided for @themeLabel.
  ///
  /// In en, this message translates to:
  /// **'Theme'**
  String get themeLabel;

  /// No description provided for @themeSystem.
  ///
  /// In en, this message translates to:
  /// **'System'**
  String get themeSystem;

  /// No description provided for @themeLight.
  ///
  /// In en, this message translates to:
  /// **'Light'**
  String get themeLight;

  /// No description provided for @themeDark.
  ///
  /// In en, this message translates to:
  /// **'Dark'**
  String get themeDark;

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

  /// No description provided for @inputDeviceGamepad.
  ///
  /// In en, this message translates to:
  /// **'Gamepad'**
  String get inputDeviceGamepad;

  /// No description provided for @connectedGamepadsTitle.
  ///
  /// In en, this message translates to:
  /// **'Connected Gamepads'**
  String get connectedGamepadsTitle;

  /// No description provided for @connectedGamepadsNone.
  ///
  /// In en, this message translates to:
  /// **'No gamepads connected'**
  String get connectedGamepadsNone;

  /// No description provided for @webGamepadActivationHint.
  ///
  /// In en, this message translates to:
  /// **'Web limit: PRESS ANY BUTTON on your gamepad to activate it.'**
  String get webGamepadActivationHint;

  /// No description provided for @connectedGamepadsPort.
  ///
  /// In en, this message translates to:
  /// **'Player {port}'**
  String connectedGamepadsPort(int port);

  /// No description provided for @connectedGamepadsUnassigned.
  ///
  /// In en, this message translates to:
  /// **'Unassigned'**
  String get connectedGamepadsUnassigned;

  /// No description provided for @inputDeviceVirtualController.
  ///
  /// In en, this message translates to:
  /// **'Virtual controller'**
  String get inputDeviceVirtualController;

  /// No description provided for @inputGamepadAssignmentLabel.
  ///
  /// In en, this message translates to:
  /// **'Gamepad Assignment'**
  String get inputGamepadAssignmentLabel;

  /// No description provided for @inputGamepadNone.
  ///
  /// In en, this message translates to:
  /// **'None/Unassigned'**
  String get inputGamepadNone;

  /// No description provided for @inputListening.
  ///
  /// In en, this message translates to:
  /// **'Listening...'**
  String get inputListening;

  /// No description provided for @inputDetected.
  ///
  /// In en, this message translates to:
  /// **'Detected: {buttons}'**
  String inputDetected(String buttons);

  /// No description provided for @inputGamepadMappingLabel.
  ///
  /// In en, this message translates to:
  /// **'Button Mapping'**
  String get inputGamepadMappingLabel;

  /// No description provided for @inputResetToDefault.
  ///
  /// In en, this message translates to:
  /// **'Reset to Default'**
  String get inputResetToDefault;

  /// No description provided for @inputButtonA.
  ///
  /// In en, this message translates to:
  /// **'A'**
  String get inputButtonA;

  /// No description provided for @inputButtonB.
  ///
  /// In en, this message translates to:
  /// **'B'**
  String get inputButtonB;

  /// No description provided for @inputButtonTurboA.
  ///
  /// In en, this message translates to:
  /// **'Turbo A'**
  String get inputButtonTurboA;

  /// No description provided for @inputButtonTurboB.
  ///
  /// In en, this message translates to:
  /// **'Turbo B'**
  String get inputButtonTurboB;

  /// No description provided for @inputButtonSelect.
  ///
  /// In en, this message translates to:
  /// **'Select'**
  String get inputButtonSelect;

  /// No description provided for @inputButtonStart.
  ///
  /// In en, this message translates to:
  /// **'Start'**
  String get inputButtonStart;

  /// No description provided for @inputButtonUp.
  ///
  /// In en, this message translates to:
  /// **'Up'**
  String get inputButtonUp;

  /// No description provided for @inputButtonDown.
  ///
  /// In en, this message translates to:
  /// **'Down'**
  String get inputButtonDown;

  /// No description provided for @inputButtonLeft.
  ///
  /// In en, this message translates to:
  /// **'Left'**
  String get inputButtonLeft;

  /// No description provided for @inputButtonRight.
  ///
  /// In en, this message translates to:
  /// **'Right'**
  String get inputButtonRight;

  /// No description provided for @inputButtonRewind.
  ///
  /// In en, this message translates to:
  /// **'Rewind'**
  String get inputButtonRewind;

  /// No description provided for @inputButtonFastForward.
  ///
  /// In en, this message translates to:
  /// **'Fast Forward'**
  String get inputButtonFastForward;

  /// No description provided for @inputButtonSaveState.
  ///
  /// In en, this message translates to:
  /// **'Save State'**
  String get inputButtonSaveState;

  /// No description provided for @inputButtonLoadState.
  ///
  /// In en, this message translates to:
  /// **'Load State'**
  String get inputButtonLoadState;

  /// No description provided for @inputButtonPause.
  ///
  /// In en, this message translates to:
  /// **'Pause'**
  String get inputButtonPause;

  /// No description provided for @globalHotkeysTitle.
  ///
  /// In en, this message translates to:
  /// **'Emulator Hotkeys'**
  String get globalHotkeysTitle;

  /// No description provided for @gamepadHotkeysTitle.
  ///
  /// In en, this message translates to:
  /// **'Gamepad Hotkeys (Player 1)'**
  String get gamepadHotkeysTitle;

  /// No description provided for @inputPortLabel.
  ///
  /// In en, this message translates to:
  /// **'Configure Player'**
  String get inputPortLabel;

  /// No description provided for @player1.
  ///
  /// In en, this message translates to:
  /// **'Player 1'**
  String get player1;

  /// No description provided for @player2.
  ///
  /// In en, this message translates to:
  /// **'Player 2'**
  String get player2;

  /// No description provided for @player3.
  ///
  /// In en, this message translates to:
  /// **'Player 3'**
  String get player3;

  /// No description provided for @player4.
  ///
  /// In en, this message translates to:
  /// **'Player 4'**
  String get player4;

  /// No description provided for @keyboardPresetLabel.
  ///
  /// In en, this message translates to:
  /// **'Keyboard preset'**
  String get keyboardPresetLabel;

  /// No description provided for @keyboardPresetNone.
  ///
  /// In en, this message translates to:
  /// **'None'**
  String get keyboardPresetNone;

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

  /// No description provided for @keyboardActionRewind.
  ///
  /// In en, this message translates to:
  /// **'Rewind'**
  String get keyboardActionRewind;

  /// No description provided for @keyboardActionFastForward.
  ///
  /// In en, this message translates to:
  /// **'Fast Forward'**
  String get keyboardActionFastForward;

  /// No description provided for @keyboardActionSaveState.
  ///
  /// In en, this message translates to:
  /// **'Save State'**
  String get keyboardActionSaveState;

  /// No description provided for @keyboardActionLoadState.
  ///
  /// In en, this message translates to:
  /// **'Load State'**
  String get keyboardActionLoadState;

  /// No description provided for @keyboardActionPause.
  ///
  /// In en, this message translates to:
  /// **'Pause'**
  String get keyboardActionPause;

  /// No description provided for @keyboardActionFullScreen.
  ///
  /// In en, this message translates to:
  /// **'Full Screen'**
  String get keyboardActionFullScreen;

  /// No description provided for @inputBindingConflictCleared.
  ///
  /// In en, this message translates to:
  /// **'{player} {action} binding cleared.'**
  String inputBindingConflictCleared(String player, String action);

  /// No description provided for @inputBindingConflictHint.
  ///
  /// In en, this message translates to:
  /// **'({player} - {action})'**
  String inputBindingConflictHint(String player, String action);

  /// No description provided for @inputBindingCapturedConflictHint.
  ///
  /// In en, this message translates to:
  /// **'Occupied by {player} - {action}'**
  String inputBindingCapturedConflictHint(String player, String action);

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

  /// No description provided for @showOverlayTitle.
  ///
  /// In en, this message translates to:
  /// **'Show status overlay'**
  String get showOverlayTitle;

  /// No description provided for @showOverlaySubtitle.
  ///
  /// In en, this message translates to:
  /// **'Show pause/rewind/fast-forward indicators on screen.'**
  String get showOverlaySubtitle;

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

  /// No description provided for @fastForwardSpeedTitle.
  ///
  /// In en, this message translates to:
  /// **'Fast Forward Speed'**
  String get fastForwardSpeedTitle;

  /// No description provided for @fastForwardSpeedSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Maximum speed while fast forward is active.'**
  String get fastForwardSpeedSubtitle;

  /// No description provided for @fastForwardSpeedValue.
  ///
  /// In en, this message translates to:
  /// **'{percent}%'**
  String fastForwardSpeedValue(int percent);

  /// No description provided for @quickSaveSlotTitle.
  ///
  /// In en, this message translates to:
  /// **'Quick Save Slot'**
  String get quickSaveSlotTitle;

  /// No description provided for @quickSaveSlotSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Slot used by quick save/load shortcuts.'**
  String get quickSaveSlotSubtitle;

  /// No description provided for @quickSaveSlotValue.
  ///
  /// In en, this message translates to:
  /// **'Slot {index}'**
  String quickSaveSlotValue(int index);

  /// No description provided for @rewindEnabledTitle.
  ///
  /// In en, this message translates to:
  /// **'Rewind'**
  String get rewindEnabledTitle;

  /// No description provided for @rewindEnabledSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Enable real-time rewind functionality.'**
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

  /// No description provided for @rewindMinutesTitle.
  ///
  /// In en, this message translates to:
  /// **'Rewind Duration'**
  String get rewindMinutesTitle;

  /// No description provided for @rewindMinutesValue.
  ///
  /// In en, this message translates to:
  /// **'{minutes} minutes'**
  String rewindMinutesValue(int minutes);

  /// No description provided for @rewindSpeedTitle.
  ///
  /// In en, this message translates to:
  /// **'Rewind Speed'**
  String get rewindSpeedTitle;

  /// No description provided for @rewindSpeedSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Speed while rewinding is active.'**
  String get rewindSpeedSubtitle;

  /// No description provided for @rewindSpeedValue.
  ///
  /// In en, this message translates to:
  /// **'{percent}%'**
  String rewindSpeedValue(int percent);

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

  /// No description provided for @virtualControlsDiscardChangesTitle.
  ///
  /// In en, this message translates to:
  /// **'Undo changes'**
  String get virtualControlsDiscardChangesTitle;

  /// No description provided for @virtualControlsDiscardChangesSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Revert to last saved layout'**
  String get virtualControlsDiscardChangesSubtitle;

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
  /// **'Power Off'**
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

  /// No description provided for @menuNetplay.
  ///
  /// In en, this message translates to:
  /// **'Netplay'**
  String get menuNetplay;

  /// No description provided for @netplayTransportLabel.
  ///
  /// In en, this message translates to:
  /// **'Transport'**
  String get netplayTransportLabel;

  /// No description provided for @netplayTransportAuto.
  ///
  /// In en, this message translates to:
  /// **'Auto (QUIC → TCP)'**
  String get netplayTransportAuto;

  /// No description provided for @netplayTransportUnknown.
  ///
  /// In en, this message translates to:
  /// **'Unknown'**
  String get netplayTransportUnknown;

  /// No description provided for @netplayTransportTcp.
  ///
  /// In en, this message translates to:
  /// **'TCP'**
  String get netplayTransportTcp;

  /// No description provided for @netplayTransportQuic.
  ///
  /// In en, this message translates to:
  /// **'QUIC'**
  String get netplayTransportQuic;

  /// No description provided for @netplayUsingTcpFallback.
  ///
  /// In en, this message translates to:
  /// **'QUIC failed, using TCP'**
  String get netplayUsingTcpFallback;

  /// No description provided for @netplayStatusDisconnected.
  ///
  /// In en, this message translates to:
  /// **'Disconnected'**
  String get netplayStatusDisconnected;

  /// No description provided for @netplayStatusConnecting.
  ///
  /// In en, this message translates to:
  /// **'Connecting...'**
  String get netplayStatusConnecting;

  /// No description provided for @netplayStatusConnected.
  ///
  /// In en, this message translates to:
  /// **'Connected (Waiting for Room)'**
  String get netplayStatusConnected;

  /// No description provided for @netplayStatusInRoom.
  ///
  /// In en, this message translates to:
  /// **'In Room'**
  String get netplayStatusInRoom;

  /// No description provided for @netplayDisconnect.
  ///
  /// In en, this message translates to:
  /// **'Disconnect'**
  String get netplayDisconnect;

  /// No description provided for @netplayServerAddress.
  ///
  /// In en, this message translates to:
  /// **'Server Address'**
  String get netplayServerAddress;

  /// No description provided for @netplayServerNameLabel.
  ///
  /// In en, this message translates to:
  /// **'Server name (SNI)'**
  String get netplayServerNameLabel;

  /// No description provided for @netplayServerNameHint.
  ///
  /// In en, this message translates to:
  /// **'localhost'**
  String get netplayServerNameHint;

  /// No description provided for @netplayPlayerName.
  ///
  /// In en, this message translates to:
  /// **'Player Name'**
  String get netplayPlayerName;

  /// No description provided for @netplayQuicFingerprintLabel.
  ///
  /// In en, this message translates to:
  /// **'QUIC cert fingerprint (optional)'**
  String get netplayQuicFingerprintLabel;

  /// No description provided for @netplayQuicFingerprintHint.
  ///
  /// In en, this message translates to:
  /// **'base64url (43 chars)'**
  String get netplayQuicFingerprintHint;

  /// No description provided for @netplayQuicFingerprintHelper.
  ///
  /// In en, this message translates to:
  /// **'Enter this to use pinned QUIC. Leave empty to use system trust (QUIC) or fallback to TCP.'**
  String get netplayQuicFingerprintHelper;

  /// No description provided for @netplayConnect.
  ///
  /// In en, this message translates to:
  /// **'Join Game'**
  String get netplayConnect;

  /// No description provided for @netplayJoinViaP2P.
  ///
  /// In en, this message translates to:
  /// **'Join via P2P'**
  String get netplayJoinViaP2P;

  /// No description provided for @netplayJoinGame.
  ///
  /// In en, this message translates to:
  /// **'Join Game'**
  String get netplayJoinGame;

  /// No description provided for @netplayCreateRoom.
  ///
  /// In en, this message translates to:
  /// **'Create Room'**
  String get netplayCreateRoom;

  /// No description provided for @netplayJoinRoom.
  ///
  /// In en, this message translates to:
  /// **'Join Game'**
  String get netplayJoinRoom;

  /// No description provided for @netplayAddressOrRoomCode.
  ///
  /// In en, this message translates to:
  /// **'Room Code or Server Address'**
  String get netplayAddressOrRoomCode;

  /// No description provided for @netplayHostingTitle.
  ///
  /// In en, this message translates to:
  /// **'Hosting'**
  String get netplayHostingTitle;

  /// No description provided for @netplayRoomCodeLabel.
  ///
  /// In en, this message translates to:
  /// **'Your Room Code'**
  String get netplayRoomCodeLabel;

  /// No description provided for @netplayP2PEnabled.
  ///
  /// In en, this message translates to:
  /// **'P2P Mode'**
  String get netplayP2PEnabled;

  /// No description provided for @netplayDirectServerLabel.
  ///
  /// In en, this message translates to:
  /// **'Server Address'**
  String get netplayDirectServerLabel;

  /// No description provided for @netplayAdvancedSettings.
  ///
  /// In en, this message translates to:
  /// **'Advanced Connection Settings'**
  String get netplayAdvancedSettings;

  /// No description provided for @netplayP2PServerLabel.
  ///
  /// In en, this message translates to:
  /// **'P2P Server'**
  String get netplayP2PServerLabel;

  /// No description provided for @netplayRoomCode.
  ///
  /// In en, this message translates to:
  /// **'Room Code'**
  String get netplayRoomCode;

  /// No description provided for @netplayRoleLabel.
  ///
  /// In en, this message translates to:
  /// **'Role'**
  String get netplayRoleLabel;

  /// No description provided for @netplayPlayerIndex.
  ///
  /// In en, this message translates to:
  /// **'Player {index}'**
  String netplayPlayerIndex(int index);

  /// No description provided for @netplaySpectator.
  ///
  /// In en, this message translates to:
  /// **'Spectator'**
  String get netplaySpectator;

  /// No description provided for @netplayClientId.
  ///
  /// In en, this message translates to:
  /// **'Client ID'**
  String get netplayClientId;

  /// No description provided for @netplayPlayerListHeader.
  ///
  /// In en, this message translates to:
  /// **'Players'**
  String get netplayPlayerListHeader;

  /// No description provided for @netplayYouIndicator.
  ///
  /// In en, this message translates to:
  /// **'(You)'**
  String get netplayYouIndicator;

  /// No description provided for @netplayOrSeparator.
  ///
  /// In en, this message translates to:
  /// **'OR'**
  String get netplayOrSeparator;

  /// No description provided for @netplayConnectFailed.
  ///
  /// In en, this message translates to:
  /// **'Connect failed: {error}'**
  String netplayConnectFailed(String error);

  /// No description provided for @netplayDisconnectFailed.
  ///
  /// In en, this message translates to:
  /// **'Disconnect failed: {error}'**
  String netplayDisconnectFailed(String error);

  /// No description provided for @netplayCreateRoomFailed.
  ///
  /// In en, this message translates to:
  /// **'Create room failed: {error}'**
  String netplayCreateRoomFailed(String error);

  /// No description provided for @netplayJoinRoomFailed.
  ///
  /// In en, this message translates to:
  /// **'Join room failed: {error}'**
  String netplayJoinRoomFailed(String error);

  /// No description provided for @netplaySwitchRoleFailed.
  ///
  /// In en, this message translates to:
  /// **'Switch role failed: {error}'**
  String netplaySwitchRoleFailed(String error);

  /// No description provided for @netplayInvalidRoomCode.
  ///
  /// In en, this message translates to:
  /// **'Invalid room code'**
  String get netplayInvalidRoomCode;

  /// No description provided for @netplayRomBroadcasted.
  ///
  /// In en, this message translates to:
  /// **'Netplay: ROM broadcasted to room'**
  String get netplayRomBroadcasted;

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

  /// No description provided for @copy.
  ///
  /// In en, this message translates to:
  /// **'Copy'**
  String get copy;

  /// No description provided for @paste.
  ///
  /// In en, this message translates to:
  /// **'Paste'**
  String get paste;

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
  /// **'Power Off'**
  String get actionEjectNes;

  /// No description provided for @actionLoadPalette.
  ///
  /// In en, this message translates to:
  /// **'Load palette'**
  String get actionLoadPalette;

  /// No description provided for @videoResetToDefault.
  ///
  /// In en, this message translates to:
  /// **'Reset to default'**
  String get videoResetToDefault;

  /// No description provided for @videoTitle.
  ///
  /// In en, this message translates to:
  /// **'Video'**
  String get videoTitle;

  /// No description provided for @videoFilterLabel.
  ///
  /// In en, this message translates to:
  /// **'Video Filter'**
  String get videoFilterLabel;

  /// No description provided for @videoFilterCategoryCpu.
  ///
  /// In en, this message translates to:
  /// **'CPU Filters'**
  String get videoFilterCategoryCpu;

  /// No description provided for @videoFilterCategoryGpu.
  ///
  /// In en, this message translates to:
  /// **'GPU Filters (Shaders)'**
  String get videoFilterCategoryGpu;

  /// No description provided for @videoFilterNone.
  ///
  /// In en, this message translates to:
  /// **'None (1x)'**
  String get videoFilterNone;

  /// No description provided for @videoFilterPrescale2x.
  ///
  /// In en, this message translates to:
  /// **'Prescale 2x'**
  String get videoFilterPrescale2x;

  /// No description provided for @videoFilterPrescale3x.
  ///
  /// In en, this message translates to:
  /// **'Prescale 3x'**
  String get videoFilterPrescale3x;

  /// No description provided for @videoFilterPrescale4x.
  ///
  /// In en, this message translates to:
  /// **'Prescale 4x'**
  String get videoFilterPrescale4x;

  /// No description provided for @videoFilterHq2x.
  ///
  /// In en, this message translates to:
  /// **'HQ2x'**
  String get videoFilterHq2x;

  /// No description provided for @videoFilterHq3x.
  ///
  /// In en, this message translates to:
  /// **'HQ3x'**
  String get videoFilterHq3x;

  /// No description provided for @videoFilterHq4x.
  ///
  /// In en, this message translates to:
  /// **'HQ4x'**
  String get videoFilterHq4x;

  /// No description provided for @videoFilter2xSai.
  ///
  /// In en, this message translates to:
  /// **'2xSaI'**
  String get videoFilter2xSai;

  /// No description provided for @videoFilterSuper2xSai.
  ///
  /// In en, this message translates to:
  /// **'Super 2xSaI'**
  String get videoFilterSuper2xSai;

  /// No description provided for @videoFilterSuperEagle.
  ///
  /// In en, this message translates to:
  /// **'Super Eagle'**
  String get videoFilterSuperEagle;

  /// No description provided for @videoFilterLcdGrid.
  ///
  /// In en, this message translates to:
  /// **'LCD Grid (2x)'**
  String get videoFilterLcdGrid;

  /// No description provided for @videoFilterScanlines.
  ///
  /// In en, this message translates to:
  /// **'Scanlines (2x)'**
  String get videoFilterScanlines;

  /// No description provided for @videoFilterXbrz2x.
  ///
  /// In en, this message translates to:
  /// **'xBRZ 2x'**
  String get videoFilterXbrz2x;

  /// No description provided for @videoFilterXbrz3x.
  ///
  /// In en, this message translates to:
  /// **'xBRZ 3x'**
  String get videoFilterXbrz3x;

  /// No description provided for @videoFilterXbrz4x.
  ///
  /// In en, this message translates to:
  /// **'xBRZ 4x'**
  String get videoFilterXbrz4x;

  /// No description provided for @videoFilterXbrz5x.
  ///
  /// In en, this message translates to:
  /// **'xBRZ 5x'**
  String get videoFilterXbrz5x;

  /// No description provided for @videoFilterXbrz6x.
  ///
  /// In en, this message translates to:
  /// **'xBRZ 6x'**
  String get videoFilterXbrz6x;

  /// No description provided for @videoLcdGridStrengthLabel.
  ///
  /// In en, this message translates to:
  /// **'LCD Grid Strength'**
  String get videoLcdGridStrengthLabel;

  /// No description provided for @videoScanlinesIntensityLabel.
  ///
  /// In en, this message translates to:
  /// **'Scanline Intensity'**
  String get videoScanlinesIntensityLabel;

  /// No description provided for @videoFilterNtscComposite.
  ///
  /// In en, this message translates to:
  /// **'NTSC (Composite)'**
  String get videoFilterNtscComposite;

  /// No description provided for @videoFilterNtscSvideo.
  ///
  /// In en, this message translates to:
  /// **'NTSC (S-Video)'**
  String get videoFilterNtscSvideo;

  /// No description provided for @videoFilterNtscRgb.
  ///
  /// In en, this message translates to:
  /// **'NTSC (RGB)'**
  String get videoFilterNtscRgb;

  /// No description provided for @videoFilterNtscMonochrome.
  ///
  /// In en, this message translates to:
  /// **'NTSC (Monochrome)'**
  String get videoFilterNtscMonochrome;

  /// No description provided for @videoFilterNtscBisqwit2x.
  ///
  /// In en, this message translates to:
  /// **'NTSC (Bisqwit) 2x'**
  String get videoFilterNtscBisqwit2x;

  /// No description provided for @videoFilterNtscBisqwit4x.
  ///
  /// In en, this message translates to:
  /// **'NTSC (Bisqwit) 4x'**
  String get videoFilterNtscBisqwit4x;

  /// No description provided for @videoFilterNtscBisqwit8x.
  ///
  /// In en, this message translates to:
  /// **'NTSC (Bisqwit) 8x'**
  String get videoFilterNtscBisqwit8x;

  /// No description provided for @videoNtscAdvancedTitle.
  ///
  /// In en, this message translates to:
  /// **'NTSC Advanced'**
  String get videoNtscAdvancedTitle;

  /// No description provided for @videoNtscMergeFieldsLabel.
  ///
  /// In en, this message translates to:
  /// **'Merge fields (reduce flicker)'**
  String get videoNtscMergeFieldsLabel;

  /// No description provided for @videoNtscHueLabel.
  ///
  /// In en, this message translates to:
  /// **'Hue'**
  String get videoNtscHueLabel;

  /// No description provided for @videoNtscSaturationLabel.
  ///
  /// In en, this message translates to:
  /// **'Saturation'**
  String get videoNtscSaturationLabel;

  /// No description provided for @videoNtscContrastLabel.
  ///
  /// In en, this message translates to:
  /// **'Contrast'**
  String get videoNtscContrastLabel;

  /// No description provided for @videoNtscBrightnessLabel.
  ///
  /// In en, this message translates to:
  /// **'Brightness'**
  String get videoNtscBrightnessLabel;

  /// No description provided for @videoNtscSharpnessLabel.
  ///
  /// In en, this message translates to:
  /// **'Sharpness'**
  String get videoNtscSharpnessLabel;

  /// No description provided for @videoNtscGammaLabel.
  ///
  /// In en, this message translates to:
  /// **'Gamma'**
  String get videoNtscGammaLabel;

  /// No description provided for @videoNtscResolutionLabel.
  ///
  /// In en, this message translates to:
  /// **'Resolution'**
  String get videoNtscResolutionLabel;

  /// No description provided for @videoNtscArtifactsLabel.
  ///
  /// In en, this message translates to:
  /// **'Artifacts'**
  String get videoNtscArtifactsLabel;

  /// No description provided for @videoNtscFringingLabel.
  ///
  /// In en, this message translates to:
  /// **'Fringing'**
  String get videoNtscFringingLabel;

  /// No description provided for @videoNtscBleedLabel.
  ///
  /// In en, this message translates to:
  /// **'Bleed'**
  String get videoNtscBleedLabel;

  /// No description provided for @videoNtscBisqwitSettingsTitle.
  ///
  /// In en, this message translates to:
  /// **'NTSC settings (Bisqwit)'**
  String get videoNtscBisqwitSettingsTitle;

  /// No description provided for @videoNtscBisqwitYFilterLengthLabel.
  ///
  /// In en, this message translates to:
  /// **'Y Filter (Horizontal Blur)'**
  String get videoNtscBisqwitYFilterLengthLabel;

  /// No description provided for @videoNtscBisqwitIFilterLengthLabel.
  ///
  /// In en, this message translates to:
  /// **'I Filter'**
  String get videoNtscBisqwitIFilterLengthLabel;

  /// No description provided for @videoNtscBisqwitQFilterLengthLabel.
  ///
  /// In en, this message translates to:
  /// **'Q Filter'**
  String get videoNtscBisqwitQFilterLengthLabel;

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

  /// No description provided for @videoFullScreenTitle.
  ///
  /// In en, this message translates to:
  /// **'Full Screen'**
  String get videoFullScreenTitle;

  /// No description provided for @videoFullScreenSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Toggle window full screen state'**
  String get videoFullScreenSubtitle;

  /// No description provided for @videoScreenVerticalOffset.
  ///
  /// In en, this message translates to:
  /// **'Screen vertical offset'**
  String get videoScreenVerticalOffset;

  /// No description provided for @videoScreenVerticalOffsetPortraitOnly.
  ///
  /// In en, this message translates to:
  /// **'Only takes effect in portrait mode.'**
  String get videoScreenVerticalOffsetPortraitOnly;

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

  /// No description provided for @videoShaderLibrashaderTitle.
  ///
  /// In en, this message translates to:
  /// **'RetroArch Shaders'**
  String get videoShaderLibrashaderTitle;

  /// No description provided for @videoShaderLibrashaderSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Requires GLES3 + Hardware backend (AHB swapchain).'**
  String get videoShaderLibrashaderSubtitle;

  /// No description provided for @videoShaderLibrashaderSubtitleWindows.
  ///
  /// In en, this message translates to:
  /// **'Requires D3D11 GPU backend.'**
  String get videoShaderLibrashaderSubtitleWindows;

  /// No description provided for @videoShaderLibrashaderSubtitleApple.
  ///
  /// In en, this message translates to:
  /// **'Requires Metal backend.'**
  String get videoShaderLibrashaderSubtitleApple;

  /// No description provided for @videoShaderLibrashaderSubtitleDisabled.
  ///
  /// In en, this message translates to:
  /// **'Switch Android backend to Hardware to enable.'**
  String get videoShaderLibrashaderSubtitleDisabled;

  /// No description provided for @videoShaderLibrashaderSubtitleDisabledWindows.
  ///
  /// In en, this message translates to:
  /// **'Switch Windows backend to D3D11 GPU to enable.'**
  String get videoShaderLibrashaderSubtitleDisabledWindows;

  /// No description provided for @videoShaderPresetLabel.
  ///
  /// In en, this message translates to:
  /// **'Preset (.slangp)'**
  String get videoShaderPresetLabel;

  /// No description provided for @videoShaderPresetNotSet.
  ///
  /// In en, this message translates to:
  /// **'Not set'**
  String get videoShaderPresetNotSet;

  /// No description provided for @shaderBrowserTitle.
  ///
  /// In en, this message translates to:
  /// **'Shaders'**
  String get shaderBrowserTitle;

  /// No description provided for @shaderBrowserNoShaders.
  ///
  /// In en, this message translates to:
  /// **'No shaders found'**
  String get shaderBrowserNoShaders;

  /// No description provided for @shaderBrowserError.
  ///
  /// In en, this message translates to:
  /// **'Error: {error}'**
  String shaderBrowserError(String error);

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
  /// **'Renderer backend'**
  String get videoBackendLabel;

  /// No description provided for @videoBackendAndroidLabel.
  ///
  /// In en, this message translates to:
  /// **'Android renderer backend'**
  String get videoBackendAndroidLabel;

  /// No description provided for @videoBackendWindowsLabel.
  ///
  /// In en, this message translates to:
  /// **'Windows renderer backend'**
  String get videoBackendWindowsLabel;

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

  /// No description provided for @videoBackendCurrent.
  ///
  /// In en, this message translates to:
  /// **'Current Backend: {backend}'**
  String videoBackendCurrent(String backend);

  /// No description provided for @windowsNativeOverlayTitle.
  ///
  /// In en, this message translates to:
  /// **'Windows Native Overlay (Experimental)'**
  String get windowsNativeOverlayTitle;

  /// No description provided for @windowsNativeOverlaySubtitle.
  ///
  /// In en, this message translates to:
  /// **'Bypasses Flutter compositor for perfect smoothness. Disables shaders and overlays UI behind the game.'**
  String get windowsNativeOverlaySubtitle;

  /// No description provided for @highPerformanceModeLabel.
  ///
  /// In en, this message translates to:
  /// **'High Performance Mode'**
  String get highPerformanceModeLabel;

  /// No description provided for @highPerformanceModeDescription.
  ///
  /// In en, this message translates to:
  /// **'Elevate process priority and optimize scheduler for smoother gameplay.'**
  String get highPerformanceModeDescription;

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

  /// No description provided for @menuTileViewer.
  ///
  /// In en, this message translates to:
  /// **'Tile Viewer'**
  String get menuTileViewer;

  /// No description provided for @tileViewerError.
  ///
  /// In en, this message translates to:
  /// **'Error: {error}'**
  String tileViewerError(String error);

  /// No description provided for @tileViewerRetry.
  ///
  /// In en, this message translates to:
  /// **'Retry'**
  String get tileViewerRetry;

  /// No description provided for @tileViewerSettings.
  ///
  /// In en, this message translates to:
  /// **'Tile Viewer Settings'**
  String get tileViewerSettings;

  /// No description provided for @tileViewerOverlays.
  ///
  /// In en, this message translates to:
  /// **'Overlays'**
  String get tileViewerOverlays;

  /// No description provided for @tileViewerShowGrid.
  ///
  /// In en, this message translates to:
  /// **'Show tile grid'**
  String get tileViewerShowGrid;

  /// No description provided for @tileViewerPalette.
  ///
  /// In en, this message translates to:
  /// **'Palette'**
  String get tileViewerPalette;

  /// No description provided for @tileViewerPaletteBg.
  ///
  /// In en, this message translates to:
  /// **'BG {index}'**
  String tileViewerPaletteBg(int index);

  /// No description provided for @tileViewerPaletteSprite.
  ///
  /// In en, this message translates to:
  /// **'Sprite {index}'**
  String tileViewerPaletteSprite(int index);

  /// No description provided for @tileViewerGrayscale.
  ///
  /// In en, this message translates to:
  /// **'Use grayscale palette'**
  String get tileViewerGrayscale;

  /// No description provided for @tileViewerSelectedTile.
  ///
  /// In en, this message translates to:
  /// **'Selected Tile'**
  String get tileViewerSelectedTile;

  /// No description provided for @tileViewerPatternTable.
  ///
  /// In en, this message translates to:
  /// **'Pattern Table'**
  String get tileViewerPatternTable;

  /// No description provided for @tileViewerTileIndex.
  ///
  /// In en, this message translates to:
  /// **'Tile Index'**
  String get tileViewerTileIndex;

  /// No description provided for @tileViewerChrAddress.
  ///
  /// In en, this message translates to:
  /// **'CHR Address'**
  String get tileViewerChrAddress;

  /// No description provided for @tileViewerClose.
  ///
  /// In en, this message translates to:
  /// **'Close'**
  String get tileViewerClose;

  /// No description provided for @tileViewerSource.
  ///
  /// In en, this message translates to:
  /// **'Source'**
  String get tileViewerSource;

  /// No description provided for @tileViewerSourcePpu.
  ///
  /// In en, this message translates to:
  /// **'PPU Memory'**
  String get tileViewerSourcePpu;

  /// No description provided for @tileViewerSourceChrRom.
  ///
  /// In en, this message translates to:
  /// **'CHR ROM'**
  String get tileViewerSourceChrRom;

  /// No description provided for @tileViewerSourceChrRam.
  ///
  /// In en, this message translates to:
  /// **'CHR RAM'**
  String get tileViewerSourceChrRam;

  /// No description provided for @tileViewerSourcePrgRom.
  ///
  /// In en, this message translates to:
  /// **'PRG ROM'**
  String get tileViewerSourcePrgRom;

  /// No description provided for @tileViewerAddress.
  ///
  /// In en, this message translates to:
  /// **'Address'**
  String get tileViewerAddress;

  /// No description provided for @tileViewerSize.
  ///
  /// In en, this message translates to:
  /// **'Size'**
  String get tileViewerSize;

  /// No description provided for @tileViewerColumns.
  ///
  /// In en, this message translates to:
  /// **'Cols'**
  String get tileViewerColumns;

  /// No description provided for @tileViewerRows.
  ///
  /// In en, this message translates to:
  /// **'Rows'**
  String get tileViewerRows;

  /// No description provided for @tileViewerLayout.
  ///
  /// In en, this message translates to:
  /// **'Layout'**
  String get tileViewerLayout;

  /// No description provided for @tileViewerLayoutNormal.
  ///
  /// In en, this message translates to:
  /// **'Normal'**
  String get tileViewerLayoutNormal;

  /// No description provided for @tileViewerLayout8x16.
  ///
  /// In en, this message translates to:
  /// **'8×16 Sprites'**
  String get tileViewerLayout8x16;

  /// No description provided for @tileViewerLayout16x16.
  ///
  /// In en, this message translates to:
  /// **'16×16 Sprites'**
  String get tileViewerLayout16x16;

  /// No description provided for @tileViewerBackground.
  ///
  /// In en, this message translates to:
  /// **'Background'**
  String get tileViewerBackground;

  /// No description provided for @tileViewerBgDefault.
  ///
  /// In en, this message translates to:
  /// **'Default'**
  String get tileViewerBgDefault;

  /// No description provided for @tileViewerBgTransparent.
  ///
  /// In en, this message translates to:
  /// **'Transparent'**
  String get tileViewerBgTransparent;

  /// No description provided for @tileViewerBgPalette.
  ///
  /// In en, this message translates to:
  /// **'Palette Color'**
  String get tileViewerBgPalette;

  /// No description provided for @tileViewerBgBlack.
  ///
  /// In en, this message translates to:
  /// **'Black'**
  String get tileViewerBgBlack;

  /// No description provided for @tileViewerBgWhite.
  ///
  /// In en, this message translates to:
  /// **'White'**
  String get tileViewerBgWhite;

  /// No description provided for @tileViewerBgMagenta.
  ///
  /// In en, this message translates to:
  /// **'Magenta'**
  String get tileViewerBgMagenta;

  /// No description provided for @tileViewerPresets.
  ///
  /// In en, this message translates to:
  /// **'Presets'**
  String get tileViewerPresets;

  /// No description provided for @tileViewerPresetPpu.
  ///
  /// In en, this message translates to:
  /// **'PPU'**
  String get tileViewerPresetPpu;

  /// No description provided for @tileViewerPresetChr.
  ///
  /// In en, this message translates to:
  /// **'CHR'**
  String get tileViewerPresetChr;

  /// No description provided for @tileViewerPresetRom.
  ///
  /// In en, this message translates to:
  /// **'ROM'**
  String get tileViewerPresetRom;

  /// No description provided for @tileViewerPresetBg.
  ///
  /// In en, this message translates to:
  /// **'BG'**
  String get tileViewerPresetBg;

  /// No description provided for @tileViewerPresetOam.
  ///
  /// In en, this message translates to:
  /// **'OAM'**
  String get tileViewerPresetOam;

  /// No description provided for @menuSpriteViewer.
  ///
  /// In en, this message translates to:
  /// **'Sprite Viewer'**
  String get menuSpriteViewer;

  /// No description provided for @menuPaletteViewer.
  ///
  /// In en, this message translates to:
  /// **'Palette Viewer'**
  String get menuPaletteViewer;

  /// No description provided for @paletteViewerPaletteRamTitle.
  ///
  /// In en, this message translates to:
  /// **'Palette RAM (32)'**
  String get paletteViewerPaletteRamTitle;

  /// No description provided for @paletteViewerSystemPaletteTitle.
  ///
  /// In en, this message translates to:
  /// **'System Palette (64)'**
  String get paletteViewerSystemPaletteTitle;

  /// No description provided for @paletteViewerSettingsTooltip.
  ///
  /// In en, this message translates to:
  /// **'Palette Viewer Settings'**
  String get paletteViewerSettingsTooltip;

  /// No description provided for @paletteViewerTooltipPaletteRam.
  ///
  /// In en, this message translates to:
  /// **'{addr} = 0x{value}'**
  String paletteViewerTooltipPaletteRam(String addr, String value);

  /// No description provided for @paletteViewerTooltipSystemIndex.
  ///
  /// In en, this message translates to:
  /// **'Index {index}'**
  String paletteViewerTooltipSystemIndex(int index);

  /// No description provided for @spriteViewerError.
  ///
  /// In en, this message translates to:
  /// **'Sprite viewer error: {error}'**
  String spriteViewerError(String error);

  /// No description provided for @spriteViewerSettingsTooltip.
  ///
  /// In en, this message translates to:
  /// **'Sprite Viewer Settings'**
  String get spriteViewerSettingsTooltip;

  /// No description provided for @spriteViewerShowGrid.
  ///
  /// In en, this message translates to:
  /// **'Show grid'**
  String get spriteViewerShowGrid;

  /// No description provided for @spriteViewerShowOutline.
  ///
  /// In en, this message translates to:
  /// **'Show outline around sprites'**
  String get spriteViewerShowOutline;

  /// No description provided for @spriteViewerShowOffscreenRegions.
  ///
  /// In en, this message translates to:
  /// **'Show offscreen regions'**
  String get spriteViewerShowOffscreenRegions;

  /// No description provided for @spriteViewerDimOffscreenSpritesGrid.
  ///
  /// In en, this message translates to:
  /// **'Dim offscreen sprites (grid)'**
  String get spriteViewerDimOffscreenSpritesGrid;

  /// No description provided for @spriteViewerShowListView.
  ///
  /// In en, this message translates to:
  /// **'Show list view'**
  String get spriteViewerShowListView;

  /// No description provided for @spriteViewerPanelSprites.
  ///
  /// In en, this message translates to:
  /// **'Sprites'**
  String get spriteViewerPanelSprites;

  /// No description provided for @spriteViewerPanelDataSource.
  ///
  /// In en, this message translates to:
  /// **'Data Source'**
  String get spriteViewerPanelDataSource;

  /// No description provided for @spriteViewerPanelSprite.
  ///
  /// In en, this message translates to:
  /// **'Sprite'**
  String get spriteViewerPanelSprite;

  /// No description provided for @spriteViewerPanelSelectedSprite.
  ///
  /// In en, this message translates to:
  /// **'Selected sprite'**
  String get spriteViewerPanelSelectedSprite;

  /// No description provided for @spriteViewerLabelMode.
  ///
  /// In en, this message translates to:
  /// **'Mode'**
  String get spriteViewerLabelMode;

  /// No description provided for @spriteViewerLabelPatternBase.
  ///
  /// In en, this message translates to:
  /// **'Pattern base'**
  String get spriteViewerLabelPatternBase;

  /// No description provided for @spriteViewerLabelThumbnailSize.
  ///
  /// In en, this message translates to:
  /// **'Thumbnail size'**
  String get spriteViewerLabelThumbnailSize;

  /// No description provided for @spriteViewerBgGray.
  ///
  /// In en, this message translates to:
  /// **'Gray'**
  String get spriteViewerBgGray;

  /// No description provided for @spriteViewerDataSourceSpriteRam.
  ///
  /// In en, this message translates to:
  /// **'Sprite RAM'**
  String get spriteViewerDataSourceSpriteRam;

  /// No description provided for @spriteViewerDataSourceCpuMemory.
  ///
  /// In en, this message translates to:
  /// **'CPU Memory'**
  String get spriteViewerDataSourceCpuMemory;

  /// No description provided for @spriteViewerTooltipTitle.
  ///
  /// In en, this message translates to:
  /// **'Sprite #{index}'**
  String spriteViewerTooltipTitle(int index);

  /// No description provided for @spriteViewerLabelIndex.
  ///
  /// In en, this message translates to:
  /// **'Index'**
  String get spriteViewerLabelIndex;

  /// No description provided for @spriteViewerLabelPos.
  ///
  /// In en, this message translates to:
  /// **'Pos'**
  String get spriteViewerLabelPos;

  /// No description provided for @spriteViewerLabelSize.
  ///
  /// In en, this message translates to:
  /// **'Size'**
  String get spriteViewerLabelSize;

  /// No description provided for @spriteViewerLabelTile.
  ///
  /// In en, this message translates to:
  /// **'Tile'**
  String get spriteViewerLabelTile;

  /// No description provided for @spriteViewerLabelTileAddr.
  ///
  /// In en, this message translates to:
  /// **'Tile addr'**
  String get spriteViewerLabelTileAddr;

  /// No description provided for @spriteViewerLabelPalette.
  ///
  /// In en, this message translates to:
  /// **'Palette'**
  String get spriteViewerLabelPalette;

  /// No description provided for @spriteViewerLabelPaletteAddr.
  ///
  /// In en, this message translates to:
  /// **'Palette addr'**
  String get spriteViewerLabelPaletteAddr;

  /// No description provided for @spriteViewerLabelFlip.
  ///
  /// In en, this message translates to:
  /// **'Flip'**
  String get spriteViewerLabelFlip;

  /// No description provided for @spriteViewerLabelPriority.
  ///
  /// In en, this message translates to:
  /// **'Priority'**
  String get spriteViewerLabelPriority;

  /// No description provided for @spriteViewerPriorityBehindBg.
  ///
  /// In en, this message translates to:
  /// **'Behind BG'**
  String get spriteViewerPriorityBehindBg;

  /// No description provided for @spriteViewerPriorityInFront.
  ///
  /// In en, this message translates to:
  /// **'In front'**
  String get spriteViewerPriorityInFront;

  /// No description provided for @spriteViewerLabelVisible.
  ///
  /// In en, this message translates to:
  /// **'Visible'**
  String get spriteViewerLabelVisible;

  /// No description provided for @spriteViewerValueYes.
  ///
  /// In en, this message translates to:
  /// **'Yes'**
  String get spriteViewerValueYes;

  /// No description provided for @spriteViewerValueNoOffscreen.
  ///
  /// In en, this message translates to:
  /// **'No (offscreen)'**
  String get spriteViewerValueNoOffscreen;

  /// No description provided for @spriteViewerVisibleStatusVisible.
  ///
  /// In en, this message translates to:
  /// **'Visible'**
  String get spriteViewerVisibleStatusVisible;

  /// No description provided for @spriteViewerVisibleStatusOffscreen.
  ///
  /// In en, this message translates to:
  /// **'Offscreen'**
  String get spriteViewerVisibleStatusOffscreen;

  /// No description provided for @longPressToClear.
  ///
  /// In en, this message translates to:
  /// **'Long press to clear'**
  String get longPressToClear;

  /// No description provided for @videoBackendD3D11.
  ///
  /// In en, this message translates to:
  /// **'D3D11 GPU (Zero-Copy)'**
  String get videoBackendD3D11;

  /// No description provided for @videoBackendSoftware.
  ///
  /// In en, this message translates to:
  /// **'Software CPU (Fallback)'**
  String get videoBackendSoftware;

  /// No description provided for @netplayBackToSetup.
  ///
  /// In en, this message translates to:
  /// **'Back to Setup'**
  String get netplayBackToSetup;

  /// No description provided for @netplayP2PMode.
  ///
  /// In en, this message translates to:
  /// **'P2P Mode'**
  String get netplayP2PMode;

  /// No description provided for @netplaySignalingServer.
  ///
  /// In en, this message translates to:
  /// **'Signaling Server'**
  String get netplaySignalingServer;

  /// No description provided for @netplayRelayServer.
  ///
  /// In en, this message translates to:
  /// **'Relay Server (Fallback)'**
  String get netplayRelayServer;

  /// No description provided for @netplayP2PRoomCode.
  ///
  /// In en, this message translates to:
  /// **'P2P Room Code'**
  String get netplayP2PRoomCode;

  /// No description provided for @netplayStartP2PSession.
  ///
  /// In en, this message translates to:
  /// **'Start P2P Session'**
  String get netplayStartP2PSession;

  /// No description provided for @netplayJoinP2PSession.
  ///
  /// In en, this message translates to:
  /// **'Join P2P Session'**
  String get netplayJoinP2PSession;

  /// No description provided for @netplayInvalidP2PServerAddr.
  ///
  /// In en, this message translates to:
  /// **'Invalid P2P server address'**
  String get netplayInvalidP2PServerAddr;

  /// No description provided for @netplayProceed.
  ///
  /// In en, this message translates to:
  /// **'Proceed'**
  String get netplayProceed;

  /// No description provided for @videoShaderParametersTitle.
  ///
  /// In en, this message translates to:
  /// **'Shader Parameters'**
  String get videoShaderParametersTitle;

  /// No description provided for @videoShaderParametersSubtitle.
  ///
  /// In en, this message translates to:
  /// **'Adjust shader parameters in real-time'**
  String get videoShaderParametersSubtitle;

  /// No description provided for @videoShaderParametersReset.
  ///
  /// In en, this message translates to:
  /// **'Reset Parameters'**
  String get videoShaderParametersReset;

  /// No description provided for @searchHint.
  ///
  /// In en, this message translates to:
  /// **'Search...'**
  String get searchHint;

  /// No description provided for @searchTooltip.
  ///
  /// In en, this message translates to:
  /// **'Search'**
  String get searchTooltip;

  /// No description provided for @noResults.
  ///
  /// In en, this message translates to:
  /// **'No matching parameters found'**
  String get noResults;

  /// No description provided for @errorFailedToCreateTexture.
  ///
  /// In en, this message translates to:
  /// **'Failed to create texture'**
  String get errorFailedToCreateTexture;
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
