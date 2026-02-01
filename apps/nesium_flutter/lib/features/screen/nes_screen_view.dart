import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_controller.dart';
import '../../platform/nes_video.dart' show VideoFilter;
import '../settings/video_settings.dart';
import '../settings/windows_shader_settings.dart';
import '../settings/apple_shader_settings.dart';
import '../settings/windows_video_backend_settings.dart';
import '../settings/android_video_backend_settings.dart';
import '../../domain/nes_texture_service.dart';
import '../../l10n/app_localizations.dart';
import '../../routing/app_route_observer.dart';
import 'emulation_status_overlay.dart';

class NesScreenView extends ConsumerStatefulWidget {
  const NesScreenView({
    super.key,
    this.error,
    required this.textureId,
    this.screenVerticalOffset = 0,
  });

  final String? error;
  final int? textureId;
  final double screenVerticalOffset;

  static const double nesWidth = 256;
  static const double nesHeight = 240;

  static Size? computeViewportSize(
    BoxConstraints constraints, {
    required bool integerScaling,
    required NesAspectRatio aspectRatio,
  }) {
    if (constraints.maxWidth <= 0 || constraints.maxHeight <= 0) {
      return null;
    }

    if (aspectRatio == NesAspectRatio.stretch) {
      return constraints.biggest;
    }

    final targetWidth = aspectRatio == NesAspectRatio.ntsc
        ? nesHeight * (4.0 / 3.0)
        : nesWidth;

    final scale = math.min(
      constraints.maxWidth / targetWidth,
      constraints.maxHeight / nesHeight,
    );
    final finalScale = scale < 1.0
        ? 1.0
        : integerScaling
        ? scale.floorToDouble().clamp(1.0, double.infinity)
        : scale;
    return Size(targetWidth * finalScale, nesHeight * finalScale);
  }

  @override
  ConsumerState<NesScreenView> createState() => _NesScreenViewState();
}

class _NesScreenViewState extends ConsumerState<NesScreenView> with RouteAware {
  static const Duration _cursorHideDelay = Duration(seconds: 2);
  static const Duration _resizeDebounceDelay = Duration(milliseconds: 100);
  static const Duration _overlayResizeDebounceDelay = Duration(
    milliseconds: 500,
  );

  Timer? _cursorTimer;
  bool _cursorHidden = false;
  Size? _lastReportedSize;
  Timer? _resizeDebounceTimer;
  Size? _pendingReportedSize;
  final _gameKey = GlobalKey();
  Rect? _lastOverlayRect;
  bool _nativeOverlayEnabled = false;
  PageRoute<dynamic>? _route;
  bool _isCurrentRoute = true;
  bool _hadRom = false;

  void _showCursorAndArmTimer() {
    if (_cursorHidden) {
      setState(() => _cursorHidden = false);
    }
    _cursorTimer?.cancel();
    _cursorTimer = Timer(_cursorHideDelay, () {
      if (!mounted) return;
      setState(() => _cursorHidden = true);
    });
  }

  void _showCursorAndCancelTimer() {
    _cursorTimer?.cancel();
    _cursorTimer = null;
    if (_cursorHidden) {
      setState(() => _cursorHidden = false);
    }
  }

  @override
  void didUpdateWidget(covariant NesScreenView oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.textureId == null || widget.error != null) {
      _showCursorAndCancelTimer();
    }

    // The Windows native overlay can be reset by backend/texture reinit.
    // Force a re-apply of the overlay rect after texture changes so the
    // native presenter is re-enabled without requiring a settings toggle.
    if (oldWidget.textureId != widget.textureId) {
      _lastOverlayRect = null;
    }
  }

  void _updateBufferSizeIfNeeded(
    Size viewport,
    bool shouldUseHighRes,
    BuildContext context,
    bool useNativeOverlay,
  ) {
    void scheduleResize(Size physical) {
      _pendingReportedSize = physical;
      _resizeDebounceTimer?.cancel();
      final debounce = useNativeOverlay
          ? _overlayResizeDebounceDelay
          : _resizeDebounceDelay;
      _resizeDebounceTimer = Timer(debounce, () {
        if (!mounted) return;
        final pending = _pendingReportedSize;
        if (pending == null) return;
        _pendingReportedSize = null;
        ref
            .read(nesControllerProvider.notifier)
            .updateWindowOutputSize(
              pending.width.round(),
              pending.height.round(),
            );
      });
    }

    if (shouldUseHighRes) {
      final dpr = MediaQuery.of(context).devicePixelRatio;
      final physicalWidth = (viewport.width * dpr).round();
      final physicalHeight = (viewport.height * dpr).round();

      if (_lastReportedSize?.width != physicalWidth.toDouble() ||
          _lastReportedSize?.height != physicalHeight.toDouble()) {
        _lastReportedSize = Size(
          physicalWidth.toDouble(),
          physicalHeight.toDouble(),
        );
        scheduleResize(_lastReportedSize!);
      }
    } else {
      // Revert to native resolution when no filters/shaders are active.
      if (_lastReportedSize?.width != 256.0 ||
          _lastReportedSize?.height != 240.0) {
        _lastReportedSize = const Size(256, 240);
        scheduleResize(_lastReportedSize!);
      }
    }
  }

  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    final route = ModalRoute.of(context);
    if (route is PageRoute<dynamic> && route != _route) {
      if (_route != null) {
        appRouteObserver.unsubscribe(this);
      }
      _route = route;
      appRouteObserver.subscribe(this, route);
      _isCurrentRoute = route.isCurrent;
    }
  }

  @override
  void dispose() {
    _cursorTimer?.cancel();
    _resizeDebounceTimer?.cancel();
    appRouteObserver.unsubscribe(this);
    super.dispose();
  }

  Widget _buildErrorContent() {
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const Icon(Icons.error_outline, color: Colors.red),
        const SizedBox(height: 8),
        Text(
          AppLocalizations.of(context)!.errorFailedToCreateTexture,
          style: Theme.of(
            context,
          ).textTheme.titleMedium?.copyWith(color: Colors.red),
        ),
        const SizedBox(height: 4),
        Text(widget.error!, textAlign: TextAlign.center),
      ],
    );
  }

  Widget _wrapWithMouseRegion(Widget child) {
    return MouseRegion(
      cursor: _cursorHidden ? SystemMouseCursors.none : MouseCursor.defer,
      onEnter: (_) => _showCursorAndArmTimer(),
      onHover: (_) => _showCursorAndArmTimer(),
      onExit: (_) => _showCursorAndCancelTimer(),
      child: child,
    );
  }

  Widget _buildAndroidContent(
    Size viewport,
    bool hasRom,
    VideoSettings settings,
    AndroidVideoBackendSettings androidBackend,
  ) {
    final useAndroidHardware =
        androidBackend.backend == AndroidVideoBackend.hardware;

    if (useAndroidHardware && _isCurrentRoute) {
      return SizedBox(
        width: viewport.width,
        height: viewport.height,
        child: Stack(
          fit: StackFit.expand,
          children: [
            const AndroidView(viewType: 'nesium_game_view'),
            if (hasRom) const EmulationStatusOverlay(),
          ],
        ),
      );
    } else {
      return _buildTextureContent(viewport, hasRom, settings);
    }
  }

  Widget _buildIosContent(Size viewport, bool hasRom, VideoSettings settings) {
    final appleShaderSettings = ref.watch(appleShaderSettingsProvider);
    final appleShaderActive =
        appleShaderSettings.enabled && appleShaderSettings.presetPath != null;

    final shouldUseHighRes =
        settings.videoFilter != VideoFilter.none || appleShaderActive;

    _updateBufferSizeIfNeeded(viewport, shouldUseHighRes, context, false);

    return SizedBox(
      width: viewport.width,
      height: viewport.height,
      child: Stack(
        fit: StackFit.expand,
        children: [
          const UiKitView(
            viewType: 'plugins.nesium.com/native_view',
            creationParams: {},
            creationParamsCodec: StandardMessageCodec(),
          ),
          if (hasRom) const EmulationStatusOverlay(),
        ],
      ),
    );
  }

  Widget _buildTextureContent(
    Size viewport,
    bool hasRom,
    VideoSettings settings,
  ) {
    final windowsShaderSettings = ref.watch(windowsShaderSettingsProvider);
    final windowsBackend = ref.watch(windowsVideoBackendSettingsProvider);
    final appleShaderSettings = ref.watch(appleShaderSettingsProvider);
    final isApple =
        !kIsWeb &&
        (defaultTargetPlatform == TargetPlatform.macOS ||
            defaultTargetPlatform == TargetPlatform.iOS);
    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;

    final appleShaderActive =
        isApple &&
        appleShaderSettings.enabled &&
        appleShaderSettings.presetPath != null;
    final windowsShaderActive =
        isWindows &&
        windowsShaderSettings.enabled &&
        windowsShaderSettings.presetPath != null;

    final shouldUseHighRes =
        settings.videoFilter != VideoFilter.none ||
        (windowsShaderActive &&
            windowsBackend.backend == WindowsVideoBackend.d3d11Gpu) ||
        appleShaderActive;

    final useNativeOverlay =
        isWindows && windowsBackend.useNativeOverlay && _isCurrentRoute;

    _updateBufferSizeIfNeeded(
      viewport,
      shouldUseHighRes,
      context,
      useNativeOverlay,
    );

    if (useNativeOverlay && _hadRom != hasRom) {
      _hadRom = hasRom;
      _lastOverlayRect = null;
    } else {
      _hadRom = hasRom;
    }

    if (useNativeOverlay) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;

        final gameBox =
            _gameKey.currentContext?.findRenderObject() as RenderBox?;
        if (gameBox == null || !gameBox.hasSize) return;

        final globalOffset = gameBox.localToGlobal(Offset.zero);
        final dpr = MediaQuery.of(context).devicePixelRatio;

        final left = (globalOffset.dx * dpr).floorToDouble();
        final top = (globalOffset.dy * dpr).floorToDouble();
        final right = ((globalOffset.dx + viewport.width) * dpr).ceilToDouble();
        final bottom = ((globalOffset.dy + viewport.height) * dpr)
            .ceilToDouble();

        final rect = Rect.fromLTWH(left, top, right - left, bottom - top);

        if (_lastOverlayRect == null ||
            (rect.left - _lastOverlayRect!.left).abs() > 0.1 ||
            (rect.top - _lastOverlayRect!.top).abs() > 0.1 ||
            (rect.width - _lastOverlayRect!.width).abs() > 0.1 ||
            (rect.height - _lastOverlayRect!.height).abs() > 0.1) {
          _lastOverlayRect = rect;
          final service = ref.read(nesTextureServiceProvider);
          if (!_nativeOverlayEnabled) {
            _nativeOverlayEnabled = true;
            service.setNativeOverlay(
              enabled: true,
              x: rect.left,
              y: rect.top,
              width: rect.width,
              height: rect.height,
            );
          } else {
            service.updateNativeOverlayRect(
              x: rect.left,
              y: rect.top,
              width: rect.width,
              height: rect.height,
            );
          }
        }
      });
    } else if (isWindows) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        if (_nativeOverlayEnabled) {
          _nativeOverlayEnabled = false;
          _lastOverlayRect = null;
          ref.read(nesTextureServiceProvider).setNativeOverlay(enabled: false);
        }
      });
    }

    return SizedBox(
      key: _gameKey,
      width: viewport.width,
      height: viewport.height,
      child: Stack(
        fit: StackFit.expand,
        children: [
          if (useNativeOverlay)
            const CustomPaint(
              painter: _HolePunchPainter(),
              child: SizedBox.expand(),
            )
          else if (widget.textureId != null)
            Texture(
              textureId: widget.textureId!,
              filterQuality: shouldUseHighRes
                  ? FilterQuality.low
                  : FilterQuality.none,
            ),
          if (hasRom) const EmulationStatusOverlay(),
        ],
      ),
    );
  }

  @override
  void didPush() {
    if (!mounted) return;
    if (_isCurrentRoute) return;
    setState(() => _isCurrentRoute = true);
  }

  @override
  void didPopNext() {
    if (!mounted) return;
    if (_isCurrentRoute) return;
    setState(() => _isCurrentRoute = true);
  }

  @override
  void didPushNext() {
    if (!mounted) return;
    if (!_isCurrentRoute) return;
    setState(() => _isCurrentRoute = false);
  }

  @override
  Widget build(BuildContext context) {
    if (widget.error != null) {
      return Container(
        color: Colors.black,
        alignment: Alignment.center,
        child: _buildErrorContent(),
      );
    }

    final hasRom = ref.watch(
      nesControllerProvider.select((s) => s.romHash != null),
    );
    final settings = ref.watch(videoSettingsProvider);
    final androidBackend = ref.watch(androidVideoBackendSettingsProvider);
    final integerScaling = settings.integerScaling;
    final aspectRatio = settings.aspectRatio;

    return Container(
      color: Colors.black,
      alignment: Alignment.center,
      child: Transform.translate(
        offset: Offset(0, widget.screenVerticalOffset),
        child: LayoutBuilder(
          builder: (context, constraints) {
            final viewport = NesScreenView.computeViewportSize(
              constraints,
              integerScaling: integerScaling,
              aspectRatio: aspectRatio,
            );
            if (viewport == null) return const SizedBox.shrink();

            Widget content;
            if (!kIsWeb && defaultTargetPlatform == TargetPlatform.android) {
              content = _buildAndroidContent(
                viewport,
                hasRom,
                settings,
                androidBackend,
              );
            } else if (!kIsWeb && defaultTargetPlatform == TargetPlatform.iOS) {
              content = _buildIosContent(viewport, hasRom, settings);
            } else {
              content = _buildTextureContent(viewport, hasRom, settings);
            }

            return _wrapWithMouseRegion(content);
          },
        ),
      ),
    );
  }
}

class _HolePunchPainter extends CustomPainter {
  const _HolePunchPainter();

  @override
  void paint(Canvas canvas, Size size) {
    canvas.drawRect(Offset.zero & size, Paint()..blendMode = BlendMode.clear);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
