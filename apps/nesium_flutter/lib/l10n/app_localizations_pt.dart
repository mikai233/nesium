// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Portuguese (`pt`).
class AppLocalizationsPt extends AppLocalizations {
  AppLocalizationsPt([String locale = 'pt']) : super(locale);

  @override
  String get settingsTitle => 'Configurações';

  @override
  String get settingsTabGeneral => 'Geral';

  @override
  String get settingsTabInput => 'Entrada';

  @override
  String get settingsTabVideo => 'Vídeo';

  @override
  String get settingsTabEmulation => 'Emulação';

  @override
  String get settingsTabServer => 'Servidor';

  @override
  String get settingsFloatingPreviewToggle => 'Pré-visualização flutuante';

  @override
  String get settingsFloatingPreviewTooltip => 'Mostrar prévia do jogo';

  @override
  String get serverTitle => 'Servidor NetPlay';

  @override
  String get serverPortLabel => 'Porta';

  @override
  String get serverStartButton => 'Iniciar servidor';

  @override
  String get serverStopButton => 'Parar servidor';

  @override
  String get serverStatusRunning => 'Em execução';

  @override
  String get serverStatusStopped => 'Parado';

  @override
  String serverClientCount(int count) {
    return 'Clientes conectados: $count';
  }

  @override
  String serverStartFailed(String error) {
    return 'Falha na inicialização do servidor: $error';
  }

  @override
  String serverStopFailed(String error) {
    return 'Falha na parada do servidor: $error';
  }

  @override
  String serverBindAddress(String address) {
    return 'Endereço de ligação: $address';
  }

  @override
  String serverQuicFingerprint(String fingerprint) {
    return 'Impressão digital QUIC: $fingerprint';
  }

  @override
  String get generalTitle => 'Geral';

  @override
  String get themeLabel => 'Tema';

  @override
  String get themeSystem => 'Sistema';

  @override
  String get themeLight => 'Luz';

  @override
  String get themeDark => 'Escuro';

  @override
  String get languageLabel => 'Linguagem';

  @override
  String get languageSystem => 'Sistema';

  @override
  String get languageEnglish => 'Inglês';

  @override
  String get languageChineseSimplified => 'Chinês simplificado';

  @override
  String get inputTitle => 'Entrada';

  @override
  String get turboTitle => 'Turbo';

  @override
  String get turboLinkPressRelease => 'Imprensa/divulgação do link';

  @override
  String get inputDeviceLabel => 'Dispositivo de entrada';

  @override
  String get inputDeviceKeyboard => 'Teclado';

  @override
  String get inputDeviceGamepad => 'Controle de jogo';

  @override
  String get connectedGamepadsTitle => 'Controles conectados';

  @override
  String get connectedGamepadsNone => 'Nenhum gamepad conectado';

  @override
  String get webGamepadActivationHint =>
      'Limite da web: PRESSIONE QUALQUER BOTÃO no seu gamepad para ativá-lo.';

  @override
  String connectedGamepadsPort(int port) {
    return 'Jogador $port';
  }

  @override
  String get connectedGamepadsUnassigned => 'Não atribuído';

  @override
  String get inputDeviceVirtualController => 'Controlador virtual';

  @override
  String get inputGamepadAssignmentLabel => 'Atribuição de gamepad';

  @override
  String get inputGamepadNone => 'Nenhum/Não atribuído';

  @override
  String get inputListening => 'Ouvindo...';

  @override
  String inputDetected(String buttons) {
    return 'Detectado: $buttons';
  }

  @override
  String get inputGamepadMappingLabel => 'Mapeamento de botões';

  @override
  String get inputResetToDefault => 'Redefinir para o padrão';

  @override
  String get inputButtonA => 'UM';

  @override
  String get inputButtonB => 'B';

  @override
  String get inputButtonTurboA => 'Turbo A';

  @override
  String get inputButtonTurboB => 'Turbo B';

  @override
  String get inputButtonSelect => 'Selecione';

  @override
  String get inputButtonStart => 'Começar';

  @override
  String get inputButtonUp => 'Acima';

  @override
  String get inputButtonDown => 'Abaixo';

  @override
  String get inputButtonLeft => 'Esquerda';

  @override
  String get inputButtonRight => 'Direita';

  @override
  String get inputButtonRewind => 'Retroceder';

  @override
  String get inputButtonFastForward => 'Avanço rápido';

  @override
  String get inputButtonSaveState => 'Salvar estado';

  @override
  String get inputButtonLoadState => 'Carregar estado';

  @override
  String get inputButtonPause => 'Pausa';

  @override
  String get globalHotkeysTitle => 'Teclas de atalho do emulador';

  @override
  String get gamepadHotkeysTitle => 'Teclas de atalho do gamepad (Jogador 1)';

  @override
  String get inputPortLabel => 'Configurar reprodutor';

  @override
  String get player1 => 'Jogador 1';

  @override
  String get player2 => 'Jogador 2';

  @override
  String get player3 => 'Jogador 3';

  @override
  String get player4 => 'Jogador 4';

  @override
  String get keyboardPresetLabel => 'Predefinição de teclado';

  @override
  String get keyboardPresetNone => 'Nenhum';

  @override
  String get keyboardPresetNesStandard => 'Padrão NES';

  @override
  String get keyboardPresetFightStick => 'Arcade Stick';

  @override
  String get keyboardPresetArcadeLayout => 'Distribuição Arcade';

  @override
  String get keyboardPresetCustom => 'Personalizado';

  @override
  String get customKeyBindingsTitle => 'Atalhos de teclas personalizados';

  @override
  String bindKeyTitle(String action) {
    return 'Vincular $action';
  }

  @override
  String get unassignedKey => 'Não atribuído';

  @override
  String get tipPressEscapeToClearBinding =>
      'Dica: pressione Escape para limpar uma ligação.';

  @override
  String get keyboardActionUp => 'Acima';

  @override
  String get keyboardActionDown => 'Abaixo';

  @override
  String get keyboardActionLeft => 'Esquerda';

  @override
  String get keyboardActionRight => 'Direita';

  @override
  String get keyboardActionA => 'UM';

  @override
  String get keyboardActionB => 'B';

  @override
  String get keyboardActionSelect => 'Selecione';

  @override
  String get keyboardActionStart => 'Começar';

  @override
  String get keyboardActionTurboA => 'Turbo A';

  @override
  String get keyboardActionTurboB => 'Turbo B';

  @override
  String get keyboardActionRewind => 'Retroceder';

  @override
  String get keyboardActionFastForward => 'Avanço rápido';

  @override
  String get keyboardActionSaveState => 'Salvar estado';

  @override
  String get keyboardActionLoadState => 'Carregar estado';

  @override
  String get keyboardActionPause => 'Pausa';

  @override
  String get keyboardActionFullScreen => 'Tela cheia';

  @override
  String inputBindingConflictCleared(String player, String action) {
    return 'Ligação $player $action desmarcada.';
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
  String get emulationTitle => 'Emulação';

  @override
  String get integerFpsTitle => 'Modo FPS inteiro (60 Hz, NTSC)';

  @override
  String get integerFpsSubtitle =>
      'Reduz a trepidação da rolagem em monitores de 60 Hz. PAL será adicionado posteriormente.';

  @override
  String get showOverlayTitle => 'Mostrar sobreposição de status';

  @override
  String get showOverlaySubtitle =>
      'Mostrar indicadores de pausa/retrocesso/avanço rápido na tela.';

  @override
  String get pauseInBackgroundTitle => 'Pausa em segundo plano';

  @override
  String get pauseInBackgroundSubtitle =>
      'Pausa automaticamente o emulador quando o aplicativo não está ativo.';

  @override
  String get autoSaveEnabledTitle => 'Salvamento automático';

  @override
  String get autoSaveEnabledSubtitle =>
      'Salve periodicamente o estado do jogo em um slot dedicado.';

  @override
  String get autoSaveIntervalTitle => 'Intervalo de salvamento automático';

  @override
  String autoSaveIntervalValue(int minutes) {
    return '$minutes minutos';
  }

  @override
  String get fastForwardSpeedTitle => 'Velocidade de avanço rápido';

  @override
  String get fastForwardSpeedSubtitle =>
      'Velocidade máxima enquanto o avanço rápido está ativo.';

  @override
  String fastForwardSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get quickSaveSlotTitle => 'Slot de salvamento rápido';

  @override
  String get quickSaveSlotSubtitle =>
      'Slot usado por atalhos rápidos para salvar/carregar.';

  @override
  String quickSaveSlotValue(int index) {
    return 'Slot $index';
  }

  @override
  String get rewindEnabledTitle => 'Retroceder';

  @override
  String get rewindEnabledSubtitle =>
      'Ative a funcionalidade de retrocesso em tempo real.';

  @override
  String get rewindSecondsTitle => 'Duração do retrocesso';

  @override
  String rewindSecondsValue(int seconds) {
    return '$seconds segundos';
  }

  @override
  String get rewindMinutesTitle => 'Duração do retrocesso';

  @override
  String rewindMinutesValue(int minutes) {
    return '$minutes minutos';
  }

  @override
  String get rewindSpeedTitle => 'Velocidade de retrocesso';

  @override
  String get rewindSpeedSubtitle =>
      'A velocidade durante o retrocesso está ativa.';

  @override
  String rewindSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get autoSlotLabel => 'Slot automático';

  @override
  String get menuAutoSave => 'Salvar automaticamente...';

  @override
  String get stateAutoSaved => 'Salvamento automático criado';

  @override
  String get virtualControlsTitle => 'Controles Virtuais';

  @override
  String get virtualControlsSwitchInputTip =>
      'Mude a entrada para \"Controlador virtual\" para usar essas configurações.';

  @override
  String get virtualControlsButtonSize => 'Tamanho do botão';

  @override
  String get virtualControlsGap => 'Brecha';

  @override
  String get virtualControlsOpacity => 'Opacidade';

  @override
  String get virtualControlsHitboxScale => 'Escala de hitbox';

  @override
  String get virtualControlsHapticFeedback => 'Feedback tátil';

  @override
  String get virtualControlsDpadDeadzone => 'Zona morta do D-pad';

  @override
  String get virtualControlsDpadDeadzoneHelp =>
      'Zona morta central: tocar próximo ao centro não acionará nenhuma direção.';

  @override
  String get virtualControlsDpadBoundaryDeadzone =>
      'Zona morta de limite do D-pad';

  @override
  String get virtualControlsDpadBoundaryDeadzoneHelp =>
      'Zona morta de limite: valores mais altos tornam as diagonais mais difíceis de acionar, reduzindo pressionamentos acidentais de vizinhos.';

  @override
  String get virtualControlsReset => 'Redefinir layout';

  @override
  String get virtualControlsDiscardChangesTitle => 'Desfazer alterações';

  @override
  String get virtualControlsDiscardChangesSubtitle =>
      'Reverter para o último layout salvo';

  @override
  String get virtualControlsTurboFramesPerToggle =>
      'Quadros turbo por alternância';

  @override
  String get virtualControlsTurboOnFrames => 'Quadros de prensa turbo';

  @override
  String get virtualControlsTurboOffFrames => 'Quadros de liberação turbo';

  @override
  String framesValue(int frames) {
    return 'Quadros $frames';
  }

  @override
  String get tipAdjustButtonsInDrawer =>
      'Dica: ajuste a posição/tamanho do botão na gaveta do jogo.';

  @override
  String get keyCapturePressKeyToBind => 'Pressione uma tecla para vincular.';

  @override
  String keyCaptureCurrent(String key) {
    return 'Atual: $key';
  }

  @override
  String keyCaptureCaptured(String key) {
    return 'Capturado: $key';
  }

  @override
  String get keyCapturePressEscToClear => 'Pressione Escape para limpar.';

  @override
  String get keyBindingsTitle => 'Atalhos de teclas';

  @override
  String get cancel => 'Cancelar';

  @override
  String get appName => 'Nesium';

  @override
  String get menuTooltip => 'Menu';

  @override
  String get menuSectionFile => 'Arquivo';

  @override
  String get menuSectionEmulation => 'Emulação';

  @override
  String get menuSectionSettings => 'Configurações';

  @override
  String get menuSectionWindows => 'Windows';

  @override
  String get menuSectionHelp => 'Ajuda';

  @override
  String get menuOpenRom => 'Abrir ROM...';

  @override
  String get menuReset => 'Reiniciar';

  @override
  String get menuPowerReset => 'Reinicialização de energia';

  @override
  String get menuEject => 'Desligar';

  @override
  String get menuSaveState => 'Salvar estado...';

  @override
  String get menuLoadState => 'Carregar estado...';

  @override
  String get menuPauseResume => 'Pausar/Retomar';

  @override
  String get menuNetplay => 'Netplay';

  @override
  String get netplayTransportLabel => 'Transporte';

  @override
  String get netplayTransportAuto => 'Automático (QUIC → TCP)';

  @override
  String get netplayTransportUnknown => 'Desconhecido';

  @override
  String get netplayTransportTcp => 'TCP';

  @override
  String get netplayTransportQuic => 'QUIC';

  @override
  String get netplayUsingTcpFallback => 'QUIC falhou, usando TCP';

  @override
  String get netplayStatusDisconnected => 'Desconectado';

  @override
  String get netplayStatusConnecting => 'Conectando...';

  @override
  String get netplayStatusConnected => 'Conectado (Aguardando Sala)';

  @override
  String get netplayStatusInRoom => 'Na sala';

  @override
  String get netplayDisconnect => 'Desconectar';

  @override
  String get netplayServerAddress => 'Endereço do servidor';

  @override
  String get netplayServerNameLabel => 'Nome do servidor (SNI)';

  @override
  String get netplayServerNameHint => 'localhost';

  @override
  String get netplayPlayerName => 'Nome do jogador';

  @override
  String get netplayQuicFingerprintLabel =>
      'Impressão digital do certificado QUIC (opcional)';

  @override
  String get netplayQuicFingerprintHint => 'base64url (43 caracteres)';

  @override
  String get netplayQuicFingerprintHelper =>
      'Insira isto para usar o QUIC fixado. Deixe em branco para usar a confiança do sistema (QUIC) ou fallback para TCP.';

  @override
  String get netplayConnect => 'Junte-se ao jogo';

  @override
  String get netplayJoinViaP2P => 'Junte-se via P2P';

  @override
  String get netplayJoinGame => 'Junte-se ao jogo';

  @override
  String get netplayCreateRoom => 'Criar sala';

  @override
  String get netplayJoinRoom => 'Junte-se ao jogo';

  @override
  String get netplayAddressOrRoomCode =>
      'Código da sala ou endereço do servidor';

  @override
  String get netplayHostingTitle => 'Hospedagem';

  @override
  String get netplayRoomCodeLabel => 'O código do seu quarto';

  @override
  String get netplayP2PEnabled => 'Modo P2P';

  @override
  String get netplayDirectServerLabel => 'Endereço do servidor';

  @override
  String get netplayAdvancedSettings => 'Configurações avançadas de conexão';

  @override
  String get netplayP2PServerLabel => 'Servidor P2P';

  @override
  String get netplayRoomCode => 'Código do quarto';

  @override
  String get netplayRoleLabel => 'Papel';

  @override
  String netplayPlayerIndex(int index) {
    return 'Jogador $index';
  }

  @override
  String get netplaySpectator => 'Espectador';

  @override
  String get netplayClientId => 'ID do cliente';

  @override
  String get netplayPlayerListHeader => 'Jogadores';

  @override
  String get netplayYouIndicator => '(Você)';

  @override
  String get netplayOrSeparator => 'OU';

  @override
  String netplayConnectFailed(String error) {
    return 'Falha na conexão: $error';
  }

  @override
  String netplayDisconnectFailed(String error) {
    return 'Falha na desconexão: $error';
  }

  @override
  String netplayCreateRoomFailed(String error) {
    return 'Falha ao criar sala: $error';
  }

  @override
  String netplayJoinRoomFailed(String error) {
    return 'Falha ao ingressar na sala: $error';
  }

  @override
  String netplaySwitchRoleFailed(String error) {
    return 'Falha na troca de função: $error';
  }

  @override
  String get netplayInvalidRoomCode => 'Código de quarto inválido';

  @override
  String get netplayRomBroadcasted => 'Netplay: ROM transmitida para a sala';

  @override
  String get menuLoadTasMovie => 'Carregar filme TAS...';

  @override
  String get menuPreferences => 'Preferências...';

  @override
  String get saveToExternalFile => 'Salvar em arquivo...';

  @override
  String get loadFromExternalFile => 'Carregar do arquivo...';

  @override
  String get slotLabel => 'Slot';

  @override
  String get slotEmpty => 'Vazio';

  @override
  String get slotHasData => 'Salvo';

  @override
  String stateSavedToSlot(int index) {
    return 'Estado salvo no slot $index';
  }

  @override
  String stateLoadedFromSlot(int index) {
    return 'Estado carregado do slot $index';
  }

  @override
  String slotCleared(int index) {
    return 'Slot $index liberado';
  }

  @override
  String get menuAbout => 'Sobre';

  @override
  String get menuDebugger => 'Depurador';

  @override
  String get menuTools => 'Ferramentas';

  @override
  String get menuOpenDebuggerWindow => 'Abrir janela do depurador';

  @override
  String get menuOpenToolsWindow => 'Abrir janela de ferramentas';

  @override
  String get menuInputMappingComingSoon => 'Mapeamento de entrada (em breve)';

  @override
  String get menuLastError => 'Último erro';

  @override
  String get lastErrorDetailsAction => 'Detalhes';

  @override
  String get lastErrorDialogTitle => 'Último erro';

  @override
  String get lastErrorCopied => 'Copiado';

  @override
  String get copy => 'Cópia';

  @override
  String get paste => 'Colar';

  @override
  String get windowDebuggerTitle => 'Depurador Nesium';

  @override
  String get windowToolsTitle => 'Ferramentas Nesium';

  @override
  String get virtualControlsEditTitle => 'Editar controles virtuais';

  @override
  String get virtualControlsEditSubtitleEnabled =>
      'Arraste para mover, aperte ou arraste o canto para redimensionar';

  @override
  String get virtualControlsEditSubtitleDisabled => 'Ativar ajuste interativo';

  @override
  String get gridSnappingTitle => 'Ajuste de grade';

  @override
  String get gridSpacingLabel => 'Espaçamento da grade';

  @override
  String get debuggerPlaceholderBody =>
      'Espaço reservado para monitores CPU/PPU, visualizadores de memória e inspetores OAM. Os mesmos widgets podem ficar em um painel lateral da área de trabalho ou em uma planilha móvel.';

  @override
  String get toolsPlaceholderBody =>
      'Gravação/reprodução, mapeamento de entrada e cheats podem compartilhar esses widgets entre os painéis laterais da área de trabalho e as folhas inferiores móveis.';

  @override
  String get actionLoadRom => 'Carregar ROM';

  @override
  String get actionResetNes => 'Redefinir NES';

  @override
  String get actionPowerResetNes => 'Reinicialização de energia NES';

  @override
  String get actionEjectNes => 'Desligar';

  @override
  String get actionLoadPalette => 'Carregar paleta';

  @override
  String get videoResetToDefault => 'Redefinir para o padrão';

  @override
  String get videoTitle => 'Vídeo';

  @override
  String get videoFilterLabel => 'Filtro de vídeo';

  @override
  String get videoFilterCategoryCpu => 'Filtros de CPU';

  @override
  String get videoFilterCategoryGpu => 'Filtros GPU (Shaders)';

  @override
  String get videoFilterNone => 'Nenhum (1x)';

  @override
  String get videoFilterPrescale2x => 'Pré-escalar 2x';

  @override
  String get videoFilterPrescale3x => 'Pré-escalar 3x';

  @override
  String get videoFilterPrescale4x => 'Pré-escalar 4x';

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
  String get videoFilterSuperEagle => 'Super Águia';

  @override
  String get videoFilterLcdGrid => 'Grade LCD (2x)';

  @override
  String get videoFilterScanlines => 'Linhas de varredura (2x)';

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
  String get videoLcdGridStrengthLabel => 'Força da grade LCD';

  @override
  String get videoScanlinesIntensityLabel =>
      'Intensidade da linha de varredura';

  @override
  String get videoFilterNtscComposite => 'NTSC (composto)';

  @override
  String get videoFilterNtscSvideo => 'NTSC (S-Vídeo)';

  @override
  String get videoFilterNtscRgb => 'NTSC (RGB)';

  @override
  String get videoFilterNtscMonochrome => 'NTSC (Monocromático)';

  @override
  String get videoFilterNtscBisqwit2x => 'NTSC (Bisqwit) 2x';

  @override
  String get videoFilterNtscBisqwit4x => 'NTSC (Bisqwit) 4x';

  @override
  String get videoFilterNtscBisqwit8x => 'NTSC (Bisqwit) 8x';

  @override
  String get videoNtscAdvancedTitle => 'NTSC Avançado';

  @override
  String get videoNtscMergeFieldsLabel => 'Mesclar campos (reduzir cintilação)';

  @override
  String get videoNtscHueLabel => 'Matiz';

  @override
  String get videoNtscSaturationLabel => 'Saturação';

  @override
  String get videoNtscContrastLabel => 'Contraste';

  @override
  String get videoNtscBrightnessLabel => 'Brilho';

  @override
  String get videoNtscSharpnessLabel => 'Nitidez';

  @override
  String get videoNtscGammaLabel => 'Gama';

  @override
  String get videoNtscResolutionLabel => 'Resolução';

  @override
  String get videoNtscArtifactsLabel => 'Artefatos';

  @override
  String get videoNtscFringingLabel => 'Franjas';

  @override
  String get videoNtscBleedLabel => 'Sangramento';

  @override
  String get videoNtscBisqwitSettingsTitle => 'Configurações NTSC (Bisqwit)';

  @override
  String get videoNtscBisqwitYFilterLengthLabel =>
      'Filtro Y (desfoque horizontal)';

  @override
  String get videoNtscBisqwitIFilterLengthLabel => 'Eu filtro';

  @override
  String get videoNtscBisqwitQFilterLengthLabel => 'Filtro Q';

  @override
  String get videoIntegerScalingTitle => 'Escala inteira';

  @override
  String get videoIntegerScalingSubtitle =>
      'Dimensionamento perfeito de pixels (reduz o brilho na rolagem).';

  @override
  String get videoFullScreenTitle => 'Tela cheia';

  @override
  String get videoFullScreenSubtitle =>
      'Alternar o estado de tela cheia da janela';

  @override
  String get videoScreenVerticalOffset => 'Deslocamento vertical da tela';

  @override
  String get videoScreenVerticalOffsetPortraitOnly =>
      'Só entra em vigor no modo retrato.';

  @override
  String get videoAspectRatio => 'Proporção de aspecto';

  @override
  String get videoAspectRatioSquare => '1:1 (pixels quadrados)';

  @override
  String get videoAspectRatioNtsc => '4:3 (NTSC)';

  @override
  String get videoAspectRatioStretch => 'Esticar';

  @override
  String get videoShaderLibrashaderTitle => 'Sombreadores RetroArch';

  @override
  String get videoShaderLibrashaderSubtitle =>
      'Requer GLES3 + back-end de hardware (swapchain AHB).';

  @override
  String get videoShaderLibrashaderSubtitleWindows =>
      'Requer back-end de GPU D3D11.';

  @override
  String get videoShaderLibrashaderSubtitleApple => 'Requer back-end de Metal.';

  @override
  String get videoShaderLibrashaderSubtitleDisabled =>
      'Mude o back-end do Android para Hardware para ativar.';

  @override
  String get videoShaderLibrashaderSubtitleDisabledWindows =>
      'Mude o back-end do Windows para GPU D3D11 para ativar.';

  @override
  String get videoShaderPresetLabel => 'Predefinição (.slangp)';

  @override
  String get videoShaderPresetNotSet => 'Não definido';

  @override
  String get shaderBrowserTitle => 'Sombreadores';

  @override
  String get shaderBrowserNoShaders => 'Nenhum sombreador encontrado';

  @override
  String shaderBrowserError(String error) {
    return 'Erro: $error';
  }

  @override
  String get aboutTitle => 'Sobre Nesium';

  @override
  String get aboutLead =>
      'Nesium: Frontend do emulador Rust NES/FC construído em nesium-core.';

  @override
  String get aboutIntro =>
      'Este frontend Flutter reutiliza o núcleo Rust para emulação. A construção da web é executada no navegador via Flutter Web + Web Worker + WASM.';

  @override
  String get aboutLinksHeading => 'Ligações';

  @override
  String get aboutGitHubLabel => 'GitHub';

  @override
  String get aboutWebDemoLabel => 'Demonstração na Web';

  @override
  String get aboutComponentsHeading => 'Componentes de código aberto';

  @override
  String get aboutComponentsHint =>
      'Toque para abrir, pressione e segure para copiar.';

  @override
  String get aboutLicenseHeading => 'Licença';

  @override
  String get aboutLicenseBody =>
      'Nesium é licenciado sob GPL-3.0 ou posterior. Consulte LICENSE.md na raiz do repositório.';

  @override
  String aboutLaunchFailed(String url) {
    return 'Não foi possível iniciar: $url';
  }

  @override
  String get videoBackendLabel => 'Back-end do renderizador';

  @override
  String get videoBackendAndroidLabel => 'Back-end do renderizador Android';

  @override
  String get videoBackendWindowsLabel => 'Back-end do renderizador do Windows';

  @override
  String get videoBackendHardware => 'Hardware (AHardwareBuffer)';

  @override
  String get videoBackendUpload => 'Compatibilidade (upload de CPU)';

  @override
  String get videoBackendRestartHint =>
      'Entra em vigor após a reinicialização do aplicativo.';

  @override
  String videoBackendCurrent(String backend) {
    return 'Back-end atual: $backend';
  }

  @override
  String get windowsNativeOverlayTitle =>
      'Sobreposição nativa do Windows (experimental)';

  @override
  String get windowsNativeOverlaySubtitle =>
      'Ignora o compositor Flutter para uma suavidade perfeita. Desativa shaders e sobreposições de UI por trás do jogo.';

  @override
  String get highPerformanceModeLabel => 'Modo de alto desempenho';

  @override
  String get highPerformanceModeDescription =>
      'Eleve a prioridade do processo e otimize o agendador para uma jogabilidade mais tranquila.';

  @override
  String get videoLowLatencyTitle => 'Vídeo de baixa latência';

  @override
  String get videoLowLatencySubtitle =>
      'Sincronize a emulação e o renderizador para reduzir o jitter. Entra em vigor após a reinicialização do aplicativo.';

  @override
  String get paletteModeLabel => 'Paleta';

  @override
  String get paletteModeBuiltin => 'Integrado';

  @override
  String get paletteModeCustom => 'Personalizado…';

  @override
  String paletteModeCustomActive(String name) {
    return 'Personalizado ($name)';
  }

  @override
  String get builtinPaletteLabel => 'Paleta integrada';

  @override
  String get customPaletteLoadTitle => 'Carregar arquivo de paleta (.pal)…';

  @override
  String get customPaletteLoadSubtitle => '192 bytes (RGB) ou 256 bytes (RGBA)';

  @override
  String commandSucceeded(String label) {
    return '$label teve sucesso';
  }

  @override
  String commandFailed(String label) {
    return '$label falhou';
  }

  @override
  String get snackPaused => 'Pausado';

  @override
  String get snackResumed => 'Retomada';

  @override
  String snackPauseFailed(String error) {
    return 'Falha na pausa: $error';
  }

  @override
  String get dialogOk => 'OK';

  @override
  String get debuggerNoRomTitle => 'Nenhuma ROM em execução';

  @override
  String get debuggerNoRomSubtitle =>
      'Carregue uma ROM para ver o estado de depuração';

  @override
  String get debuggerCpuRegisters => 'Registros de CPU';

  @override
  String get debuggerPpuState => 'Estado da UPU';

  @override
  String get debuggerCpuStatusTooltip =>
      'Registro de status da CPU (P)\nN: Negativo - definido se o bit de resultado 7 estiver definido\nV: Overflow - definido como overflow assinado\nB: Break - definido pela instrução BRK\nD: Decimal - modo BCD (ignorado no NES)\nI: Interrupção Desativada - bloqueia IRQ\nZ: Zero - definido se o resultado for zero\nC: Carry - definido para overflow não assinado\n\nMaiúsculas = definir, minúsculas = limpar';

  @override
  String get debuggerPpuCtrlTooltip =>
      'Registro de controle PPU (US\$ 2.000)\nV: NMI habilitado\nP: PPU mestre/escravo (não utilizado)\nH: Altura do Sprite (0=8x8, 1=8x16)\nB: Endereço da tabela de padrão de fundo\nS: endereço da tabela de padrões Sprite\nI: incremento de endereço VRAM (0=1, 1=32)\nNN: Endereço base da tabela de nomes\n\nMaiúsculas = definir, minúsculas = limpar';

  @override
  String get debuggerPpuMaskTooltip =>
      'Registro de máscara PPU (\$ 2.001)\nBGR: bits de ênfase de cor\ns: Mostrar sprites\nb: Mostrar plano de fundo\nM: Mostrar sprites nos 8 pixels mais à esquerda\nm: Mostra o fundo nos 8 pixels mais à esquerda\ng: Escala de cinza\n\nMaiúsculas = definir, minúsculas = limpar';

  @override
  String get debuggerPpuStatusTooltip =>
      'Registro de status PPU (\$ 2002)\nV: VBlank começou\nS: Sprite 0 acerto\nO: Estouro de Sprite\n\nMaiúsculas = definir, minúsculas = limpar';

  @override
  String get debuggerScanlineTooltip =>
      'Números da linha de varredura:\n0-239: Visível (Renderização)\n240: Pós-renderização (inativo)\n241-260: VBlank (Supressão Vertical)\n-1: Pré-renderização (linha de varredura fictícia)';

  @override
  String get tilemapSettings => 'Configurações';

  @override
  String get tilemapOverlay => 'Sobreposição';

  @override
  String get tilemapDisplayMode => 'Modo de exibição';

  @override
  String get tilemapDisplayModeDefault => 'Padrão';

  @override
  String get tilemapDisplayModeGrayscale => 'Tons de cinza';

  @override
  String get tilemapDisplayModeAttributeView => 'Visualização de atributos';

  @override
  String get tilemapTileGrid => 'Grade de azulejos (8×8)';

  @override
  String get tilemapAttrGrid => 'Grade de atributos (16×16)';

  @override
  String get tilemapAttrGrid32 => 'Grade de atributos (32×32)';

  @override
  String get tilemapNtBounds => 'Limites do NT';

  @override
  String get tilemapScrollOverlay => 'Sobreposição de rolagem';

  @override
  String get tilemapPanelDisplay => 'Mostrar';

  @override
  String get tilemapPanelTilemap => 'Mapa de blocos';

  @override
  String get tilemapPanelSelectedTile => 'Bloco selecionado';

  @override
  String get tilemapHidePanel => 'Ocultar painel';

  @override
  String get tilemapShowPanel => 'Mostrar painel';

  @override
  String get tilemapInfoSize => 'Tamanho';

  @override
  String get tilemapInfoSizePx => 'Tamanho (px)';

  @override
  String get tilemapInfoTilemapAddress => 'Endereço do mapa de blocos';

  @override
  String get tilemapInfoTilesetAddress => 'Endereço do conjunto de blocos';

  @override
  String get tilemapInfoMirroring => 'Espelhamento';

  @override
  String get tilemapInfoTileFormat => 'Formato de bloco';

  @override
  String get tilemapInfoTileFormat2bpp => '2 pb';

  @override
  String get tilemapMirroringHorizontal => 'Horizontal';

  @override
  String get tilemapMirroringVertical => 'Vertical';

  @override
  String get tilemapMirroringFourScreen => 'Quatro telas';

  @override
  String get tilemapMirroringSingleScreenLower => 'Tela única (inferior)';

  @override
  String get tilemapMirroringSingleScreenUpper => 'Tela única (superior)';

  @override
  String get tilemapMirroringMapperControlled => 'Controlado por mapeador';

  @override
  String get tilemapLabelColumnRow => 'Coluna, Linha';

  @override
  String get tilemapLabelXY => 'X, Y';

  @override
  String get tilemapLabelSize => 'Tamanho';

  @override
  String get tilemapLabelTilemapAddress => 'Endereço do mapa de blocos';

  @override
  String get tilemapLabelTileIndex => 'Índice de blocos';

  @override
  String get tilemapLabelTileAddressPpu => 'Endereço do bloco (PPU)';

  @override
  String get tilemapLabelPaletteIndex => 'Índice de paleta';

  @override
  String get tilemapLabelPaletteAddress => 'Endereço da paleta';

  @override
  String get tilemapLabelAttributeAddress => 'Endereço de atributo';

  @override
  String get tilemapLabelAttributeData => 'Dados de atributos';

  @override
  String get tilemapSelectedTileTilemap => 'Mapa de blocos';

  @override
  String get tilemapSelectedTileTileIdx => 'Bloco idx';

  @override
  String get tilemapSelectedTileTilePpu => 'Bloco (PPU)';

  @override
  String get tilemapSelectedTilePalette => 'Paleta';

  @override
  String get tilemapSelectedTileAttr => 'Atributo';

  @override
  String get tilemapCapture => 'Capturar';

  @override
  String get tilemapCaptureFrameStart => 'Início do quadro';

  @override
  String get tilemapCaptureVblankStart => 'Início em branco';

  @override
  String get tilemapCaptureManual => 'Manual';

  @override
  String get tilemapScanline => 'Linha de varredura';

  @override
  String get tilemapDot => 'Ponto';

  @override
  String tilemapError(String error) {
    return 'Erro: $error';
  }

  @override
  String get tilemapRetry => 'Tentar novamente';

  @override
  String get tilemapResetZoom => 'Redefinir zoom';

  @override
  String get menuTilemapViewer => 'Visualizador de mapa de blocos';

  @override
  String get menuTileViewer => 'Visualizador de blocos';

  @override
  String tileViewerError(String error) {
    return 'Erro: $error';
  }

  @override
  String get tileViewerRetry => 'Tentar novamente';

  @override
  String get tileViewerSettings => 'Configurações do visualizador de blocos';

  @override
  String get tileViewerOverlays => 'Sobreposições';

  @override
  String get tileViewerShowGrid => 'Mostrar grade de blocos';

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
  String get tileViewerGrayscale => 'Usar paleta de tons de cinza';

  @override
  String get tileViewerSelectedTile => 'Bloco selecionado';

  @override
  String get tileViewerPatternTable => 'Tabela de padrões';

  @override
  String get tileViewerTileIndex => 'Índice de blocos';

  @override
  String get tileViewerChrAddress => 'Endereço CHR';

  @override
  String get tileViewerClose => 'Fechar';

  @override
  String get tileViewerSource => 'Fonte';

  @override
  String get tileViewerSourcePpu => 'Memória PPU';

  @override
  String get tileViewerSourceChrRom => 'ROM CHR';

  @override
  String get tileViewerSourceChrRam => 'RAM CHR';

  @override
  String get tileViewerSourcePrgRom => 'ROM PRG';

  @override
  String get tileViewerAddress => 'Endereço';

  @override
  String get tileViewerSize => 'Tamanho';

  @override
  String get tileViewerColumns => 'Colunas';

  @override
  String get tileViewerRows => 'Linhas';

  @override
  String get tileViewerLayout => 'Disposição';

  @override
  String get tileViewerLayoutNormal => 'Normal';

  @override
  String get tileViewerLayout8x16 => 'Sprites 8x16';

  @override
  String get tileViewerLayout16x16 => 'Sprites 16×16';

  @override
  String get tileViewerBackground => 'Fundo';

  @override
  String get tileViewerBgDefault => 'Padrão';

  @override
  String get tileViewerBgTransparent => 'Transparente';

  @override
  String get tileViewerBgPalette => 'Cor da paleta';

  @override
  String get tileViewerBgBlack => 'Preto';

  @override
  String get tileViewerBgWhite => 'Branco';

  @override
  String get tileViewerBgMagenta => 'Magenta';

  @override
  String get tileViewerPresets => 'Predefinições';

  @override
  String get tileViewerPresetPpu => 'UPU';

  @override
  String get tileViewerPresetChr => 'CDH';

  @override
  String get tileViewerPresetRom => 'ROM';

  @override
  String get tileViewerPresetBg => 'GB';

  @override
  String get tileViewerPresetOam => 'OAM';

  @override
  String get menuSpriteViewer => 'Visualizador de Sprites';

  @override
  String get menuPaletteViewer => 'Visualizador de paleta';

  @override
  String get paletteViewerPaletteRamTitle => 'Paleta RAM (32)';

  @override
  String get paletteViewerSystemPaletteTitle => 'Paleta do Sistema (64)';

  @override
  String get paletteViewerSettingsTooltip =>
      'Configurações do visualizador de paleta';

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
    return 'Erro do visualizador de Sprite: $error';
  }

  @override
  String get spriteViewerSettingsTooltip =>
      'Configurações do visualizador de Sprite';

  @override
  String get spriteViewerShowGrid => 'Mostrar grade';

  @override
  String get spriteViewerShowOutline => 'Mostrar contorno em torno dos sprites';

  @override
  String get spriteViewerShowOffscreenRegions => 'Mostrar regiões fora da tela';

  @override
  String get spriteViewerDimOffscreenSpritesGrid =>
      'Escurecer sprites fora da tela (grade)';

  @override
  String get spriteViewerShowListView => 'Mostrar visualização de lista';

  @override
  String get spriteViewerPanelSprites => 'Sprites';

  @override
  String get spriteViewerPanelDataSource => 'Fonte de dados';

  @override
  String get spriteViewerPanelSprite => 'Sprite';

  @override
  String get spriteViewerPanelSelectedSprite => 'ator selecionado';

  @override
  String get spriteViewerLabelMode => 'Modo';

  @override
  String get spriteViewerLabelPatternBase => 'Base padrão';

  @override
  String get spriteViewerLabelThumbnailSize => 'Tamanho da miniatura';

  @override
  String get spriteViewerBgGray => 'Cinza';

  @override
  String get spriteViewerDataSourceSpriteRam => 'Sprite RAM';

  @override
  String get spriteViewerDataSourceCpuMemory => 'Memória CPU';

  @override
  String spriteViewerTooltipTitle(int index) {
    return 'Sprite#$index';
  }

  @override
  String get spriteViewerLabelIndex => 'Índice';

  @override
  String get spriteViewerLabelPos => 'Posição';

  @override
  String get spriteViewerLabelSize => 'Tamanho';

  @override
  String get spriteViewerLabelTile => 'Telha';

  @override
  String get spriteViewerLabelTileAddr => 'Endereço do bloco';

  @override
  String get spriteViewerLabelPalette => 'Paleta';

  @override
  String get spriteViewerLabelPaletteAddr => 'Endereço da paleta';

  @override
  String get spriteViewerLabelFlip => 'Virar';

  @override
  String get spriteViewerLabelPriority => 'Prioridade';

  @override
  String get spriteViewerPriorityBehindBg => 'Atrás de BG';

  @override
  String get spriteViewerPriorityInFront => 'Na frente';

  @override
  String get spriteViewerLabelVisible => 'Visível';

  @override
  String get spriteViewerValueYes => 'Sim';

  @override
  String get spriteViewerValueNoOffscreen => 'Não (fora da tela)';

  @override
  String get spriteViewerVisibleStatusVisible => 'Visível';

  @override
  String get spriteViewerVisibleStatusOffscreen => 'Fora da tela';

  @override
  String get longPressToClear => 'Pressione longamente para limpar';

  @override
  String get videoBackendD3D11 => 'GPU D3D11 (cópia zero)';

  @override
  String get videoBackendSoftware => 'CPU de software (substituição)';

  @override
  String get netplayBackToSetup => 'Voltar à configuração';

  @override
  String get netplayP2PMode => 'Modo P2P';

  @override
  String get netplaySignalingServer => 'Servidor de sinalização';

  @override
  String get netplayRelayServer => 'Servidor de retransmissão (substituto)';

  @override
  String get netplayP2PRoomCode => 'Código da sala P2P';

  @override
  String get netplayStartP2PSession => 'Iniciar sessão P2P';

  @override
  String get netplayJoinP2PSession => 'Junte-se à sessão P2P';

  @override
  String get netplayInvalidP2PServerAddr => 'Endereço de servidor P2P inválido';

  @override
  String get netplayProceed => 'Prosseguir';

  @override
  String get videoShaderParametersTitle => 'Parâmetros de sombreador';

  @override
  String get videoShaderParametersSubtitle =>
      'Ajuste os parâmetros do shader em tempo real';

  @override
  String get videoShaderParametersReset => 'Redefinir parâmetros';

  @override
  String get searchHint => 'Procurar...';

  @override
  String get searchTooltip => 'Procurar';

  @override
  String get noResults => 'Nenhum parâmetro correspondente encontrado';

  @override
  String get errorFailedToCreateTexture => 'Falha ao criar textura';

  @override
  String get languageJapanese => 'Japonês';

  @override
  String get languageSpanish => 'Espanhol';

  @override
  String get languagePortuguese => 'Português';

  @override
  String get languageRussian => 'Russo';

  @override
  String get languageFrench => 'Francês';

  @override
  String get languageGerman => 'Alemão';
}
