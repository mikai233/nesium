// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for French (`fr`).
class AppLocalizationsFr extends AppLocalizations {
  AppLocalizationsFr([String locale = 'fr']) : super(locale);

  @override
  String get settingsTitle => 'Paramètres';

  @override
  String get settingsTabGeneral => 'Général';

  @override
  String get settingsTabInput => 'Entrées';

  @override
  String get settingsTabVideo => 'Vidéo';

  @override
  String get settingsTabEmulation => 'Émulation';

  @override
  String get settingsTabServer => 'Serveur';

  @override
  String get settingsFloatingPreviewToggle => 'Aperçu flottant';

  @override
  String get settingsFloatingPreviewTooltip => 'Afficher l\'aperçu du jeu';

  @override
  String get serverTitle => 'Serveur Netplay';

  @override
  String get serverPortLabel => 'Port';

  @override
  String get serverStartButton => 'Démarrer le serveur';

  @override
  String get serverStopButton => 'Arrêter le serveur';

  @override
  String get serverStatusRunning => 'En cours d\'exécution';

  @override
  String get serverStatusStopped => 'Arrêté';

  @override
  String serverClientCount(int count) {
    return 'Clients connectés : $count';
  }

  @override
  String serverStartFailed(String error) {
    return 'Échec du démarrage du serveur : $error';
  }

  @override
  String serverStopFailed(String error) {
    return 'Échec de l\'arrêt du serveur : $error';
  }

  @override
  String serverBindAddress(String address) {
    return 'Adresse de liaison : $address';
  }

  @override
  String serverQuicFingerprint(String fingerprint) {
    return 'Empreinte digitale QUIC : $fingerprint';
  }

  @override
  String get generalTitle => 'Général';

  @override
  String get themeLabel => 'Thème';

  @override
  String get themeSystem => 'Système';

  @override
  String get themeLight => 'Lumière';

  @override
  String get themeDark => 'Sombre';

  @override
  String get languageLabel => 'Langue';

  @override
  String get languageSystem => 'Système';

  @override
  String get languageEnglish => 'Anglais';

  @override
  String get languageChineseSimplified => 'Chinois simplifié';

  @override
  String get inputTitle => 'Entrées';

  @override
  String get turboTitle => 'Turbo';

  @override
  String get turboLinkPressRelease => 'Lien presse/communiqué';

  @override
  String get inputDeviceLabel => 'Périphérique d\'entrée';

  @override
  String get inputDeviceKeyboard => 'Clavier';

  @override
  String get inputDeviceGamepad => 'Manette de jeu';

  @override
  String get connectedGamepadsTitle => 'Manettes connectées';

  @override
  String get connectedGamepadsNone => 'Aucune manette de jeu connectée';

  @override
  String get webGamepadActivationHint =>
      'Limite Web : APPUYEZ SUR N\'IMPORTE QUEL BOUTON de votre manette de jeu pour l\'activer.';

  @override
  String connectedGamepadsPort(int port) {
    return 'Joueur $port';
  }

  @override
  String get connectedGamepadsUnassigned => 'Non attribué';

  @override
  String get inputDeviceVirtualController => 'Contrôleur virtuel';

  @override
  String get inputGamepadAssignmentLabel => 'Affectation de la manette de jeu';

  @override
  String get inputGamepadNone => 'Aucun/Non attribué';

  @override
  String get inputListening => 'Écoute...';

  @override
  String inputDetected(String buttons) {
    return 'Détecté : $buttons';
  }

  @override
  String get inputGamepadMappingLabel => 'Mappage des boutons';

  @override
  String get inputResetToDefault => 'Réinitialiser aux valeurs par défaut';

  @override
  String get inputButtonA => 'UN';

  @override
  String get inputButtonB => 'B';

  @override
  String get inputButtonTurboA => 'Turbo-A';

  @override
  String get inputButtonTurboB => 'TurboB';

  @override
  String get inputButtonSelect => 'Select';

  @override
  String get inputButtonStart => 'Start';

  @override
  String get inputButtonUp => 'Haut';

  @override
  String get inputButtonDown => 'Bas';

  @override
  String get inputButtonLeft => 'Gauche';

  @override
  String get inputButtonRight => 'Droite';

  @override
  String get inputButtonRewind => 'Rembobiner';

  @override
  String get inputButtonFastForward => 'Avance rapide';

  @override
  String get inputButtonSaveState => 'Enregistrer l\'état';

  @override
  String get inputButtonLoadState => 'Charger l\'état';

  @override
  String get inputButtonPause => 'Pause';

  @override
  String get globalHotkeysTitle => 'Raccourcis clavier de l\'émulateur';

  @override
  String get gamepadHotkeysTitle =>
      'Raccourcis clavier de la manette de jeu (Joueur 1)';

  @override
  String get inputPortLabel => 'Configurer le lecteur';

  @override
  String get player1 => 'Joueur 1';

  @override
  String get player2 => 'Joueur 2';

  @override
  String get player3 => 'Joueur 3';

  @override
  String get player4 => 'Joueur 4';

  @override
  String get keyboardPresetLabel => 'Préréglage du clavier';

  @override
  String get keyboardPresetNone => 'Aucun';

  @override
  String get keyboardPresetNesStandard => 'Norme NES';

  @override
  String get keyboardPresetFightStick => 'Arcade Stick';

  @override
  String get keyboardPresetArcadeLayout => 'Disposition Arcade';

  @override
  String get keyboardPresetCustom => 'Personnalisé';

  @override
  String get customKeyBindingsTitle => 'Raccourcis de touches personnalisés';

  @override
  String bindKeyTitle(String action) {
    return 'Lier $action';
  }

  @override
  String get unassignedKey => 'Non attribué';

  @override
  String get tipPressEscapeToClearBinding =>
      'Astuce : appuyez sur Échap pour effacer une liaison.';

  @override
  String get keyboardActionUp => 'Haut';

  @override
  String get keyboardActionDown => 'Bas';

  @override
  String get keyboardActionLeft => 'Gauche';

  @override
  String get keyboardActionRight => 'Droite';

  @override
  String get keyboardActionA => 'UN';

  @override
  String get keyboardActionB => 'B';

  @override
  String get keyboardActionSelect => 'Select';

  @override
  String get keyboardActionStart => 'Start';

  @override
  String get keyboardActionTurboA => 'Turbo-A';

  @override
  String get keyboardActionTurboB => 'TurboB';

  @override
  String get keyboardActionRewind => 'Rembobiner';

  @override
  String get keyboardActionFastForward => 'Avance rapide';

  @override
  String get keyboardActionSaveState => 'Enregistrer l\'état';

  @override
  String get keyboardActionLoadState => 'Charger l\'état';

  @override
  String get keyboardActionPause => 'Pause';

  @override
  String get keyboardActionFullScreen => 'Plein écran';

  @override
  String inputBindingConflictCleared(String player, String action) {
    return 'La liaison $player $action a été supprimée.';
  }

  @override
  String inputBindingConflictHint(String player, String action) {
    return '($player - $action)';
  }

  @override
  String inputBindingCapturedConflictHint(String player, String action) {
    return 'Occupé par $player - $action';
  }

  @override
  String get emulationTitle => 'Émulation';

  @override
  String get integerFpsTitle => 'Mode FPS entier (60 Hz, NTSC)';

  @override
  String get integerFpsSubtitle =>
      'Réduit les saccades de défilement sur les écrans 60 Hz. PAL sera ajouté plus tard.';

  @override
  String get showOverlayTitle => 'Afficher la superposition d\'état';

  @override
  String get showOverlaySubtitle =>
      'Afficher les indicateurs pause/rembobinage/avance rapide à l’écran.';

  @override
  String get pauseInBackgroundTitle => 'Pause en arrière-plan';

  @override
  String get pauseInBackgroundSubtitle =>
      'Met automatiquement l\'émulateur en pause lorsque l\'application n\'est pas active.';

  @override
  String get autoSaveEnabledTitle => 'Sauvegarde automatique';

  @override
  String get autoSaveEnabledSubtitle =>
      'Enregistrez périodiquement l’état du jeu dans un emplacement dédié.';

  @override
  String get autoSaveIntervalTitle =>
      'Intervalle d\'enregistrement automatique';

  @override
  String autoSaveIntervalValue(int minutes) {
    return '$minutes minutes';
  }

  @override
  String get fastForwardSpeedTitle => 'Vitesse d\'avance rapide';

  @override
  String get fastForwardSpeedSubtitle =>
      'Vitesse maximale lorsque l’avance rapide est active.';

  @override
  String fastForwardSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get quickSaveSlotTitle => 'Emplacement de sauvegarde rapide';

  @override
  String get quickSaveSlotSubtitle =>
      'Emplacement utilisé par les raccourcis de sauvegarde/chargement rapide.';

  @override
  String quickSaveSlotValue(int index) {
    return 'Emplacement $index';
  }

  @override
  String get rewindEnabledTitle => 'Rembobiner';

  @override
  String get rewindEnabledSubtitle =>
      'Activez la fonctionnalité de rembobinage en temps réel.';

  @override
  String get rewindSecondsTitle => 'Durée de rembobinage';

  @override
  String rewindSecondsValue(int seconds) {
    return '$seconds secondes';
  }

  @override
  String get rewindMinutesTitle => 'Durée de rembobinage';

  @override
  String rewindMinutesValue(int minutes) {
    return '$minutes minutes';
  }

  @override
  String get rewindSpeedTitle => 'Vitesse de rembobinage';

  @override
  String get rewindSpeedSubtitle =>
      'La vitesse pendant le rembobinage est active.';

  @override
  String rewindSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get autoSlotLabel => 'Emplacement automatique';

  @override
  String get menuAutoSave => 'Sauvegarde automatique...';

  @override
  String get stateAutoSaved => 'Sauvegarde automatique créée';

  @override
  String get virtualControlsTitle => 'Contrôles virtuels';

  @override
  String get virtualControlsSwitchInputTip =>
      'Basculez l\'entrée sur \"Contrôleur virtuel\" pour utiliser ces paramètres.';

  @override
  String get virtualControlsButtonSize => 'Taille du bouton';

  @override
  String get virtualControlsGap => 'Écart';

  @override
  String get virtualControlsOpacity => 'Opacité';

  @override
  String get virtualControlsHitboxScale => 'Échelle de la hitbox';

  @override
  String get virtualControlsHapticFeedback => 'Retour haptique';

  @override
  String get virtualControlsDpadDeadzone => 'Zone morte du D-pad';

  @override
  String get virtualControlsDpadDeadzoneHelp =>
      'Zone morte centrale : toucher près du centre ne déclenchera aucune direction.';

  @override
  String get virtualControlsDpadBoundaryDeadzone =>
      'Zone morte de la limite du D-pad';

  @override
  String get virtualControlsDpadBoundaryDeadzoneHelp =>
      'Zone morte de limite : des valeurs plus élevées rendent les diagonales plus difficiles à déclencher, réduisant ainsi les pressions accidentelles des voisins.';

  @override
  String get virtualControlsReset => 'Réinitialiser la mise en page';

  @override
  String get virtualControlsDiscardChangesTitle => 'Annuler les modifications';

  @override
  String get virtualControlsDiscardChangesSubtitle =>
      'Revenir à la dernière mise en page enregistrée';

  @override
  String get virtualControlsTurboFramesPerToggle => 'Images Turbo par bascule';

  @override
  String get virtualControlsTurboOnFrames => 'Cadres de presse turbo';

  @override
  String get virtualControlsTurboOffFrames => 'Cadres de libération turbo';

  @override
  String framesValue(int frames) {
    return 'Cadres $frames';
  }

  @override
  String get tipAdjustButtonsInDrawer =>
      'Astuce : ajustez la position/la taille des boutons depuis le tiroir du jeu.';

  @override
  String get keyCapturePressKeyToBind => 'Appuyez sur une touche pour lier.';

  @override
  String keyCaptureCurrent(String key) {
    return 'Actuel : $key';
  }

  @override
  String keyCaptureCaptured(String key) {
    return 'Capturé : $key';
  }

  @override
  String get keyCapturePressEscToClear => 'Appuyez sur Échap pour effacer.';

  @override
  String get keyBindingsTitle => 'Raccourcis clavier';

  @override
  String get cancel => 'Annuler';

  @override
  String get appName => 'Nesium';

  @override
  String get menuTooltip => 'Menu';

  @override
  String get menuSectionFile => 'Fichier';

  @override
  String get menuSectionEmulation => 'Émulation';

  @override
  String get menuSectionSettings => 'Paramètres';

  @override
  String get menuSectionWindows => 'Fenêtres';

  @override
  String get menuSectionHelp => 'Aide';

  @override
  String get menuOpenRom => 'Ouvrir la ROM...';

  @override
  String get menuReset => 'Réinitialiser';

  @override
  String get menuPowerReset => 'Réinitialisation de l\'alimentation';

  @override
  String get menuEject => 'Mise hors tension';

  @override
  String get menuSaveState => 'Enregistrer l\'état...';

  @override
  String get menuLoadState => 'État de chargement...';

  @override
  String get menuPauseResume => 'Pause/Reprise';

  @override
  String get menuNetplay => 'Jeu en réseau';

  @override
  String get netplayTransportLabel => 'Transport';

  @override
  String get netplayTransportAuto => 'Automatique (QUIC → TCP)';

  @override
  String get netplayTransportUnknown => 'Inconnu';

  @override
  String get netplayTransportTcp => 'TCP';

  @override
  String get netplayTransportQuic => 'QUIC';

  @override
  String get netplayUsingTcpFallback => 'QUIC a échoué, en utilisant TCP';

  @override
  String get netplayStatusDisconnected => 'Déconnecté';

  @override
  String get netplayStatusConnecting => 'De liaison...';

  @override
  String get netplayStatusConnected => 'Connecté (en attente de chambre)';

  @override
  String get netplayStatusInRoom => 'Dans la salle';

  @override
  String get netplayDisconnect => 'Déconnecter';

  @override
  String get netplayServerAddress => 'Adresse du serveur';

  @override
  String get netplayServerNameLabel => 'Nom du serveur (SNI)';

  @override
  String get netplayServerNameHint => 'localhost';

  @override
  String get netplayPlayerName => 'Nom du joueur';

  @override
  String get netplayQuicFingerprintLabel =>
      'Empreinte digitale du certificat QUIC (facultatif)';

  @override
  String get netplayQuicFingerprintHint => 'base64url (43 caractères)';

  @override
  String get netplayQuicFingerprintHelper =>
      'Entrez ceci pour utiliser QUIC épinglé. Laissez vide pour utiliser la confiance du système (QUIC) ou revenir à TCP.';

  @override
  String get netplayConnect => 'Rejoindre le jeu';

  @override
  String get netplayJoinViaP2P => 'Rejoignez via P2P';

  @override
  String get netplayJoinGame => 'Rejoindre le jeu';

  @override
  String get netplayCreateRoom => 'Créer une salle';

  @override
  String get netplayJoinRoom => 'Rejoindre le jeu';

  @override
  String get netplayAddressOrRoomCode => 'Code de salle ou adresse du serveur';

  @override
  String get netplayHostingTitle => 'Hébergement';

  @override
  String get netplayRoomCodeLabel => 'Votre code de chambre';

  @override
  String get netplayP2PEnabled => 'Mode P2P';

  @override
  String get netplayDirectServerLabel => 'Adresse du serveur';

  @override
  String get netplayAdvancedSettings => 'Paramètres de connexion avancés';

  @override
  String get netplayP2PServerLabel => 'Serveur P2P';

  @override
  String get netplayRoomCode => 'Code de la chambre';

  @override
  String get netplayRoleLabel => 'Rôle';

  @override
  String netplayPlayerIndex(int index) {
    return 'Joueur $index';
  }

  @override
  String get netplaySpectator => 'Spectateur';

  @override
  String get netplayClientId => 'Identifiant client';

  @override
  String get netplayPlayerListHeader => 'Joueurs';

  @override
  String get netplayYouIndicator => '(Toi)';

  @override
  String get netplayOrSeparator => 'OU';

  @override
  String netplayConnectFailed(String error) {
    return 'Échec de la connexion : $error';
  }

  @override
  String netplayDisconnectFailed(String error) {
    return 'Échec de la déconnexion : $error';
  }

  @override
  String netplayCreateRoomFailed(String error) {
    return 'Échec de la création de la salle : $error';
  }

  @override
  String netplayJoinRoomFailed(String error) {
    return 'Échec de la connexion au salon : $error';
  }

  @override
  String netplaySwitchRoleFailed(String error) {
    return 'Échec du changement de rôle : $error';
  }

  @override
  String get netplayInvalidRoomCode => 'Code de chambre invalide';

  @override
  String get netplayRomBroadcasted => 'Netplay : ROM diffusée dans la salle';

  @override
  String get menuLoadTasMovie => 'Charger le film TAS...';

  @override
  String get menuPreferences => 'Préférences...';

  @override
  String get saveToExternalFile => 'Enregistrer dans un fichier...';

  @override
  String get loadFromExternalFile => 'Charger à partir du fichier...';

  @override
  String get slotLabel => 'Fente';

  @override
  String get slotEmpty => 'Vide';

  @override
  String get slotHasData => 'Enregistré';

  @override
  String stateSavedToSlot(int index) {
    return 'État enregistré dans l\'emplacement $index';
  }

  @override
  String stateLoadedFromSlot(int index) {
    return 'État chargé depuis l\'emplacement $index';
  }

  @override
  String slotCleared(int index) {
    return 'Emplacement $index effacé';
  }

  @override
  String get menuAbout => 'À propos';

  @override
  String get menuDebugger => 'Débogueur';

  @override
  String get menuTools => 'Outils';

  @override
  String get menuOpenDebuggerWindow => 'Ouvrir la fenêtre du débogueur';

  @override
  String get menuOpenToolsWindow => 'Ouvrir la fenêtre Outils';

  @override
  String get menuInputMappingComingSoon => 'Mappage d\'entrée (à venir)';

  @override
  String get menuLastError => 'Dernière erreur';

  @override
  String get lastErrorDetailsAction => 'Détails';

  @override
  String get lastErrorDialogTitle => 'Dernière erreur';

  @override
  String get lastErrorCopied => 'Copié';

  @override
  String get copy => 'Copie';

  @override
  String get paste => 'Coller';

  @override
  String get windowDebuggerTitle => 'Débogueur Nesium';

  @override
  String get windowToolsTitle => 'Outils Nesium';

  @override
  String get virtualControlsEditTitle => 'Modifier les contrôles virtuels';

  @override
  String get virtualControlsEditSubtitleEnabled =>
      'Faites glisser pour déplacer, pincez ou faites glisser le coin pour redimensionner';

  @override
  String get virtualControlsEditSubtitleDisabled =>
      'Activer l\'ajustement interactif';

  @override
  String get gridSnappingTitle => 'Capture de grille';

  @override
  String get gridSpacingLabel => 'Espacement des grilles';

  @override
  String get debuggerPlaceholderBody =>
      'Espace réservé aux moniteurs CPU/PPU, aux visualiseurs de mémoire et aux inspecteurs OAM. Les mêmes widgets peuvent résider dans un panneau latéral de bureau ou dans une feuille mobile.';

  @override
  String get toolsPlaceholderBody =>
      'L\'enregistrement/lecture, le mappage des entrées et les astuces peuvent partager ces widgets entre les volets latéraux du bureau et les feuilles de fond mobiles.';

  @override
  String get actionLoadRom => 'Charger la ROM';

  @override
  String get actionResetNes => 'Réinitialiser la NES';

  @override
  String get actionPowerResetNes => 'Réinitialisation de l\'alimentation NES';

  @override
  String get actionEjectNes => 'Mise hors tension';

  @override
  String get actionLoadPalette => 'Charger la palette';

  @override
  String get videoResetToDefault => 'Réinitialiser aux valeurs par défaut';

  @override
  String get videoTitle => 'Vidéo';

  @override
  String get videoFilterLabel => 'Filtre vidéo';

  @override
  String get videoFilterCategoryCpu => 'Filtres du processeur';

  @override
  String get videoFilterCategoryGpu => 'Filtres GPU (Shaders)';

  @override
  String get videoFilterNone => 'Aucun (1x)';

  @override
  String get videoFilterPrescale2x => 'Pré-échelle 2x';

  @override
  String get videoFilterPrescale3x => 'Pré-échelle 3x';

  @override
  String get videoFilterPrescale4x => 'Pré-échelle 4x';

  @override
  String get videoFilterHq2x => 'QG2x';

  @override
  String get videoFilterHq3x => 'QG3x';

  @override
  String get videoFilterHq4x => 'QG4x';

  @override
  String get videoFilter2xSai => '2xSaI';

  @override
  String get videoFilterSuper2xSai => 'Super 2xSaI';

  @override
  String get videoFilterSuperEagle => 'Super Aigle';

  @override
  String get videoFilterLcdGrid => 'Grille LCD (2x)';

  @override
  String get videoFilterScanlines => 'Lignes de balayage (2x)';

  @override
  String get videoFilterXbrz2x => 'xBRZ2x';

  @override
  String get videoFilterXbrz3x => 'xBRZ 3x';

  @override
  String get videoFilterXbrz4x => 'xBRZ4x';

  @override
  String get videoFilterXbrz5x => 'xBRZ5x';

  @override
  String get videoFilterXbrz6x => 'xBRZ6x';

  @override
  String get videoLcdGridStrengthLabel => 'Force de la grille LCD';

  @override
  String get videoScanlinesIntensityLabel =>
      'Intensité de la ligne de balayage';

  @override
  String get videoFilterNtscComposite => 'NTSC (Composite)';

  @override
  String get videoFilterNtscSvideo => 'NTSC (S-Vidéo)';

  @override
  String get videoFilterNtscRgb => 'NTSC (RVB)';

  @override
  String get videoFilterNtscMonochrome => 'NTSC (Monochrome)';

  @override
  String get videoFilterNtscBisqwit2x => 'NTSC (Bisqwit) 2x';

  @override
  String get videoFilterNtscBisqwit4x => 'NTSC (Bisqwit) 4x';

  @override
  String get videoFilterNtscBisqwit8x => 'NTSC (Bisqwit) 8x';

  @override
  String get videoNtscAdvancedTitle => 'NTSC Avancé';

  @override
  String get videoNtscMergeFieldsLabel =>
      'Fusionner les champs (réduire le scintillement)';

  @override
  String get videoNtscHueLabel => 'Teinte';

  @override
  String get videoNtscSaturationLabel => 'Saturation';

  @override
  String get videoNtscContrastLabel => 'Contraste';

  @override
  String get videoNtscBrightnessLabel => 'Luminosité';

  @override
  String get videoNtscSharpnessLabel => 'Acuité';

  @override
  String get videoNtscGammaLabel => 'Gamma';

  @override
  String get videoNtscResolutionLabel => 'Résolution';

  @override
  String get videoNtscArtifactsLabel => 'Artefacts';

  @override
  String get videoNtscFringingLabel => 'Franges';

  @override
  String get videoNtscBleedLabel => 'Franges de couleur (Bleed)';

  @override
  String get videoNtscBisqwitSettingsTitle => 'Paramètres NTSC (Bisqwit)';

  @override
  String get videoNtscBisqwitYFilterLengthLabel => 'Filtre Y (flou horizontal)';

  @override
  String get videoNtscBisqwitIFilterLengthLabel => 'Je filtre';

  @override
  String get videoNtscBisqwitQFilterLengthLabel => 'Filtre Q';

  @override
  String get videoIntegerScalingTitle => 'Mise à l\'échelle entière';

  @override
  String get videoIntegerScalingSubtitle =>
      'Mise à l\'échelle parfaite au pixel près (réduit les reflets lors du défilement).';

  @override
  String get videoFullScreenTitle => 'Plein écran';

  @override
  String get videoFullScreenSubtitle =>
      'Basculer l\'état plein écran de la fenêtre';

  @override
  String get videoScreenVerticalOffset => 'Décalage vertical de l\'écran';

  @override
  String get videoScreenVerticalOffsetPortraitOnly =>
      'Ne prend effet qu\'en mode portrait.';

  @override
  String get videoAspectRatio => 'Rapport hauteur/largeur';

  @override
  String get videoAspectRatioSquare => '1:1 (pixels carrés)';

  @override
  String get videoAspectRatioNtsc => '4:3 (NTSC)';

  @override
  String get videoAspectRatioStretch => 'Extensible';

  @override
  String get videoShaderLibrashaderTitle => 'Shaders RétroArch';

  @override
  String get videoShaderLibrashaderSubtitle =>
      'Nécessite GLES3 + Backend matériel (swapchain AHB).';

  @override
  String get videoShaderLibrashaderSubtitleWindows =>
      'Nécessite le back-end GPU D3D11.';

  @override
  String get videoShaderLibrashaderSubtitleApple =>
      'Nécessite un back-end Metal.';

  @override
  String get videoShaderLibrashaderSubtitleDisabled =>
      'Basculez le backend Android sur Matériel pour l’activer.';

  @override
  String get videoShaderLibrashaderSubtitleDisabledWindows =>
      'Basculez le backend Windows vers le GPU D3D11 pour l\'activer.';

  @override
  String get videoShaderPresetLabel => 'Préréglage (.slangp)';

  @override
  String get videoShaderPresetNotSet => 'Non défini';

  @override
  String get shaderBrowserTitle => 'Shaders';

  @override
  String get shaderBrowserNoShaders => 'Aucun shader trouvé';

  @override
  String shaderBrowserError(String error) {
    return 'Erreur : $error';
  }

  @override
  String get aboutTitle => 'À propos de Nesium';

  @override
  String get aboutLead =>
      'Nesium : interface d\'émulateur Rust NES/FC construite sur un noyau nesium.';

  @override
  String get aboutIntro =>
      'Cette interface Flutter réutilise le noyau Rust pour l\'émulation. La version Web s\'exécute dans le navigateur via Flutter Web + Web Worker + WASM.';

  @override
  String get aboutLinksHeading => 'Links';

  @override
  String get aboutGitHubLabel => 'GitHub';

  @override
  String get aboutWebDemoLabel => 'Démo Web';

  @override
  String get aboutComponentsHeading => 'Composants open source';

  @override
  String get aboutComponentsHint =>
      'Appuyez pour ouvrir, appuyez longuement pour copier.';

  @override
  String get aboutLicenseHeading => 'Licence';

  @override
  String get aboutLicenseBody =>
      'Nesium est sous licence GPL-3.0 ou version ultérieure. Voir LICENSE.md à la racine du référentiel.';

  @override
  String aboutLaunchFailed(String url) {
    return 'Impossible de lancer : $url';
  }

  @override
  String get videoBackendLabel => 'Moteur de rendu';

  @override
  String get videoBackendAndroidLabel => 'Backend du moteur de rendu Android';

  @override
  String get videoBackendWindowsLabel => 'Backend du moteur de rendu Windows';

  @override
  String get videoBackendHardware => 'Matériel (AHardwareBuffer)';

  @override
  String get videoBackendUpload => 'Compatibilité (téléchargement CPU)';

  @override
  String get videoBackendRestartHint =>
      'Prend effet après le redémarrage de l\'application.';

  @override
  String videoBackendCurrent(String backend) {
    return 'Backend actuel : $backend';
  }

  @override
  String get windowsNativeOverlayTitle =>
      'Superposition native Windows (expérimentale)';

  @override
  String get windowsNativeOverlaySubtitle =>
      'Contourne le compositeur Flutter pour une douceur parfaite. Désactive les shaders et superpose l\'interface utilisateur derrière le jeu.';

  @override
  String get highPerformanceModeLabel => 'Mode hautes performances';

  @override
  String get highPerformanceModeDescription =>
      'Élevez la priorité des processus et optimisez le planificateur pour un gameplay plus fluide.';

  @override
  String get videoLowLatencyTitle => 'Vidéo à faible latence';

  @override
  String get videoLowLatencySubtitle =>
      'Synchronisez l\'émulation et le moteur de rendu pour réduire la gigue. Prend effet après le redémarrage de l\'application.';

  @override
  String get paletteModeLabel => 'Palette';

  @override
  String get paletteModeBuiltin => 'Intégré';

  @override
  String get paletteModeCustom => 'Coutume…';

  @override
  String paletteModeCustomActive(String name) {
    return 'Personnalisé ($name)';
  }

  @override
  String get builtinPaletteLabel => 'Palette intégrée';

  @override
  String get customPaletteLoadTitle => 'Charger le fichier de palette (.pal)…';

  @override
  String get customPaletteLoadSubtitle =>
      '192 octets (RVB) ou 256 octets (RGBA)';

  @override
  String commandSucceeded(String label) {
    return '$label a réussi';
  }

  @override
  String commandFailed(String label) {
    return 'Échec de $label';
  }

  @override
  String get snackPaused => 'En pause';

  @override
  String get snackResumed => 'Reprise';

  @override
  String snackPauseFailed(String error) {
    return 'Échec de la pause : $error';
  }

  @override
  String get dialogOk => 'D\'ACCORD';

  @override
  String get debuggerNoRomTitle => 'Aucune ROM en cours d\'exécution';

  @override
  String get debuggerNoRomSubtitle =>
      'Chargez une ROM pour voir l\'état du débogage';

  @override
  String get debuggerCpuRegisters => 'Registres du processeur';

  @override
  String get debuggerPpuState => 'État du PPU';

  @override
  String get debuggerCpuStatusTooltip =>
      'Registre d\'état du processeur (P)\nN : Négatif - défini si le bit de résultat 7 est défini\nV : Débordement - défini sur le débordement signé\nB : Pause - défini par l\'instruction BRK\nD : Décimal - mode BCD (ignoré sur NES)\nI : Désactivation de l\'interruption - bloque l\'IRQ\nZ : Zéro - défini si le résultat est zéro\nC : Carry – défini sur un débordement non signé\n\nMajuscule = défini, minuscule = clair';

  @override
  String get debuggerPpuCtrlTooltip =>
      'Registre de contrôle PPU (2 000 \$)\nV : activation NMI\nP : PPU maître/esclave (inutilisé)\nH : hauteur du sprite (0=8x8, 1=8x16)\nB : adresse de la table des motifs d’arrière-plan\nS : adresse de la table de modèles de sprites\nI : incrément d\'adresse VRAM (0=1, 1=32)\nNN : adresse de table nommable de base\n\nMajuscule = défini, minuscule = clair';

  @override
  String get debuggerPpuMaskTooltip =>
      'Registre des masques PPU (2 001 \$)\nBGR : bits d\'accentuation des couleurs\ns : Afficher les sprites\nb : Afficher l\'arrière-plan\nM : Afficher les sprites dans les 8 pixels les plus à gauche\nm : Afficher l\'arrière-plan dans les 8 pixels les plus à gauche\ng : niveaux de gris\n\nMajuscule = défini, minuscule = clair';

  @override
  String get debuggerPpuStatusTooltip =>
      'Registre de statut du PPU (2002 \$)\nV : VBlank a démarré\nS : Sprite 0 touché\nO : Débordement de sprites\n\nMajuscule = défini, minuscule = clair';

  @override
  String get debuggerScanlineTooltip =>
      'Numéros de lignes de balayage :\n0-239 : Visible (Rendu)\n240 : Post-rendu (inactif)\n241-260 : VBlank (suppression verticale)\n-1 : Pré-rendu (ligne de balayage factice)';

  @override
  String get tilemapSettings => 'Paramètres';

  @override
  String get tilemapOverlay => 'Recouvrir';

  @override
  String get tilemapDisplayMode => 'Mode d\'affichage';

  @override
  String get tilemapDisplayModeDefault => 'Défaut';

  @override
  String get tilemapDisplayModeGrayscale => 'Niveaux de gris';

  @override
  String get tilemapDisplayModeAttributeView => 'Vue des attributs';

  @override
  String get tilemapTileGrid => 'Grille de tuiles (8 × 8)';

  @override
  String get tilemapAttrGrid => 'Grille d\'attraction (16×16)';

  @override
  String get tilemapAttrGrid32 => 'Grille d\'attraction (32 × 32)';

  @override
  String get tilemapNtBounds => 'Limites NT';

  @override
  String get tilemapScrollOverlay => 'Superposition de défilement';

  @override
  String get tilemapPanelDisplay => 'Afficher';

  @override
  String get tilemapPanelTilemap => 'Plan de tuiles';

  @override
  String get tilemapPanelSelectedTile => 'Tuile sélectionnée';

  @override
  String get tilemapHidePanel => 'Masquer le panneau';

  @override
  String get tilemapShowPanel => 'Afficher le panneau';

  @override
  String get tilemapInfoSize => 'Taille';

  @override
  String get tilemapInfoSizePx => 'Taille (px)';

  @override
  String get tilemapInfoTilemapAddress => 'Adresse de la carte de tuiles';

  @override
  String get tilemapInfoTilesetAddress => 'Adresse de l\'ensemble de tuiles';

  @override
  String get tilemapInfoMirroring => 'Mise en miroir';

  @override
  String get tilemapInfoTileFormat => 'Format de tuile';

  @override
  String get tilemapInfoTileFormat2bpp => '2 cuillères à café';

  @override
  String get tilemapMirroringHorizontal => 'Horizontal';

  @override
  String get tilemapMirroringVertical => 'Verticale';

  @override
  String get tilemapMirroringFourScreen => 'Quatre écrans';

  @override
  String get tilemapMirroringSingleScreenLower => 'Écran unique (inférieur)';

  @override
  String get tilemapMirroringSingleScreenUpper => 'Écran unique (supérieur)';

  @override
  String get tilemapMirroringMapperControlled => 'Contrôlé par le mappeur';

  @override
  String get tilemapLabelColumnRow => 'Colonne, Ligne';

  @override
  String get tilemapLabelXY => 'X, Oui';

  @override
  String get tilemapLabelSize => 'Taille';

  @override
  String get tilemapLabelTilemapAddress => 'Adresse du plan de tuile';

  @override
  String get tilemapLabelTileIndex => 'Index des tuiles';

  @override
  String get tilemapLabelTileAddressPpu => 'Adresse de la tuile (PPU)';

  @override
  String get tilemapLabelPaletteIndex => 'Index des palettes';

  @override
  String get tilemapLabelPaletteAddress => 'Adresse de la palette';

  @override
  String get tilemapLabelAttributeAddress => 'Adresse d\'attribut';

  @override
  String get tilemapLabelAttributeData => 'Données d\'attribut';

  @override
  String get tilemapSelectedTileTilemap => 'Plan de tuiles';

  @override
  String get tilemapSelectedTileTileIdx => 'Idx de tuile';

  @override
  String get tilemapSelectedTileTilePpu => 'Tuile (PPU)';

  @override
  String get tilemapSelectedTilePalette => 'Palette';

  @override
  String get tilemapSelectedTileAttr => 'Attr.';

  @override
  String get tilemapCapture => 'Capturer';

  @override
  String get tilemapCaptureFrameStart => 'Début de l\'image';

  @override
  String get tilemapCaptureVblankStart => 'VBlank Démarrage';

  @override
  String get tilemapCaptureManual => 'Manuel';

  @override
  String get tilemapScanline => 'Ligne de balayage';

  @override
  String get tilemapDot => 'Point';

  @override
  String tilemapError(String error) {
    return 'Erreur : $error';
  }

  @override
  String get tilemapRetry => 'Réessayer';

  @override
  String get tilemapResetZoom => 'Réinitialiser le zoom';

  @override
  String get menuTilemapViewer => 'Visionneuse de cartes de tuiles';

  @override
  String get menuTileViewer => 'Visionneuse de tuiles';

  @override
  String tileViewerError(String error) {
    return 'Erreur : $error';
  }

  @override
  String get tileViewerRetry => 'Réessayer';

  @override
  String get tileViewerSettings => 'Paramètres de la visionneuse de vignettes';

  @override
  String get tileViewerOverlays => 'Superpositions';

  @override
  String get tileViewerShowGrid => 'Afficher la grille de tuiles';

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
  String get tileViewerGrayscale => 'Utiliser la palette de niveaux de gris';

  @override
  String get tileViewerSelectedTile => 'Tuile sélectionnée';

  @override
  String get tileViewerPatternTable => 'Tableau des modèles';

  @override
  String get tileViewerTileIndex => 'Index des tuiles';

  @override
  String get tileViewerChrAddress => 'Adresse du CHR';

  @override
  String get tileViewerClose => 'Fermer';

  @override
  String get tileViewerSource => 'Source';

  @override
  String get tileViewerSourcePpu => 'Mémoire PPU';

  @override
  String get tileViewerSourceChrRom => 'ROM CHR';

  @override
  String get tileViewerSourceChrRam => 'RAM du CHR';

  @override
  String get tileViewerSourcePrgRom => 'ROM PRG';

  @override
  String get tileViewerAddress => 'Adresse';

  @override
  String get tileViewerSize => 'Taille';

  @override
  String get tileViewerColumns => 'Cols';

  @override
  String get tileViewerRows => 'Lignes';

  @override
  String get tileViewerLayout => 'Mise en page';

  @override
  String get tileViewerLayoutNormal => 'Normale';

  @override
  String get tileViewerLayout8x16 => 'Sprites 8x16';

  @override
  String get tileViewerLayout16x16 => 'Sprites 16x16';

  @override
  String get tileViewerBackground => 'Arrière-plan';

  @override
  String get tileViewerBgDefault => 'Défaut';

  @override
  String get tileViewerBgTransparent => 'Transparent';

  @override
  String get tileViewerBgPalette => 'Couleur de la palette';

  @override
  String get tileViewerBgBlack => 'Noir';

  @override
  String get tileViewerBgWhite => 'Blanc';

  @override
  String get tileViewerBgMagenta => 'Magenta';

  @override
  String get tileViewerPresets => 'Préréglages';

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
  String get menuSpriteViewer => 'Visionneuse de sprites';

  @override
  String get menuPaletteViewer => 'Visionneuse de palettes';

  @override
  String get paletteViewerPaletteRamTitle => 'RAM des palettes (32)';

  @override
  String get paletteViewerSystemPaletteTitle => 'Palette système (64)';

  @override
  String get paletteViewerSettingsTooltip =>
      'Paramètres de la visionneuse de palettes';

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
    return 'Erreur du visualiseur de sprites : $error';
  }

  @override
  String get spriteViewerSettingsTooltip =>
      'Paramètres de la visionneuse de sprites';

  @override
  String get spriteViewerShowGrid => 'Afficher la grille';

  @override
  String get spriteViewerShowOutline =>
      'Afficher le contour autour des sprites';

  @override
  String get spriteViewerShowOffscreenRegions =>
      'Afficher les régions hors écran';

  @override
  String get spriteViewerDimOffscreenSpritesGrid =>
      'Atténuer les sprites hors écran (grille)';

  @override
  String get spriteViewerShowListView => 'Afficher la vue liste';

  @override
  String get spriteViewerPanelSprites => 'Sprites';

  @override
  String get spriteViewerPanelDataSource => 'Source de données';

  @override
  String get spriteViewerPanelSprite => 'Sprite';

  @override
  String get spriteViewerPanelSelectedSprite => 'Sprite sélectionné';

  @override
  String get spriteViewerLabelMode => 'Mode';

  @override
  String get spriteViewerLabelPatternBase => 'Base du motif';

  @override
  String get spriteViewerLabelThumbnailSize => 'Taille de la vignette';

  @override
  String get spriteViewerBgGray => 'Gris';

  @override
  String get spriteViewerDataSourceSpriteRam => 'RAM des sprites';

  @override
  String get spriteViewerDataSourceCpuMemory => 'Mémoire du processeur';

  @override
  String spriteViewerTooltipTitle(int index) {
    return 'Sprite #$index';
  }

  @override
  String get spriteViewerLabelIndex => 'Indice';

  @override
  String get spriteViewerLabelPos => 'Pos';

  @override
  String get spriteViewerLabelSize => 'Taille';

  @override
  String get spriteViewerLabelTile => 'Tuile';

  @override
  String get spriteViewerLabelTileAddr => 'Adresse de la tuile';

  @override
  String get spriteViewerLabelPalette => 'Palette';

  @override
  String get spriteViewerLabelPaletteAddr => 'Adresse de la palette';

  @override
  String get spriteViewerLabelFlip => 'Retourner';

  @override
  String get spriteViewerLabelPriority => 'Priorité';

  @override
  String get spriteViewerPriorityBehindBg => 'Derrière BG';

  @override
  String get spriteViewerPriorityInFront => 'Devant';

  @override
  String get spriteViewerLabelVisible => 'Visible';

  @override
  String get spriteViewerValueYes => 'Oui';

  @override
  String get spriteViewerValueNoOffscreen => 'Non (hors écran)';

  @override
  String get spriteViewerVisibleStatusVisible => 'Visible';

  @override
  String get spriteViewerVisibleStatusOffscreen => 'Hors écran';

  @override
  String get longPressToClear => 'Appuyez longuement pour effacer';

  @override
  String get videoBackendD3D11 => 'GPU D3D11 (zéro copie)';

  @override
  String get videoBackendSoftware => 'Processeur logiciel (de secours)';

  @override
  String get netplayBackToSetup => 'Retour à la configuration';

  @override
  String get netplayP2PMode => 'Mode P2P';

  @override
  String get netplaySignalingServer => 'Serveur de signalisation';

  @override
  String get netplayRelayServer => 'Serveur relais (de secours)';

  @override
  String get netplayP2PRoomCode => 'Code de chambre P2P';

  @override
  String get netplayStartP2PSession => 'Démarrer une session P2P';

  @override
  String get netplayJoinP2PSession => 'Rejoignez la session P2P';

  @override
  String get netplayInvalidP2PServerAddr => 'Adresse du serveur P2P invalide';

  @override
  String get netplayProceed => 'Procéder';

  @override
  String get videoShaderParametersTitle => 'Paramètres du shader';

  @override
  String get videoShaderParametersSubtitle =>
      'Ajustez les paramètres du shader en temps réel';

  @override
  String get videoShaderParametersReset => 'Réinitialiser les paramètres';

  @override
  String get searchHint => 'Recherche...';

  @override
  String get searchTooltip => 'Recherche';

  @override
  String get noResults => 'Aucun paramètre correspondant trouvé';

  @override
  String get errorFailedToCreateTexture => 'Échec de la création de la texture';

  @override
  String get languageJapanese => 'japonais';

  @override
  String get languageSpanish => 'Espagnol';

  @override
  String get languagePortuguese => 'portugais';

  @override
  String get languageRussian => 'russe';

  @override
  String get languageFrench => 'Français';

  @override
  String get languageGerman => 'Allemand';
}
