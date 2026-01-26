import 'dart:async';
import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_controller.dart';
import '../../platform/platform_capabilities.dart';
import '../../platform/nes_video.dart' show VideoFilter;
import '../settings/video_settings.dart';
import '../settings/windows_shader_settings.dart';
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

          final shouldUseHighRes =
              settings.videoFilter != VideoFilter.none ||
              windowsShaderSettings.enabled;

          _updateBufferSizeIfNeeded(viewport, shouldUseHighRes, context);

          final child = SizedBox(
            width: viewport.width,
            height: viewport.height,
            child: Stack(
              fit: StackFit.expand,
              children: [
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
