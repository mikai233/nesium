import 'dart:async';
import 'dart:js_interop';
import 'dart:js_interop_unsafe';
import 'dart:ui_web' as ui_web;

import 'package:file_selector/file_selector.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path/path.dart' as p;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:logging/logging.dart';
import 'package:web/web.dart' as web;

import '../domain/nes_controller.dart';
import '../domain/nes_input_masks.dart';
import '../domain/gamepad_service.dart';
import '../domain/pad_button.dart';
import '../domain/emulation_status.dart';
import '../features/controls/input_settings.dart';
import '../features/controls/virtual_controls_editor.dart';
import '../features/controls/turbo_settings.dart';
import '../features/controls/virtual_controls_overlay.dart';
import '../features/controls/virtual_controls_settings.dart';
import '../features/about/about_page.dart';
import '../features/save_state/auto_save_service.dart';
import '../features/save_state/save_state_dialog.dart';
import '../features/save_state/save_state_repository.dart';
import '../features/screen/emulation_status_overlay.dart';
import '../features/screen/nes_screen_view.dart';
import '../features/settings/emulation_settings.dart';
import '../features/settings/settings_page.dart';
import '../features/settings/video_settings.dart';
import '../l10n/app_localizations.dart';
import '../logging/app_logger.dart';
import '../platform/platform_capabilities.dart';
import '../platform/web_cmd_sender.dart';
import '../platform/nes_emulation.dart' as nes_emulation;
import 'nes_actions.dart';
import 'nes_menu_bar.dart';
import 'nes_menu_model.dart';

class WebShell extends ConsumerStatefulWidget {
  const WebShell({super.key});

  @override
  ConsumerState<WebShell> createState() => _WebShellState();
}

class _WebShellState extends ConsumerState<WebShell> {
  static const int _nesWidth = 256;
  static const int _nesHeight = 240;
  static const double _desktopMenuMinWidth = 820;
  static const Duration _cursorHideDelay = Duration(seconds: 2);

  final FocusNode _focusNode = FocusNode();

  late final String _viewType;
  web.HTMLCanvasElement? _canvas;
  web.OffscreenCanvas? _offscreenCanvas;

  web.Worker? _worker;

  bool _workerInitialized = false;
  Completer<void>? _initCompleter;
  bool _running = false;
  String? _error;

  JSObject? _audioContext;
  JSObject? _audioPort;

  String? _lastSnackMessage;
  DateTime? _lastSnackAt;
  String? _lastError;
  String? _workerBlobUrl;

  Timer? _cursorTimer;
  bool _cursorHidden = false;
  bool _menuVisible = false;

  @override
  void initState() {
    super.initState();
    _viewType = 'nesium-canvas-${DateTime.now().microsecondsSinceEpoch}';
    _initCanvasView();
    unawaitedLogged(
      _warmupNesWasm(),
      message: 'warmup NES wasm',
      logger: 'web_shell',
    );
  }

  @override
  void dispose() {
    _cursorTimer?.cancel();
    final worker = _worker;
    if (worker != null) {
      worker.onmessage = null;
      worker.onerror = null;
      worker.onmessageerror = null;
      worker.terminate();
    }
    _worker = null;
    final blobUrl = _workerBlobUrl;
    if (blobUrl != null) {
      web.URL.revokeObjectURL(blobUrl);
      _workerBlobUrl = null;
    }
    _initCompleter = null;
    setWebCmdSender(null);
    setWebNesReady(false);
    _stopAudio();
    _focusNode.dispose();
    super.dispose();
  }

  void _showCursorAndArmTimer() {
    if (_cursorHidden) {
      setState(() => _cursorHidden = false);
      _setCanvasCursorHidden(false);
    }
    _cursorTimer?.cancel();
    _cursorTimer = Timer(_cursorHideDelay, () {
      if (!mounted) return;
      if (_cursorHidden) return;
      setState(() => _cursorHidden = true);
      _setCanvasCursorHidden(true);
    });
  }

  void _showCursorAndCancelTimer() {
    _cursorTimer?.cancel();
    _cursorTimer = null;
    if (_cursorHidden) {
      setState(() => _cursorHidden = false);
      _setCanvasCursorHidden(false);
    }
  }

  void _setCanvasCursorHidden(bool hidden) {
    final canvas = _canvas;
    if (canvas == null) return;
    canvas.style.cursor = hidden ? 'none' : 'default';
  }

  void _initCanvasView() {
    final canvas = web.HTMLCanvasElement()
      ..width = _nesWidth
      ..height = _nesHeight;
    canvas.style
      ..width = '100%'
      ..height = '100%'
      ..backgroundColor = 'black'
      ..setProperty('image-rendering', 'pixelated')
      ..setProperty('image-rendering', 'crisp-edges')
      ..setProperty('touch-action', 'none');

    canvas.style.cursor = 'default';
    canvas.onmousemove = ((web.Event _) => _showCursorAndArmTimer()).toJS;
    canvas.onmouseenter = ((web.Event _) => _showCursorAndArmTimer()).toJS;
    canvas.onmouseleave = ((web.Event _) => _showCursorAndCancelTimer()).toJS;

    _canvas = canvas;
    ui_web.platformViewRegistry.registerViewFactory(_viewType, (_) => canvas);
  }

  Future<web.Worker> _createBlobWorker(String scriptUrl) async {
    final baseUrl = web.window.location.href.substring(
      0,
      web.window.location.href.lastIndexOf('/') + 1,
    );
    final absoluteScriptUrl = Uri.parse(baseUrl).resolve(scriptUrl).toString();

    // 1. Fetch the worker script
    final response = await web.window.fetch(absoluteScriptUrl.toJS).toDart;
    if (!response.ok) {
      throw StateError(
        'Failed to fetch worker script: ${response.status} ${response.statusText}',
      );
    }
    var scriptText = (await response.text().toDart).toDart;

    // 2. Rewrite relative imports because Blob URLs don't have a base directory.
    // Example: import("./pkg/nesium_wasm.js") -> import("http://host/nes/pkg/nesium_wasm.js")
    final workerBaseUrl = absoluteScriptUrl.substring(
      0,
      absoluteScriptUrl.lastIndexOf('/') + 1,
    );
    scriptText = scriptText.replaceAll('import("./', 'import("$workerBaseUrl');
    scriptText = scriptText.replaceAll("import('./", "import('$workerBaseUrl");

    // 3. Create the Blob and ObjectURL
    // We cast to BlobPart which is required by the Blob constructor.
    final blob = web.Blob(
      JSArray<web.BlobPart>()..add(scriptText.toJS as web.BlobPart),
      web.BlobPropertyBag(type: 'application/javascript'),
    );
    final objectUrl = web.URL.createObjectURL(blob);
    _workerBlobUrl = objectUrl;

    // 4. Instantiate the worker
    return web.Worker(objectUrl.toJS, web.WorkerOptions(type: 'module'));
  }

  Future<void> _ensureWorker() async {
    if (_worker != null) return;

    final url = 'nes/nes_worker.js';
    Logger('web_shell').info(
      'Creating blob worker from $url (current location: ${web.window.location.href})',
    );

    try {
      final worker = await _createBlobWorker(url);
      _worker = worker;
      setWebCmdSender((cmd, extra) => _postCmd(cmd, extra));
      setWebRequestSender(_requestWorker);
      worker.onmessage = ((web.MessageEvent e) => _onWorkerMessage(e)).toJS;
      worker.onerror = ((web.Event e) {
        final details = _formatWorkerErrorEvent(e);
        _initCompleter?.completeError(StateError(details ?? 'Worker error'));
        _handleFatalWorkerFailure(details ?? 'Worker error', error: e);
      }).toJS;
      worker.onmessageerror = ((web.Event e) {
        final details = _formatWorkerErrorEvent(e);
        _initCompleter?.completeError(
          StateError(details ?? 'Worker message error'),
        );
        _handleFatalWorkerFailure(details ?? 'Worker message error', error: e);
      }).toJS;
    } catch (e, st) {
      _handleFatalWorkerFailure(
        'Failed to create Blob Worker: $e',
        error: e,
        stackTrace: st,
      );
      rethrow;
    }
  }

  Future<void> _warmupNesWasm() async {
    await _ensureWorker();
    final payload = JSObject()..['type'] = 'preload'.toJS;
    _worker?.postMessage(payload);
  }

  Future<void> _ensureInitialized() async {
    await _ensureWorker();
    if (_workerInitialized) return;
    final existing = _initCompleter;
    if (existing != null) return existing.future;

    await _preflightWebAssetsOrThrow();
    final completer = Completer<void>();
    _initCompleter = completer;

    final canvas = _canvas;
    if (canvas == null) {
      final err = StateError('Canvas not initialized');
      _initCompleter = null;
      throw err;
    }

    final sampleRate = await _startAudio();

    _offscreenCanvas ??= canvas.transferControlToOffscreen();
    final payload = JSObject()
      ..['type'] = 'init'.toJS
      ..['canvas'] = _offscreenCanvas
      ..['width'] = _nesWidth.toJS
      ..['height'] = _nesHeight.toJS
      ..['sampleRate'] = sampleRate.toJS;
    final transfer = JSArray<JSAny?>()..add(_offscreenCanvas!);
    _worker!.postMessage(payload, transfer);

    try {
      await completer.future.timeout(const Duration(seconds: 10));
    } on TimeoutException {
      _initCompleter = null;
      throw StateError(
        'Worker init timed out.\n'
        'Check DevTools Network for `nes/nes_worker.js`, `nes/pkg/nesium_wasm.js`, and `nes/pkg/nesium_wasm_bg.wasm`.',
      );
    }
  }

  Future<int> _startAudio() async {
    if (_audioContext != null) {
      try {
        return (_audioContext!['sampleRate'] as JSNumber).toDartInt;
      } catch (e, st) {
        Logger('web_shell').fine(
          'Failed to read existing AudioContext.sampleRate; defaulting to 48000',
          e,
          st,
        );
        return 48_000;
      }
    }

    final audioContextCtor =
        (globalContext['AudioContext'] as JSFunction?) ??
        (globalContext['webkitAudioContext'] as JSFunction?);
    if (audioContextCtor == null) {
      throw UnsupportedError('WebAudio not available');
    }

    final ctx = audioContextCtor.callAsConstructor<JSObject>(
      (JSObject()..['latencyHint'] = 'interactive'.toJS),
    );

    final worklet = ctx['audioWorklet'] as JSObject;
    await worklet
        .callMethod<JSPromise<JSAny?>>(
          'addModule'.toJS,
          'nes/audio_worklet.js'.toJS,
        )
        .toDart;

    final nodeCtor = globalContext['AudioWorkletNode'] as JSFunction;
    final outputChannelCount = JSArray<JSNumber>()..add(2.toJS);
    final nodeOpts = JSObject()
      ..['numberOfOutputs'] = 1.toJS
      ..['outputChannelCount'] = outputChannelCount;
    final node = nodeCtor.callAsConstructor<JSObject>(
      ctx,
      'nes-audio'.toJS,
      nodeOpts,
    );

    node.callMethod<JSAny?>('connect'.toJS, ctx['destination']);
    await ctx.callMethod<JSPromise<JSAny?>>('resume'.toJS).toDart;

    _audioContext = ctx;
    _audioPort = node['port'] as JSObject?;

    try {
      return (ctx['sampleRate'] as JSNumber).toDartInt;
    } catch (e, st) {
      Logger('web_shell').fine(
        'Failed to read AudioContext.sampleRate; defaulting to 48000',
        e,
        st,
      );
      return 48_000;
    }
  }

  void _stopAudio() {
    final ctx = _audioContext;
    _audioPort = null;
    _audioContext = null;
    if (ctx != null) {
      try {
        ctx.callMethod<JSAny?>('close'.toJS);
      } catch (e, st) {
        Logger('web_shell').fine('Failed to close AudioContext', e, st);
      }
    }
  }

  final Map<String, Completer<Object?>> _pendingRequests = {};
  int _requestIdCounter = 0;

  void _postCmd(String cmd, [Map<String, Object?>? extra]) {
    final payload = JSObject()
      ..['type'] = 'cmd'.toJS
      ..['cmd'] = cmd.toJS;
    extra?.forEach((k, v) => payload[k] = _toJsAny(v));
    _worker?.postMessage(payload);
  }

  Future<T> _requestWorker<T>(String cmd, [Map<String, Object?>? extra]) {
    final id = 'req_${_requestIdCounter++}';
    final completer = Completer<T>();
    _pendingRequests[id] = completer;

    final payload = JSObject()
      ..['type'] = 'cmd'.toJS
      ..['cmd'] = cmd.toJS
      ..['requestId'] = id.toJS;
    extra?.forEach((k, v) => payload[k] = _toJsAny(v));
    _worker?.postMessage(payload);

    return completer.future;
  }

  void _onWorkerMessage(web.MessageEvent event) {
    final dataAny = event.data;
    if (dataAny == null) return;
    final data = dataAny as JSObject;
    final type = (data['type'] as JSString?)?.toDart;
    if (type == null) return;

    if (type == 'log') {
      final message = (data['message'] as JSString?)?.toDart;
      if (message != null && message.isNotEmpty) {
        // Useful when the worker errors before it can post a structured error.
        // ignore: avoid_print
        print('[nes_worker] $message');
      }
      return;
    }

    if (type == 'romLoaded') {
      final hashList = (data['hash'] as JSArray?)?.toDart;
      if (hashList != null) {
        final bytes = Uint8List.fromList(
          hashList.map((e) => (e as JSNumber).toDartInt).toList(),
        );
        final hashStr = bytes
            .map((b) => b.toRadixString(16).padLeft(2, '0'))
            .join();
        ref.read(nesControllerProvider.notifier).updateRomHash(hashStr);
      } else {
        ref.read(nesControllerProvider.notifier).updateRomHash(null);
      }
      return;
    }

    if (type == 'saveStateResult') {
      final requestId = (data['requestId'] as JSString?)?.toDart;
      final buffer = data['data'];
      if (requestId != null && _pendingRequests.containsKey(requestId)) {
        if (buffer != null) {
          final bytes = (buffer as JSArrayBuffer).toDart.asUint8List();
          _pendingRequests.remove(requestId)!.complete(bytes);
        } else {
          _pendingRequests
              .remove(requestId)!
              .completeError(StateError('Save state failed'));
        }
      }
      return;
    }

    if (type == 'loadStateResult') {
      final requestId = (data['requestId'] as JSString?)?.toDart;
      final success = (data['success'] as JSBoolean?)?.toDart ?? false;
      if (requestId != null && _pendingRequests.containsKey(requestId)) {
        if (success) {
          _pendingRequests.remove(requestId)!.complete(null);
        } else {
          _pendingRequests
              .remove(requestId)!
              .completeError(StateError('Load state failed'));
        }
      }
      return;
    }

    if (type == 'ready') {
      _initCompleter?.complete();
      setWebNesReady(true);
      ref.read(nesInputMasksProvider.notifier).flushToNative();
      ref.read(emulationSettingsProvider.notifier).applyToRuntime();
      ref.read(turboSettingsProvider.notifier).applyToRuntime();
      unawaited(ref.read(videoSettingsProvider.notifier).applyToRuntime());
      setState(() {
        _workerInitialized = true;
        _error = null;
      });
      return;
    }

    if (type == 'running') {
      final value = data['value'] as JSBoolean?;
      final running = value?.toDart ?? false;
      setState(() => _running = running);
      ref.read(emulationStatusProvider.notifier).setPaused(!running);
      return;
    }

    if (type == 'error') {
      final message = (data['message'] as JSString?)?.toDart;
      final err = StateError(message ?? 'Unknown worker error');
      if (!_workerInitialized) {
        _initCompleter?.completeError(err);
        _initCompleter = null;
        setWebNesReady(false);
      } else {
        // Keep the runtime "ready" so input stays active. Worker-reported errors
        // are usually recoverable (e.g. bad ROM, paused run loop).
        setWebNesReady(true);
      }
      _reportError(
        message ?? 'Unknown worker error',
        persistent: true,
        error: err,
      );
      setState(() {
        _error = message ?? 'Unknown worker error';
        _running = false;
      });
      ref.read(emulationStatusProvider.notifier).setPaused(true);
      return;
    }

    if (type == 'audio' && _audioPort != null) {
      final buffer = data['buffer'];
      if (buffer == null) return;
      final transfer = JSArray<JSAny?>()..add(buffer);
      _audioPort!.callMethodVarArgs<JSAny?>('postMessage'.toJS, [
        buffer,
        transfer,
      ]);
    }
  }

  String? _formatWorkerErrorEvent(web.Event e) {
    String? type;
    try {
      final obj = e as JSObject;
      type = (obj['type'] as JSString?)?.toDart;

      // Try to get structured error info if e is ErrorEvent
      final message = (obj['message'] as JSString?)?.toDart;
      final filename = (obj['filename'] as JSString?)?.toDart;
      final lineno = (obj['lineno'] as JSNumber?)?.toDartInt;
      final colno = (obj['colno'] as JSNumber?)?.toDartInt;

      final parts = <String>[];
      if (message != null && message.isNotEmpty) parts.add(message);
      if (filename != null && filename.isNotEmpty) {
        final loc = (lineno != null && colno != null)
            ? '$filename:$lineno:$colno'
            : filename;
        parts.add(loc);
      }

      final out = parts.join('\n');
      if (out.isNotEmpty) {
        if (out.contains('nesium_wasm') || out.contains('nes/nes_worker.js')) {
          return '$out\n\nHint: run `dart run tool/build_wasm.dart` to build `web/nes/pkg/` (wasm-pack output).';
        }
        return out;
      }
    } catch (err) {
      logWarning(
        err,
        message: 'Failed to format worker error',
        logger: 'web_shell',
      );
    }

    // Fallback: search for any info on the event object.
    try {
      final obj = e as JSObject;
      final target = obj['target'];
      logWarning(
        e,
        message: 'Worker error event (type: $type)',
        logger: 'web_shell',
      );

      if (target != null) {
        final targetObj = target as JSObject;
        final url = (targetObj['url'] as JSString?)?.toDart;
        if (url != null) {
          return 'Failed to load worker at $url';
        }
      }

      final str = e.toString();
      if (str != '[object Event]') return str;
      if (type != null) return 'Worker error ($type)';
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Worker error fallback formatting failed',
        logger: 'web_shell',
      );
    }

    return null;
  }

  Future<void> _preflightWebAssetsOrThrow() async {
    // If wasm-pack output is missing, the worker's dynamic import will fail.
    // We preflight here to provide a clearer error message.
    final workerUrl = 'nes/nes_worker.js';
    final jsUrl = 'nes/pkg/nesium_wasm.js';
    final wasmUrl = 'nes/pkg/nesium_wasm_bg.wasm';

    final workerOk = await _fetchOk(workerUrl);
    final jsOk = await _fetchOk(jsUrl);
    final wasmOk = await _fetchOk(wasmUrl);
    if (workerOk && jsOk && wasmOk) return;

    final missing = <String>[
      if (!workerOk) workerUrl,
      if (!jsOk) jsUrl,
      if (!wasmOk) wasmUrl,
    ].join('\n');

    final msg =
        'Missing Web assets:\n$missing\n\n'
        'Run: `cd apps/nesium_flutter && dart run tool/build_wasm.dart`';
    _reportError(msg, persistent: true);
    throw StateError(msg);
  }

  Future<bool> _fetchOk(String url) async {
    try {
      final resp = await web.window.fetch(url.toJS).toDart;
      return resp.ok;
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'fetch failed: $url',
        logger: 'web_shell',
      );
      return false;
    }
  }

  Future<void> _pickAndLoadRom() async {
    setState(() => _error = null);
    final result = await FilePicker.platform.pickFiles(
      withData: true,
      allowMultiple: false,
      type: FileType.custom,
      allowedExtensions: const ['nes'],
    );
    if (!mounted || result == null || result.files.isEmpty) return;

    final file = result.files.single;
    final bytes = file.bytes;
    if (bytes == null) {
      _reportError('Failed to read ROM bytes');
      return;
    }

    final name = p.basenameWithoutExtension(file.name);

    try {
      await _ensureInitialized();
      ref.read(nesControllerProvider.notifier).updateRomInfo(name: name);
      final u8 = bytes.toJS;
      final arrayBuffer = (u8 as JSObject)['buffer'] as JSArrayBuffer;
      final payload = JSObject()
        ..['type'] = 'cmd'.toJS
        ..['cmd'] = 'loadRom'.toJS
        ..['rom'] = arrayBuffer;
      final transfer = JSArray<JSAny?>()..add(arrayBuffer);
      _worker?.postMessage(payload, transfer);
      // Match native behavior: start running immediately after loading a ROM.
      _postCmd('run', {'emitAudio': true});
      setState(() {});
    } catch (e) {
      _reportError(e.toString());
    }
  }

  Future<void> _toggleRun() async {
    setState(() => _error = null);
    try {
      await _ensureInitialized();
      if (_running) {
        _postCmd('pause');
        ref.read(emulationStatusProvider.notifier).setPaused(true);
      } else {
        _postCmd('run', {'emitAudio': true});
        ref.read(emulationStatusProvider.notifier).setPaused(false);
      }
    } catch (e) {
      _reportError(e.toString());
    }
  }

  Future<void> _togglePause() async {
    await _toggleRun();
  }

  Future<void> _reset({required bool powerOn}) async {
    setState(() => _error = null);
    try {
      await _ensureInitialized();
      _postCmd(powerOn ? 'powerOnReset' : 'softReset');
    } catch (e) {
      _reportError(e.toString());
    }
  }

  Future<void> _saveState() async {
    await showDialog<void>(
      context: context,
      builder: (_) => const SaveStateDialog(isSaving: true),
    );
  }

  Future<void> _loadState() async {
    await showDialog<void>(
      context: context,
      builder: (_) => const SaveStateDialog(isSaving: false),
    );
  }

  int _quickSaveSlot() => ref.read(emulationSettingsProvider).quickSaveSlot;

  void _quickSaveState() {
    final slot = _quickSaveSlot();
    unawaitedLogged(
      _saveToSlot(slot),
      message: 'saveState to repository (slot $slot)',
      logger: 'web_shell',
    );
  }

  void _quickLoadState() {
    final slot = _quickSaveSlot();
    unawaitedLogged(
      _loadFromSlot(slot),
      message: 'loadState from repository (slot $slot)',
      logger: 'web_shell',
    );
  }

  Future<void> _openAutoSaveDialog() async {
    await showDialog<void>(
      context: context,
      builder: (_) => const SaveStateDialog(isSaving: false, isAutoSave: true),
    );
  }

  Future<void> _saveToSlot(int slot) async {
    final l10n = AppLocalizations.of(context)!;
    final repository = ref.read(saveStateRepositoryProvider.notifier);
    try {
      final data = await _requestWorker<Uint8List>('saveState');
      await repository.saveState(slot, data);
      if (mounted) {
        _showSnack(l10n.stateSavedToSlot(slot));
      }
    } catch (e) {
      if (mounted) {
        _showSnack('${l10n.commandFailed('Save to slot $slot')}: $e');
      }
    }
  }

  Future<void> _loadFromSlot(int slot) async {
    final l10n = AppLocalizations.of(context)!;
    final repository = ref.read(saveStateRepositoryProvider.notifier);
    try {
      if (!repository.hasSave(slot)) return;
      final data = await repository.loadState(slot);
      if (data != null) {
        await _requestWorker<void>('loadState', {'data': data});
        if (mounted) {
          _showSnack(l10n.stateLoadedFromSlot(slot));
        }
      }
    } catch (e) {
      if (mounted) {
        _showSnack('${l10n.commandFailed('Load from slot $slot')}: $e');
      }
    }
  }

  Future<void> _saveToFile() async {
    final l10n = AppLocalizations.of(context)!;
    const XTypeGroup typeGroup = XTypeGroup(
      label: 'Nesium State',
      extensions: <String>['nesium'],
    );

    try {
      final data = await _requestWorker<Uint8List>('saveState');
      final romName = ref.read(nesControllerProvider).romName ?? 'save';
      final suggestedName = '$romName.nesium';

      final FileSaveLocation? result = await getSaveLocation(
        acceptedTypeGroups: <XTypeGroup>[typeGroup],
        suggestedName: suggestedName,
      );

      if (result != null) {
        final file = XFile.fromData(
          data,
          name: suggestedName,
          mimeType: 'application/octet-stream',
        );
        await file.saveTo(result.path);
      }
    } catch (e) {
      if (mounted) {
        _showSnack('${l10n.commandFailed('Save to file')}: $e');
      }
    }
  }

  Future<void> _loadFromFile() async {
    final l10n = AppLocalizations.of(context)!;
    const XTypeGroup typeGroup = XTypeGroup(
      label: 'Nesium State',
      extensions: <String>['nesium'],
    );

    try {
      final XFile? result = await openFile(
        acceptedTypeGroups: <XTypeGroup>[typeGroup],
      );

      if (result != null) {
        final bytes = await result.readAsBytes();
        await _requestWorker<void>('loadState', {'data': bytes});
        if (mounted) {
          _showSnack(l10n.commandSucceeded('Load from file'));
        }
      }
    } catch (e) {
      if (mounted) {
        _showSnack('${l10n.commandFailed('Load from file')}: $e');
      }
    }
  }

  Future<void> _loadTasMovie() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['fm2'],
      withData: true,
      withReadStream: false,
    );
    final file = result?.files.single;
    if (file == null) return;

    final bytes = file.bytes;
    if (bytes == null) return;

    final data = String.fromCharCodes(bytes);

    if (!mounted) return;
    await _runRustCommand('Load TAS Movie', () async {
      await nes_emulation.loadTasMovie(data: data);
    });
  }

  KeyEventResult _handleKeyEvent(FocusNode _, KeyEvent event) {
    // Avoid sending key events to the emulator when a different route (e.g. settings)
    // is on top.
    final route = ModalRoute.of(context);
    if (route != null && !route.isCurrent) {
      return KeyEventResult.ignored;
    }

    // Treat key repeat as a continued key down to avoid system beeps.
    if (event is! KeyDownEvent &&
        event is! KeyUpEvent &&
        event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }

    final pressed = event is KeyDownEvent || event is KeyRepeatEvent;
    final key = event.logicalKey;

    final inputState = ref.read(inputSettingsProvider);

    // TODO: support Netplay in Web? For now just local
    var handled = false;
    for (var i = 0; i < 4; i++) {
      final settings = inputState.ports[i]!;
      if (settings.device != InputDevice.keyboard) continue;

      final action = settings.resolveKeyboardBindings()[key];
      if (action == null) continue;

      final input = ref.read(nesInputMasksProvider.notifier);
      switch (action) {
        case KeyboardBindingAction.up:
          input.setPressed(PadButton.up, pressed, pad: i);
          break;
        case KeyboardBindingAction.down:
          input.setPressed(PadButton.down, pressed, pad: i);
          break;
        case KeyboardBindingAction.left:
          input.setPressed(PadButton.left, pressed, pad: i);
          break;
        case KeyboardBindingAction.right:
          input.setPressed(PadButton.right, pressed, pad: i);
          break;
        case KeyboardBindingAction.a:
          input.setPressed(PadButton.a, pressed, pad: i);
          break;
        case KeyboardBindingAction.b:
          input.setPressed(PadButton.b, pressed, pad: i);
          break;
        case KeyboardBindingAction.select:
          input.setPressed(PadButton.select, pressed, pad: i);
          break;
        case KeyboardBindingAction.start:
          input.setPressed(PadButton.start, pressed, pad: i);
          break;
        case KeyboardBindingAction.turboA:
          input.setTurboEnabled(PadButton.a, pressed, pad: i);
          break;
        case KeyboardBindingAction.turboB:
          input.setTurboEnabled(PadButton.b, pressed, pad: i);
          break;
        case KeyboardBindingAction.rewind:
          ref.read(emulationStatusProvider.notifier).setRewinding(pressed);
          unawaitedLogged(
            nes_emulation.setRewinding(rewinding: pressed),
            message: 'setRewinding ($pressed)',
            logger: 'web_shell',
          );
          break;
        case KeyboardBindingAction.fastForward:
          ref.read(emulationStatusProvider.notifier).setFastForwarding(pressed);
          unawaitedLogged(
            nes_emulation.setFastForwarding(fastForwarding: pressed),
            message: 'setFastForwarding ($pressed)',
            logger: 'web_shell',
          );
          break;
        case KeyboardBindingAction.saveState:
          if (pressed) {
            _quickSaveState();
          }
          break;
        case KeyboardBindingAction.loadState:
          if (pressed) {
            _quickLoadState();
          }
          break;
        case KeyboardBindingAction.pause:
          if (pressed) _togglePause();
          break;
        case KeyboardBindingAction.fullScreen:
          if (pressed) {
            final videoSettings = ref.read(videoSettingsProvider);
            unawaited(
              ref
                  .read(videoSettingsProvider.notifier)
                  .setFullScreen(!videoSettings.fullScreen),
            );
          }
          break;
      }
      handled = true;
    }

    return handled ? KeyEventResult.handled : KeyEventResult.ignored;
  }

  JSAny? _toJsAny(Object? value) {
    if (value == null) return null;
    if (value is bool) return value.toJS;
    if (value is num) return value.toJS;
    if (value is String) return value.toJS;
    if (value is Uint8List) return value.toJS;
    if (value is BigInt) return value.toInt().toJS;
    throw ArgumentError.value(value, 'value', 'Unsupported JS interop value');
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    ref.watch(gamepadServiceProvider); // Keep gamepad polling running on web
    ref.watch(autoSaveServiceProvider); // Keep auto-save timer running on web
    final actions = NesActions(
      openRom: _pickAndLoadRom,
      saveState: _saveState,
      loadState: _loadState,
      openAutoSave: _openAutoSaveDialog,
      saveStateSlot: _saveToSlot,
      loadStateSlot: _loadFromSlot,
      saveStateFile: _saveToFile,
      loadStateFile: _loadFromFile,
      loadTasMovie: _loadTasMovie,
      reset: () => _reset(powerOn: false),
      powerReset: () => _reset(powerOn: true),
      powerOff: () async {
        if (!mounted) return;
        ScaffoldMessenger.of(context).showSnackBar(
          const SnackBar(content: Text('Power Off is not supported on Web')),
        );
      },
      togglePause: _togglePause,
      setRewinding: (active) {
        ref.read(emulationStatusProvider.notifier).setRewinding(active);
        unawaitedLogged(
          nes_emulation.setRewinding(rewinding: active),
          message: 'setRewinding ($active)',
          logger: 'web_shell',
        );
      },
      setFastForwarding: (active) {
        ref.read(emulationStatusProvider.notifier).setFastForwarding(active);
        unawaitedLogged(
          nes_emulation.setFastForwarding(fastForwarding: active),
          message: 'setFastForwarding ($active)',
          logger: 'web_shell',
        );
      },
      openSettings: () async {
        if (!mounted) return;
        await Navigator.of(
          context,
        ).push(MaterialPageRoute<void>(builder: (_) => const SettingsPage()));
      },
      openAbout: () async {
        if (!mounted) return;
        await Navigator.of(
          context,
        ).push(MaterialPageRoute<void>(builder: (_) => const AboutPage()));
      },
      openDebugger: () async {},
      openTools: () async {},
      openTilemapViewer: () async {},
      openTileViewer: () async {},
    );

    final videoSettings = ref.watch(videoSettingsProvider);
    final slotStates = ref.watch(saveStateRepositoryProvider);
    final hasRom = ref.watch(
      nesControllerProvider.select((s) => s.romHash != null),
    );
    final lastError = _error ?? _lastError;

    final isLandscape =
        MediaQuery.orientationOf(context) == Orientation.landscape;
    final useDrawerMenu =
        MediaQuery.sizeOf(context).width < _desktopMenuMinWidth;

    const double menuHeight = 28;
    final isFullScreen = videoSettings.fullScreen;

    return Scaffold(
      appBar: useDrawerMenu && !isLandscape
          ? AppBar(
              title: Text(l10n.appName),
              actions: [
                if (lastError != null)
                  IconButton(
                    tooltip: l10n.menuLastError,
                    icon: const Icon(Icons.error_outline),
                    onPressed: _showLastErrorDialog,
                  ),
              ],
            )
          : null,
      drawer: useDrawerMenu
          ? Drawer(child: _buildDrawer(context, actions, hasRom))
          : null,
      body: Focus(
        focusNode: _focusNode,
        autofocus: true,
        onKeyEvent: _handleKeyEvent,
        child: MouseRegion(
          onHover: (event) {
            if (!isFullScreen || useDrawerMenu) return;
            // Show menu if mouse is within top 40px
            final bool nearTop = event.localPosition.dy < 40;
            if (nearTop != _menuVisible) {
              setState(() => _menuVisible = nearTop);
            }
          },
          child: Stack(
            children: [
              // Main Content
              Positioned.fill(
                child: Column(
                  children: [
                    if (!useDrawerMenu)
                      AnimatedContainer(
                        duration: const Duration(milliseconds: 200),
                        height: isFullScreen ? 0 : menuHeight,
                      ),
                    Expanded(
                      child: LayoutBuilder(
                        builder: (context, constraints) {
                          final videoSettings = ref.watch(
                            videoSettingsProvider,
                          );
                          final inputState = ref.watch(inputSettingsProvider);
                          final editor = ref.watch(
                            virtualControlsEditorProvider,
                          );
                          final controlsSettings = ref.watch(
                            virtualControlsSettingsProvider,
                          );

                          final usingVirtual =
                              editor.enabled ||
                              inputState.ports[0]!.device ==
                                  InputDevice.virtualController;
                          final autoOffsetY = (!isLandscape && usingVirtual)
                              ? -(controlsSettings.buttonSize * 0.55)
                              : 0.0;
                          final screenOffsetY =
                              videoSettings.screenVerticalOffset + autoOffsetY;

                          final viewport = NesScreenView.computeViewportSize(
                            constraints,
                            integerScaling: videoSettings.integerScaling,
                            aspectRatio: videoSettings.aspectRatio,
                          );
                          if (viewport == null) return const SizedBox.shrink();

                          final view = Transform.translate(
                            offset: Offset(0, screenOffsetY),
                            child: SizedBox(
                              width: viewport.width,
                              height: viewport.height,
                              child: GestureDetector(
                                onTap: () => _focusNode.requestFocus(),
                                child: Stack(
                                  fit: StackFit.expand,
                                  children: [
                                    HtmlElementView(viewType: _viewType),
                                    if (hasRom) const EmulationStatusOverlay(),
                                  ],
                                ),
                              ),
                            ),
                          );

                          return Container(
                            color: Colors.black,
                            alignment: Alignment.center,
                            child: Stack(
                              alignment: Alignment.center,
                              children: [
                                view,
                                if (useDrawerMenu && isLandscape)
                                  Positioned(
                                    left: 0,
                                    top: 0,
                                    child: SafeArea(
                                      child: Padding(
                                        padding: const EdgeInsets.all(8),
                                        child: Builder(
                                          builder: (context) => Material(
                                            color: Colors.black54,
                                            borderRadius: BorderRadius.circular(
                                              12,
                                            ),
                                            clipBehavior: Clip.antiAlias,
                                            child: Row(
                                              mainAxisSize: MainAxisSize.min,
                                              children: [
                                                IconButton(
                                                  onPressed: () => Scaffold.of(
                                                    context,
                                                  ).openDrawer(),
                                                  icon: const Icon(Icons.menu),
                                                  color: Colors.white,
                                                  tooltip: l10n.menuTooltip,
                                                ),
                                                if (lastError != null)
                                                  IconButton(
                                                    onPressed:
                                                        _showLastErrorDialog,
                                                    icon: const Icon(
                                                      Icons.error_outline,
                                                    ),
                                                    color: Colors.white,
                                                    tooltip: l10n.menuLastError,
                                                  ),
                                              ],
                                            ),
                                          ),
                                        ),
                                      ),
                                    ),
                                  ),
                                VirtualControlsOverlay(
                                  isLandscape: isLandscape,
                                ),
                              ],
                            ),
                          );
                        },
                      ),
                    ),
                  ],
                ),
              ),

              // Menu Bar (Overlay)
              if (!useDrawerMenu)
                AnimatedPositioned(
                  duration: const Duration(milliseconds: 200),
                  curve: Curves.easeInOut,
                  top: (isFullScreen && !_menuVisible) ? -menuHeight : 0,
                  left: 0,
                  right: 0,
                  height: menuHeight,
                  child: MouseRegion(
                    onEnter: (_) {
                      if (isFullScreen) setState(() => _menuVisible = true);
                    },
                    onExit: (_) {
                      if (isFullScreen) setState(() => _menuVisible = false);
                    },
                    child: NesMenuBar(
                      actions: actions,
                      sections: NesMenus.webMenuSections(),
                      slotStates: slotStates,
                      hasRom: hasRom,
                      trailing: Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 8),
                        child: lastError != null
                            ? IconButton(
                                tooltip: l10n.menuLastError,
                                icon: const Icon(
                                  Icons.error_outline,
                                  color: Colors.red,
                                  size: 18,
                                ),
                                onPressed: _showLastErrorDialog,
                              )
                            : const SizedBox.shrink(),
                      ),
                    ),
                  ),
                ),
            ],
          ),
        ),
      ),
    );
  }

  void _reportError(
    String message, {
    Object? error,
    StackTrace? stackTrace,
    bool persistent = false,
  }) {
    if (message.trim().isEmpty) return;

    final l10n = AppLocalizations.of(context)!;
    logWarning(
      error ?? message,
      stackTrace: stackTrace,
      message: message,
      logger: 'web_shell',
    );

    final now = DateTime.now();
    final lastAt = _lastSnackAt;
    final lastMessage = _lastSnackMessage;
    if (lastAt != null &&
        lastMessage == message &&
        now.difference(lastAt) < const Duration(seconds: 2)) {
      if (persistent && mounted) {
        setState(() => _error = message);
      }
      return;
    }

    _lastSnackAt = now;
    _lastSnackMessage = message;

    if (mounted) {
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(message),
          behavior: SnackBarBehavior.floating,
          duration: persistent
              ? const Duration(seconds: 8)
              : const Duration(seconds: 4),
          action: SnackBarAction(
            label: l10n.lastErrorDetailsAction,
            onPressed: _showLastErrorDialog,
          ),
        ),
      );
    }

    _lastError = message;
    _lastSnackMessage = message;
  }

  void _showSnack(String message) {
    if (!mounted) return;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  Future<void> _runRustCommand(
    String label,
    Future<void> Function() action,
  ) async {
    try {
      await action();
    } catch (e) {
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      _showSnack('${l10n.commandFailed(label)}: $e');
    }
  }

  void _handleFatalWorkerFailure(
    String message, {
    Object? error,
    StackTrace? stackTrace,
  }) {
    final worker = _worker;
    if (worker != null) {
      worker.onmessage = null;
      worker.onerror = null;
      worker.onmessageerror = null;
      worker.terminate();
    }
    _worker = null;
    final blobUrl = _workerBlobUrl;
    if (blobUrl != null) {
      web.URL.revokeObjectURL(blobUrl);
      _workerBlobUrl = null;
    }
    _initCompleter = null;
    _workerInitialized = false;
    _running = false;

    setWebCmdSender(null);
    setWebNesReady(false);

    _reportError(
      '$message\n\nHint: refresh the page to fully reset the Web runtime.',
      persistent: true,
      error: error ?? message,
      stackTrace: stackTrace,
    );
  }

  void _showLastErrorDialog() {
    final message = _lastError ?? _error ?? _lastSnackMessage;
    if (message == null || message.trim().isEmpty) return;
    final l10n = AppLocalizations.of(context)!;
    final localizations = MaterialLocalizations.of(context);
    showDialog<void>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(l10n.lastErrorDialogTitle),
        content: SelectableText(message),
        actions: [
          TextButton(
            onPressed: () async {
              await Clipboard.setData(ClipboardData(text: message));
              if (context.mounted) {
                Navigator.of(context).pop();
                ScaffoldMessenger.of(context).showSnackBar(
                  SnackBar(
                    content: Text(l10n.lastErrorCopied),
                    behavior: SnackBarBehavior.floating,
                    duration: const Duration(seconds: 2),
                  ),
                );
              }
            },
            child: Text(localizations.copyButtonLabel),
          ),
          TextButton(
            onPressed: () => Navigator.of(context).pop(),
            child: Text(localizations.closeButtonLabel),
          ),
        ],
      ),
    );
  }

  Widget _buildDrawer(BuildContext context, NesActions actions, bool hasRom) {
    final l10n = AppLocalizations.of(context)!;
    void closeDrawer() => Navigator.of(context).pop();

    final inputState = ref.watch(inputSettingsProvider);
    final inputCtrl = ref.read(inputSettingsProvider.notifier);
    final editor = ref.watch(virtualControlsEditorProvider);
    final editorCtrl = ref.read(virtualControlsEditorProvider.notifier);

    return SafeArea(
      child: ListView(
        padding: EdgeInsets.zero,
        children: [
          DrawerHeader(
            margin: EdgeInsets.zero,
            child: Align(
              alignment: Alignment.bottomLeft,
              child: Text(l10n.appName, style: const TextStyle(fontSize: 24)),
            ),
          ),
          for (final section in NesMenus.webMenuSections()) ...[
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
              child: Text(
                section.title(l10n),
                style: Theme.of(context).textTheme.labelLarge,
              ),
            ),
            for (final item in section.items)
              ListTile(
                enabled:
                    ((item.id != NesMenuItemId.saveState &&
                            item.id != NesMenuItemId.loadState &&
                            item.id != NesMenuItemId.autoSave) ||
                        hasRom) &&
                    item.id != NesMenuItemId.loadTasMovie,
                leading: Icon(item.icon),
                title: Text(item.label(l10n)),
                onTap:
                    ((item.id != NesMenuItemId.saveState &&
                                item.id != NesMenuItemId.loadState &&
                                item.id != NesMenuItemId.autoSave) ||
                            hasRom) &&
                        item.id != NesMenuItemId.loadTasMovie
                    ? () {
                        closeDrawer();
                        _dispatchDrawerAction(item.id, actions);
                      }
                    : null,
              ),
            const Divider(height: 1),
          ],
          if (supportsVirtualControls)
            AnimatedSize(
              duration: const Duration(milliseconds: 300),
              curve: Curves.easeInOut,
              child: AnimatedSwitcher(
                duration: const Duration(milliseconds: 300),
                transitionBuilder: (Widget child, Animation<double> animation) {
                  return FadeTransition(
                    opacity: animation,
                    child: SizeTransition(
                      sizeFactor: animation,
                      axisAlignment: -1.0,
                      child: child,
                    ),
                  );
                },
                child:
                    (editor.enabled ||
                        inputState.ports[0]!.device ==
                            InputDevice.virtualController)
                    ? Column(
                        key: const ValueKey('virtual_controls_edit_group'),
                        children: [
                          ListTile(
                            leading: const Icon(Icons.tune),
                            title: Text(l10n.virtualControlsEditTitle),
                            subtitle: Text(
                              editor.enabled
                                  ? l10n.virtualControlsEditSubtitleEnabled
                                  : l10n.virtualControlsEditSubtitleDisabled,
                            ),
                            trailing: Switch(
                              value: editor.enabled,
                              onChanged: (enabled) {
                                if (enabled &&
                                    inputState.ports[0]!.device !=
                                        InputDevice.virtualController) {
                                  inputCtrl.setDevice(
                                    InputDevice.virtualController,
                                  );
                                }
                                editorCtrl.setEnabled(enabled);
                                closeDrawer();
                              },
                            ),
                          ),
                          if (editor.enabled) ...[
                            SwitchListTile(
                              secondary: const Icon(Icons.grid_4x4),
                              title: Text(l10n.gridSnappingTitle),
                              value: editor.gridSnapEnabled,
                              onChanged: editorCtrl.setGridSnapEnabled,
                            ),
                            if (editor.gridSnapEnabled)
                              Padding(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 16,
                                  vertical: 4,
                                ),
                                child: Column(
                                  crossAxisAlignment: CrossAxisAlignment.start,
                                  children: [
                                    Row(
                                      children: [
                                        Expanded(
                                          child: Text(l10n.gridSpacingLabel),
                                        ),
                                        Text(
                                          '${editor.gridSpacing.toStringAsFixed(0)} px',
                                        ),
                                      ],
                                    ),
                                    Slider(
                                      value: editor.gridSpacing.clamp(4, 64),
                                      min: 4,
                                      max: 64,
                                      divisions: 60,
                                      onChanged: editorCtrl.setGridSpacing,
                                    ),
                                  ],
                                ),
                              ),
                          ],
                        ],
                      )
                    : const SizedBox.shrink(
                        key: ValueKey('virtual_controls_edit_hidden'),
                      ),
              ),
            ),
        ],
      ),
    );
  }

  void _dispatchDrawerAction(NesMenuItemId id, NesActions actions) {
    switch (id) {
      case NesMenuItemId.openRom:
        unawaited(actions.openRom?.call());
        break;
      case NesMenuItemId.saveState:
        unawaited(actions.saveState?.call());
        break;
      case NesMenuItemId.loadState:
        unawaited(actions.loadState?.call());
        break;
      case NesMenuItemId.autoSave:
        unawaited(actions.openAutoSave?.call());
        break;
      case NesMenuItemId.reset:
        unawaited(actions.reset?.call());
        break;
      case NesMenuItemId.powerReset:
        unawaited(actions.powerReset?.call());
        break;
      case NesMenuItemId.powerOff:
        unawaited(actions.powerOff?.call());
        break;
      case NesMenuItemId.togglePause:
        unawaited(actions.togglePause?.call());
        break;
      case NesMenuItemId.loadTasMovie:
        unawaited(actions.loadTasMovie?.call());
        break;
      case NesMenuItemId.settings:
        unawaited(actions.openSettings?.call());
        break;
      case NesMenuItemId.about:
        unawaited(actions.openAbout?.call());
        break;
      case NesMenuItemId.debugger:
        unawaited(actions.openDebugger?.call());
        break;
      case NesMenuItemId.tools:
        unawaited(actions.openTools?.call());
        break;
      case NesMenuItemId.tilemapViewer:
      case NesMenuItemId.tileViewer:
      case NesMenuItemId.spriteViewer:
      case NesMenuItemId.paletteViewer:
      case NesMenuItemId.autoSaveSlot:
      case NesMenuItemId.saveStateSlot:
      case NesMenuItemId.loadStateSlot:
      case NesMenuItemId.saveStateFile:
      case NesMenuItemId.loadStateFile:
      case NesMenuItemId.netplay:
        break;
    }
  }

  // Viewport size is computed via `NesScreenView.computeViewportSize(...)`.
}
