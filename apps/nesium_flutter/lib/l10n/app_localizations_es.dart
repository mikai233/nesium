// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Spanish Castilian (`es`).
class AppLocalizationsEs extends AppLocalizations {
  AppLocalizationsEs([String locale = 'es']) : super(locale);

  @override
  String get settingsTitle => 'Ajustes';

  @override
  String get settingsTabGeneral => 'General';

  @override
  String get settingsTabInput => 'Entrada';

  @override
  String get settingsTabVideo => 'Video';

  @override
  String get settingsTabEmulation => 'Emulación';

  @override
  String get settingsTabServer => 'Servidor';

  @override
  String get settingsFloatingPreviewToggle => 'Vista previa flotante';

  @override
  String get settingsFloatingPreviewTooltip => 'Mostrar vista previa del juego';

  @override
  String get serverTitle => 'Servidor netplay';

  @override
  String get serverPortLabel => 'Puerto';

  @override
  String get serverStartButton => 'Iniciar servidor';

  @override
  String get serverStopButton => 'Detener servidor';

  @override
  String get serverStatusRunning => 'En ejecución';

  @override
  String get serverStatusStopped => 'Detenido';

  @override
  String serverClientCount(int count) {
    return 'Clientes conectados: $count';
  }

  @override
  String serverStartFailed(String error) {
    return 'Error al iniciar el servidor: $error';
  }

  @override
  String serverStopFailed(String error) {
    return 'Error al detener el servidor: $error';
  }

  @override
  String serverBindAddress(String address) {
    return 'Dirección de enlace: $address';
  }

  @override
  String serverQuicFingerprint(String fingerprint) {
    return 'Huella digital QUIC: $fingerprint';
  }

  @override
  String get generalTitle => 'General';

  @override
  String get themeLabel => 'Tema';

  @override
  String get themeSystem => 'Sistema';

  @override
  String get themeLight => 'Luz';

  @override
  String get themeDark => 'Oscuro';

  @override
  String get languageLabel => 'Idioma';

  @override
  String get languageSystem => 'Sistema';

  @override
  String get languageEnglish => 'Inglés';

  @override
  String get languageChineseSimplified => 'Chino simplificado';

  @override
  String get inputTitle => 'Entrada';

  @override
  String get turboTitle => 'Turbo';

  @override
  String get turboLinkPressRelease => 'Enlace de prensa/comunicado';

  @override
  String get inputDeviceLabel => 'Dispositivo de entrada';

  @override
  String get inputDeviceKeyboard => 'Teclado';

  @override
  String get inputDeviceGamepad => 'Mando de juegos';

  @override
  String get connectedGamepadsTitle => 'Mandos conectados';

  @override
  String get connectedGamepadsNone => 'No hay mandos conectados';

  @override
  String get webGamepadActivationHint =>
      'Límite web: PRESIONA CUALQUIER BOTÓN en tu gamepad para activarlo.';

  @override
  String connectedGamepadsPort(int port) {
    return 'Jugador $port';
  }

  @override
  String get connectedGamepadsUnassigned => 'No asignado';

  @override
  String get inputDeviceVirtualController => 'Controlador virtual';

  @override
  String get inputGamepadAssignmentLabel => 'Asignación de gamepad';

  @override
  String get inputGamepadNone => 'Ninguno/Sin asignar';

  @override
  String get inputListening => 'Escuchando...';

  @override
  String inputDetected(String buttons) {
    return 'Detectado: $buttons';
  }

  @override
  String get inputGamepadMappingLabel => 'Mapeo de botones';

  @override
  String get inputResetToDefault => 'Restablecer los valores predeterminados';

  @override
  String get inputButtonA => 'A';

  @override
  String get inputButtonB => 'B';

  @override
  String get inputButtonTurboA => 'Turbo A';

  @override
  String get inputButtonTurboB => 'TurboB';

  @override
  String get inputButtonSelect => 'Seleccionar';

  @override
  String get inputButtonStart => 'Comenzar';

  @override
  String get inputButtonUp => 'Arriba';

  @override
  String get inputButtonDown => 'Abajo';

  @override
  String get inputButtonLeft => 'Izquierda';

  @override
  String get inputButtonRight => 'Derecha';

  @override
  String get inputButtonRewind => 'Rebobinar';

  @override
  String get inputButtonFastForward => 'Avance rápido';

  @override
  String get inputButtonSaveState => 'Guardar estado';

  @override
  String get inputButtonLoadState => 'Cargar estado';

  @override
  String get inputButtonPause => 'Pausa';

  @override
  String get globalHotkeysTitle => 'Teclas de acceso rápido del emulador';

  @override
  String get gamepadHotkeysTitle =>
      'Teclas de acceso rápido del gamepad (Jugador 1)';

  @override
  String get inputPortLabel => 'Configurar reproductor';

  @override
  String get player1 => 'Jugador 1';

  @override
  String get player2 => 'Jugador 2';

  @override
  String get player3 => 'Jugador 3';

  @override
  String get player4 => 'Jugador 4';

  @override
  String get keyboardPresetLabel => 'Preajuste de teclado';

  @override
  String get keyboardPresetNone => 'Ninguno';

  @override
  String get keyboardPresetNesStandard => 'Estándar NES';

  @override
  String get keyboardPresetFightStick => 'Arcade Stick';

  @override
  String get keyboardPresetArcadeLayout => 'Distribución Arcade';

  @override
  String get keyboardPresetCustom => 'Costumbre';

  @override
  String get customKeyBindingsTitle => 'Atajos de teclas personalizados';

  @override
  String bindKeyTitle(String action) {
    return 'Enlazar $action';
  }

  @override
  String get unassignedKey => 'No asignado';

  @override
  String get tipPressEscapeToClearBinding =>
      'Consejo: presione Escape para borrar un enlace.';

  @override
  String get keyboardActionUp => 'Arriba';

  @override
  String get keyboardActionDown => 'Abajo';

  @override
  String get keyboardActionLeft => 'Izquierda';

  @override
  String get keyboardActionRight => 'Derecha';

  @override
  String get keyboardActionA => 'A';

  @override
  String get keyboardActionB => 'B';

  @override
  String get keyboardActionSelect => 'Seleccionar';

  @override
  String get keyboardActionStart => 'Comenzar';

  @override
  String get keyboardActionTurboA => 'Turbo A';

  @override
  String get keyboardActionTurboB => 'TurboB';

  @override
  String get keyboardActionRewind => 'Rebobinar';

  @override
  String get keyboardActionFastForward => 'Avance rápido';

  @override
  String get keyboardActionSaveState => 'Guardar estado';

  @override
  String get keyboardActionLoadState => 'Cargar estado';

  @override
  String get keyboardActionPause => 'Pausa';

  @override
  String get keyboardActionFullScreen => 'Pantalla completa';

  @override
  String inputBindingConflictCleared(String player, String action) {
    return '$player $action enlace borrado.';
  }

  @override
  String inputBindingConflictHint(String player, String action) {
    return '($player - $action)';
  }

  @override
  String inputBindingCapturedConflictHint(String player, String action) {
    return 'Ocupado por $player - $action';
  }

  @override
  String get emulationTitle => 'Emulación';

  @override
  String get integerFpsTitle => 'Modo FPS entero (60 Hz, NTSC)';

  @override
  String get integerFpsSubtitle =>
      'Reduce la vibración del desplazamiento en pantallas de 60 Hz. PAL se agregará más tarde.';

  @override
  String get showOverlayTitle => 'Mostrar superposición de estado';

  @override
  String get showOverlaySubtitle =>
      'Muestra indicadores de pausa/rebobinado/avance rápido en la pantalla.';

  @override
  String get pauseInBackgroundTitle => 'Pausa en segundo plano';

  @override
  String get pauseInBackgroundSubtitle =>
      'Pausa automáticamente el emulador cuando la aplicación no está activa.';

  @override
  String get autoSaveEnabledTitle => 'Guardar automáticamente';

  @override
  String get autoSaveEnabledSubtitle =>
      'Guarde periódicamente el estado del juego en una ranura dedicada.';

  @override
  String get autoSaveIntervalTitle => 'Intervalo de guardado automático';

  @override
  String autoSaveIntervalValue(int minutes) {
    return '$minutes minutos';
  }

  @override
  String get fastForwardSpeedTitle => 'Velocidad de avance rápido';

  @override
  String get fastForwardSpeedSubtitle =>
      'Velocidad máxima mientras el avance rápido está activo.';

  @override
  String fastForwardSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get quickSaveSlotTitle => 'Ranura de guardado rápido';

  @override
  String get quickSaveSlotSubtitle =>
      'Ranura utilizada por atajos de carga/guardado rápido.';

  @override
  String quickSaveSlotValue(int index) {
    return 'Ranura $index';
  }

  @override
  String get rewindEnabledTitle => 'Rebobinar';

  @override
  String get rewindEnabledSubtitle =>
      'Habilite la funcionalidad de rebobinado en tiempo real.';

  @override
  String get rewindSecondsTitle => 'Duración del rebobinado';

  @override
  String rewindSecondsValue(int seconds) {
    return '$seconds segundos';
  }

  @override
  String get rewindMinutesTitle => 'Duración del rebobinado';

  @override
  String rewindMinutesValue(int minutes) {
    return '$minutes minutos';
  }

  @override
  String get rewindSpeedTitle => 'Velocidad de rebobinado';

  @override
  String get rewindSpeedSubtitle =>
      'La velocidad mientras el rebobinado está activo.';

  @override
  String rewindSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get autoSlotLabel => 'Ranura automática';

  @override
  String get menuAutoSave => 'Guardar automáticamente...';

  @override
  String get stateAutoSaved => 'Guardado automático creado';

  @override
  String get virtualControlsTitle => 'Controles virtuales';

  @override
  String get virtualControlsSwitchInputTip =>
      'Cambie la entrada a \"Controlador virtual\" para usar esta configuración.';

  @override
  String get virtualControlsButtonSize => 'Tamaño del botón';

  @override
  String get virtualControlsGap => 'Brecha';

  @override
  String get virtualControlsOpacity => 'Opacidad';

  @override
  String get virtualControlsHitboxScale => 'escala de hitbox';

  @override
  String get virtualControlsHapticFeedback => 'Retroalimentación háptica';

  @override
  String get virtualControlsDpadDeadzone => 'Zona muerta del pad direccional';

  @override
  String get virtualControlsDpadDeadzoneHelp =>
      'Center deadzone: touching near the center won’t trigger any direction.';

  @override
  String get virtualControlsDpadBoundaryDeadzone =>
      'Zona muerta límite del pad direccional';

  @override
  String get virtualControlsDpadBoundaryDeadzoneHelp =>
      'Zona muerta de límites: los valores más altos hacen que las diagonales sean más difíciles de activar, lo que reduce las pulsaciones accidentales de vecinos.';

  @override
  String get virtualControlsReset => 'Restablecer diseño';

  @override
  String get virtualControlsDiscardChangesTitle => 'Deshacer cambios';

  @override
  String get virtualControlsDiscardChangesSubtitle =>
      'Volver al último diseño guardado';

  @override
  String get virtualControlsTurboFramesPerToggle => 'Cuadros turbo por palanca';

  @override
  String get virtualControlsTurboOnFrames => 'Marcos de prensa turbo';

  @override
  String get virtualControlsTurboOffFrames => 'Marcos de liberación turbo';

  @override
  String framesValue(int frames) {
    return 'Marcos $frames';
  }

  @override
  String get tipAdjustButtonsInDrawer =>
      'Consejo: ajusta la posición/tamaño de los botones desde el cajón del juego.';

  @override
  String get keyCapturePressKeyToBind => 'Presione una tecla para vincularse.';

  @override
  String keyCaptureCurrent(String key) {
    return 'Actual: $key';
  }

  @override
  String keyCaptureCaptured(String key) {
    return 'Capturado: $key';
  }

  @override
  String get keyCapturePressEscToClear => 'Presione Escape para borrar.';

  @override
  String get keyBindingsTitle => 'Atajos de teclas';

  @override
  String get cancel => 'Cancelar';

  @override
  String get appName => 'Nesium';

  @override
  String get menuTooltip => 'Menú';

  @override
  String get menuSectionFile => 'Archivo';

  @override
  String get menuSectionEmulation => 'Emulación';

  @override
  String get menuSectionSettings => 'Ajustes';

  @override
  String get menuSectionWindows => 'ventanas';

  @override
  String get menuSectionHelp => 'Ayuda';

  @override
  String get menuOpenRom => 'Abrir ROM...';

  @override
  String get menuReset => 'Reiniciar';

  @override
  String get menuPowerReset => 'Reinicio de energía';

  @override
  String get menuEject => 'Apagar';

  @override
  String get menuSaveState => 'Guardar estado...';

  @override
  String get menuLoadState => 'Estado de carga...';

  @override
  String get menuPauseResume => 'Pausar / Reanudar';

  @override
  String get menuNetplay => 'Netplay';

  @override
  String get netplayTransportLabel => 'Transporte';

  @override
  String get netplayTransportAuto => 'Automático (QUIC → TCP)';

  @override
  String get netplayTransportUnknown => 'Desconocido';

  @override
  String get netplayTransportTcp => 'tcp';

  @override
  String get netplayTransportQuic => 'QUIC';

  @override
  String get netplayUsingTcpFallback => 'QUIC falló al usar TCP';

  @override
  String get netplayStatusDisconnected => 'Desconectado';

  @override
  String get netplayStatusConnecting => 'Conectando...';

  @override
  String get netplayStatusConnected => 'Conectado (esperando habitación)';

  @override
  String get netplayStatusInRoom => 'En la sala';

  @override
  String get netplayDisconnect => 'Desconectar';

  @override
  String get netplayServerAddress => 'Dirección del servidor';

  @override
  String get netplayServerNameLabel => 'Nombre del servidor (SNI)';

  @override
  String get netplayServerNameHint => 'localhost';

  @override
  String get netplayPlayerName => 'Nombre del jugador';

  @override
  String get netplayQuicFingerprintLabel =>
      'Huella digital del certificado QUIC (opcional)';

  @override
  String get netplayQuicFingerprintHint => 'base64url (43 caracteres)';

  @override
  String get netplayQuicFingerprintHelper =>
      'Ingrese esto para usar QUIC anclado. Déjelo vacío para utilizar la confianza del sistema (QUIC) o recurrir a TCP.';

  @override
  String get netplayConnect => 'Unirse al juego';

  @override
  String get netplayJoinViaP2P => 'Unirse a través de P2P';

  @override
  String get netplayJoinGame => 'Unirse al juego';

  @override
  String get netplayCreateRoom => 'Crear habitación';

  @override
  String get netplayJoinRoom => 'Unirse al juego';

  @override
  String get netplayAddressOrRoomCode =>
      'Código de habitación o dirección del servidor';

  @override
  String get netplayHostingTitle => 'Alojamiento';

  @override
  String get netplayRoomCodeLabel => 'Tu código de habitación';

  @override
  String get netplayP2PEnabled => 'Modo P2P';

  @override
  String get netplayDirectServerLabel => 'Dirección del servidor';

  @override
  String get netplayAdvancedSettings => 'Configuración de conexión avanzada';

  @override
  String get netplayP2PServerLabel => 'Servidor P2P';

  @override
  String get netplayRoomCode => 'Código de habitación';

  @override
  String get netplayRoleLabel => 'Role';

  @override
  String netplayPlayerIndex(int index) {
    return 'Jugador $index';
  }

  @override
  String get netplaySpectator => 'Espectador';

  @override
  String get netplayClientId => 'ID de cliente';

  @override
  String get netplayPlayerListHeader => 'Jugadores';

  @override
  String get netplayYouIndicator => '(Tú)';

  @override
  String get netplayOrSeparator => 'O';

  @override
  String netplayConnectFailed(String error) {
    return 'Error de conexión: $error';
  }

  @override
  String netplayDisconnectFailed(String error) {
    return 'Error de desconexión: $error';
  }

  @override
  String netplayCreateRoomFailed(String error) {
    return 'Error al crear sala: $error';
  }

  @override
  String netplayJoinRoomFailed(String error) {
    return 'Error al unirse a la sala: $error';
  }

  @override
  String netplaySwitchRoleFailed(String error) {
    return 'Error al cambiar de rol: $error';
  }

  @override
  String get netplayInvalidRoomCode => 'Código de habitación no válido';

  @override
  String get netplayRomBroadcasted =>
      'Netplay: ROM transmitida a la habitación';

  @override
  String get menuLoadTasMovie => 'Cargar película TAS...';

  @override
  String get menuPreferences => 'Preferences...';

  @override
  String get saveToExternalFile => 'Guardar en archivo...';

  @override
  String get loadFromExternalFile => 'Cargar desde archivo...';

  @override
  String get slotLabel => 'Ranura';

  @override
  String get slotEmpty => 'Vacío';

  @override
  String get slotHasData => 'Guardado';

  @override
  String stateSavedToSlot(int index) {
    return 'Estado guardado en la ranura $index';
  }

  @override
  String stateLoadedFromSlot(int index) {
    return 'Estado cargado desde la ranura $index';
  }

  @override
  String slotCleared(int index) {
    return 'Ranura $index borrada';
  }

  @override
  String get menuAbout => 'Acerca de';

  @override
  String get menuDebugger => 'Depurador';

  @override
  String get menuTools => 'Herramientas';

  @override
  String get menuOpenDebuggerWindow => 'Abrir ventana del depurador';

  @override
  String get menuOpenToolsWindow => 'Abrir ventana de herramientas';

  @override
  String get menuInputMappingComingSoon => 'Mapeo de entrada (próximamente)';

  @override
  String get menuLastError => 'último error';

  @override
  String get lastErrorDetailsAction => 'Detalles';

  @override
  String get lastErrorDialogTitle => 'último error';

  @override
  String get lastErrorCopied => 'Copiado';

  @override
  String get copy => 'Copiar';

  @override
  String get paste => 'Pasta';

  @override
  String get windowDebuggerTitle => 'Depurador de Nesium';

  @override
  String get windowToolsTitle => 'Herramientas de nesio';

  @override
  String get virtualControlsEditTitle => 'Editar controles virtuales';

  @override
  String get virtualControlsEditSubtitleEnabled =>
      'Arrastra para mover, pellizca o arrastra la esquina para cambiar el tamaño';

  @override
  String get virtualControlsEditSubtitleDisabled =>
      'Habilitar ajuste interactivo';

  @override
  String get gridSnappingTitle => 'Ajuste de cuadrícula';

  @override
  String get gridSpacingLabel => 'Espaciado de cuadrícula';

  @override
  String get debuggerPlaceholderBody =>
      'Espacio reservado para monitores CPU/PPU, visores de memoria e inspectores OAM. Los mismos widgets pueden vivir en un panel lateral de escritorio o en una hoja móvil.';

  @override
  String get toolsPlaceholderBody =>
      'La grabación/reproducción, el mapeo de entradas y los trucos pueden compartir estos widgets entre los paneles laterales del escritorio y las hojas inferiores del móvil.';

  @override
  String get actionLoadRom => 'Cargar ROM';

  @override
  String get actionResetNes => 'Restablecer NES';

  @override
  String get actionPowerResetNes => 'Reinicio de energía NES';

  @override
  String get actionEjectNes => 'Apagar';

  @override
  String get actionLoadPalette => 'Cargar paleta';

  @override
  String get videoResetToDefault => 'Restablecer los valores predeterminados';

  @override
  String get videoTitle => 'Video';

  @override
  String get videoFilterLabel => 'Filtro de vídeo';

  @override
  String get videoFilterCategoryCpu => 'Filtros de CPU';

  @override
  String get videoFilterCategoryGpu => 'Filtros GPU (sombreadores)';

  @override
  String get videoFilterNone => 'Ninguno (1x)';

  @override
  String get videoFilterPrescale2x => 'Preescala 2x';

  @override
  String get videoFilterPrescale3x => 'Preescala 3x';

  @override
  String get videoFilterPrescale4x => 'Preescala 4x';

  @override
  String get videoFilterHq2x => 'HQ2x';

  @override
  String get videoFilterHq3x => 'HQ3x';

  @override
  String get videoFilterHq4x => 'HQ4x';

  @override
  String get videoFilter2xSai => '2xSaI';

  @override
  String get videoFilterSuper2xSai => 'Súper 2xSaI';

  @override
  String get videoFilterSuperEagle => 'Súper Águila';

  @override
  String get videoFilterLcdGrid => 'Rejilla LCD (2x)';

  @override
  String get videoFilterScanlines => 'Líneas de exploración (2x)';

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
  String get videoLcdGridStrengthLabel => 'Intensidad de la rejilla LCD';

  @override
  String get videoScanlinesIntensityLabel =>
      'Intensidad de la línea de exploración';

  @override
  String get videoFilterNtscComposite => 'NTSC (compuesto)';

  @override
  String get videoFilterNtscSvideo => 'NTSC (S-Vídeo)';

  @override
  String get videoFilterNtscRgb => 'NTSC (RGB)';

  @override
  String get videoFilterNtscMonochrome => 'NTSC (monocromo)';

  @override
  String get videoFilterNtscBisqwit2x => 'NTSC (Bisqwit) 2x';

  @override
  String get videoFilterNtscBisqwit4x => 'NTSC (Bisqwit) 4x';

  @override
  String get videoFilterNtscBisqwit8x => 'NTSC (Bisqwit) 8x';

  @override
  String get videoNtscAdvancedTitle => 'NTSC avanzado';

  @override
  String get videoNtscMergeFieldsLabel =>
      'Fusionar campos (reducir el parpadeo)';

  @override
  String get videoNtscHueLabel => 'Matiz';

  @override
  String get videoNtscSaturationLabel => 'Saturación';

  @override
  String get videoNtscContrastLabel => 'Contraste';

  @override
  String get videoNtscBrightnessLabel => 'Brillo';

  @override
  String get videoNtscSharpnessLabel => 'Nitidez';

  @override
  String get videoNtscGammaLabel => 'Gama';

  @override
  String get videoNtscResolutionLabel => 'Resolución';

  @override
  String get videoNtscArtifactsLabel => 'Artefactos';

  @override
  String get videoNtscFringingLabel => 'Franjas';

  @override
  String get videoNtscBleedLabel => 'Sangrado';

  @override
  String get videoNtscBisqwitSettingsTitle => 'Configuración NTSC (Bisqwit)';

  @override
  String get videoNtscBisqwitYFilterLengthLabel =>
      'Filtro Y (desenfoque horizontal)';

  @override
  String get videoNtscBisqwitIFilterLengthLabel => 'Yo filtro';

  @override
  String get videoNtscBisqwitQFilterLengthLabel => 'Filtro Q';

  @override
  String get videoIntegerScalingTitle => 'Escalado de enteros';

  @override
  String get videoIntegerScalingSubtitle =>
      'Escala de píxeles perfectos (reduce el brillo al desplazarse).';

  @override
  String get videoFullScreenTitle => 'Pantalla completa';

  @override
  String get videoFullScreenSubtitle =>
      'Alternar estado de pantalla completa de la ventana';

  @override
  String get videoScreenVerticalOffset =>
      'Desplazamiento vertical de la pantalla';

  @override
  String get videoScreenVerticalOffsetPortraitOnly =>
      'Sólo tiene efecto en modo retrato.';

  @override
  String get videoAspectRatio => 'Relación de aspecto';

  @override
  String get videoAspectRatioSquare => '1:1 (píxeles cuadrados)';

  @override
  String get videoAspectRatioNtsc => '4:3 (NTSC)';

  @override
  String get videoAspectRatioStretch => 'Estirar';

  @override
  String get videoShaderLibrashaderTitle => 'Sombreadores RetroArch';

  @override
  String get videoShaderLibrashaderSubtitle =>
      'Requiere GLES3 + backend de hardware (cadena de intercambio AHB).';

  @override
  String get videoShaderLibrashaderSubtitleWindows =>
      'Requiere backend de GPU D3D11.';

  @override
  String get videoShaderLibrashaderSubtitleApple =>
      'Requiere backend de metal.';

  @override
  String get videoShaderLibrashaderSubtitleDisabled =>
      'Cambie el backend de Android a Hardware para habilitarlo.';

  @override
  String get videoShaderLibrashaderSubtitleDisabledWindows =>
      'Cambie el backend de Windows a la GPU D3D11 para habilitarlo.';

  @override
  String get videoShaderPresetLabel => 'Preestablecido (.slangp)';

  @override
  String get videoShaderPresetNotSet => 'No establecido';

  @override
  String get shaderBrowserTitle => 'Sombreadores';

  @override
  String get shaderBrowserNoShaders => 'No se encontraron sombreadores';

  @override
  String shaderBrowserError(String error) {
    return 'Error: $error';
  }

  @override
  String get aboutTitle => 'Acerca de Nesium';

  @override
  String get aboutLead =>
      'Nesium: interfaz del emulador Rust NES/FC construida sobre nesium-core.';

  @override
  String get aboutIntro =>
      'Esta interfaz de Flutter reutiliza el núcleo de Rust para la emulación. La compilación web se ejecuta en el navegador a través de Flutter Web + Web Worker + WASM.';

  @override
  String get aboutLinksHeading => 'Campo de golf';

  @override
  String get aboutGitHubLabel => 'GitHub';

  @override
  String get aboutWebDemoLabel => 'Demostración web';

  @override
  String get aboutComponentsHeading => 'Componentes de código abierto';

  @override
  String get aboutComponentsHint =>
      'Toque para abrir, mantenga presionado para copiar.';

  @override
  String get aboutLicenseHeading => 'Licencia';

  @override
  String get aboutLicenseBody =>
      'Nesium tiene licencia GPL-3.0 o posterior. Consulte LICENSE.md en la raíz del repositorio.';

  @override
  String aboutLaunchFailed(String url) {
    return 'No se pudo iniciar: $url';
  }

  @override
  String get videoBackendLabel => 'Renderer backend';

  @override
  String get videoBackendAndroidLabel => 'Backend del renderizador de Android';

  @override
  String get videoBackendWindowsLabel => 'Backend del renderizador de Windows';

  @override
  String get videoBackendHardware => 'Hardware (AHardwareBuffer)';

  @override
  String get videoBackendUpload => 'Compatibilidad (carga de CPU)';

  @override
  String get videoBackendRestartHint =>
      'Entra en vigor después de reiniciar la aplicación.';

  @override
  String videoBackendCurrent(String backend) {
    return 'Backend actual: $backend';
  }

  @override
  String get windowsNativeOverlayTitle =>
      'Superposición nativa de Windows (experimental)';

  @override
  String get windowsNativeOverlaySubtitle =>
      'Omite el compositor Flutter para una suavidad perfecta. Desactiva los sombreadores y superpone la interfaz de usuario detrás del juego.';

  @override
  String get highPerformanceModeLabel => 'Modo de alto rendimiento';

  @override
  String get highPerformanceModeDescription =>
      'Eleve la prioridad del proceso y optimice el programador para una jugabilidad más fluida.';

  @override
  String get videoLowLatencyTitle => 'Vídeo de baja latencia';

  @override
  String get videoLowLatencySubtitle =>
      'Sincronice la emulación y el renderizador para reducir la fluctuación. Entra en vigor después de reiniciar la aplicación.';

  @override
  String get paletteModeLabel => 'Paleta';

  @override
  String get paletteModeBuiltin => 'Incorporado';

  @override
  String get paletteModeCustom => 'Costumbre…';

  @override
  String paletteModeCustomActive(String name) {
    return 'Personalizado ($name)';
  }

  @override
  String get builtinPaletteLabel => 'Paleta incorporada';

  @override
  String get customPaletteLoadTitle => 'Cargar archivo de paleta (.pal)…';

  @override
  String get customPaletteLoadSubtitle => '192 bytes (RGB) o 256 bytes (RGBA)';

  @override
  String commandSucceeded(String label) {
    return '$label tuvo éxito';
  }

  @override
  String commandFailed(String label) {
    return '$label falló';
  }

  @override
  String get snackPaused => 'En pausa';

  @override
  String get snackResumed => 'Resumed';

  @override
  String snackPauseFailed(String error) {
    return 'Pausa fallida: $error';
  }

  @override
  String get dialogOk => 'DE ACUERDO';

  @override
  String get debuggerNoRomTitle => 'No hay ROM en ejecución';

  @override
  String get debuggerNoRomSubtitle =>
      'Cargue una ROM para ver el estado de depuración';

  @override
  String get debuggerCpuRegisters => 'Registros de CPU';

  @override
  String get debuggerPpuState => 'Estado de la UPP';

  @override
  String get debuggerCpuStatusTooltip =>
      'Registro de estado de la CPU (P)\nN: Negativo: se establece si el bit de resultado 7 está establecido\nV: Desbordamiento: configurado en desbordamiento firmado\nB: Descanso - establecido por la instrucción BRK\nD: Decimal - modo BCD (ignorado en NES)\nI: Interrupción Desactivada - bloquea IRQ\nZ: Cero: se establece si el resultado es cero\nC: Llevar - establecido en desbordamiento sin firmar\n\nMayúsculas = establecer, minúsculas = borrar';

  @override
  String get debuggerPpuCtrlTooltip =>
      'Registro de control de PPU (\$2000)\nV: habilitación NMI\nP: PPU maestro/esclavo (sin usar)\nH: altura del sprite (0=8x8, 1=8x16)\nB: dirección de la tabla de patrón de fondo\nS: dirección de la tabla de patrones de Sprite\nI: incremento de dirección VRAM (0=1, 1=32)\nNN: dirección de la tabla de nombres base\n\nMayúsculas = establecer, minúsculas = borrar';

  @override
  String get debuggerPpuMaskTooltip =>
      'Registro de máscara PPU (\$2001)\nBGR: bits de énfasis de color\ns: Mostrar sprites\nb: Mostrar fondo\nM: muestra sprites en los 8 píxeles más a la izquierda\nm: muestra el fondo en los 8 píxeles más a la izquierda\ng: escala de grises\n\nMayúsculas = establecer, minúsculas = borrar';

  @override
  String get debuggerPpuStatusTooltip =>
      'Registro de estado de PPU (\$2002)\nV: Vblank ha comenzado\nS: Sprite 0 hit\nO: desbordamiento de sprites\n\nMayúsculas = establecer, minúsculas = borrar';

  @override
  String get debuggerScanlineTooltip =>
      'Números de línea de exploración:\n0-239: Visible (Renderizar)\n240: Post-renderizado (inactivo)\n241-260: Vblank (supresión vertical)\n-1: Pre-renderizado (línea de exploración ficticia)';

  @override
  String get tilemapSettings => 'Ajustes';

  @override
  String get tilemapOverlay => 'Cubrir';

  @override
  String get tilemapDisplayMode => 'Modo de visualización';

  @override
  String get tilemapDisplayModeDefault => 'Por defecto';

  @override
  String get tilemapDisplayModeGrayscale => 'Escala de grises';

  @override
  String get tilemapDisplayModeAttributeView => 'Vista de atributos';

  @override
  String get tilemapTileGrid => 'Cuadrícula de azulejos (8×8)';

  @override
  String get tilemapAttrGrid => 'Cuadrícula de atributos (16×16)';

  @override
  String get tilemapAttrGrid32 => 'Attr Grid (32×32)';

  @override
  String get tilemapNtBounds => 'Límites del Nuevo Testamento';

  @override
  String get tilemapScrollOverlay => 'Scroll Overlay';

  @override
  String get tilemapPanelDisplay => 'Mostrar';

  @override
  String get tilemapPanelTilemap => 'Mapa de mosaicos';

  @override
  String get tilemapPanelSelectedTile => 'Azulejo seleccionado';

  @override
  String get tilemapHidePanel => 'Ocultar panel';

  @override
  String get tilemapShowPanel => 'Mostrar panel';

  @override
  String get tilemapInfoSize => 'Tamaño';

  @override
  String get tilemapInfoSizePx => 'Tamaño (px)';

  @override
  String get tilemapInfoTilemapAddress => 'Dirección del mapa de mosaicos';

  @override
  String get tilemapInfoTilesetAddress => 'Dirección del conjunto de mosaicos';

  @override
  String get tilemapInfoMirroring => 'Duplicación';

  @override
  String get tilemapInfoTileFormat => 'Formato de mosaico';

  @override
  String get tilemapInfoTileFormat2bpp => '2 pb';

  @override
  String get tilemapMirroringHorizontal => 'Horizontal';

  @override
  String get tilemapMirroringVertical => 'Vertical';

  @override
  String get tilemapMirroringFourScreen => 'Cuatro pantallas';

  @override
  String get tilemapMirroringSingleScreenLower => 'Pantalla única (inferior)';

  @override
  String get tilemapMirroringSingleScreenUpper => 'Pantalla única (superior)';

  @override
  String get tilemapMirroringMapperControlled => 'controlado por mapeador';

  @override
  String get tilemapLabelColumnRow => 'columna, fila';

  @override
  String get tilemapLabelXY => 'X,Y';

  @override
  String get tilemapLabelSize => 'Tamaño';

  @override
  String get tilemapLabelTilemapAddress => 'Dirección del mapa de mosaicos';

  @override
  String get tilemapLabelTileIndex => 'Índice de mosaicos';

  @override
  String get tilemapLabelTileAddressPpu => 'Dirección de mosaico (PPU)';

  @override
  String get tilemapLabelPaletteIndex => 'Índice de paleta';

  @override
  String get tilemapLabelPaletteAddress => 'Dirección de paleta';

  @override
  String get tilemapLabelAttributeAddress => 'Dirección de atributo';

  @override
  String get tilemapLabelAttributeData => 'Datos de atributos';

  @override
  String get tilemapSelectedTileTilemap => 'Mapa de mosaicos';

  @override
  String get tilemapSelectedTileTileIdx => 'ID de mosaico';

  @override
  String get tilemapSelectedTileTilePpu => 'Azulejo (PPU)';

  @override
  String get tilemapSelectedTilePalette => 'Paleta';

  @override
  String get tilemapSelectedTileAttr => 'Atributo';

  @override
  String get tilemapCapture => 'Captura';

  @override
  String get tilemapCaptureFrameStart => 'Inicio del cuadro';

  @override
  String get tilemapCaptureVblankStart => 'Inicio en blanco';

  @override
  String get tilemapCaptureManual => 'Manual';

  @override
  String get tilemapScanline => 'Línea de exploración';

  @override
  String get tilemapDot => 'Punto';

  @override
  String tilemapError(String error) {
    return 'Error: $error';
  }

  @override
  String get tilemapRetry => 'Reintentar';

  @override
  String get tilemapResetZoom => 'Restablecer zoom';

  @override
  String get menuTilemapViewer => 'Visor de mapas de mosaicos';

  @override
  String get menuTileViewer => 'Tile Viewer';

  @override
  String tileViewerError(String error) {
    return 'Error: $error';
  }

  @override
  String get tileViewerRetry => 'Reintentar';

  @override
  String get tileViewerSettings => 'Configuración del visor de mosaicos';

  @override
  String get tileViewerOverlays => 'Superposiciones';

  @override
  String get tileViewerShowGrid => 'Mostrar cuadrícula de mosaicos';

  @override
  String get tileViewerPalette => 'Paleta';

  @override
  String tileViewerPaletteBg(int index) {
    return 'BG $index';
  }

  @override
  String tileViewerPaletteSprite(int index) {
    return 'Sprite $index';
  }

  @override
  String get tileViewerGrayscale => 'Usar paleta de escala de grises';

  @override
  String get tileViewerSelectedTile => 'Azulejo seleccionado';

  @override
  String get tileViewerPatternTable => 'Pattern Table';

  @override
  String get tileViewerTileIndex => 'Índice de mosaicos';

  @override
  String get tileViewerChrAddress => 'Dirección de la CDH';

  @override
  String get tileViewerClose => 'Cerrar';

  @override
  String get tileViewerSource => 'Fuente';

  @override
  String get tileViewerSourcePpu => 'Memoria PPU';

  @override
  String get tileViewerSourceChrRom => 'ROM CHR';

  @override
  String get tileViewerSourceChrRam => 'RAM CR';

  @override
  String get tileViewerSourcePrgRom => 'ROM PRG';

  @override
  String get tileViewerAddress => 'DIRECCIÓN';

  @override
  String get tileViewerSize => 'Tamaño';

  @override
  String get tileViewerColumns => 'columnas';

  @override
  String get tileViewerRows => 'Filas';

  @override
  String get tileViewerLayout => 'Disposición';

  @override
  String get tileViewerLayoutNormal => 'Normal';

  @override
  String get tileViewerLayout8x16 => 'Duendes 8×16';

  @override
  String get tileViewerLayout16x16 => 'Duendes 16×16';

  @override
  String get tileViewerBackground => 'Fondo';

  @override
  String get tileViewerBgDefault => 'Por defecto';

  @override
  String get tileViewerBgTransparent => 'Transparente';

  @override
  String get tileViewerBgPalette => 'Color de paleta';

  @override
  String get tileViewerBgBlack => 'Negro';

  @override
  String get tileViewerBgWhite => 'Blanco';

  @override
  String get tileViewerBgMagenta => 'Magenta';

  @override
  String get tileViewerPresets => 'Preajustes';

  @override
  String get tileViewerPresetPpu => 'PUP';

  @override
  String get tileViewerPresetChr => 'CDH';

  @override
  String get tileViewerPresetRom => 'memoria de sólo lectura';

  @override
  String get tileViewerPresetBg => 'bg';

  @override
  String get tileViewerPresetOam => 'OAM';

  @override
  String get menuSpriteViewer => 'Visor de sprites';

  @override
  String get menuPaletteViewer => 'Visor de paletas';

  @override
  String get paletteViewerPaletteRamTitle => 'RAM de paleta (32)';

  @override
  String get paletteViewerSystemPaletteTitle => 'Paleta del sistema (64)';

  @override
  String get paletteViewerSettingsTooltip =>
      'Configuración del visor de paletas';

  @override
  String paletteViewerTooltipPaletteRam(String addr, String value) {
    return '$addr = 0x$value';
  }

  @override
  String paletteViewerTooltipSystemIndex(int index) {
    return 'Índice $index';
  }

  @override
  String spriteViewerError(String error) {
    return 'Error del visor de sprites: $error';
  }

  @override
  String get spriteViewerSettingsTooltip =>
      'Configuración del visor de sprites';

  @override
  String get spriteViewerShowGrid => 'Mostrar cuadrícula';

  @override
  String get spriteViewerShowOutline =>
      'Mostrar contorno alrededor de los sprites';

  @override
  String get spriteViewerShowOffscreenRegions =>
      'Mostrar regiones fuera de pantalla';

  @override
  String get spriteViewerDimOffscreenSpritesGrid =>
      'Sprites apagados fuera de la pantalla (cuadrícula)';

  @override
  String get spriteViewerShowListView => 'Mostrar vista de lista';

  @override
  String get spriteViewerPanelSprites => 'duendes';

  @override
  String get spriteViewerPanelDataSource => 'Fuente de datos';

  @override
  String get spriteViewerPanelSprite => 'Duende';

  @override
  String get spriteViewerPanelSelectedSprite => 'objeto seleccionado';

  @override
  String get spriteViewerLabelMode => 'Modo';

  @override
  String get spriteViewerLabelPatternBase => 'Base del patrón';

  @override
  String get spriteViewerLabelThumbnailSize => 'Tamaño de miniatura';

  @override
  String get spriteViewerBgGray => 'Gris';

  @override
  String get spriteViewerDataSourceSpriteRam => 'RAM de sprites';

  @override
  String get spriteViewerDataSourceCpuMemory => 'Memoria de la CPU';

  @override
  String spriteViewerTooltipTitle(int index) {
    return 'Sprite #$index';
  }

  @override
  String get spriteViewerLabelIndex => 'Índice';

  @override
  String get spriteViewerLabelPos => 'Pos.';

  @override
  String get spriteViewerLabelSize => 'Tamaño';

  @override
  String get spriteViewerLabelTile => 'Teja';

  @override
  String get spriteViewerLabelTileAddr => 'Dirección de mosaico';

  @override
  String get spriteViewerLabelPalette => 'Paleta';

  @override
  String get spriteViewerLabelPaletteAddr => 'Dirección de paleta';

  @override
  String get spriteViewerLabelFlip => 'Voltear';

  @override
  String get spriteViewerLabelPriority => 'Prioridad';

  @override
  String get spriteViewerPriorityBehindBg => 'Behind BG';

  @override
  String get spriteViewerPriorityInFront => 'Al frente';

  @override
  String get spriteViewerLabelVisible => 'Visible';

  @override
  String get spriteViewerValueYes => 'Sí';

  @override
  String get spriteViewerValueNoOffscreen => 'No (fuera de pantalla)';

  @override
  String get spriteViewerVisibleStatusVisible => 'Visible';

  @override
  String get spriteViewerVisibleStatusOffscreen => 'Fuera de la pantalla';

  @override
  String get longPressToClear => 'Mantenga pulsado para borrar';

  @override
  String get videoBackendD3D11 => 'GPU D3D11 (copia cero)';

  @override
  String get videoBackendSoftware => 'CPU de software (reserva)';

  @override
  String get netplayBackToSetup => 'Volver a la configuración';

  @override
  String get netplayP2PMode => 'Modo P2P';

  @override
  String get netplaySignalingServer => 'Servidor de señalización';

  @override
  String get netplayRelayServer => 'Servidor de retransmisión (alternativo)';

  @override
  String get netplayP2PRoomCode => 'Código de habitación P2P';

  @override
  String get netplayStartP2PSession => 'Iniciar sesión P2P';

  @override
  String get netplayJoinP2PSession => 'Únase a la sesión P2P';

  @override
  String get netplayInvalidP2PServerAddr =>
      'Dirección de servidor P2P no válida';

  @override
  String get netplayProceed => 'Proceder';

  @override
  String get videoShaderParametersTitle => 'Parámetros del sombreador';

  @override
  String get videoShaderParametersSubtitle =>
      'Ajuste los parámetros del sombreador en tiempo real';

  @override
  String get videoShaderParametersReset => 'Restablecer parámetros';

  @override
  String get searchHint => 'Buscar...';

  @override
  String get searchTooltip => 'Buscar';

  @override
  String get noResults => 'No se encontraron parámetros coincidentes';

  @override
  String get errorFailedToCreateTexture => 'No se pudo crear textura';

  @override
  String get languageJapanese => 'Japonés';

  @override
  String get languageSpanish => 'Español';

  @override
  String get languagePortuguese => 'Portugués';

  @override
  String get languageRussian => 'Ruso';

  @override
  String get languageFrench => 'Francés';

  @override
  String get languageGerman => 'Alemán';
}
