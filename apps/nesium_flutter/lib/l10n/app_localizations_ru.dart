// ignore: unused_import
import 'package:intl/intl.dart' as intl;
import 'app_localizations.dart';

// ignore_for_file: type=lint

/// The translations for Russian (`ru`).
class AppLocalizationsRu extends AppLocalizations {
  AppLocalizationsRu([String locale = 'ru']) : super(locale);

  @override
  String get settingsTitle => 'Настройки';

  @override
  String get settingsTabGeneral => 'Общие';

  @override
  String get settingsTabInput => 'Управление';

  @override
  String get settingsTabVideo => 'Видео';

  @override
  String get settingsTabEmulation => 'Эмуляция';

  @override
  String get settingsTabServer => 'Сервер';

  @override
  String get settingsFloatingPreviewToggle =>
      'Плавающий предварительный просмотр';

  @override
  String get settingsFloatingPreviewTooltip =>
      'Показать предварительный просмотр игры';

  @override
  String get serverTitle => 'Сервер Netplay';

  @override
  String get serverPortLabel => 'Порт';

  @override
  String get serverStartButton => 'Запустить сервер';

  @override
  String get serverStopButton => 'Остановить сервер';

  @override
  String get serverStatusRunning => 'Запущено';

  @override
  String get serverStatusStopped => 'Остановлено';

  @override
  String serverClientCount(int count) {
    return 'Подключенные клиенты: $count';
  }

  @override
  String serverStartFailed(String error) {
    return 'Не удалось запустить сервер: $error.';
  }

  @override
  String serverStopFailed(String error) {
    return 'Не удалось остановить сервер: $error.';
  }

  @override
  String serverBindAddress(String address) {
    return 'Адрес привязки: $address';
  }

  @override
  String serverQuicFingerprint(String fingerprint) {
    return 'Отпечаток QUIC: $fingerprint';
  }

  @override
  String get generalTitle => 'Общие';

  @override
  String get themeLabel => 'Тема';

  @override
  String get themeSystem => 'Система';

  @override
  String get themeLight => 'Свет';

  @override
  String get themeDark => 'Темный';

  @override
  String get languageLabel => 'Язык';

  @override
  String get languageSystem => 'Система';

  @override
  String get languageEnglish => 'Английский';

  @override
  String get languageChineseSimplified => 'Упрощенный китайский';

  @override
  String get inputTitle => 'Управление';

  @override
  String get turboTitle => 'Турбо';

  @override
  String get turboLinkPressRelease => 'Link press/release';

  @override
  String get inputDeviceLabel => 'Устройство ввода';

  @override
  String get inputDeviceKeyboard => 'Клавиатура';

  @override
  String get inputDeviceGamepad => 'Геймпад';

  @override
  String get connectedGamepadsTitle => 'Подключенные геймпады';

  @override
  String get connectedGamepadsNone => 'Геймпады не подключены';

  @override
  String get webGamepadActivationHint =>
      'Веб-лимит: НАЖМИТЕ ЛЮБУЮ КНОПКУ на геймпаде, чтобы активировать его.';

  @override
  String connectedGamepadsPort(int port) {
    return 'Игрок $port';
  }

  @override
  String get connectedGamepadsUnassigned => 'Неназначенный';

  @override
  String get inputDeviceVirtualController => 'Виртуальный контроллер';

  @override
  String get inputGamepadAssignmentLabel => 'Назначение геймпада';

  @override
  String get inputGamepadNone => 'Нет/не назначено';

  @override
  String get inputListening => 'Слушаю...';

  @override
  String inputDetected(String buttons) {
    return 'Обнаружено: $buttons';
  }

  @override
  String get inputGamepadMappingLabel => 'Сопоставление кнопок';

  @override
  String get inputResetToDefault => 'Сбросить настройки по умолчанию';

  @override
  String get inputButtonA => 'А';

  @override
  String get inputButtonB => 'Б';

  @override
  String get inputButtonTurboA => 'Турбо А';

  @override
  String get inputButtonTurboB => 'Турбо Б';

  @override
  String get inputButtonSelect => 'Select';

  @override
  String get inputButtonStart => 'Start';

  @override
  String get inputButtonUp => 'Вверх';

  @override
  String get inputButtonDown => 'Вниз';

  @override
  String get inputButtonLeft => 'Влево';

  @override
  String get inputButtonRight => 'Вправо';

  @override
  String get inputButtonRewind => 'Перемотка назад';

  @override
  String get inputButtonFastForward => 'Быстрая перемотка вперед';

  @override
  String get inputButtonSaveState => 'Сохранить состояние';

  @override
  String get inputButtonLoadState => 'Загрузить состояние';

  @override
  String get inputButtonPause => 'Пауза';

  @override
  String get globalHotkeysTitle => 'Горячие клавиши эмулятора';

  @override
  String get gamepadHotkeysTitle => 'Горячие клавиши геймпада (Игрок 1)';

  @override
  String get inputPortLabel => 'Настроить плеер';

  @override
  String get player1 => 'Игрок 1';

  @override
  String get player2 => 'Игрок 2';

  @override
  String get player3 => 'Игрок 3';

  @override
  String get player4 => 'Игрок 4';

  @override
  String get keyboardPresetLabel => 'Предустановка клавиатуры';

  @override
  String get keyboardPresetNone => 'Нет';

  @override
  String get keyboardPresetNesStandard => 'стандарт РЭШ';

  @override
  String get keyboardPresetFightStick => 'Arcade Stick';

  @override
  String get keyboardPresetArcadeLayout => 'Arcade';

  @override
  String get keyboardPresetCustom => 'Пользовательский';

  @override
  String get customKeyBindingsTitle => 'Пользовательские привязки клавиш';

  @override
  String bindKeyTitle(String action) {
    return 'Привязать $action';
  }

  @override
  String get unassignedKey => 'Неназначенный';

  @override
  String get tipPressEscapeToClearBinding =>
      'Совет: нажмите Escape, чтобы очистить привязку.';

  @override
  String get keyboardActionUp => 'Вверх';

  @override
  String get keyboardActionDown => 'Вниз';

  @override
  String get keyboardActionLeft => 'Влево';

  @override
  String get keyboardActionRight => 'Вправо';

  @override
  String get keyboardActionA => 'А';

  @override
  String get keyboardActionB => 'Б';

  @override
  String get keyboardActionSelect => 'Select';

  @override
  String get keyboardActionStart => 'Start';

  @override
  String get keyboardActionTurboA => 'Турбо А';

  @override
  String get keyboardActionTurboB => 'Турбо Б';

  @override
  String get keyboardActionRewind => 'Перемотка назад';

  @override
  String get keyboardActionFastForward => 'Быстрая перемотка вперед';

  @override
  String get keyboardActionSaveState => 'Сохранить состояние';

  @override
  String get keyboardActionLoadState => 'Загрузить состояние';

  @override
  String get keyboardActionPause => 'Пауза';

  @override
  String get keyboardActionFullScreen => 'Полноэкранный';

  @override
  String inputBindingConflictCleared(String player, String action) {
    return 'Привязка $player $action очищена.';
  }

  @override
  String inputBindingConflictHint(String player, String action) {
    return '($player - $action)';
  }

  @override
  String inputBindingCapturedConflictHint(String player, String action) {
    return 'Занят $player - $action';
  }

  @override
  String get emulationTitle => 'Эмуляция';

  @override
  String get integerFpsTitle => 'Целочисленный режим FPS (60 Гц, NTSC)';

  @override
  String get integerFpsSubtitle =>
      'Уменьшает дрожание при прокрутке на дисплеях с частотой 60 Гц. PAL будет добавлен позже.';

  @override
  String get showOverlayTitle => 'Показать наложение статуса';

  @override
  String get showOverlaySubtitle =>
      'Показывать индикаторы паузы/перемотки назад/вперед на экране.';

  @override
  String get pauseInBackgroundTitle => 'Пауза в фоновом режиме';

  @override
  String get pauseInBackgroundSubtitle =>
      'Автоматически приостанавливает работу эмулятора, когда приложение не активно.';

  @override
  String get autoSaveEnabledTitle => 'Автосохранение';

  @override
  String get autoSaveEnabledSubtitle =>
      'Периодически сохраняйте состояние игры в выделенном слоте.';

  @override
  String get autoSaveIntervalTitle => 'Интервал автоматического сохранения';

  @override
  String autoSaveIntervalValue(int minutes) {
    return '$minutes минут';
  }

  @override
  String get fastForwardSpeedTitle => 'Скорость быстрой перемотки вперед';

  @override
  String get fastForwardSpeedSubtitle =>
      'Максимальная скорость при активной перемотке вперед.';

  @override
  String fastForwardSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get quickSaveSlotTitle => 'Слот быстрого сохранения';

  @override
  String get quickSaveSlotSubtitle =>
      'Слот, используемый ярлыками быстрого сохранения/загрузки.';

  @override
  String quickSaveSlotValue(int index) {
    return 'Слот $index';
  }

  @override
  String get rewindEnabledTitle => 'Перемотка назад';

  @override
  String get rewindEnabledSubtitle =>
      'Включите функцию перемотки в реальном времени.';

  @override
  String get rewindSecondsTitle => 'Продолжительность перемотки назад';

  @override
  String rewindSecondsValue(int seconds) {
    return '$seconds секунд';
  }

  @override
  String get rewindMinutesTitle => 'Продолжительность перемотки назад';

  @override
  String rewindMinutesValue(int minutes) {
    return '$minutes минут';
  }

  @override
  String get rewindSpeedTitle => 'Скорость перемотки назад';

  @override
  String get rewindSpeedSubtitle => 'Скорость при перемотке активна.';

  @override
  String rewindSpeedValue(int percent) {
    return '$percent%';
  }

  @override
  String get autoSlotLabel => 'Автоматический слот';

  @override
  String get menuAutoSave => 'Автосохранение...';

  @override
  String get stateAutoSaved => 'Автосохранение создано';

  @override
  String get virtualControlsTitle => 'Виртуальное управление';

  @override
  String get virtualControlsSwitchInputTip =>
      'Переключите вход на «Виртуальный контроллер», чтобы использовать эти настройки.';

  @override
  String get virtualControlsButtonSize => 'Размер кнопки';

  @override
  String get virtualControlsGap => 'Зазор';

  @override
  String get virtualControlsOpacity => 'Непрозрачность';

  @override
  String get virtualControlsHitboxScale => 'Масштаб хитбокса';

  @override
  String get virtualControlsHapticFeedback => 'Тактильная обратная связь';

  @override
  String get virtualControlsDpadDeadzone => 'Мертвая зона крестовины';

  @override
  String get virtualControlsDpadDeadzoneHelp =>
      'Центральная мертвая зона: прикосновение к центру не активирует какое-либо направление.';

  @override
  String get virtualControlsDpadBoundaryDeadzone =>
      'Граница мертвой зоны крестовины';

  @override
  String get virtualControlsDpadBoundaryDeadzoneHelp =>
      'Граничная мертвая зона: более высокие значения затрудняют срабатывание диагоналей, уменьшая случайное нажатие соседей.';

  @override
  String get virtualControlsReset => 'Сбросить макет';

  @override
  String get virtualControlsDiscardChangesTitle => 'Отменить изменения';

  @override
  String get virtualControlsDiscardChangesSubtitle =>
      'Вернуться к последнему сохраненному макету';

  @override
  String get virtualControlsTurboFramesPerToggle =>
      'Турбо-кадров на переключатель';

  @override
  String get virtualControlsTurboOnFrames => 'Турбо пресс-рамы';

  @override
  String get virtualControlsTurboOffFrames => 'Рамки турбо-релиза';

  @override
  String framesValue(int frames) {
    return '$frames кадры';
  }

  @override
  String get tipAdjustButtonsInDrawer =>
      'Совет: отрегулируйте положение/размер кнопки в панели управления игрой.';

  @override
  String get keyCapturePressKeyToBind => 'Нажмите клавишу для привязки.';

  @override
  String keyCaptureCurrent(String key) {
    return 'Текущий: $key';
  }

  @override
  String keyCaptureCaptured(String key) {
    return 'Снято: $key';
  }

  @override
  String get keyCapturePressEscToClear => 'Нажмите Escape, чтобы очистить.';

  @override
  String get keyBindingsTitle => 'Привязки клавиш';

  @override
  String get cancel => 'Отмена';

  @override
  String get appName => 'Nesium';

  @override
  String get menuTooltip => 'Меню';

  @override
  String get menuSectionFile => 'Файл';

  @override
  String get menuSectionEmulation => 'Эмуляция';

  @override
  String get menuSectionSettings => 'Настройки';

  @override
  String get menuSectionWindows => 'Окна';

  @override
  String get menuSectionHelp => 'Помощь';

  @override
  String get menuOpenRom => 'Открыть ПЗУ...';

  @override
  String get menuReset => 'Перезагрузить';

  @override
  String get menuPowerReset => 'Сброс питания';

  @override
  String get menuEject => 'Выключить питание';

  @override
  String get menuSaveState => 'Сохранить состояние...';

  @override
  String get menuLoadState => 'Загрузить состояние...';

  @override
  String get menuPauseResume => 'Пауза/возобновить';

  @override
  String get menuNetplay => 'Сетевая игра';

  @override
  String get netplayTransportLabel => 'Транспорт';

  @override
  String get netplayTransportAuto => 'Авто (QUIC → TCP)';

  @override
  String get netplayTransportUnknown => 'Неизвестный';

  @override
  String get netplayTransportTcp => 'TCP';

  @override
  String get netplayTransportQuic => 'QUIC';

  @override
  String get netplayUsingTcpFallback => 'Ошибка QUIC при использовании TCP';

  @override
  String get netplayStatusDisconnected => 'Отключено';

  @override
  String get netplayStatusConnecting => 'Подключение...';

  @override
  String get netplayStatusConnected => 'Подключено (ожидание места)';

  @override
  String get netplayStatusInRoom => 'В комнате';

  @override
  String get netplayDisconnect => 'Отключить';

  @override
  String get netplayServerAddress => 'Адрес сервера';

  @override
  String get netplayServerNameLabel => 'Имя сервера (SNI)';

  @override
  String get netplayServerNameHint => 'локальный хост';

  @override
  String get netplayPlayerName => 'Имя игрока';

  @override
  String get netplayQuicFingerprintLabel =>
      'Отпечаток сертификата QUIC (необязательно)';

  @override
  String get netplayQuicFingerprintHint => 'base64url (43 символа)';

  @override
  String get netplayQuicFingerprintHelper =>
      'Введите это для использования привязанного QUIC. Оставьте пустым для использования системных сертификатов (QUIC) или отката на TCP.';

  @override
  String get netplayConnect => 'Присоединиться к игре';

  @override
  String get netplayJoinViaP2P => 'Присоединяйтесь через P2P';

  @override
  String get netplayJoinGame => 'Присоединиться к игре';

  @override
  String get netplayCreateRoom => 'Создать комнату';

  @override
  String get netplayJoinRoom => 'Присоединиться к игре';

  @override
  String get netplayAddressOrRoomCode => 'Код комнаты или адрес сервера';

  @override
  String get netplayHostingTitle => 'Хостинг';

  @override
  String get netplayRoomCodeLabel => 'Код вашего номера';

  @override
  String get netplayP2PEnabled => 'P2P-режим';

  @override
  String get netplayDirectServerLabel => 'Адрес сервера';

  @override
  String get netplayAdvancedSettings => 'Расширенные настройки подключения';

  @override
  String get netplayP2PServerLabel => 'P2P-сервер';

  @override
  String get netplayRoomCode => 'Код номера';

  @override
  String get netplayRoleLabel => 'Роль';

  @override
  String netplayPlayerIndex(int index) {
    return 'Игрок $index';
  }

  @override
  String get netplaySpectator => 'Зритель';

  @override
  String get netplayClientId => 'Идентификатор клиента';

  @override
  String get netplayPlayerListHeader => 'Игроки';

  @override
  String get netplayYouIndicator => '(Ты)';

  @override
  String get netplayOrSeparator => 'ИЛИ';

  @override
  String netplayConnectFailed(String error) {
    return 'Не удалось подключиться: $error';
  }

  @override
  String netplayDisconnectFailed(String error) {
    return 'Отключиться не удалось: $error';
  }

  @override
  String netplayCreateRoomFailed(String error) {
    return 'Не удалось создать комнату: $error.';
  }

  @override
  String netplayJoinRoomFailed(String error) {
    return 'Не удалось присоединиться к комнате: $error.';
  }

  @override
  String netplaySwitchRoleFailed(String error) {
    return 'Не удалось переключить роль: $error.';
  }

  @override
  String get netplayInvalidRoomCode => 'Неверный код комнаты';

  @override
  String get netplayRomBroadcasted => 'Netplay: ПЗУ транслируется в комнату';

  @override
  String get menuLoadTasMovie => 'Загрузить фильм ТАС...';

  @override
  String get menuPreferences => 'Предпочтения...';

  @override
  String get saveToExternalFile => 'Сохранить в файл...';

  @override
  String get loadFromExternalFile => 'Загрузить из файла...';

  @override
  String get slotLabel => 'Слот';

  @override
  String get slotEmpty => 'Пусто';

  @override
  String get slotHasData => 'Сохранено';

  @override
  String stateSavedToSlot(int index) {
    return 'Состояние сохранено в слоте $index.';
  }

  @override
  String stateLoadedFromSlot(int index) {
    return 'Состояние загружено из слота $index';
  }

  @override
  String slotCleared(int index) {
    return 'Слот $index очищен';
  }

  @override
  String get menuAbout => 'О программе';

  @override
  String get menuDebugger => 'Отладчик';

  @override
  String get menuTools => 'Инструменты';

  @override
  String get menuOpenDebuggerWindow => 'Открыть окно отладчика';

  @override
  String get menuOpenToolsWindow => 'Открыть окно инструментов';

  @override
  String get menuInputMappingComingSoon =>
      'Сопоставление входных данных (скоро)';

  @override
  String get menuLastError => 'Последняя ошибка';

  @override
  String get lastErrorDetailsAction => 'Подробности';

  @override
  String get lastErrorDialogTitle => 'Последняя ошибка';

  @override
  String get lastErrorCopied => 'Скопировано';

  @override
  String get copy => 'Копировать';

  @override
  String get paste => 'Вставить';

  @override
  String get windowDebuggerTitle => 'Отладчик Nesium';

  @override
  String get windowToolsTitle => 'Инструменты Nesium';

  @override
  String get virtualControlsEditTitle =>
      'Редактировать виртуальные элементы управления';

  @override
  String get virtualControlsEditSubtitleEnabled =>
      'Перетащите, чтобы переместить, зажмите или перетащите угол, чтобы изменить размер';

  @override
  String get virtualControlsEditSubtitleDisabled =>
      'Включить интерактивную настройку';

  @override
  String get gridSnappingTitle => 'Привязка к сетке';

  @override
  String get gridSpacingLabel => 'Шаг сетки';

  @override
  String get debuggerPlaceholderBody =>
      'Место, зарезервированное для мониторов CPU/PPU, средств просмотра памяти и инспекторов OAM. Одни и те же виджеты могут располагаться на боковой панели рабочего стола или на листе мобильного устройства.';

  @override
  String get toolsPlaceholderBody =>
      'Запись/воспроизведение, сопоставление ввода и читы могут использовать эти виджеты для совместного использования между боковыми панелями рабочего стола и нижними листами мобильных устройств.';

  @override
  String get actionLoadRom => 'Загрузить ПЗУ';

  @override
  String get actionResetNes => 'Сбросить РЭШ';

  @override
  String get actionPowerResetNes => 'Сброс питания NES';

  @override
  String get actionEjectNes => 'Выключить питание';

  @override
  String get actionLoadPalette => 'Загрузить палитру';

  @override
  String get videoResetToDefault => 'Сбросить настройки по умолчанию';

  @override
  String get videoTitle => 'Видео';

  @override
  String get videoFilterLabel => 'Видео фильтр';

  @override
  String get videoFilterCategoryCpu => 'Фильтры ЦП';

  @override
  String get videoFilterCategoryGpu =>
      'Фильтры графического процессора (шейдеры)';

  @override
  String get videoFilterNone => 'Нет (1x)';

  @override
  String get videoFilterPrescale2x => 'Предварительное масштабирование 2x';

  @override
  String get videoFilterPrescale3x => 'Предварительное масштабирование 3x';

  @override
  String get videoFilterPrescale4x => 'Предварительное масштабирование 4x';

  @override
  String get videoFilterHq2x => 'HQ2x';

  @override
  String get videoFilterHq3x => 'HQ3x';

  @override
  String get videoFilterHq4x => 'HQ4x';

  @override
  String get videoFilter2xSai => '2xSaI';

  @override
  String get videoFilterSuper2xSai => 'Супер 2xSaI';

  @override
  String get videoFilterSuperEagle => 'Супер Орел';

  @override
  String get videoFilterLcdGrid => 'ЖК-сетка (2 шт.)';

  @override
  String get videoFilterScanlines => 'Строки развертки (2x)';

  @override
  String get videoFilterXbrz2x => 'хБРЗ 2x';

  @override
  String get videoFilterXbrz3x => 'хБРЗ 3x';

  @override
  String get videoFilterXbrz4x => 'хБРЗ 4x';

  @override
  String get videoFilterXbrz5x => 'хБРЗ 5x';

  @override
  String get videoFilterXbrz6x => 'хБРЗ 6x';

  @override
  String get videoLcdGridStrengthLabel => 'Прочность ЖК-сетки';

  @override
  String get videoScanlinesIntensityLabel => 'Интенсивность линии развертки';

  @override
  String get videoFilterNtscComposite => 'NTSC (композитный)';

  @override
  String get videoFilterNtscSvideo => 'NTSC (S-Видео)';

  @override
  String get videoFilterNtscRgb => 'NTSC (РГБ)';

  @override
  String get videoFilterNtscMonochrome => 'NTSC (монохромный)';

  @override
  String get videoFilterNtscBisqwit2x => 'NTSC (Бисквит) 2x';

  @override
  String get videoFilterNtscBisqwit4x => 'NTSC (Бисквит) 4x';

  @override
  String get videoFilterNtscBisqwit8x => 'NTSC (Бисквит) 8x';

  @override
  String get videoNtscAdvancedTitle => 'NTSC продвинутый';

  @override
  String get videoNtscMergeFieldsLabel =>
      'Объединить поля (уменьшить мерцание)';

  @override
  String get videoNtscHueLabel => 'Оттенок (Hue)';

  @override
  String get videoNtscSaturationLabel => 'Насыщенность';

  @override
  String get videoNtscContrastLabel => 'Контраст';

  @override
  String get videoNtscBrightnessLabel => 'Яркость';

  @override
  String get videoNtscSharpnessLabel => 'Резкость';

  @override
  String get videoNtscGammaLabel => 'Гамма';

  @override
  String get videoNtscResolutionLabel => 'Разрешение';

  @override
  String get videoNtscArtifactsLabel => 'Артефакты';

  @override
  String get videoNtscFringingLabel => 'окантовка';

  @override
  String get videoNtscBleedLabel => 'Смешивание цветов (Bleed)';

  @override
  String get videoNtscBisqwitSettingsTitle => 'Настройки NTSC (Бисквит)';

  @override
  String get videoNtscBisqwitYFilterLengthLabel =>
      'Y-фильтр (горизонтальное размытие)';

  @override
  String get videoNtscBisqwitIFilterLengthLabel => 'Я фильтрую';

  @override
  String get videoNtscBisqwitQFilterLengthLabel => 'Q-фильтр';

  @override
  String get videoIntegerScalingTitle => 'Целочисленное масштабирование';

  @override
  String get videoIntegerScalingSubtitle =>
      'Идеальное масштабирование до пикселя (уменьшает мерцание при прокрутке).';

  @override
  String get videoFullScreenTitle => 'Полноэкранный';

  @override
  String get videoFullScreenSubtitle =>
      'Переключить полноэкранное состояние окна';

  @override
  String get videoScreenVerticalOffset => 'Вертикальное смещение экрана';

  @override
  String get videoScreenVerticalOffsetPortraitOnly =>
      'Действует только в портретном режиме.';

  @override
  String get videoAspectRatio => 'Соотношение сторон';

  @override
  String get videoAspectRatioSquare => '1:1 (квадратные пиксели)';

  @override
  String get videoAspectRatioNtsc => '4:3 (НТСК)';

  @override
  String get videoAspectRatioStretch => 'Растянуть';

  @override
  String get videoShaderLibrashaderTitle => 'Шейдеры RetroArch';

  @override
  String get videoShaderLibrashaderSubtitle =>
      'Требуется GLES3 + аппаратный бэкэнд (цепочка обмена AHB).';

  @override
  String get videoShaderLibrashaderSubtitleWindows =>
      'Требуется серверная часть графического процессора D3D11.';

  @override
  String get videoShaderLibrashaderSubtitleApple =>
      'Требуется металлический бэкэнд.';

  @override
  String get videoShaderLibrashaderSubtitleDisabled =>
      'Переключите серверную часть Android на Аппаратное обеспечение, чтобы включить.';

  @override
  String get videoShaderLibrashaderSubtitleDisabledWindows =>
      'Чтобы включить серверную часть Windows, переключите ее на графический процессор D3D11.';

  @override
  String get videoShaderPresetLabel => 'Предустановка (.slangp)';

  @override
  String get videoShaderPresetNotSet => 'Не установлено';

  @override
  String get shaderBrowserTitle => 'Шейдеры';

  @override
  String get shaderBrowserNoShaders => 'Шейдеры не найдены';

  @override
  String shaderBrowserError(String error) {
    return 'Ошибка: $error.';
  }

  @override
  String get aboutTitle => 'О Nesium';

  @override
  String get aboutLead =>
      'Nesium: Интерфейс эмулятора NES/FC на языке Rust, построенный на ядре nesium-core.';

  @override
  String get aboutIntro =>
      'Этот интерфейс Flutter повторно использует ядро ​​Rust для эмуляции. Веб-сборка запускается в браузере через Flutter Web + Web Worker + WASM.';

  @override
  String get aboutLinksHeading => 'Ссылки';

  @override
  String get aboutGitHubLabel => 'GitHub';

  @override
  String get aboutWebDemoLabel => 'Веб-демо';

  @override
  String get aboutComponentsHeading => 'Компоненты с открытым исходным кодом';

  @override
  String get aboutComponentsHint =>
      'Нажмите, чтобы открыть, нажмите и удерживайте, чтобы скопировать.';

  @override
  String get aboutLicenseHeading => 'Лицензия';

  @override
  String get aboutLicenseBody =>
      'Nesium распространяется под лицензией GPL-3.0 или более поздней версии. См. LICENSE.md в корне репозитория.';

  @override
  String aboutLaunchFailed(String url) {
    return 'Не удалось запустить: $url';
  }

  @override
  String get videoBackendLabel => 'Серверная часть рендерера';

  @override
  String get videoBackendAndroidLabel => 'Серверная часть рендеринга Android';

  @override
  String get videoBackendWindowsLabel => 'Серверная часть рендеринга Windows';

  @override
  String get videoBackendHardware => 'Аппаратное обеспечение (AHardwareBuffer)';

  @override
  String get videoBackendUpload => 'Совместимость (загрузка процессора)';

  @override
  String get videoBackendRestartHint =>
      'Вступает в силу после перезапуска приложения.';

  @override
  String videoBackendCurrent(String backend) {
    return 'Текущий бэкэнд: $backend';
  }

  @override
  String get windowsNativeOverlayTitle =>
      'Собственное наложение Windows (экспериментальное)';

  @override
  String get windowsNativeOverlaySubtitle =>
      'Обходит компоновщик Flutter для идеальной плавности. Отключает шейдеры и наложения пользовательского интерфейса игры.';

  @override
  String get highPerformanceModeLabel => 'Режим высокой производительности';

  @override
  String get highPerformanceModeDescription =>
      'Повысьте приоритет процесса и оптимизируйте планировщик для более плавного игрового процесса.';

  @override
  String get videoLowLatencyTitle => 'Видео с низкой задержкой';

  @override
  String get videoLowLatencySubtitle =>
      'Синхронизируйте эмуляцию и рендеринг, чтобы уменьшить дрожание. Вступает в силу после перезапуска приложения.';

  @override
  String get paletteModeLabel => 'Палитра';

  @override
  String get paletteModeBuiltin => 'Встроенный';

  @override
  String get paletteModeCustom => 'Обычай…';

  @override
  String paletteModeCustomActive(String name) {
    return 'Пользовательский ($name)';
  }

  @override
  String get builtinPaletteLabel => 'Встроенная палитра';

  @override
  String get customPaletteLoadTitle => 'Загрузить файл палитры (.pal)…';

  @override
  String get customPaletteLoadSubtitle =>
      '192 байта (RGB) или 256 байтов (RGBA)';

  @override
  String commandSucceeded(String label) {
    return '$label удалось';
  }

  @override
  String commandFailed(String label) {
    return '$label не удалось';
  }

  @override
  String get snackPaused => 'Приостановлено';

  @override
  String get snackResumed => 'Возобновлено';

  @override
  String snackPauseFailed(String error) {
    return 'Не удалось приостановить: $error.';
  }

  @override
  String get dialogOk => 'ХОРОШО';

  @override
  String get debuggerNoRomTitle => 'ПЗУ не работает';

  @override
  String get debuggerNoRomSubtitle =>
      'Загрузите ПЗУ, чтобы увидеть состояние отладки';

  @override
  String get debuggerCpuRegisters => 'Регистры ЦП';

  @override
  String get debuggerPpuState => 'ППУ Государство';

  @override
  String get debuggerCpuStatusTooltip =>
      'Регистр состояния ЦП (P)\nN: Отрицательный — устанавливается, если установлен бит результата 7.\nV: Переполнение – устанавливается на знаковое переполнение.\nB: Break – устанавливается инструкцией BRK.\nD: Десятичный — режим BCD (игнорируется на NES)\nI: Прерывание отключено – блокирует IRQ\nZ: Ноль — устанавливается, если результат равен нулю.\nC: Carry – устанавливается на беззнаковое переполнение.\n\nПрописные буквы = заданы, строчные = очищены.';

  @override
  String get debuggerPpuCtrlTooltip =>
      'Регистр управления ППУ (2000 долларов США)\nV: включение NMI\nP: ведущий/ведомый PPU (не используется)\nH: Высота спрайта (0=8x8, 1=8x16)\nB: Адрес таблицы фоновых шаблонов\nS: Адрес таблицы шаблонов спрайтов.\nI: приращение адреса VRAM (0=1, 1=32)\nNN: адрес базовой таблицы имен.\n\nПрописные буквы = заданы, строчные = очищены.';

  @override
  String get debuggerPpuMaskTooltip =>
      'Реестр масок ППУ (2001 долл. США)\nBGR: биты выделения цвета.\ns: Показать спрайты\nб: Показать фон\nM: показывать спрайты в 8 крайних левых пикселях.\nm: показать фон в крайних левых 8 пикселях.\nг: оттенки серого\n\nПрописные буквы = заданы, строчные = очищены.';

  @override
  String get debuggerPpuStatusTooltip =>
      'Регистр состояния PPU (\$2002)\nВ: VBlank запущен.\nS: Спрайт 0 попаданий\nО: переполнение спрайта\n\nПрописные буквы = заданы, строчные = очищены.';

  @override
  String get debuggerScanlineTooltip =>
      'Scanline Numbers:\n0-239: Видимый (рендеринг)\n240: Пост-рендеринг (в режиме ожидания)\n241-260: VBlank (вертикальное гашение)\n-1: Предварительный рендеринг (фиктивная строка сканирования)';

  @override
  String get tilemapSettings => 'Настройки';

  @override
  String get tilemapOverlay => 'Наложение';

  @override
  String get tilemapDisplayMode => 'Режим отображения';

  @override
  String get tilemapDisplayModeDefault => 'По умолчанию';

  @override
  String get tilemapDisplayModeGrayscale => 'Оттенки серого';

  @override
  String get tilemapDisplayModeAttributeView => 'Просмотр атрибутов';

  @override
  String get tilemapTileGrid => 'Плиточная сетка (8×8)';

  @override
  String get tilemapAttrGrid => 'Сетка атрибутов (16×16)';

  @override
  String get tilemapAttrGrid32 => 'Сетка атрибутов (32×32)';

  @override
  String get tilemapNtBounds => 'Границы Северной Америки';

  @override
  String get tilemapScrollOverlay => 'Наложение прокрутки';

  @override
  String get tilemapPanelDisplay => 'Отображать';

  @override
  String get tilemapPanelTilemap => 'Тайловая карта';

  @override
  String get tilemapPanelSelectedTile => 'Выбранная плитка';

  @override
  String get tilemapHidePanel => 'Скрыть панель';

  @override
  String get tilemapShowPanel => 'Показать панель';

  @override
  String get tilemapInfoSize => 'Размер';

  @override
  String get tilemapInfoSizePx => 'Размер (пикселей)';

  @override
  String get tilemapInfoTilemapAddress => 'Адрес тайловой карты';

  @override
  String get tilemapInfoTilesetAddress => 'Адрес набора тайлов';

  @override
  String get tilemapInfoMirroring => 'Зеркальное отображение';

  @override
  String get tilemapInfoTileFormat => 'Формат плитки';

  @override
  String get tilemapInfoTileFormat2bpp => '2 бит на пиксель';

  @override
  String get tilemapMirroringHorizontal => 'Горизонтальный';

  @override
  String get tilemapMirroringVertical => 'Вертикальный';

  @override
  String get tilemapMirroringFourScreen => 'Четырехэкранный';

  @override
  String get tilemapMirroringSingleScreenLower => 'Одноэкранный (нижний)';

  @override
  String get tilemapMirroringSingleScreenUpper => 'Одноэкранный (верхний)';

  @override
  String get tilemapMirroringMapperControlled => 'Контролируется картографом';

  @override
  String get tilemapLabelColumnRow => 'Столбец, Строка';

  @override
  String get tilemapLabelXY => 'Х, Ю';

  @override
  String get tilemapLabelSize => 'Размер';

  @override
  String get tilemapLabelTilemapAddress => 'Адрес тайловой карты';

  @override
  String get tilemapLabelTileIndex => 'Индекс плитки';

  @override
  String get tilemapLabelTileAddressPpu => 'Адрес плитки (PPU)';

  @override
  String get tilemapLabelPaletteIndex => 'Индекс палитры';

  @override
  String get tilemapLabelPaletteAddress => 'Адрес палитры';

  @override
  String get tilemapLabelAttributeAddress => 'Адрес атрибута';

  @override
  String get tilemapLabelAttributeData => 'Данные атрибута';

  @override
  String get tilemapSelectedTileTilemap => 'Тайловая карта';

  @override
  String get tilemapSelectedTileTileIdx => 'Идентификатор плитки';

  @override
  String get tilemapSelectedTileTilePpu => 'Плитка (ППУ)';

  @override
  String get tilemapSelectedTilePalette => 'Палитра';

  @override
  String get tilemapSelectedTileAttr => 'Атрибут';

  @override
  String get tilemapCapture => 'Захватывать';

  @override
  String get tilemapCaptureFrameStart => 'Начало кадра';

  @override
  String get tilemapCaptureVblankStart => 'VПустой старт';

  @override
  String get tilemapCaptureManual => 'Руководство';

  @override
  String get tilemapScanline => 'Сканлайн';

  @override
  String get tilemapDot => 'Точка';

  @override
  String tilemapError(String error) {
    return 'Ошибка: $error.';
  }

  @override
  String get tilemapRetry => 'Повторить попытку';

  @override
  String get tilemapResetZoom => 'Сбросить масштаб';

  @override
  String get menuTilemapViewer => 'Просмотрщик тайловых карт';

  @override
  String get menuTileViewer => 'Средство просмотра плиток';

  @override
  String tileViewerError(String error) {
    return 'Ошибка: $error.';
  }

  @override
  String get tileViewerRetry => 'Повторить попытку';

  @override
  String get tileViewerSettings => 'Настройки просмотра плиток';

  @override
  String get tileViewerOverlays => 'Наложения';

  @override
  String get tileViewerShowGrid => 'Показать сетку плиток';

  @override
  String get tileViewerPalette => 'Палитра';

  @override
  String tileViewerPaletteBg(int index) {
    return 'БГ $index';
  }

  @override
  String tileViewerPaletteSprite(int index) {
    return 'Спрайт $index';
  }

  @override
  String get tileViewerGrayscale => 'Использовать палитру оттенков серого';

  @override
  String get tileViewerSelectedTile => 'Выбранная плитка';

  @override
  String get tileViewerPatternTable => 'Таблица шаблонов';

  @override
  String get tileViewerTileIndex => 'Индекс плитки';

  @override
  String get tileViewerChrAddress => 'Адрес ЦРП';

  @override
  String get tileViewerClose => 'Закрыть';

  @override
  String get tileViewerSource => 'Источник';

  @override
  String get tileViewerSourcePpu => 'Память ППУ';

  @override
  String get tileViewerSourceChrRom => 'ХР ПЗУ';

  @override
  String get tileViewerSourceChrRam => 'ЧР ОЗУ';

  @override
  String get tileViewerSourcePrgRom => 'ПРГ ПЗУ';

  @override
  String get tileViewerAddress => 'Адрес';

  @override
  String get tileViewerSize => 'Размер';

  @override
  String get tileViewerColumns => 'Колс';

  @override
  String get tileViewerRows => 'Строки';

  @override
  String get tileViewerLayout => 'Макет';

  @override
  String get tileViewerLayoutNormal => 'Нормальный';

  @override
  String get tileViewerLayout8x16 => 'Спрайты 8×16';

  @override
  String get tileViewerLayout16x16 => 'Спрайты 16×16';

  @override
  String get tileViewerBackground => 'Фон';

  @override
  String get tileViewerBgDefault => 'По умолчанию';

  @override
  String get tileViewerBgTransparent => 'Прозрачный';

  @override
  String get tileViewerBgPalette => 'Палитра цветов';

  @override
  String get tileViewerBgBlack => 'Черный';

  @override
  String get tileViewerBgWhite => 'Белый';

  @override
  String get tileViewerBgMagenta => 'Пурпурный';

  @override
  String get tileViewerPresets => 'Пресеты';

  @override
  String get tileViewerPresetPpu => 'ППУ';

  @override
  String get tileViewerPresetChr => 'ЧР';

  @override
  String get tileViewerPresetRom => 'ПЗУ';

  @override
  String get tileViewerPresetBg => 'БГ';

  @override
  String get tileViewerPresetOam => 'ОАМ';

  @override
  String get menuSpriteViewer => 'Просмотр спрайтов';

  @override
  String get menuPaletteViewer => 'Средство просмотра палитры';

  @override
  String get paletteViewerPaletteRamTitle => 'Палитра ОЗУ (32)';

  @override
  String get paletteViewerSystemPaletteTitle => 'Системная палитра (64)';

  @override
  String get paletteViewerSettingsTooltip =>
      'Настройки средства просмотра палитры';

  @override
  String paletteViewerTooltipPaletteRam(String addr, String value) {
    return '$addr = 0x$value';
  }

  @override
  String paletteViewerTooltipSystemIndex(int index) {
    return 'Индекс $index';
  }

  @override
  String spriteViewerError(String error) {
    return 'Ошибка просмотра спрайтов: $error.';
  }

  @override
  String get spriteViewerSettingsTooltip => 'Настройки просмотра спрайтов';

  @override
  String get spriteViewerShowGrid => 'Показать сетку';

  @override
  String get spriteViewerShowOutline => 'Показать контур вокруг спрайтов';

  @override
  String get spriteViewerShowOffscreenRegions => 'Показать закадровые регионы';

  @override
  String get spriteViewerDimOffscreenSpritesGrid =>
      'Затемнение закадровых спрайтов (сетка)';

  @override
  String get spriteViewerShowListView => 'Показать список';

  @override
  String get spriteViewerPanelSprites => 'Спрайты';

  @override
  String get spriteViewerPanelDataSource => 'Источник данных';

  @override
  String get spriteViewerPanelSprite => 'Спрайт';

  @override
  String get spriteViewerPanelSelectedSprite => 'Выбранный спрайт';

  @override
  String get spriteViewerLabelMode => 'Режим';

  @override
  String get spriteViewerLabelPatternBase => 'Основа узора';

  @override
  String get spriteViewerLabelThumbnailSize => 'Размер миниатюры';

  @override
  String get spriteViewerBgGray => 'Серый';

  @override
  String get spriteViewerDataSourceSpriteRam => 'Спрайтовая оперативная память';

  @override
  String get spriteViewerDataSourceCpuMemory => 'Память процессора';

  @override
  String spriteViewerTooltipTitle(int index) {
    return 'Спрайт #$index';
  }

  @override
  String get spriteViewerLabelIndex => 'Индекс';

  @override
  String get spriteViewerLabelPos => 'Поз.';

  @override
  String get spriteViewerLabelSize => 'Размер';

  @override
  String get spriteViewerLabelTile => 'Плитка';

  @override
  String get spriteViewerLabelTileAddr => 'Адрес плитки';

  @override
  String get spriteViewerLabelPalette => 'Палитра';

  @override
  String get spriteViewerLabelPaletteAddr => 'Адрес палитры';

  @override
  String get spriteViewerLabelFlip => 'Подбросить';

  @override
  String get spriteViewerLabelPriority => 'Приоритет';

  @override
  String get spriteViewerPriorityBehindBg => 'За БГ';

  @override
  String get spriteViewerPriorityInFront => 'Спереди';

  @override
  String get spriteViewerLabelVisible => 'Видимый';

  @override
  String get spriteViewerValueYes => 'Да';

  @override
  String get spriteViewerValueNoOffscreen => 'Нет (за кадром)';

  @override
  String get spriteViewerVisibleStatusVisible => 'Видимый';

  @override
  String get spriteViewerVisibleStatusOffscreen => 'За кадром';

  @override
  String get longPressToClear => 'Длительное нажатие, чтобы очистить';

  @override
  String get videoBackendD3D11 => 'Графический процессор D3D11 (нулевая копия)';

  @override
  String get videoBackendSoftware => 'Программный процессор (резервный)';

  @override
  String get netplayBackToSetup => 'Вернуться к настройке';

  @override
  String get netplayP2PMode => 'P2P-режим';

  @override
  String get netplaySignalingServer => 'Сигнальный сервер';

  @override
  String get netplayRelayServer => 'Ретрансляционный сервер (резервный)';

  @override
  String get netplayP2PRoomCode => 'P2P-код комнаты';

  @override
  String get netplayStartP2PSession => 'Начать P2P-сессию';

  @override
  String get netplayJoinP2PSession => 'Присоединиться к P2P-сессии';

  @override
  String get netplayInvalidP2PServerAddr => 'Неверный адрес P2P-сервера.';

  @override
  String get netplayProceed => 'Продолжить';

  @override
  String get videoShaderParametersTitle => 'Параметры шейдера';

  @override
  String get videoShaderParametersSubtitle =>
      'Настраивайте параметры шейдера в режиме реального времени';

  @override
  String get videoShaderParametersReset => 'Сбросить параметры';

  @override
  String get searchHint => 'Поиск...';

  @override
  String get searchTooltip => 'Поиск';

  @override
  String get noResults => 'Соответствующие параметры не найдены';

  @override
  String get errorFailedToCreateTexture => 'Не удалось создать текстуру.';

  @override
  String get languageJapanese => 'Японский';

  @override
  String get languageSpanish => 'Испанский';

  @override
  String get languagePortuguese => 'Португальский';

  @override
  String get languageRussian => 'Русский';

  @override
  String get languageFrench => 'Французский';

  @override
  String get languageGerman => 'Немецкий';
}
