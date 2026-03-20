// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Japanese (`ja`).
class AppLocalizationsJa extends AppLocalizations {
  AppLocalizationsJa([String locale = 'ja']) : super(locale);

  @override
  String get settingsTitle => '設定';

  @override
  String get settingsTabGeneral => '全般';

  @override
  String get settingsTabInput => '入力';

  @override
  String get settingsTabVideo => 'ビデオ';

  @override
  String get settingsTabEmulation => 'エミュレーション';

  @override
  String get settingsTabServer => 'サーバ';

  @override
  String get settingsFloatingPreviewToggle => 'フローティングプレビュー';

  @override
  String get settingsFloatingPreviewTooltip => 'ゲームのプレビューを表示する';

  @override
  String get serverTitle => 'ネットプレイサーバー';

  @override
  String get serverPortLabel => 'ポート';

  @override
  String get serverStartButton => 'サーバーの起動';

  @override
  String get serverStopButton => 'サーバーを停止する';

  @override
  String get serverStatusRunning => 'ランニング';

  @override
  String get serverStatusStopped => '停止しました';

  @override
  String serverClientCount(int count) {
    return '接続されているクライアント: $count';
  }

  @override
  String serverStartFailed(String error) {
    return 'サーバーの起動に失敗しました: $error';
  }

  @override
  String serverStopFailed(String error) {
    return 'サーバーの停止に失敗しました: $error';
  }

  @override
  String serverBindAddress(String address) {
    return 'バインドアドレス: $address';
  }

  @override
  String serverQuicFingerprint(String fingerprint) {
    return 'QUIC フィンガープリント: $fingerprint';
  }

  @override
  String get generalTitle => '全般';

  @override
  String get themeLabel => 'テーマ';

  @override
  String get themeSystem => 'システム';

  @override
  String get themeLight => 'ライト';

  @override
  String get themeDark => 'ダーク';

  @override
  String get languageLabel => '言語';

  @override
  String get languageSystem => 'システム';

  @override
  String get languageEnglish => '英語';

  @override
  String get languageChineseSimplified => '簡体字中国語';

  @override
  String get inputTitle => '入力';

  @override
  String get turboTitle => 'ターボ';

  @override
  String get turboLinkPressRelease => 'プレス/リリースをリンク';

  @override
  String get inputDeviceLabel => '入力デバイス';

  @override
  String get inputDeviceKeyboard => 'キーボード';

  @override
  String get inputDeviceGamepad => 'ゲームパッド';

  @override
  String get connectedGamepadsTitle => '接続されたゲームパッド';

  @override
  String get connectedGamepadsNone => 'ゲームパッドが接続されていません';

  @override
  String get webGamepadActivationHint => 'ウェブ制限: ゲームパッドのいずれかのボタンを押して有効にします。';

  @override
  String connectedGamepadsPort(int port) {
    return 'プレイヤー $port';
  }

  @override
  String get connectedGamepadsUnassigned => '未割り当て';

  @override
  String get inputDeviceVirtualController => '仮想コントローラー';

  @override
  String get inputGamepadAssignmentLabel => 'ゲームパッドの割り当て';

  @override
  String get inputGamepadNone => 'なし/未割り当て';

  @override
  String get inputListening => 'リスニング...';

  @override
  String inputDetected(String buttons) {
    return '検出されました: $buttons';
  }

  @override
  String get inputGamepadMappingLabel => 'ボタンのマッピング';

  @override
  String get inputResetToDefault => 'デフォルトにリセット';

  @override
  String get inputButtonA => 'A';

  @override
  String get inputButtonB => 'B';

  @override
  String get inputButtonTurboA => 'ターボA';

  @override
  String get inputButtonTurboB => 'ターボB';

  @override
  String get inputButtonSelect => '選択';

  @override
  String get inputButtonStart => 'スタート';

  @override
  String get inputButtonUp => '上';

  @override
  String get inputButtonDown => '下';

  @override
  String get inputButtonLeft => '左';

  @override
  String get inputButtonRight => '右';

  @override
  String get inputButtonRewind => '巻き戻し';

  @override
  String get inputButtonFastForward => '早送り';

  @override
  String get inputButtonSaveState => 'ステート保存';

  @override
  String get inputButtonLoadState => 'ステートロード';

  @override
  String get inputButtonPause => '一時停止';

  @override
  String get globalHotkeysTitle => 'エミュレータのホットキー';

  @override
  String get gamepadHotkeysTitle => 'ゲームパッドのホットキー (プレイヤー 1)';

  @override
  String get inputPortLabel => 'プレーヤーの設定';

  @override
  String get player1 => 'プレイヤー 1';

  @override
  String get player2 => 'プレイヤー2';

  @override
  String get player3 => 'プレイヤー3';

  @override
  String get player4 => 'プレイヤー 4';

  @override
  String get keyboardPresetLabel => 'キーボードプリセット';

  @override
  String get keyboardPresetNone => 'なし';

  @override
  String get keyboardPresetNesStandard => 'ファミコン規格';

  @override
  String get keyboardPresetFightStick => 'ファイトスティック';

  @override
  String get keyboardPresetArcadeLayout => 'アーケードのレイアウト';

  @override
  String get keyboardPresetCustom => 'カスタム';

  @override
  String get customKeyBindingsTitle => 'カスタムキーバインディング';

  @override
  String bindKeyTitle(String action) {
    return '$actionをバインドする';
  }

  @override
  String get unassignedKey => '未割り当て';

  @override
  String get tipPressEscapeToClearBinding =>
      'ヒント: バインディングをクリアするには Esc キーを押します。';

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
  String get keyboardActionSelect => '選択';

  @override
  String get keyboardActionStart => 'スタート';

  @override
  String get keyboardActionTurboA => 'ターボA';

  @override
  String get keyboardActionTurboB => 'ターボB';

  @override
  String get keyboardActionRewind => '巻き戻し';

  @override
  String get keyboardActionFastForward => '早送り';

  @override
  String get keyboardActionSaveState => 'ステート保存';

  @override
  String get keyboardActionLoadState => 'ステートロード';

  @override
  String get keyboardActionPause => '一時停止';

  @override
  String get keyboardActionFullScreen => '全画面表示';

  @override
  String inputBindingConflictCleared(String player, String action) {
    return '$player $action バインディングがクリアされました。';
  }

  @override
  String inputBindingConflictHint(String player, String action) {
    return '($player - $action)';
  }

  @override
  String inputBindingCapturedConflictHint(String player, String action) {
    return '$player - $action が占有';
  }

  @override
  String get emulationTitle => 'エミュレーション';

  @override
  String get integerFpsTitle => '整数 FPS モード (60Hz、NTSC)';

  @override
  String get integerFpsSubtitle =>
      '60Hz ディスプレイでのスクロールのジャダーを軽減します。 PALは今後追加される予定です。';

  @override
  String get showOverlayTitle => 'ステータスオーバーレイを表示';

  @override
  String get showOverlaySubtitle => '画面上に一時停止/巻き戻し/早送りインジケーターを表示します。';

  @override
  String get pauseInBackgroundTitle => 'バックグラウンドで一時停止する';

  @override
  String get pauseInBackgroundSubtitle => 'アプリがアクティブでない場合、エミュレーターを自動的に一時停止します。';

  @override
  String get autoSaveEnabledTitle => '自動保存';

  @override
  String get autoSaveEnabledSubtitle => 'ゲームの状態を定期的に専用スロットに保存します。';

  @override
  String get autoSaveIntervalTitle => '自動保存間隔';

  @override
  String autoSaveIntervalValue(int minutes) {
    return '$minutes 分';
  }

  @override
  String get fastForwardSpeedTitle => '早送り速度';

  @override
  String get fastForwardSpeedSubtitle => '早送り中の最大速度。';

  @override
  String fastForwardSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get quickSaveSlotTitle => 'クイックセーブスロット';

  @override
  String get quickSaveSlotSubtitle => 'クイック セーブ/ロード ショートカットで使用されるスロット。';

  @override
  String quickSaveSlotValue(int index) {
    return 'スロット $index';
  }

  @override
  String get rewindEnabledTitle => '巻き戻し';

  @override
  String get rewindEnabledSubtitle => 'リアルタイム巻き戻し機能を有効にします。';

  @override
  String get rewindSecondsTitle => '巻き戻し期間';

  @override
  String rewindSecondsValue(int seconds) {
    return '$seconds 秒';
  }

  @override
  String get rewindMinutesTitle => '巻き戻し期間';

  @override
  String rewindMinutesValue(int minutes) {
    return '$minutes 分';
  }

  @override
  String get rewindSpeedTitle => '巻き戻し速度';

  @override
  String get rewindSpeedSubtitle => '巻き戻し中の速度。';

  @override
  String rewindSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get autoSlotLabel => 'オートスロット';

  @override
  String get menuAutoSave => '自動保存...';

  @override
  String get stateAutoSaved => '自動保存が作成されました';

  @override
  String get virtualControlsTitle => '仮想コントロール';

  @override
  String get virtualControlsSwitchInputTip =>
      'これらの設定を使用するには、入力を「仮想コントローラー」に切り替えます。';

  @override
  String get virtualControlsButtonSize => 'ボタンのサイズ';

  @override
  String get virtualControlsGap => 'ギャップ';

  @override
  String get virtualControlsOpacity => '不透明度';

  @override
  String get virtualControlsHitboxScale => 'ヒットボックススケール';

  @override
  String get virtualControlsHapticFeedback => '触覚フィードバック';

  @override
  String get virtualControlsDpadDeadzone => '十字キーのデッドゾーン';

  @override
  String get virtualControlsDpadDeadzoneHelp =>
      '中央のデッドゾーン: 中央付近に触れても、どの方向にもトリガーされません。';

  @override
  String get virtualControlsDpadBoundaryDeadzone => 'D-パッド境界デッドゾーン';

  @override
  String get virtualControlsDpadBoundaryDeadzoneHelp =>
      '境界デッドゾーン: 値を大きくすると、対角線がトリガーされにくくなり、誤って隣接するボタンが押されることが少なくなります。';

  @override
  String get virtualControlsReset => 'レイアウトをリセットする';

  @override
  String get virtualControlsDiscardChangesTitle => '変更を元に戻す';

  @override
  String get virtualControlsDiscardChangesSubtitle => '最後に保存したレイアウトに戻す';

  @override
  String get virtualControlsTurboFramesPerToggle => 'トグルごとのターボ フレーム';

  @override
  String get virtualControlsTurboOnFrames => 'ターボプレスフレーム';

  @override
  String get virtualControlsTurboOffFrames => 'ターボリリースフレーム';

  @override
  String framesValue(int frames) {
    return '$frames フレーム';
  }

  @override
  String get tipAdjustButtonsInDrawer => 'ヒント: ゲーム内のドロワーからボタンの位置/サイズを調整します。';

  @override
  String get keyCapturePressKeyToBind => 'キーを押してバインドします。';

  @override
  String keyCaptureCurrent(String key) {
    return '現在: $key';
  }

  @override
  String keyCaptureCaptured(String key) {
    return 'キャプチャ: $key';
  }

  @override
  String get keyCapturePressEscToClear => 'Esc キーを押してクリアします。';

  @override
  String get keyBindingsTitle => 'キーバインディング';

  @override
  String get cancel => 'キャンセル';

  @override
  String get appName => 'Nesium';

  @override
  String get menuTooltip => 'メニュー';

  @override
  String get menuSectionFile => 'ファイル';

  @override
  String get menuSectionEmulation => 'エミュレーション';

  @override
  String get menuSectionSettings => '設定';

  @override
  String get menuSectionWindows => 'ウィンドウ';

  @override
  String get menuSectionHelp => 'ヘルプ';

  @override
  String get menuOpenRom => 'ROMを開く...';

  @override
  String get menuReset => 'リセット';

  @override
  String get menuPowerReset => 'パワーリセット';

  @override
  String get menuEject => '電源オフ';

  @override
  String get menuSaveState => '状態を保存...';

  @override
  String get menuLoadState => '状態をロード...';

  @override
  String get menuPauseResume => '一時停止/再開';

  @override
  String get menuNetplay => 'ネットプレイ';

  @override
  String get netplayTransportLabel => '輸送';

  @override
  String get netplayTransportAuto => '自動 (QUIC → TCP)';

  @override
  String get netplayTransportUnknown => '不明';

  @override
  String get netplayTransportTcp => 'TCP';

  @override
  String get netplayTransportQuic => 'QUIC';

  @override
  String get netplayUsingTcpFallback => 'QUIC が失敗しました。TCP を使用しています';

  @override
  String get netplayStatusDisconnected => '切断されました';

  @override
  String get netplayStatusConnecting => '接続中...';

  @override
  String get netplayStatusConnected => '接続済み (ルーム待機中)';

  @override
  String get netplayStatusInRoom => '入室中';

  @override
  String get netplayDisconnect => '切断する';

  @override
  String get netplayServerAddress => 'サーバーアドレス';

  @override
  String get netplayServerNameLabel => 'サーバー名 (SN​​I)';

  @override
  String get netplayServerNameHint => 'ローカルホスト';

  @override
  String get netplayPlayerName => 'プレイヤー名';

  @override
  String get netplayQuicFingerprintLabel => 'QUIC 証明書フィンガープリント (オプション)';

  @override
  String get netplayQuicFingerprintHint => 'Base64url (43 文字)';

  @override
  String get netplayQuicFingerprintHelper =>
      'ピン留めされた QUIC を使用するにはこれを入力します。システムの信頼（QUIC）を使用するか TCP にフォールバックする場合は空のままにしてください。';

  @override
  String get netplayConnect => 'ゲームに参加する';

  @override
  String get netplayJoinViaP2P => 'P2P 経由で参加する';

  @override
  String get netplayJoinGame => 'ゲームに参加する';

  @override
  String get netplayCreateRoom => 'ルームの作成';

  @override
  String get netplayJoinRoom => 'ゲームに参加する';

  @override
  String get netplayAddressOrRoomCode => 'ルームコードまたはサーバーアドレス';

  @override
  String get netplayHostingTitle => 'ホスティング';

  @override
  String get netplayRoomCodeLabel => 'あなたのルームコード';

  @override
  String get netplayP2PEnabled => 'P2Pモード';

  @override
  String get netplayDirectServerLabel => 'サーバーアドレス';

  @override
  String get netplayAdvancedSettings => '詳細な接続設定';

  @override
  String get netplayP2PServerLabel => 'P2Pサーバー';

  @override
  String get netplayRoomCode => 'ルームコード';

  @override
  String get netplayRoleLabel => '役割';

  @override
  String netplayPlayerIndex(int index) {
    return 'プレイヤー $index';
  }

  @override
  String get netplaySpectator => '観客';

  @override
  String get netplayClientId => 'クライアントID';

  @override
  String get netplayPlayerListHeader => 'プレイヤー';

  @override
  String get netplayYouIndicator => '（あなた）';

  @override
  String get netplayOrSeparator => 'または';

  @override
  String netplayConnectFailed(String error) {
    return '接続に失敗しました: $error';
  }

  @override
  String netplayDisconnectFailed(String error) {
    return '切断に失敗しました: $error';
  }

  @override
  String netplayCreateRoomFailed(String error) {
    return 'ルームの作成に失敗しました: $error';
  }

  @override
  String netplayJoinRoomFailed(String error) {
    return 'ルームへの参加に失敗しました: $error';
  }

  @override
  String netplaySwitchRoleFailed(String error) {
    return '役割の切り替えに失敗しました: $error';
  }

  @override
  String get netplayInvalidRoomCode => '部屋コードが無効です';

  @override
  String get netplayRomBroadcasted => 'ネットプレイ: ルームにブロードキャストされた ROM';

  @override
  String get menuLoadTasMovie => 'TAS ムービーをロード...';

  @override
  String get menuPreferences => '設定...';

  @override
  String get saveToExternalFile => 'ファイルに保存...';

  @override
  String get loadFromExternalFile => 'ファイルからロード...';

  @override
  String get slotLabel => 'スロット';

  @override
  String get slotEmpty => '空';

  @override
  String get slotHasData => '保存されました';

  @override
  String stateSavedToSlot(int index) {
    return '状態はスロット $index に保存されました';
  }

  @override
  String stateLoadedFromSlot(int index) {
    return 'スロット $index からロードされた状態';
  }

  @override
  String slotCleared(int index) {
    return 'スロット $index がクリアされました';
  }

  @override
  String get menuAbout => 'Nesium について';

  @override
  String get menuDebugger => 'デバッガ';

  @override
  String get menuTools => 'ツール';

  @override
  String get menuOpenDebuggerWindow => 'デバッガーウィンドウを開く';

  @override
  String get menuOpenToolsWindow => 'ツールウィンドウを開く';

  @override
  String get menuInputMappingComingSoon => '入力マッピング (近日公開予定)';

  @override
  String get menuLastError => '最後のエラー';

  @override
  String get lastErrorDetailsAction => '詳細';

  @override
  String get lastErrorDialogTitle => '最後のエラー';

  @override
  String get lastErrorCopied => 'コピーされました';

  @override
  String get copy => 'コピー';

  @override
  String get paste => 'ペースト';

  @override
  String get windowDebuggerTitle => 'Nesium デバッガー';

  @override
  String get windowToolsTitle => 'ネシウムツール';

  @override
  String get virtualControlsEditTitle => '仮想コントロールを編集する';

  @override
  String get virtualControlsEditSubtitleEnabled =>
      'ドラッグして移動し、角をピンチまたはドラッグしてサイズを変更します';

  @override
  String get virtualControlsEditSubtitleDisabled => 'インタラクティブな調整を有効にする';

  @override
  String get gridSnappingTitle => 'グリッドスナップ';

  @override
  String get gridSpacingLabel => 'グリッド間隔';

  @override
  String get debuggerPlaceholderBody =>
      'CPU/PPU モニタ、メモリ ビューア、および OAM インスペクタ用に予約されたスペース。同じウィジェットをデスクトップのサイド パネルまたはモバイル シートに配置できます。';

  @override
  String get toolsPlaceholderBody =>
      '記録/再生、入力マッピング、およびチートは、デスクトップのサイド ペインとモバイルのボトム シート間でこれらのウィジェットを共有できます。';

  @override
  String get actionLoadRom => 'ROMをロードする';

  @override
  String get actionResetNes => 'ファミコンをリセットする';

  @override
  String get actionPowerResetNes => 'パワーリセット';

  @override
  String get actionEjectNes => '電源オフ';

  @override
  String get actionLoadPalette => 'パレットをロードする';

  @override
  String get videoResetToDefault => 'デフォルトにリセットする';

  @override
  String get videoTitle => 'ビデオ';

  @override
  String get videoFilterLabel => 'ビデオフィルター';

  @override
  String get videoFilterCategoryCpu => 'CPU フィルター';

  @override
  String get videoFilterCategoryGpu => 'GPU フィルター (シェーダー)';

  @override
  String get videoFilterNone => 'なし (1x)';

  @override
  String get videoFilterPrescale2x => 'プリスケール 2x';

  @override
  String get videoFilterPrescale3x => 'プリスケール 3x';

  @override
  String get videoFilterPrescale4x => 'プリスケール 4x';

  @override
  String get videoFilterHq2x => 'HQ2x';

  @override
  String get videoFilterHq3x => 'HQ3x';

  @override
  String get videoFilterHq4x => 'HQ4x';

  @override
  String get videoFilter2xSai => '2xSaI';

  @override
  String get videoFilterSuper2xSai => 'スーパー 2xSaI';

  @override
  String get videoFilterSuperEagle => 'スーパーイーグル';

  @override
  String get videoFilterLcdGrid => 'LCD グリッド (2x)';

  @override
  String get videoFilterScanlines => 'スキャンライン (2x)';

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
  String get videoLcdGridStrengthLabel => 'LCD グリッド強度';

  @override
  String get videoScanlinesIntensityLabel => '走査線の強度';

  @override
  String get videoFilterNtscComposite => 'NTSC（コンポジット）';

  @override
  String get videoFilterNtscSvideo => 'NTSC (Sビデオ)';

  @override
  String get videoFilterNtscRgb => 'NTSC (RGB)';

  @override
  String get videoFilterNtscMonochrome => 'NTSC（モノクロ）';

  @override
  String get videoFilterNtscBisqwit2x => 'NTSC (ビスクウィット) 2x';

  @override
  String get videoFilterNtscBisqwit4x => 'NTSC (ビスクウィット) 4x';

  @override
  String get videoFilterNtscBisqwit8x => 'NTSC (ビスクウィット) 8x';

  @override
  String get videoNtscAdvancedTitle => 'NTSCアドバンスト';

  @override
  String get videoNtscMergeFieldsLabel => 'フィールドを結合 (ちらつきを軽減)';

  @override
  String get videoNtscHueLabel => '色相';

  @override
  String get videoNtscSaturationLabel => '飽和';

  @override
  String get videoNtscContrastLabel => '対比';

  @override
  String get videoNtscBrightnessLabel => '輝度';

  @override
  String get videoNtscSharpnessLabel => 'シャープネス';

  @override
  String get videoNtscGammaLabel => 'ガンマ';

  @override
  String get videoNtscResolutionLabel => '解決';

  @override
  String get videoNtscArtifactsLabel => 'アーティファクト';

  @override
  String get videoNtscFringingLabel => 'フリンジ';

  @override
  String get videoNtscBleedLabel => 'にじみ';

  @override
  String get videoNtscBisqwitSettingsTitle => 'NTSC設定（ビスクウィット）';

  @override
  String get videoNtscBisqwitYFilterLengthLabel => 'Yフィルター（横ぼかし）';

  @override
  String get videoNtscBisqwitIFilterLengthLabel => 'Iフィルター';

  @override
  String get videoNtscBisqwitQFilterLengthLabel => 'Qフィルター';

  @override
  String get videoIntegerScalingTitle => '整数スケーリング';

  @override
  String get videoIntegerScalingSubtitle =>
      'ピクセルパーフェクトなスケーリング（スクロール時のガタつきを軽減）。';

  @override
  String get videoFullScreenTitle => '全画面表示';

  @override
  String get videoFullScreenSubtitle => 'ウィンドウの全画面状態を切り替えます';

  @override
  String get videoScreenVerticalOffset => '画面の垂直オフセット';

  @override
  String get videoScreenVerticalOffsetPortraitOnly => 'ポートレート モードでのみ有効です。';

  @override
  String get videoAspectRatio => 'アスペクト比';

  @override
  String get videoAspectRatioSquare => '1:1 (正方形ピクセル)';

  @override
  String get videoAspectRatioNtsc => '4:3 (NTSC)';

  @override
  String get videoAspectRatioStretch => 'ストレッチ';

  @override
  String get videoShaderLibrashaderTitle => 'RetroArch シェーダ';

  @override
  String get videoShaderLibrashaderSubtitle =>
      'GLES3 + ハードウェア バックエンド (AHB スワップチェーン) が必要です。';

  @override
  String get videoShaderLibrashaderSubtitleWindows => 'D3D11 GPU バックエンドが必要です。';

  @override
  String get videoShaderLibrashaderSubtitleApple => 'Metal バックエンドが必要です。';

  @override
  String get videoShaderLibrashaderSubtitleDisabled =>
      'Android バックエンドをハードウェアに切り替えて有効にします。';

  @override
  String get videoShaderLibrashaderSubtitleDisabledWindows =>
      'Windows バックエンドを D3D11 GPU に切り替えて有効にします。';

  @override
  String get videoShaderPresetLabel => 'Preset (.slangp)';

  @override
  String get videoShaderPresetNotSet => '未設定';

  @override
  String get shaderBrowserTitle => 'シェーダ';

  @override
  String get shaderBrowserNoShaders => 'シェーダが見つかりません';

  @override
  String shaderBrowserError(String error) {
    return 'エラー: $error';
  }

  @override
  String get aboutTitle => 'ネシウムについて';

  @override
  String get aboutLead =>
      'Nesium: nesium-core 上に構築された Rust NES/FC エミュレータ フロントエンド。';

  @override
  String get aboutIntro =>
      'この Flutter フロントエンドはエミュレーションに Rust コアを再利用します。 Web ビルドは、Flutter Web + Web Worker + WASM を介してブラウザーで実行されます。';

  @override
  String get aboutLinksHeading => 'リンク';

  @override
  String get aboutGitHubLabel => 'GitHub';

  @override
  String get aboutWebDemoLabel => 'ウェブデモ';

  @override
  String get aboutComponentsHeading => 'オープンソースコンポーネント';

  @override
  String get aboutComponentsHint => 'タップして開き、長押ししてコピーします。';

  @override
  String get aboutLicenseHeading => 'ライセンス';

  @override
  String get aboutLicenseBody =>
      'Nesium は GPL-3.0 以降に基づいてライセンスされています。リポジトリのルートにある LICENSE.md を参照してください。';

  @override
  String aboutLaunchFailed(String url) {
    return '起動できませんでした: $url';
  }

  @override
  String get videoBackendLabel => 'レンダラ バックエンド';

  @override
  String get videoBackendAndroidLabel => 'Android レンダラー バックエンド';

  @override
  String get videoBackendWindowsLabel => 'Windows レンダラー バックエンド';

  @override
  String get videoBackendHardware => 'ハードウェア (AHardwareBuffer)';

  @override
  String get videoBackendUpload => '互換性 (CPU アップロード)';

  @override
  String get videoBackendRestartHint => 'アプリの再起動後に有効になります。';

  @override
  String videoBackendCurrent(String backend) {
    return '現在のバックエンド: $backend';
  }

  @override
  String get windowsNativeOverlayTitle => 'Windows ネイティブ オーバーレイ (実験的)';

  @override
  String get windowsNativeOverlaySubtitle =>
      'Flutter コンポジターをバイパスして、完璧な滑らかさを実現します。シェーダーを無効にし、ゲームの背後にある UI をオーバーレイします。';

  @override
  String get highPerformanceModeLabel => 'ハイパフォーマンスモード';

  @override
  String get highPerformanceModeDescription =>
      'プロセスの優先順位を高め、スケジューラーを最適化してゲームプレイをよりスムーズにします。';

  @override
  String get videoLowLatencyTitle => '低遅延ビデオ';

  @override
  String get videoLowLatencySubtitle =>
      'エミュレーションとレンダラーを同期してジッターを軽減します。アプリの再起動後に有効になります。';

  @override
  String get paletteModeLabel => 'パレット';

  @override
  String get paletteModeBuiltin => '内蔵';

  @override
  String get paletteModeCustom => 'カスタム…';

  @override
  String paletteModeCustomActive(String name) {
    return 'カスタム ($name)';
  }

  @override
  String get builtinPaletteLabel => '内蔵パレット';

  @override
  String get customPaletteLoadTitle => 'パレット ファイル (.pal) をロード…';

  @override
  String get customPaletteLoadSubtitle => '192バイト（RGB）または256バイト（RGBA）';

  @override
  String commandSucceeded(String label) {
    return '$label が成功しました';
  }

  @override
  String commandFailed(String label) {
    return '$label が失敗しました';
  }

  @override
  String get snackPaused => '一時停止中';

  @override
  String get snackResumed => '再開しました';

  @override
  String snackPauseFailed(String error) {
    return '一時停止に失敗しました: $error';
  }

  @override
  String get dialogOk => 'わかりました';

  @override
  String get debuggerNoRomTitle => 'ROMが実行されていません';

  @override
  String get debuggerNoRomSubtitle => 'ROMをロードしてデバッグ状態を確認します';

  @override
  String get debuggerCpuRegisters => 'CPUレジスタ';

  @override
  String get debuggerPpuState => 'PPU の状態';

  @override
  String get debuggerCpuStatusTooltip =>
      'CPUステータスレジスタ(P)\nN: 負 - 結果ビット 7 が設定されている場合に設定されます。\nV: オーバーフロー - 符号付きオーバーフローに設定\nB: ブレーク - BRK 命令で設定\nD: 10 進数 - BCD モード (NES では無視されます)\nI: 割り込みディスエーブル - IRQ をブロックします\nZ: ゼロ - 結果がゼロの場合に設定されます\nC: キャリー - 符号なしオーバーフローに設定\n\n大文字 = 設定、小文字 = クリア';

  @override
  String get debuggerPpuCtrlTooltip =>
      'PPU 制御レジスタ (\$2000)\nV: NMIイネーブル\nP：PPUマスター/スレーブ（未使用）\nH: スプライトの高さ (0=8x8、1=8x16)\nB：地紋テーブルアドレス\nS: スプライトパターンテーブルアドレス\nI：VRAMアドレスインクリメント（0=1、1=32）\nNN: ベースネームテーブルアドレス\n\n大文字 = 設定、小文字 = クリア';

  @override
  String get debuggerPpuMaskTooltip =>
      'PPU マスク レジスタ (\$2001)\nBGR: カラー強調ビット\ns: スプライトを表示\nb: 背景を表示\nM: スプライトを左端の 8 ピクセルで表示します\nm: 左端の 8 ピクセルに背景を表示します\ng: グレースケール\n\n大文字 = 設定、小文字 = クリア';

  @override
  String get debuggerPpuStatusTooltip =>
      'PPU ステータス レジスタ (\$2002)\nV: VBlank が開始されました\nS: スプライト 0 ヒット\nO：スプライトオーバーフロー\n\n大文字 = 設定、小文字 = クリア';

  @override
  String get debuggerScanlineTooltip =>
      'スキャンライン番号:\n0-239: 表示 (レンダリング)\n240: ポストレンダリング (アイドル)\n241-260: VBlank (垂直ブランキング)\n-1: プリレンダリング (ダミースキャンライン)';

  @override
  String get tilemapSettings => '設定';

  @override
  String get tilemapOverlay => 'かぶせる';

  @override
  String get tilemapDisplayMode => '表示モード';

  @override
  String get tilemapDisplayModeDefault => 'デフォルト';

  @override
  String get tilemapDisplayModeGrayscale => 'グレースケール';

  @override
  String get tilemapDisplayModeAttributeView => '属性ビュー';

  @override
  String get tilemapTileGrid => 'タイルグリッド (8×8)';

  @override
  String get tilemapAttrGrid => '属性グリッド (16×16)';

  @override
  String get tilemapAttrGrid32 => '属性グリッド (32×32)';

  @override
  String get tilemapNtBounds => 'NT 境界';

  @override
  String get tilemapScrollOverlay => 'スクロールオーバーレイ';

  @override
  String get tilemapPanelDisplay => '画面';

  @override
  String get tilemapPanelTilemap => 'タイルマップ';

  @override
  String get tilemapPanelSelectedTile => '選択したタイル';

  @override
  String get tilemapHidePanel => 'パネルを非表示にする';

  @override
  String get tilemapShowPanel => 'パネルを表示';

  @override
  String get tilemapInfoSize => 'サイズ';

  @override
  String get tilemapInfoSizePx => 'サイズ (ピクセル)';

  @override
  String get tilemapInfoTilemapAddress => 'タイルマップアドレス';

  @override
  String get tilemapInfoTilesetAddress => 'タイルセットアドレス';

  @override
  String get tilemapInfoMirroring => 'ミラーリング';

  @override
  String get tilemapInfoTileFormat => 'タイル形式';

  @override
  String get tilemapInfoTileFormat2bpp => '2bpp';

  @override
  String get tilemapMirroringHorizontal => '水平';

  @override
  String get tilemapMirroringVertical => '垂直';

  @override
  String get tilemapMirroringFourScreen => '4画面';

  @override
  String get tilemapMirroringSingleScreenLower => 'シングルスクリーン（下）';

  @override
  String get tilemapMirroringSingleScreenUpper => '単一画面（上）';

  @override
  String get tilemapMirroringMapperControlled => 'マッパー制御';

  @override
  String get tilemapLabelColumnRow => '列、行';

  @override
  String get tilemapLabelXY => 'X、Y';

  @override
  String get tilemapLabelSize => 'サイズ';

  @override
  String get tilemapLabelTilemapAddress => 'タイルマップアドレス';

  @override
  String get tilemapLabelTileIndex => 'タイルインデックス';

  @override
  String get tilemapLabelTileAddressPpu => 'タイルアドレス(PPU)';

  @override
  String get tilemapLabelPaletteIndex => 'パレットインデックス';

  @override
  String get tilemapLabelPaletteAddress => 'パレットアドレス';

  @override
  String get tilemapLabelAttributeAddress => '属性アドレス';

  @override
  String get tilemapLabelAttributeData => '属性データ';

  @override
  String get tilemapSelectedTileTilemap => 'タイルマップ';

  @override
  String get tilemapSelectedTileTileIdx => 'タイルIDX';

  @override
  String get tilemapSelectedTileTilePpu => 'タイル(PPU)';

  @override
  String get tilemapSelectedTilePalette => 'パレット';

  @override
  String get tilemapSelectedTileAttr => '属性';

  @override
  String get tilemapCapture => '捕獲';

  @override
  String get tilemapCaptureFrameStart => 'フレーム開始';

  @override
  String get tilemapCaptureVblankStart => 'Vブランクスタート';

  @override
  String get tilemapCaptureManual => 'マニュアル';

  @override
  String get tilemapScanline => 'スキャンライン';

  @override
  String get tilemapDot => 'ドット';

  @override
  String tilemapError(String error) {
    return 'エラー: $error';
  }

  @override
  String get tilemapRetry => 'リトライ';

  @override
  String get tilemapResetZoom => 'ズームをリセット';

  @override
  String get menuTilemapViewer => 'タイルマップビューア';

  @override
  String get menuTileViewer => 'タイルビューア';

  @override
  String tileViewerError(String error) {
    return 'エラー: $error';
  }

  @override
  String get tileViewerRetry => 'リトライ';

  @override
  String get tileViewerSettings => 'タイルビューアの設定';

  @override
  String get tileViewerOverlays => 'オーバーレイ';

  @override
  String get tileViewerShowGrid => 'タイルグリッドを表示';

  @override
  String get tileViewerPalette => 'パレット';

  @override
  String tileViewerPaletteBg(int index) {
    return 'BG $index';
  }

  @override
  String tileViewerPaletteSprite(int index) {
    return 'スプライト $index';
  }

  @override
  String get tileViewerGrayscale => 'グレースケールパレットを使用する';

  @override
  String get tileViewerSelectedTile => '選択したタイル';

  @override
  String get tileViewerPatternTable => 'パターンテーブル';

  @override
  String get tileViewerTileIndex => 'タイルインデックス';

  @override
  String get tileViewerChrAddress => 'CHRアドレス';

  @override
  String get tileViewerClose => '近い';

  @override
  String get tileViewerSource => 'ソース';

  @override
  String get tileViewerSourcePpu => 'PPUメモリ';

  @override
  String get tileViewerSourceChrRom => 'CHRROM';

  @override
  String get tileViewerSourceChrRam => 'CHR RAM';

  @override
  String get tileViewerSourcePrgRom => 'PRGROM';

  @override
  String get tileViewerAddress => '住所';

  @override
  String get tileViewerSize => 'サイズ';

  @override
  String get tileViewerColumns => 'コル';

  @override
  String get tileViewerRows => '行';

  @override
  String get tileViewerLayout => 'レイアウト';

  @override
  String get tileViewerLayoutNormal => '普通';

  @override
  String get tileViewerLayout8x16 => '8×16 スプライト';

  @override
  String get tileViewerLayout16x16 => '16×16 スプライト';

  @override
  String get tileViewerBackground => '背景';

  @override
  String get tileViewerBgDefault => 'デフォルト';

  @override
  String get tileViewerBgTransparent => '透明';

  @override
  String get tileViewerBgPalette => 'パレットの色';

  @override
  String get tileViewerBgBlack => '黒';

  @override
  String get tileViewerBgWhite => '白';

  @override
  String get tileViewerBgMagenta => 'マゼンタ';

  @override
  String get tileViewerPresets => 'プリセット';

  @override
  String get tileViewerPresetPpu => 'PPU';

  @override
  String get tileViewerPresetChr => 'CHR';

  @override
  String get tileViewerPresetRom => 'ロム';

  @override
  String get tileViewerPresetBg => 'BG';

  @override
  String get tileViewerPresetOam => 'OAM';

  @override
  String get menuSpriteViewer => 'スプライトビューア';

  @override
  String get menuPaletteViewer => 'パレットビューア';

  @override
  String get paletteViewerPaletteRamTitle => 'パレットRAM (32)';

  @override
  String get paletteViewerSystemPaletteTitle => 'システムパレット (64)';

  @override
  String get paletteViewerSettingsTooltip => 'パレットビューアの設定';

  @override
  String paletteViewerTooltipPaletteRam(String addr, String value) {
    return '$addr = 0x$value';
  }

  @override
  String paletteViewerTooltipSystemIndex(int index) {
    return 'インデックス $index';
  }

  @override
  String spriteViewerError(String error) {
    return 'スプライト ビューア エラー: $error';
  }

  @override
  String get spriteViewerSettingsTooltip => 'スプライトビューアの設定';

  @override
  String get spriteViewerShowGrid => 'グリッドを表示';

  @override
  String get spriteViewerShowOutline => 'スプライトの周囲に輪郭を表示';

  @override
  String get spriteViewerShowOffscreenRegions => 'オフスクリーン領域を表示する';

  @override
  String get spriteViewerDimOffscreenSpritesGrid => 'オフスクリーン スプライトを暗くする (グリッド)';

  @override
  String get spriteViewerShowListView => 'リストビューを表示';

  @override
  String get spriteViewerPanelSprites => 'スプライト';

  @override
  String get spriteViewerPanelDataSource => 'データソース';

  @override
  String get spriteViewerPanelSprite => 'スプライト';

  @override
  String get spriteViewerPanelSelectedSprite => '選択したスプライト';

  @override
  String get spriteViewerLabelMode => 'モード';

  @override
  String get spriteViewerLabelPatternBase => 'パターンベース';

  @override
  String get spriteViewerLabelThumbnailSize => 'サムネイルのサイズ';

  @override
  String get spriteViewerBgGray => 'グレー';

  @override
  String get spriteViewerDataSourceSpriteRam => 'スプライトRAM';

  @override
  String get spriteViewerDataSourceCpuMemory => 'CPUメモリ';

  @override
  String spriteViewerTooltipTitle(int index) {
    return 'スプライト #$index';
  }

  @override
  String get spriteViewerLabelIndex => '索引';

  @override
  String get spriteViewerLabelPos => '位置';

  @override
  String get spriteViewerLabelSize => 'サイズ';

  @override
  String get spriteViewerLabelTile => 'タイル';

  @override
  String get spriteViewerLabelTileAddr => 'タイルアドレス';

  @override
  String get spriteViewerLabelPalette => 'パレット';

  @override
  String get spriteViewerLabelPaletteAddr => 'パレットアドレス';

  @override
  String get spriteViewerLabelFlip => 'フリップ';

  @override
  String get spriteViewerLabelPriority => '優先度';

  @override
  String get spriteViewerPriorityBehindBg => 'BGの後ろ';

  @override
  String get spriteViewerPriorityInFront => '前に';

  @override
  String get spriteViewerLabelVisible => '見える';

  @override
  String get spriteViewerValueYes => 'はい';

  @override
  String get spriteViewerValueNoOffscreen => 'いいえ (画面外)';

  @override
  String get spriteViewerVisibleStatusVisible => '見える';

  @override
  String get spriteViewerVisibleStatusOffscreen => 'オフスクリーン';

  @override
  String get longPressToClear => '長押しするとクリアされます';

  @override
  String get videoBackendD3D11 => 'D3D11 GPU (ゼロコピー)';

  @override
  String get videoBackendSoftware => 'ソフトウェアCPU（フォールバック）';

  @override
  String get netplayBackToSetup => 'セットアップに戻る';

  @override
  String get netplayP2PMode => 'P2Pモード';

  @override
  String get netplaySignalingServer => 'シグナリングサーバー';

  @override
  String get netplayRelayServer => 'リレーサーバー (フォールバック)';

  @override
  String get netplayP2PRoomCode => 'P2Pルームコード';

  @override
  String get netplayStartP2PSession => 'P2Pセッションを開始する';

  @override
  String get netplayJoinP2PSession => 'P2Pセッションに参加する';

  @override
  String get netplayInvalidP2PServerAddr => 'P2Pサーバーアドレスが無効です';

  @override
  String get netplayProceed => '進む';

  @override
  String get videoShaderParametersTitle => 'シェーダパラメータ';

  @override
  String get videoShaderParametersSubtitle => 'シェーダーパラメータをリアルタイムで調整';

  @override
  String get videoShaderParametersReset => 'パラメータのリセット';

  @override
  String get searchHint => '検索...';

  @override
  String get searchTooltip => '検索';

  @override
  String get noResults => '一致するパラメータが見つかりません';

  @override
  String get errorFailedToCreateTexture => 'テクスチャの作成に失敗しました';

  @override
  String get languageJapanese => '日本語';

  @override
  String get languageSpanish => 'スペイン語';

  @override
  String get languagePortuguese => 'ポルトガル語';

  @override
  String get languageRussian => 'ロシア語';

  @override
  String get languageFrench => 'フランス語';

  @override
  String get languageGerman => 'ドイツ語';
}
