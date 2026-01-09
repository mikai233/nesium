import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/bridge/api/history.dart' as bridge;
import 'package:nesium_flutter/domain/aux_texture_ids.dart';
import 'package:nesium_flutter/domain/nes_controller.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/features/settings/emulation_settings.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';
import 'package:nesium_flutter/widgets/single_position_scrollbar.dart';

class HistoryViewer extends ConsumerStatefulWidget {
  const HistoryViewer({super.key});

  @override
  ConsumerState<HistoryViewer> createState() => _HistoryViewerState();
}

class _HistoryViewerState extends ConsumerState<HistoryViewer> {
  static const double _sidePanelWidth = 280;

  final NesTextureService _textureService = NesTextureService();
  int? _historyTextureId;
  int? _flutterTextureId;
  bool _isCreating = false;

  StreamSubscription<bridge.HistorySnapshot>? _subscription;
  bool _hasReceivedData = false;
  bridge.HistorySnapshot? _lastSnapshot;
  String? _error;

  bool _showSidePanel = true;

  // Playback controls state
  bool _isPlaying = false;
  double _playbackSpeed = 1.0;
  Timer? _playbackTimer;

  @override
  void initState() {
    super.initState();
    _createTexture();
  }

  @override
  void dispose() {
    _playbackTimer?.cancel();
    final textureId = _historyTextureId;
    if (textureId != null) {
      _textureService.pauseAuxTexture(textureId);
    }
    unawaited(_subscription?.cancel());
    unawaited(_unsubscribe());
    if (textureId != null) {
      _textureService.disposeAuxTexture(textureId);
    }
    super.dispose();
  }

  Future<void> _unsubscribe() async {
    try {
      await bridge.unsubscribeHistoryState();
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to unsubscribe history state',
        logger: 'history_viewer',
      );
    }
  }

  Future<void> _createTexture() async {
    if (_isCreating) return;
    setState(() {
      _isCreating = true;
      _error = null;
    });

    try {
      final ids = await AuxTextureIdsCache.get();
      _historyTextureId ??= ids.history;

      final textureId = await _textureService.createAuxTexture(
        id: _historyTextureId!,
        width: 256,
        height: 240,
      );

      await _subscription?.cancel();
      _subscription = bridge.historyStateStream().listen(
        (snap) {
          if (!mounted) return;
          if (!_hasReceivedData) {
            _hasReceivedData = true;
          }
          // Update snapshot data without rebuilding the whole widget tree.
          // The Texture widget updates independently via the native texture.
          _lastSnapshot = snap;
        },
        onError: (e, st) {
          logWarning(
            e,
            stackTrace: st,
            message: 'Failed to subscribe to history state',
            logger: 'history_viewer',
          );
          if (!mounted) return;
          setState(() => _error = e.toString());
        },
      );

      if (mounted) {
        setState(() {
          _flutterTextureId = textureId;
          _isCreating = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isCreating = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 16),
            Text(l10n.tilemapError(_error ?? ''), textAlign: TextAlign.center),
            const SizedBox(height: 16),
            FilledButton.tonal(
              onPressed: _createTexture,
              child: Text(l10n.tilemapRetry),
            ),
          ],
        ),
      );
    }

    final hasRom = ref.watch(nesControllerProvider).romHash != null;
    final loading = !hasRom || _isCreating || _flutterTextureId == null;
    final content = _buildHistoryContent(context);

    if (!isNativeDesktop) {
      return ViewerSkeletonizer(
        enabled: loading,
        child: Stack(
          children: [
            content,
            Positioned(
              top: 12,
              right: 12,
              child: _buildMobileSettingsButton(context),
            ),
          ],
        ),
      );
    }

    return ViewerSkeletonizer(
      enabled: loading,
      child: Stack(
        children: [
          Row(
            children: [
              Expanded(child: content),
              _buildDesktopSidePanelWrapper(context),
            ],
          ),
          Positioned(
            top: 12,
            right: 12,
            child: _buildPanelToggleButton(context),
          ),
        ],
      ),
    );
  }

  Widget _buildHistoryContent(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Expanded(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: AspectRatio(
                aspectRatio: 256 / 240,
                child: Container(
                  clipBehavior: Clip.antiAlias,
                  decoration: BoxDecoration(
                    color: Colors.black,
                    borderRadius: BorderRadius.circular(8),
                    border: Border.all(
                      color: Theme.of(context).colorScheme.outlineVariant,
                      width: 1,
                    ),
                  ),
                  child:
                      (_flutterTextureId != null &&
                          !ViewerSkeletonScope.enabledOf(context))
                      ? Texture(
                          textureId: _flutterTextureId!,
                          filterQuality: FilterQuality.none,
                        )
                      : const SizedBox.shrink(),
                ),
              ),
            ),
          ),
          _buildControls(context),
        ],
      ),
    );
  }

  Widget _buildControls(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final snap = _lastSnapshot;
    if (snap == null) return const SizedBox.shrink();

    final BigInt maxPos = (snap.frameCount > BigInt.zero)
        ? snap.frameCount - BigInt.one
        : BigInt.zero;

    // Calculate absolute frame numbers using first_frame_seq
    final absoluteCurrentFrame =
        snap.firstFrameSeq + snap.currentPosition + BigInt.one;
    final absoluteLatestFrame = snap.firstFrameSeq + snap.frameCount;

    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Row(
            children: [
              Text(
                l10n.historyViewerFrame(
                  absoluteCurrentFrame.toString(),
                  absoluteLatestFrame.toString(),
                ),
              ),
              const Spacer(),
              if (snap.frameCount > BigInt.zero)
                Text(
                  '${((snap.currentPosition.toDouble() / maxPos.toDouble().clamp(1.0, double.infinity)) * 100).toStringAsFixed(1)}%',
                ),
            ],
          ),
          const SizedBox(height: 8),
          // Playback control buttons
          Row(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              // Go to start
              IconButton(
                icon: const Icon(Icons.skip_previous),
                tooltip: l10n.historyViewerGoToStart,
                onPressed: snap.currentPosition > BigInt.zero
                    ? () => _goToStart()
                    : null,
              ),
              // Step back
              IconButton(
                icon: const Icon(Icons.fast_rewind),
                tooltip: l10n.historyViewerStepBack,
                onPressed: snap.currentPosition > BigInt.zero
                    ? () => _stepBack()
                    : null,
              ),
              // Play/Pause
              IconButton(
                icon: Icon(_isPlaying ? Icons.pause : Icons.play_arrow),
                tooltip: _isPlaying
                    ? l10n.historyViewerPause
                    : l10n.historyViewerPlay,
                onPressed: snap.frameCount > BigInt.zero
                    ? () => _togglePlayback()
                    : null,
              ),
              // Step forward
              IconButton(
                icon: const Icon(Icons.fast_forward),
                tooltip: l10n.historyViewerStepForward,
                onPressed: snap.currentPosition < maxPos
                    ? () => _stepForward()
                    : null,
              ),
              // Go to end
              IconButton(
                icon: const Icon(Icons.skip_next),
                tooltip: l10n.historyViewerGoToEnd,
                onPressed: snap.currentPosition < maxPos
                    ? () => _goToEnd()
                    : null,
              ),
              const SizedBox(width: 16),
              // Speed selector
              _buildSpeedSelector(context),
            ],
          ),
          const SizedBox(height: 8),
          Slider(
            value: snap.currentPosition.toDouble(),
            min: 0,
            max: maxPos.toDouble(),
            onChanged: (val) {
              _stopPlayback();
              unawaited(bridge.historySeek(position: BigInt.from(val.toInt())));
            },
          ),
        ],
      ),
    );
  }

  Widget _buildSpeedSelector(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          l10n.historyViewerPlaybackSpeed,
          style: Theme.of(context).textTheme.bodySmall,
        ),
        const SizedBox(width: 8),
        DropdownButton<double>(
          value: _playbackSpeed,
          underline: const SizedBox.shrink(),
          isDense: true,
          items: const [
            DropdownMenuItem(value: 0.25, child: Text('0.25x')),
            DropdownMenuItem(value: 0.5, child: Text('0.5x')),
            DropdownMenuItem(value: 1.0, child: Text('1x')),
            DropdownMenuItem(value: 2.0, child: Text('2x')),
            DropdownMenuItem(value: 4.0, child: Text('4x')),
          ],
          onChanged: (val) {
            if (val != null) {
              setState(() => _playbackSpeed = val);
              if (_isPlaying) {
                _stopPlayback();
                _startPlayback();
              }
            }
          },
        ),
      ],
    );
  }

  void _togglePlayback() {
    if (_isPlaying) {
      _stopPlayback();
    } else {
      _startPlayback();
    }
  }

  void _startPlayback() {
    if (_isPlaying) return;
    setState(() => _isPlaying = true);
    final intervalMs = (1000 / 60 / _playbackSpeed).round();
    _playbackTimer = Timer.periodic(
      Duration(milliseconds: intervalMs.clamp(1, 1000)),
      (_) => _stepForward(),
    );
  }

  void _stopPlayback() {
    _playbackTimer?.cancel();
    _playbackTimer = null;
    if (_isPlaying) {
      setState(() => _isPlaying = false);
    }
  }

  void _stepForward() {
    final snap = _lastSnapshot;
    if (snap == null) return;
    final maxPos = snap.frameCount - BigInt.one;
    if (snap.currentPosition >= maxPos) {
      _stopPlayback();
      return;
    }
    unawaited(bridge.historySeek(position: snap.currentPosition + BigInt.one));
  }

  void _stepBack() {
    final snap = _lastSnapshot;
    if (snap == null || snap.currentPosition <= BigInt.zero) return;
    _stopPlayback();
    unawaited(bridge.historySeek(position: snap.currentPosition - BigInt.one));
  }

  void _goToStart() {
    _stopPlayback();
    unawaited(bridge.historySeek(position: BigInt.zero));
  }

  void _goToEnd() {
    final snap = _lastSnapshot;
    if (snap == null) return;
    _stopPlayback();
    final maxPos = snap.frameCount - BigInt.one;
    if (maxPos > BigInt.zero) {
      unawaited(bridge.historySeek(position: maxPos));
    }
  }

  Widget _buildMobileSettingsButton(BuildContext context) {
    final theme = Theme.of(context);
    return IconButton(
      icon: Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: theme.colorScheme.surfaceContainerHighest.withValues(
            alpha: 0.8,
          ),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Icon(
          Icons.settings,
          color: theme.colorScheme.onSurface,
          size: 20,
        ),
      ),
      onPressed: () {
        // Show controls in bottom sheet if they aren't on screen,
        // but for now history viewer has them on screen.
      },
    );
  }

  Widget _buildPanelToggleButton(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;
    final icon = _showSidePanel ? Icons.chevron_right : Icons.chevron_left;

    return IconButton(
      icon: Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: theme.colorScheme.surfaceContainerHighest.withValues(
            alpha: 0.8,
          ),
          borderRadius: BorderRadius.circular(8),
        ),
        child: Icon(icon, color: theme.colorScheme.onSurface, size: 20),
      ),
      tooltip: _showSidePanel ? l10n.tilemapHidePanel : l10n.tilemapShowPanel,
      onPressed: () => setState(() => _showSidePanel = !_showSidePanel),
    );
  }

  Widget _buildDesktopSidePanelWrapper(BuildContext context) {
    return ClipRect(
      child: TweenAnimationBuilder<double>(
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOut,
        tween: Tween<double>(end: _showSidePanel ? 1.0 : 0.0),
        builder: (context, factor, child) {
          return IgnorePointer(
            ignoring: factor == 0.0,
            child: Align(
              alignment: Alignment.centerLeft,
              widthFactor: factor,
              child: child,
            ),
          );
        },
        child: SizedBox(
          width: _sidePanelWidth,
          child: _buildDesktopSidePanel(context),
        ),
      ),
    );
  }

  Widget _buildDesktopSidePanel(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final l10n = AppLocalizations.of(context)!;

    return Container(
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainerLowest,
        border: Border(
          left: BorderSide(color: colorScheme.outlineVariant, width: 1),
        ),
      ),
      child: SinglePositionScrollbar(
        thumbVisibility: true,
        builder: (context, controller) {
          return ListView(
            controller: controller,
            padding: const EdgeInsets.all(12),
            children: [
              _sideSection(
                context,
                title: l10n.historyViewerInfo,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      l10n.historyViewerBufferSize(
                        (_lastSnapshot?.frameCount ?? BigInt.zero).toString(),
                      ),
                    ),
                    const SizedBox(height: 8),
                    if (_lastSnapshot != null &&
                        _lastSnapshot!.frameCount > BigInt.zero) ...[
                      Text(l10n.historyViewerHint),
                      const SizedBox(height: 12),
                      SizedBox(
                        width: double.infinity,
                        child: FilledButton.icon(
                          onPressed: () async {
                            final messenger = ScaffoldMessenger.of(context);
                            final successMsg = l10n.historyViewerStateRestored;
                            try {
                              await bridge.historyApply(
                                position: _lastSnapshot!.currentPosition,
                              );
                              if (mounted) {
                                messenger.showSnackBar(
                                  SnackBar(
                                    content: Text(successMsg),
                                    duration: const Duration(seconds: 1),
                                  ),
                                );
                              }
                            } catch (e) {
                              if (mounted) {
                                messenger.showSnackBar(
                                  SnackBar(
                                    content: Text(
                                      l10n.historyViewerApplyFailed(
                                        e.toString(),
                                      ),
                                    ),
                                    backgroundColor: Colors.red,
                                  ),
                                );
                              }
                            }
                          },
                          icon: const Icon(Icons.restore),
                          label: Text(l10n.historyViewerApplyState),
                        ),
                      ),
                    ] else ...[
                      if (ref.watch(nesControllerProvider).romHash == null)
                        Text(
                          l10n.historyViewerNoRom,
                          style: const TextStyle(color: Colors.orange),
                        )
                      else if (ref
                          .watch(emulationSettingsProvider)
                          .rewindEnabled)
                        Text(
                          l10n.historyViewerEmpty,
                          style: const TextStyle(color: Colors.orange),
                        )
                      else
                        Text(
                          l10n.historyViewerDisabled,
                          style: const TextStyle(color: Colors.orange),
                        ),
                    ],
                  ],
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _sideSection(
    BuildContext context, {
    required String title,
    required Widget child,
  }) {
    final theme = Theme.of(context);
    return Card(
      elevation: 0,
      color: theme.colorScheme.surface,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              title,
              style: theme.textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            child,
          ],
        ),
      ),
    );
  }
}
