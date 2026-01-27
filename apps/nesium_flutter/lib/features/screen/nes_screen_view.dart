import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/foundation.dart';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_controller.dart';
import '../../platform/platform_capabilities.dart';
import '../../platform/nes_video.dart' show VideoFilter;
import '../settings/video_settings.dart';
import '../settings/windows_shader_settings.dart';
import '../settings/macos_shader_settings.dart';
import '../settings/windows_video_backend_settings.dart';
import '../../domain/nes_texture_service.dart';
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

class _NesScreenViewState extends ConsumerState<NesScreenView> {
  static const Duration _cursorHideDelay = Duration(seconds: 2);

  Timer? _cursorTimer;
  bool _cursorHidden = false;
  Size? _lastReportedSize;
  final _gameKey = GlobalKey();
  Rect? _lastOverlayRect;

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
  }

  void _updateBufferSizeIfNeeded(
    Size viewport,
    bool shouldUseHighRes,
    BuildContext context,
  ) {
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
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted) {
            ref
                .read(nesControllerProvider.notifier)
                .updateWindowOutputSize(physicalWidth, physicalHeight);
          }
        });
      }
    } else {
      // Revert to native resolution when no filters/shaders are active.
      if (_lastReportedSize?.width != 256.0 ||
          _lastReportedSize?.height != 240.0) {
        _lastReportedSize = const Size(256, 240);
        WidgetsBinding.instance.addPostFrameCallback((_) {
          if (mounted) {
            ref
                .read(nesControllerProvider.notifier)
                .updateWindowOutputSize(256, 240);
          }
        });
      }
    }
  }

  @override
  void dispose() {
    _cursorTimer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final hasRom = ref.watch(
      nesControllerProvider.select((s) => s.romHash != null),
    );
    final settings = ref.watch(videoSettingsProvider);
    final windowsShaderSettings = ref.watch(windowsShaderSettingsProvider);
    final windowsBackend = ref.watch(windowsVideoBackendSettingsProvider);
    final integerScaling = settings.integerScaling;
    final aspectRatio = settings.aspectRatio;

    Widget content;
    if (widget.error != null) {
      content = Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline, color: Colors.red),
          const SizedBox(height: 8),
          Text(
            'Failed to create texture',
            style: Theme.of(
              context,
            ).textTheme.titleMedium?.copyWith(color: Colors.red),
          ),
          const SizedBox(height: 4),
          Text(widget.error!, textAlign: TextAlign.center),
        ],
      );
    } else if (useAndroidNativeGameView) {
      content = LayoutBuilder(
        builder: (context, constraints) {
          final viewport = NesScreenView.computeViewportSize(
            constraints,
            integerScaling: integerScaling,
            aspectRatio: aspectRatio,
          );
          if (viewport == null) return const SizedBox.shrink();

          final child = SizedBox(
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

          return MouseRegion(
            cursor: _cursorHidden ? SystemMouseCursors.none : MouseCursor.defer,
            onEnter: (_) => _showCursorAndArmTimer(),
            onHover: (_) => _showCursorAndArmTimer(),
            onExit: (_) => _showCursorAndCancelTimer(),
            child: child,
          );
        },
      );
    } else if (widget.textureId != null) {
      content = LayoutBuilder(
        builder: (context, constraints) {
          final viewport = NesScreenView.computeViewportSize(
            constraints,
            integerScaling: integerScaling,
            aspectRatio: aspectRatio,
          );
          if (viewport == null) return const SizedBox.shrink();

          final macosShaderSettings = ref.watch(macosShaderSettingsProvider);
          final isMacos =
              !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;

          final shouldUseHighRes =
              settings.videoFilter != VideoFilter.none ||
              (windowsShaderSettings.enabled &&
                  windowsBackend.backend == WindowsVideoBackend.d3d11Gpu) ||
              (isMacos && macosShaderSettings.enabled);

          _updateBufferSizeIfNeeded(viewport, shouldUseHighRes, context);

          final isWindows =
              !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;

          // Only enable overlay if this is the current active route
          final isCurrentRoute = ModalRoute.of(context)?.isCurrent ?? true;
          final useNativeOverlay =
              isWindows && windowsBackend.useNativeOverlay && isCurrentRoute;

          if (useNativeOverlay) {
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (!mounted) return;

              // We use the GlobalKey of the actual game SizedBox to get its exact position
              // This is much more reliable as it avoids double-counting any
              // parent alignment or Transform.translate offsets.
              final gameBox =
                  _gameKey.currentContext?.findRenderObject() as RenderBox?;
              if (gameBox == null || !gameBox.hasSize) return;

              final globalOffset = gameBox.localToGlobal(Offset.zero);
              final dpr = MediaQuery.of(context).devicePixelRatio;

              final rect = Rect.fromLTWH(
                globalOffset.dx * dpr,
                globalOffset.dy * dpr,
                viewport.width * dpr,
                viewport.height * dpr,
              );

              // Only update if significantly changed
              if (_lastOverlayRect == null ||
                  (rect.left - _lastOverlayRect!.left).abs() > 0.1 ||
                  (rect.top - _lastOverlayRect!.top).abs() > 0.1 ||
                  (rect.width - _lastOverlayRect!.width).abs() > 0.1 ||
                  (rect.height - _lastOverlayRect!.height).abs() > 0.1) {
                _lastOverlayRect = rect;
                ref
                    .read(nesTextureServiceProvider)
                    .setNativeOverlay(
                      enabled: true,
                      x: rect.left,
                      y: rect.top,
                      width: rect.width,
                      height: rect.height,
                    );
              }
            });
          } else if (isWindows) {
            // Ensure overlay is disabled if setting is off
            WidgetsBinding.instance.addPostFrameCallback((_) {
              if (!mounted) return;
              if (_lastOverlayRect != null) {
                _lastOverlayRect = null;
                ref
                    .read(nesTextureServiceProvider)
                    .setNativeOverlay(enabled: false);
              }
            });
          }

          final child = SizedBox(
            key: _gameKey,
            width: viewport.width,
            height: viewport.height,
            child: Stack(
              fit: StackFit.expand,
              children: [
                if (useNativeOverlay)
                  // Use BlendMode.clear to punch a hole through to the DWM transparency
                  CustomPaint(
                    painter: const _HolePunchPainter(),
                    child: const SizedBox.expand(),
                  )
                else
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

          return MouseRegion(
            cursor: _cursorHidden ? SystemMouseCursors.none : MouseCursor.defer,
            onEnter: (_) => _showCursorAndArmTimer(),
            onHover: (_) => _showCursorAndArmTimer(),
            onExit: (_) => _showCursorAndCancelTimer(),
            child: child,
          );
        },
      );
    } else {
      content = const SizedBox.shrink();
    }

    return Container(
      color: Colors.black,
      alignment: Alignment.center,
      child: Transform.translate(
        offset: Offset(0, widget.screenVerticalOffset),
        child: content,
      ),
    );
  }
}

class _HolePunchPainter extends CustomPainter {
  const _HolePunchPainter();

  @override
  void paint(Canvas canvas, Size size) {
    // Clears the pixels to (0,0,0,0) - Transparent Black
    // This allows DwmExtendFrameIntoClientArea to show the window behind.
    canvas.drawRect(Offset.zero & size, Paint()..blendMode = BlendMode.clear);
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
