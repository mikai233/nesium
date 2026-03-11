// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for German (`de`).
class AppLocalizationsDe extends AppLocalizations {
  AppLocalizationsDe([String locale = 'de']) : super(locale);

  @override
  String get settingsTitle => 'Einstellungen';

  @override
  String get settingsTabGeneral => 'Allgemein';

  @override
  String get settingsTabInput => 'Steuerung';

  @override
  String get settingsTabVideo => 'Video';

  @override
  String get settingsTabEmulation => 'Emulation';

  @override
  String get settingsTabServer => 'Server';

  @override
  String get settingsFloatingPreviewToggle => 'Schwebende Vorschau';

  @override
  String get settingsFloatingPreviewTooltip => 'Spielvorschau anzeigen';

  @override
  String get serverTitle => 'Netplay-Server';

  @override
  String get serverPortLabel => 'Port';

  @override
  String get serverStartButton => 'Starten Sie den Server';

  @override
  String get serverStopButton => 'Stoppen Sie den Server';

  @override
  String get serverStatusRunning => 'Läuft';

  @override
  String get serverStatusStopped => 'Angehalten';

  @override
  String serverClientCount(int count) {
    return 'Verbundene Clients: $count';
  }

  @override
  String serverStartFailed(String error) {
    return 'Serverstart fehlgeschlagen: $error';
  }

  @override
  String serverStopFailed(String error) {
    return 'Serverstopp fehlgeschlagen: $error';
  }

  @override
  String serverBindAddress(String address) {
    return 'Bindungsadresse: $address';
  }

  @override
  String serverQuicFingerprint(String fingerprint) {
    return 'QUIC-Fingerabdruck: $fingerprint';
  }

  @override
  String get generalTitle => 'Allgemein';

  @override
  String get themeLabel => 'Thema';

  @override
  String get themeSystem => 'System';

  @override
  String get themeLight => 'Licht';

  @override
  String get themeDark => 'Dunkel';

  @override
  String get languageLabel => 'Sprache';

  @override
  String get languageSystem => 'System';

  @override
  String get languageEnglish => 'Englisch';

  @override
  String get languageChineseSimplified => 'Vereinfachtes Chinesisch';

  @override
  String get inputTitle => 'Steuerung';

  @override
  String get turboTitle => 'Turbo';

  @override
  String get turboLinkPressRelease =>
      'Pressemitteilung/Pressemitteilung verlinken';

  @override
  String get inputDeviceLabel => 'Eingabegerät';

  @override
  String get inputDeviceKeyboard => 'Tastatur';

  @override
  String get inputDeviceGamepad => 'Gamepad';

  @override
  String get connectedGamepadsTitle => 'Verbundene Gamepads';

  @override
  String get connectedGamepadsNone => 'Keine Gamepads angeschlossen';

  @override
  String get webGamepadActivationHint =>
      'Web-Limit: DRÜCKEN SIE EINE JEDE TASTE auf Ihrem Gamepad, um es zu aktivieren.';

  @override
  String connectedGamepadsPort(int port) {
    return 'Spieler $port';
  }

  @override
  String get connectedGamepadsUnassigned => 'Nicht zugewiesen';

  @override
  String get inputDeviceVirtualController => 'Virtueller Controller';

  @override
  String get inputGamepadAssignmentLabel => 'Gamepad-Zuweisung';

  @override
  String get inputGamepadNone => 'Keine/Nicht zugewiesen';

  @override
  String get inputListening => 'Hören...';

  @override
  String inputDetected(String buttons) {
    return 'Erkannt: $buttons';
  }

  @override
  String get inputGamepadMappingLabel => 'Tastenbelegung';

  @override
  String get inputResetToDefault => 'Auf Standard zurücksetzen';

  @override
  String get inputButtonA => 'A';

  @override
  String get inputButtonB => 'B';

  @override
  String get inputButtonTurboA => 'Turbo A';

  @override
  String get inputButtonTurboB => 'Turbo B';

  @override
  String get inputButtonSelect => 'Select';

  @override
  String get inputButtonStart => 'Start';

  @override
  String get inputButtonUp => 'Oben';

  @override
  String get inputButtonDown => 'Unten';

  @override
  String get inputButtonLeft => 'Links';

  @override
  String get inputButtonRight => 'Rechts';

  @override
  String get inputButtonRewind => 'Zurückspulen';

  @override
  String get inputButtonFastForward => 'Schneller Vorlauf';

  @override
  String get inputButtonSaveState => 'Zustand speichern';

  @override
  String get inputButtonLoadState => 'Zustand laden';

  @override
  String get inputButtonPause => 'Pause';

  @override
  String get globalHotkeysTitle => 'Emulator-Hotkeys';

  @override
  String get gamepadHotkeysTitle => 'Gamepad-Hotkeys (Spieler 1)';

  @override
  String get inputPortLabel => 'Player konfigurieren';

  @override
  String get player1 => 'Spieler 1';

  @override
  String get player2 => 'Spieler 2';

  @override
  String get player3 => 'Spieler 3';

  @override
  String get player4 => 'Spieler 4';

  @override
  String get keyboardPresetLabel => 'Tastaturvoreinstellung';

  @override
  String get keyboardPresetNone => 'Keiner';

  @override
  String get keyboardPresetNesStandard => 'NES-Standard';

  @override
  String get keyboardPresetFightStick => 'Arcade Stick';

  @override
  String get keyboardPresetArcadeLayout => 'Arcade-Layout';

  @override
  String get keyboardPresetCustom => 'Benutzerdefiniert';

  @override
  String get customKeyBindingsTitle => 'Benutzerdefinierte Tastenkombinationen';

  @override
  String bindKeyTitle(String action) {
    return 'Binden Sie $action';
  }

  @override
  String get unassignedKey => 'Nicht zugewiesen';

  @override
  String get tipPressEscapeToClearBinding =>
      'Tipp: Drücken Sie Escape, um eine Bindung zu löschen.';

  @override
  String get keyboardActionUp => 'Oben';

  @override
  String get keyboardActionDown => 'Unten';

  @override
  String get keyboardActionLeft => 'Links';

  @override
  String get keyboardActionRight => 'Rechts';

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
  String get keyboardActionRewind => 'Zurückspulen';

  @override
  String get keyboardActionFastForward => 'Schneller Vorlauf';

  @override
  String get keyboardActionSaveState => 'Zustand speichern';

  @override
  String get keyboardActionLoadState => 'Zustand laden';

  @override
  String get keyboardActionPause => 'Pause';

  @override
  String get keyboardActionFullScreen => 'Vollbild';

  @override
  String inputBindingConflictCleared(String player, String action) {
    return '$player $action-Bindung gelöscht.';
  }

  @override
  String inputBindingConflictHint(String player, String action) {
    return '($player - $action)';
  }

  @override
  String inputBindingCapturedConflictHint(String player, String action) {
    return 'Besetzt durch $player - $action';
  }

  @override
  String get emulationTitle => 'Emulation';

  @override
  String get integerFpsTitle => 'Ganzzahliger FPS-Modus (60 Hz, NTSC)';

  @override
  String get integerFpsSubtitle =>
      'Reduziert Bildlaufruckeln auf 60-Hz-Displays. PAL wird später hinzugefügt.';

  @override
  String get showOverlayTitle => 'Status-Overlay anzeigen';

  @override
  String get showOverlaySubtitle =>
      'Anzeigen von Pause-/Rücklauf-/Schnellvorlauf-Anzeigen auf dem Bildschirm.';

  @override
  String get pauseInBackgroundTitle => 'Pause im Hintergrund';

  @override
  String get pauseInBackgroundSubtitle =>
      'Pausiert den Emulator automatisch, wenn die App nicht aktiv ist.';

  @override
  String get autoSaveEnabledTitle => 'Automatisch speichern';

  @override
  String get autoSaveEnabledSubtitle =>
      'Speichern Sie den Spielstatus regelmäßig in einem dedizierten Slot.';

  @override
  String get autoSaveIntervalTitle => 'Automatisches Speicherintervall';

  @override
  String autoSaveIntervalValue(int minutes) {
    return '$minutes Minuten';
  }

  @override
  String get fastForwardSpeedTitle => 'Schnelle Vorlaufgeschwindigkeit';

  @override
  String get fastForwardSpeedSubtitle =>
      'Maximale Geschwindigkeit bei aktivem Schnellvorlauf.';

  @override
  String fastForwardSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get quickSaveSlotTitle => 'Schnellspeicherplatz';

  @override
  String get quickSaveSlotSubtitle =>
      'Steckplatz, der von Verknüpfungen zum schnellen Speichern/Laden verwendet wird.';

  @override
  String quickSaveSlotValue(int index) {
    return 'Steckplatz $index';
  }

  @override
  String get rewindEnabledTitle => 'Zurückspulen';

  @override
  String get rewindEnabledSubtitle =>
      'Aktivieren Sie die Echtzeit-Rückspulfunktion.';

  @override
  String get rewindSecondsTitle => 'Rückspuldauer';

  @override
  String rewindSecondsValue(int seconds) {
    return '$seconds Sekunden';
  }

  @override
  String get rewindMinutesTitle => 'Rückspuldauer';

  @override
  String rewindMinutesValue(int minutes) {
    return '$minutes Minuten';
  }

  @override
  String get rewindSpeedTitle => 'Rückspulgeschwindigkeit';

  @override
  String get rewindSpeedSubtitle =>
      'Geschwindigkeit beim Zurückspulen ist aktiv.';

  @override
  String rewindSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get autoSlotLabel => 'Automatischer Slot';

  @override
  String get menuAutoSave => 'Automatisch speichern...';

  @override
  String get stateAutoSaved => 'Automatische Speicherung erstellt';

  @override
  String get virtualControlsTitle => 'Virtuelle Kontrollen';

  @override
  String get virtualControlsSwitchInputTip =>
      'Schalten Sie den Eingang auf „Virtual Controller“ um, um diese Einstellungen zu verwenden.';

  @override
  String get virtualControlsButtonSize => 'Knopfgröße';

  @override
  String get virtualControlsGap => 'Lücke';

  @override
  String get virtualControlsOpacity => 'Opazität';

  @override
  String get virtualControlsHitboxScale => 'Hitbox-Skala';

  @override
  String get virtualControlsHapticFeedback => 'Haptisches Feedback';

  @override
  String get virtualControlsDpadDeadzone => 'D-Pad-Totzone';

  @override
  String get virtualControlsDpadDeadzoneHelp =>
      'Totzone in der Mitte: Eine Berührung in der Nähe der Mitte löst keine Richtung aus.';

  @override
  String get virtualControlsDpadBoundaryDeadzone => 'Totzone der D-Pad-Grenze';

  @override
  String get virtualControlsDpadBoundaryDeadzoneHelp =>
      'Grenz-Totzone: Höhere Werte erschweren das Auslösen von Diagonalen und reduzieren so versehentliche Nachbardrücke.';

  @override
  String get virtualControlsReset => 'Layout zurücksetzen';

  @override
  String get virtualControlsDiscardChangesTitle =>
      'Änderungen rückgängig machen';

  @override
  String get virtualControlsDiscardChangesSubtitle =>
      'Zum zuletzt gespeicherten Layout zurückkehren';

  @override
  String get virtualControlsTurboFramesPerToggle =>
      'Turbo-Frames pro Umschalter';

  @override
  String get virtualControlsTurboOnFrames => 'Turbopressenrahmen';

  @override
  String get virtualControlsTurboOffFrames => 'Turbo-Release-Rahmen';

  @override
  String framesValue(int frames) {
    return '$frames-Frames';
  }

  @override
  String get tipAdjustButtonsInDrawer =>
      'Tipp: Passen Sie die Position/Größe der Tasten in der Schublade im Spiel an.';

  @override
  String get keyCapturePressKeyToBind => 'Zum Binden eine Taste drücken.';

  @override
  String keyCaptureCurrent(String key) {
    return 'Aktuell: $key';
  }

  @override
  String keyCaptureCaptured(String key) {
    return 'Erfasst: $key';
  }

  @override
  String get keyCapturePressEscToClear =>
      'Drücken Sie zum Löschen die Esc-Taste.';

  @override
  String get keyBindingsTitle => 'Schlüsselbindungen';

  @override
  String get cancel => 'Abbrechen';

  @override
  String get appName => 'Nesium';

  @override
  String get menuTooltip => 'Menü';

  @override
  String get menuSectionFile => 'Datei';

  @override
  String get menuSectionEmulation => 'Emulation';

  @override
  String get menuSectionSettings => 'Einstellungen';

  @override
  String get menuSectionWindows => 'Windows';

  @override
  String get menuSectionHelp => 'Hilfe';

  @override
  String get menuOpenRom => 'ROM öffnen...';

  @override
  String get menuReset => 'Zurücksetzen';

  @override
  String get menuPowerReset => 'Power-Reset';

  @override
  String get menuEject => 'Ausschalten';

  @override
  String get menuSaveState => 'Status speichern...';

  @override
  String get menuLoadState => 'Zustand laden...';

  @override
  String get menuPauseResume => 'Pause / Fortsetzen';

  @override
  String get menuNetplay => 'Netplay';

  @override
  String get netplayTransportLabel => 'Transport';

  @override
  String get netplayTransportAuto => 'Auto (QUIC → TCP)';

  @override
  String get netplayTransportUnknown => 'Unbekannt';

  @override
  String get netplayTransportTcp => 'TCP';

  @override
  String get netplayTransportQuic => 'QUIC';

  @override
  String get netplayUsingTcpFallback =>
      'QUIC ist bei Verwendung von TCP fehlgeschlagen';

  @override
  String get netplayStatusDisconnected => 'Getrennt';

  @override
  String get netplayStatusConnecting => 'Verbinden...';

  @override
  String get netplayStatusConnected => 'Verbunden (Warten auf Zimmer)';

  @override
  String get netplayStatusInRoom => 'Im Raum';

  @override
  String get netplayDisconnect => 'Trennen';

  @override
  String get netplayServerAddress => 'Serveradresse';

  @override
  String get netplayServerNameLabel => 'Servername (SNI)';

  @override
  String get netplayServerNameHint => 'localhost';

  @override
  String get netplayPlayerName => 'Spielername';

  @override
  String get netplayQuicFingerprintLabel =>
      'QUIC-Zertifikat-Fingerabdruck (optional)';

  @override
  String get netplayQuicFingerprintHint => 'base64url (43 Zeichen)';

  @override
  String get netplayQuicFingerprintHelper =>
      'Geben Sie dies ein, um angeheftetes QUIC zu verwenden. Lassen Sie das Feld leer, um die Systemvertrauensstellung (QUIC) oder den Fallback auf TCP zu verwenden.';

  @override
  String get netplayConnect => 'Dem Spiel beitreten';

  @override
  String get netplayJoinViaP2P => 'Treten Sie über P2P bei';

  @override
  String get netplayJoinGame => 'Dem Spiel beitreten';

  @override
  String get netplayCreateRoom => 'Raum schaffen';

  @override
  String get netplayJoinRoom => 'Dem Spiel beitreten';

  @override
  String get netplayAddressOrRoomCode => 'Raumcode oder Serveradresse';

  @override
  String get netplayHostingTitle => 'Hosting';

  @override
  String get netplayRoomCodeLabel => 'Ihr Zimmercode';

  @override
  String get netplayP2PEnabled => 'P2P-Modus';

  @override
  String get netplayDirectServerLabel => 'Serveradresse';

  @override
  String get netplayAdvancedSettings => 'Erweiterte Verbindungseinstellungen';

  @override
  String get netplayP2PServerLabel => 'P2P-Server';

  @override
  String get netplayRoomCode => 'Zimmercode';

  @override
  String get netplayRoleLabel => 'Rolle';

  @override
  String netplayPlayerIndex(int index) {
    return 'Spieler $index';
  }

  @override
  String get netplaySpectator => 'Zuschauer';

  @override
  String get netplayClientId => 'Kunden-ID';

  @override
  String get netplayPlayerListHeader => 'Spieler';

  @override
  String get netplayYouIndicator => '(Du)';

  @override
  String get netplayOrSeparator => 'ODER';

  @override
  String netplayConnectFailed(String error) {
    return 'Verbindung fehlgeschlagen: $error';
  }

  @override
  String netplayDisconnectFailed(String error) {
    return 'Verbindungstrennung fehlgeschlagen: $error';
  }

  @override
  String netplayCreateRoomFailed(String error) {
    return 'Raum erstellen fehlgeschlagen: $error';
  }

  @override
  String netplayJoinRoomFailed(String error) {
    return 'Dem Raum beitreten fehlgeschlagen: $error';
  }

  @override
  String netplaySwitchRoleFailed(String error) {
    return 'Rollenwechsel fehlgeschlagen: $error';
  }

  @override
  String get netplayInvalidRoomCode => 'Ungültiger Zimmercode';

  @override
  String get netplayRomBroadcasted => 'Netplay: ROM ins Zimmer übertragen';

  @override
  String get menuLoadTasMovie => 'TAS-Film laden...';

  @override
  String get menuPreferences => 'Präferenzen...';

  @override
  String get saveToExternalFile => 'In Datei speichern...';

  @override
  String get loadFromExternalFile => 'Aus Datei laden...';

  @override
  String get slotLabel => 'Slot';

  @override
  String get slotEmpty => 'Leer';

  @override
  String get slotHasData => 'Gespeichert';

  @override
  String stateSavedToSlot(int index) {
    return 'Status im Steckplatz $index gespeichert';
  }

  @override
  String stateLoadedFromSlot(int index) {
    return 'Status vom Steckplatz $index geladen';
  }

  @override
  String slotCleared(int index) {
    return 'Steckplatz $index gelöscht';
  }

  @override
  String get menuAbout => 'Über Nesium';

  @override
  String get menuDebugger => 'Debugger';

  @override
  String get menuTools => 'Werkzeuge';

  @override
  String get menuOpenDebuggerWindow => 'Öffnen Sie das Debugger-Fenster';

  @override
  String get menuOpenToolsWindow => 'Öffnen Sie das Tools-Fenster';

  @override
  String get menuInputMappingComingSoon => 'Eingabezuordnung (bald verfügbar)';

  @override
  String get menuLastError => 'Letzter Fehler';

  @override
  String get lastErrorDetailsAction => 'Einzelheiten';

  @override
  String get lastErrorDialogTitle => 'Letzter Fehler';

  @override
  String get lastErrorCopied => 'Kopiert';

  @override
  String get copy => 'Kopie';

  @override
  String get paste => 'Paste';

  @override
  String get windowDebuggerTitle => 'Nesium-Debugger';

  @override
  String get windowToolsTitle => 'Nesium-Tools';

  @override
  String get virtualControlsEditTitle =>
      'Bearbeiten Sie virtuelle Steuerelemente';

  @override
  String get virtualControlsEditSubtitleEnabled =>
      'Ziehen Sie zum Verschieben, ziehen Sie die Ecke zusammen oder ziehen Sie sie zusammen, um die Größe zu ändern';

  @override
  String get virtualControlsEditSubtitleDisabled =>
      'Aktivieren Sie die interaktive Anpassung';

  @override
  String get gridSnappingTitle => 'Gitterraster';

  @override
  String get gridSpacingLabel => 'Rasterabstand';

  @override
  String get debuggerPlaceholderBody =>
      'Platz reserviert für CPU/PPU-Monitore, Speicherbetrachter und OAM-Inspektoren. Dieselben Widgets können in einem Desktop-Seitenbereich oder einem mobilen Blatt vorhanden sein.';

  @override
  String get toolsPlaceholderBody =>
      'Aufnahme/Wiedergabe, Eingabezuordnung und Cheats können diese Widgets zwischen Desktop-Seitenfenstern und mobilen Unterseiten teilen.';

  @override
  String get actionLoadRom => 'ROM laden';

  @override
  String get actionResetNes => 'NES zurücksetzen';

  @override
  String get actionPowerResetNes => 'Power-Reset NES';

  @override
  String get actionEjectNes => 'Ausschalten';

  @override
  String get actionLoadPalette => 'Palette laden';

  @override
  String get videoResetToDefault => 'Auf Standard zurücksetzen';

  @override
  String get videoTitle => 'Video';

  @override
  String get videoFilterLabel => 'Videofilter';

  @override
  String get videoFilterCategoryCpu => 'CPU-Filter';

  @override
  String get videoFilterCategoryGpu => 'GPU-Filter (Shader)';

  @override
  String get videoFilterNone => 'Keine (1x)';

  @override
  String get videoFilterPrescale2x => '2x vorskalieren';

  @override
  String get videoFilterPrescale3x => '3x vorskalieren';

  @override
  String get videoFilterPrescale4x => '4x vorskalieren';

  @override
  String get videoFilterHq2x => 'HQ2x';

  @override
  String get videoFilterHq3x => 'HQ3x';

  @override
  String get videoFilterHq4x => 'HQ4x';

  @override
  String get videoFilter2xSai => '2xSaI';

  @override
  String get videoFilterSuper2xSai => 'Super 2xSaI';

  @override
  String get videoFilterSuperEagle => 'Super Eagle';

  @override
  String get videoFilterLcdGrid => 'LCD-Gitter (2x)';

  @override
  String get videoFilterScanlines => 'Scanlinien (2x)';

  @override
  String get videoFilterXbrz2x => 'xBRZ 2x';

  @override
  String get videoFilterXbrz3x => 'xBRZ 3x';

  @override
  String get videoFilterXbrz4x => 'xBRZ 4x';

  @override
  String get videoFilterXbrz5x => 'xBRZ 5x';

  @override
  String get videoFilterXbrz6x => 'xBRZ 6x';

  @override
  String get videoLcdGridStrengthLabel => 'LCD-Rasterstärke';

  @override
  String get videoScanlinesIntensityLabel => 'Scanline-Intensität';

  @override
  String get videoFilterNtscComposite => 'NTSC (Composite)';

  @override
  String get videoFilterNtscSvideo => 'NTSC (S-Video)';

  @override
  String get videoFilterNtscRgb => 'NTSC (RGB)';

  @override
  String get videoFilterNtscMonochrome => 'NTSC (Monochrom)';

  @override
  String get videoFilterNtscBisqwit2x => 'NTSC (Bisqwit) 2x';

  @override
  String get videoFilterNtscBisqwit4x => 'NTSC (Bisqwit) 4x';

  @override
  String get videoFilterNtscBisqwit8x => 'NTSC (Bisqwit) 8x';

  @override
  String get videoNtscAdvancedTitle => 'NTSC Advanced';

  @override
  String get videoNtscMergeFieldsLabel =>
      'Felder zusammenführen (Flimmern reduzieren)';

  @override
  String get videoNtscHueLabel => 'Farbton';

  @override
  String get videoNtscSaturationLabel => 'Sättigung';

  @override
  String get videoNtscContrastLabel => 'Kontrast';

  @override
  String get videoNtscBrightnessLabel => 'Helligkeit';

  @override
  String get videoNtscSharpnessLabel => 'Schärfe';

  @override
  String get videoNtscGammaLabel => 'Gamma';

  @override
  String get videoNtscResolutionLabel => 'Auflösung';

  @override
  String get videoNtscArtifactsLabel => 'Artefakte';

  @override
  String get videoNtscFringingLabel => 'Fransen';

  @override
  String get videoNtscBleedLabel => 'Farbauslaufen (Bleed)';

  @override
  String get videoNtscBisqwitSettingsTitle => 'NTSC-Einstellungen (Bisqwit)';

  @override
  String get videoNtscBisqwitYFilterLengthLabel =>
      'Y-Filter (horizontale Unschärfe)';

  @override
  String get videoNtscBisqwitIFilterLengthLabel => 'Ich filtere';

  @override
  String get videoNtscBisqwitQFilterLengthLabel => 'Q-Filter';

  @override
  String get videoIntegerScalingTitle => 'Ganzzahlige Skalierung';

  @override
  String get videoIntegerScalingSubtitle =>
      'Pixelgenaue Skalierung (reduziert Flimmern beim Scrollen).';

  @override
  String get videoFullScreenTitle => 'Vollbild';

  @override
  String get videoFullScreenSubtitle => 'Fenster-Vollbildstatus umschalten';

  @override
  String get videoScreenVerticalOffset => 'Vertikaler Bildschirmversatz';

  @override
  String get videoScreenVerticalOffsetPortraitOnly =>
      'Wirkt nur im Porträtmodus.';

  @override
  String get videoAspectRatio => 'Seitenverhältnis';

  @override
  String get videoAspectRatioSquare => '1:1 (quadratische Pixel)';

  @override
  String get videoAspectRatioNtsc => '4:3 (NTSC)';

  @override
  String get videoAspectRatioStretch => 'Strecken';

  @override
  String get videoShaderLibrashaderTitle => 'RetroArch-Shader';

  @override
  String get videoShaderLibrashaderSubtitle =>
      'Erfordert GLES3 + Hardware-Backend (AHB-Swapchain).';

  @override
  String get videoShaderLibrashaderSubtitleWindows =>
      'Erfordert D3D11-GPU-Backend.';

  @override
  String get videoShaderLibrashaderSubtitleApple => 'Erfordert Metal-Backend.';

  @override
  String get videoShaderLibrashaderSubtitleDisabled =>
      'Schalten Sie das Android-Backend zur Aktivierung auf Hardware um.';

  @override
  String get videoShaderLibrashaderSubtitleDisabledWindows =>
      'Wechseln Sie zum Aktivieren zum Windows-Backend auf die D3D11-GPU.';

  @override
  String get videoShaderPresetLabel => 'Voreinstellung (.slangp)';

  @override
  String get videoShaderPresetNotSet => 'Nicht festgelegt';

  @override
  String get shaderBrowserTitle => 'Shader';

  @override
  String get shaderBrowserNoShaders => 'Keine Shader gefunden';

  @override
  String shaderBrowserError(String error) {
    return 'Fehler: $error';
  }

  @override
  String get aboutTitle => 'Über Nesium';

  @override
  String get aboutLead =>
      'Nesium: Rust NES/FC-Emulator-Frontend basierend auf Nesium-Core.';

  @override
  String get aboutIntro =>
      'Dieses Flutter-Frontend verwendet den Rust-Kern für die Emulation wieder. Der Web-Build läuft im Browser über Flutter Web + Web Worker + WASM.';

  @override
  String get aboutLinksHeading => 'Links';

  @override
  String get aboutGitHubLabel => 'GitHub';

  @override
  String get aboutWebDemoLabel => 'Web-Demo';

  @override
  String get aboutComponentsHeading => 'Open-Source-Komponenten';

  @override
  String get aboutComponentsHint =>
      'Zum Öffnen tippen, zum Kopieren lange drücken.';

  @override
  String get aboutLicenseHeading => 'Lizenz';

  @override
  String get aboutLicenseBody =>
      'Nesium ist unter GPL-3.0 oder höher lizenziert. Siehe LICENSE.md im Repository-Stammverzeichnis.';

  @override
  String aboutLaunchFailed(String url) {
    return 'Konnte nicht gestartet werden: $url';
  }

  @override
  String get videoBackendLabel => 'Renderer-Backend';

  @override
  String get videoBackendAndroidLabel => 'Android-Renderer-Backend';

  @override
  String get videoBackendWindowsLabel => 'Windows-Renderer-Backend';

  @override
  String get videoBackendHardware => 'Hardware (AHardwareBuffer)';

  @override
  String get videoBackendUpload => 'Kompatibilität (CPU-Upload)';

  @override
  String get videoBackendRestartHint =>
      'Wird nach dem Neustart der App wirksam.';

  @override
  String videoBackendCurrent(String backend) {
    return 'Aktuelles Backend: $backend';
  }

  @override
  String get windowsNativeOverlayTitle =>
      'Natives Windows-Overlay (experimentell)';

  @override
  String get windowsNativeOverlaySubtitle =>
      'Umgeht den Flutter-Compositor für perfekte Glätte. Deaktiviert Shader und überlagert die Benutzeroberfläche hinter dem Spiel.';

  @override
  String get highPerformanceModeLabel => 'Hochleistungsmodus';

  @override
  String get highPerformanceModeDescription =>
      'Erhöhen Sie die Prozesspriorität und optimieren Sie den Zeitplaner für ein reibungsloseres Gameplay.';

  @override
  String get videoLowLatencyTitle => 'Video mit geringer Latenz';

  @override
  String get videoLowLatencySubtitle =>
      'Synchronisieren Sie Emulation und Renderer, um Jitter zu reduzieren. Wird nach dem Neustart der App wirksam.';

  @override
  String get paletteModeLabel => 'Palette';

  @override
  String get paletteModeBuiltin => 'Eingebaut';

  @override
  String get paletteModeCustom => 'Brauch…';

  @override
  String paletteModeCustomActive(String name) {
    return 'Benutzerdefiniert ($name)';
  }

  @override
  String get builtinPaletteLabel => 'Integrierte Palette';

  @override
  String get customPaletteLoadTitle => 'Palettendatei (.pal) laden…';

  @override
  String get customPaletteLoadSubtitle => '192 Byte (RGB) oder 256 Byte (RGBA)';

  @override
  String commandSucceeded(String label) {
    return '$label war erfolgreich';
  }

  @override
  String commandFailed(String label) {
    return '$label ist fehlgeschlagen';
  }

  @override
  String get snackPaused => 'Angehalten';

  @override
  String get snackResumed => 'Wieder aufgenommen';

  @override
  String snackPauseFailed(String error) {
    return 'Pause fehlgeschlagen: $error';
  }

  @override
  String get dialogOk => 'OK';

  @override
  String get debuggerNoRomTitle => 'Kein ROM läuft';

  @override
  String get debuggerNoRomSubtitle =>
      'Laden Sie ein ROM, um den Debug-Status anzuzeigen';

  @override
  String get debuggerCpuRegisters => 'CPU-Register';

  @override
  String get debuggerPpuState => 'PPU-Staat';

  @override
  String get debuggerCpuStatusTooltip =>
      'CPU-Statusregister (P)\nN: Negativ – gesetzt, wenn Ergebnisbit 7 gesetzt ist\nV: Überlauf – auf vorzeichenbehafteten Überlauf eingestellt\nB: Pause – gesetzt durch BRK-Anweisung\nD: Dezimal – BCD-Modus (auf NES ignoriert)\nI: Interrupt Disable – blockiert IRQ\nZ: Null – gesetzt, wenn das Ergebnis Null ist\nC: Carry – wird bei vorzeichenlosem Überlauf gesetzt\n\nGroßbuchstabe = gesetzt, Kleinbuchstabe = klar';

  @override
  String get debuggerPpuCtrlTooltip =>
      'PPU-Kontrollregister (\$2000)\nV: NMI-Aktivierung\nP: PPU-Master/Slave (unbenutzt)\nH: Sprite-Höhe (0=8x8, 1=8x16)\nB: Adresse der Hintergrundmustertabelle\nS: Adresse der Sprite-Mustertabelle\nI: VRAM-Adresserhöhung (0=1, 1=32)\nNN: Basisnametabellenadresse\n\nGroßbuchstabe = gesetzt, Kleinbuchstabe = klar';

  @override
  String get debuggerPpuMaskTooltip =>
      'PPU-Maskenregister (\$2001)\nBGR: Farbbetonungsbits\ns: Sprites anzeigen\nb: Hintergrund anzeigen\nM: Sprites in den 8 Pixeln ganz links anzeigen\nm: Hintergrund in den 8 Pixeln ganz links anzeigen\ng: Graustufen\n\nGroßbuchstabe = gesetzt, Kleinbuchstabe = klar';

  @override
  String get debuggerPpuStatusTooltip =>
      'PPU-Statusregister (\$2002)\nV: VBlank wurde gestartet\nS: Sprite 0 Treffer\nO: Sprite-Überlauf\n\nGroßbuchstabe = gesetzt, Kleinbuchstabe = klar';

  @override
  String get debuggerScanlineTooltip =>
      'Scanline-Nummern:\n0-239: Sichtbar (Rendern)\n240: Nach-Rendering (Leerlauf)\n241-260: VBlank (vertikale Ausblendung)\n-1: Vorrendern (Dummy-Scanline)';

  @override
  String get tilemapSettings => 'Einstellungen';

  @override
  String get tilemapOverlay => 'Überlagerung';

  @override
  String get tilemapDisplayMode => 'Anzeigemodus';

  @override
  String get tilemapDisplayModeDefault => 'Standard';

  @override
  String get tilemapDisplayModeGrayscale => 'Graustufen';

  @override
  String get tilemapDisplayModeAttributeView => 'Attributansicht';

  @override
  String get tilemapTileGrid => 'Kachelgitter (8×8)';

  @override
  String get tilemapAttrGrid => 'Attr-Gitter (16×16)';

  @override
  String get tilemapAttrGrid32 => 'Attr-Gitter (32×32)';

  @override
  String get tilemapNtBounds => 'NT-Grenzen';

  @override
  String get tilemapScrollOverlay => 'Scroll-Overlay';

  @override
  String get tilemapPanelDisplay => 'Anzeige';

  @override
  String get tilemapPanelTilemap => 'Tilemap';

  @override
  String get tilemapPanelSelectedTile => 'Ausgewählte Kachel';

  @override
  String get tilemapHidePanel => 'Panel ausblenden';

  @override
  String get tilemapShowPanel => 'Panel anzeigen';

  @override
  String get tilemapInfoSize => 'Größe';

  @override
  String get tilemapInfoSizePx => 'Größe (px)';

  @override
  String get tilemapInfoTilemapAddress => 'Tilemap-Adresse';

  @override
  String get tilemapInfoTilesetAddress => 'Tileset-Adresse';

  @override
  String get tilemapInfoMirroring => 'Spiegelung';

  @override
  String get tilemapInfoTileFormat => 'Kachelformat';

  @override
  String get tilemapInfoTileFormat2bpp => '2 bpp';

  @override
  String get tilemapMirroringHorizontal => 'Horizontal';

  @override
  String get tilemapMirroringVertical => 'Vertikal';

  @override
  String get tilemapMirroringFourScreen => 'Vier Bildschirme';

  @override
  String get tilemapMirroringSingleScreenLower => 'Einzelbildschirm (unten)';

  @override
  String get tilemapMirroringSingleScreenUpper => 'Einzelbildschirm (oben)';

  @override
  String get tilemapMirroringMapperControlled => 'Mapper-gesteuert';

  @override
  String get tilemapLabelColumnRow => 'Spalte, Zeile';

  @override
  String get tilemapLabelXY => 'X, Y';

  @override
  String get tilemapLabelSize => 'Größe';

  @override
  String get tilemapLabelTilemapAddress => 'Tilemap-Adresse';

  @override
  String get tilemapLabelTileIndex => 'Kachelindex';

  @override
  String get tilemapLabelTileAddressPpu => 'Kacheladresse (PPU)';

  @override
  String get tilemapLabelPaletteIndex => 'Palettenindex';

  @override
  String get tilemapLabelPaletteAddress => 'Palettenadresse';

  @override
  String get tilemapLabelAttributeAddress => 'Attributadresse';

  @override
  String get tilemapLabelAttributeData => 'Attributdaten';

  @override
  String get tilemapSelectedTileTilemap => 'Tilemap';

  @override
  String get tilemapSelectedTileTileIdx => 'Kachel-IDX';

  @override
  String get tilemapSelectedTileTilePpu => 'Kachel (PPU)';

  @override
  String get tilemapSelectedTilePalette => 'Palette';

  @override
  String get tilemapSelectedTileAttr => 'Attr';

  @override
  String get tilemapCapture => 'Erfassen';

  @override
  String get tilemapCaptureFrameStart => 'Rahmenanfang';

  @override
  String get tilemapCaptureVblankStart => 'VBlank Start';

  @override
  String get tilemapCaptureManual => 'Handbuch';

  @override
  String get tilemapScanline => 'Scanline';

  @override
  String get tilemapDot => 'Punkt';

  @override
  String tilemapError(String error) {
    return 'Fehler: $error';
  }

  @override
  String get tilemapRetry => 'Wiederholen';

  @override
  String get tilemapResetZoom => 'Zoom zurücksetzen';

  @override
  String get menuTilemapViewer => 'Tilemap-Viewer';

  @override
  String get menuTileViewer => 'Kachelbetrachter';

  @override
  String tileViewerError(String error) {
    return 'Fehler: $error';
  }

  @override
  String get tileViewerRetry => 'Wiederholen';

  @override
  String get tileViewerSettings => 'Kachel-Viewer-Einstellungen';

  @override
  String get tileViewerOverlays => 'Überlagerungen';

  @override
  String get tileViewerShowGrid => 'Kachelraster anzeigen';

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
  String get tileViewerGrayscale => 'Verwenden Sie eine Graustufenpalette';

  @override
  String get tileViewerSelectedTile => 'Ausgewählte Kachel';

  @override
  String get tileViewerPatternTable => 'Mustertabelle';

  @override
  String get tileViewerTileIndex => 'Kachelindex';

  @override
  String get tileViewerChrAddress => 'CHR-Adresse';

  @override
  String get tileViewerClose => 'Schließen';

  @override
  String get tileViewerSource => 'Quelle';

  @override
  String get tileViewerSourcePpu => 'PPU-Speicher';

  @override
  String get tileViewerSourceChrRom => 'CHR ROM';

  @override
  String get tileViewerSourceChrRam => 'CHR-RAM';

  @override
  String get tileViewerSourcePrgRom => 'PRG-ROM';

  @override
  String get tileViewerAddress => 'Adresse';

  @override
  String get tileViewerSize => 'Größe';

  @override
  String get tileViewerColumns => 'Spalten';

  @override
  String get tileViewerRows => 'Reihen';

  @override
  String get tileViewerLayout => 'Layout';

  @override
  String get tileViewerLayoutNormal => 'Normal';

  @override
  String get tileViewerLayout8x16 => '8×16 Sprites';

  @override
  String get tileViewerLayout16x16 => '16×16 Sprites';

  @override
  String get tileViewerBackground => 'Hintergrund';

  @override
  String get tileViewerBgDefault => 'Standard';

  @override
  String get tileViewerBgTransparent => 'Transparent';

  @override
  String get tileViewerBgPalette => 'Palettenfarbe';

  @override
  String get tileViewerBgBlack => 'Schwarz';

  @override
  String get tileViewerBgWhite => 'Weiß';

  @override
  String get tileViewerBgMagenta => 'Magenta';

  @override
  String get tileViewerPresets => 'Voreinstellungen';

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
  String get menuSpriteViewer => 'Sprite-Viewer';

  @override
  String get menuPaletteViewer => 'Palettenbetrachter';

  @override
  String get paletteViewerPaletteRamTitle => 'Paletten-RAM (32)';

  @override
  String get paletteViewerSystemPaletteTitle => 'Systempalette (64)';

  @override
  String get paletteViewerSettingsTooltip =>
      'Einstellungen für den Paletten-Viewer';

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
    return 'Sprite-Viewer-Fehler: $error';
  }

  @override
  String get spriteViewerSettingsTooltip => 'Sprite Viewer-Einstellungen';

  @override
  String get spriteViewerShowGrid => 'Raster anzeigen';

  @override
  String get spriteViewerShowOutline => 'Umrisse um Sprites anzeigen';

  @override
  String get spriteViewerShowOffscreenRegions => 'Offscreen-Bereiche anzeigen';

  @override
  String get spriteViewerDimOffscreenSpritesGrid =>
      'Offscreen-Sprites dimmen (Raster)';

  @override
  String get spriteViewerShowListView => 'Listenansicht anzeigen';

  @override
  String get spriteViewerPanelSprites => 'Sprites';

  @override
  String get spriteViewerPanelDataSource => 'Datenquelle';

  @override
  String get spriteViewerPanelSprite => 'Sprite';

  @override
  String get spriteViewerPanelSelectedSprite => 'Ausgewähltes Sprite';

  @override
  String get spriteViewerLabelMode => 'Modus';

  @override
  String get spriteViewerLabelPatternBase => 'Musterbasis';

  @override
  String get spriteViewerLabelThumbnailSize => 'Miniaturbildgröße';

  @override
  String get spriteViewerBgGray => 'Grau';

  @override
  String get spriteViewerDataSourceSpriteRam => 'Sprite-RAM';

  @override
  String get spriteViewerDataSourceCpuMemory => 'CPU-Speicher';

  @override
  String spriteViewerTooltipTitle(int index) {
    return 'Sprite #$index';
  }

  @override
  String get spriteViewerLabelIndex => 'Index';

  @override
  String get spriteViewerLabelPos => 'Pos';

  @override
  String get spriteViewerLabelSize => 'Größe';

  @override
  String get spriteViewerLabelTile => 'Fliese';

  @override
  String get spriteViewerLabelTileAddr => 'Kacheladr';

  @override
  String get spriteViewerLabelPalette => 'Palette';

  @override
  String get spriteViewerLabelPaletteAddr => 'Palettenadr';

  @override
  String get spriteViewerLabelFlip => 'Umdrehen';

  @override
  String get spriteViewerLabelPriority => 'Priorität';

  @override
  String get spriteViewerPriorityBehindBg => 'Behind BG';

  @override
  String get spriteViewerPriorityInFront => 'Vorne';

  @override
  String get spriteViewerLabelVisible => 'Sichtbar';

  @override
  String get spriteViewerValueYes => 'Ja';

  @override
  String get spriteViewerValueNoOffscreen => 'Nein (außerhalb des Bildschirms)';

  @override
  String get spriteViewerVisibleStatusVisible => 'Sichtbar';

  @override
  String get spriteViewerVisibleStatusOffscreen => 'Offscreen';

  @override
  String get longPressToClear => 'Zum Löschen lange drücken';

  @override
  String get videoBackendD3D11 => 'D3D11-GPU (Zero-Copy)';

  @override
  String get videoBackendSoftware => 'Software-CPU (Fallback)';

  @override
  String get netplayBackToSetup => 'Zurück zum Setup';

  @override
  String get netplayP2PMode => 'P2P-Modus';

  @override
  String get netplaySignalingServer => 'Signalisierungsserver';

  @override
  String get netplayRelayServer => 'Relay-Server (Fallback)';

  @override
  String get netplayP2PRoomCode => 'P2P-Raumcode';

  @override
  String get netplayStartP2PSession => 'Starten Sie eine P2P-Sitzung';

  @override
  String get netplayJoinP2PSession => 'Treten Sie der P2P-Sitzung bei';

  @override
  String get netplayInvalidP2PServerAddr => 'Ungültige P2P-Serveradresse';

  @override
  String get netplayProceed => 'Fortfahren';

  @override
  String get videoShaderParametersTitle => 'Shader-Parameter';

  @override
  String get videoShaderParametersSubtitle =>
      'Passen Sie die Shader-Parameter in Echtzeit an';

  @override
  String get videoShaderParametersReset => 'Parameter zurücksetzen';

  @override
  String get searchHint => 'Suchen...';

  @override
  String get searchTooltip => 'Suchen';

  @override
  String get noResults => 'Keine passenden Parameter gefunden';

  @override
  String get errorFailedToCreateTexture =>
      'Textur konnte nicht erstellt werden';

  @override
  String get languageJapanese => 'japanisch';

  @override
  String get languageSpanish => 'Spanisch';

  @override
  String get languagePortuguese => 'Portugiesisch';

  @override
  String get languageRussian => 'Russisch';

  @override
  String get languageFrench => 'Französisch';

  @override
  String get languageGerman => 'Deutsch';
}
