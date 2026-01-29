import 'dart:math' as math;
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter/services.dart';

import '../../domain/emulation_status.dart';
import '../../domain/nes_input_masks.dart';
import '../../domain/pad_button.dart';
import '../netplay/netplay_state.dart';
import '../../logging/app_logger.dart';
import '../../platform/nes_emulation.dart' as frb_emulation;
import '../screen/nes_screen_view.dart';
import '../settings/emulation_settings.dart';
import '../settings/video_settings.dart';
import 'input_settings.dart';
import 'virtual_controls_editor.dart';
import 'virtual_controls_settings.dart';

final Path _kDpadCrossPath24 = _buildDpadCrossPath24();
const Rect _kDpadCrossBounds24 = Rect.fromLTWH(2, 2, 20, 20);

class VirtualControlsOverlay extends ConsumerStatefulWidget {
  const VirtualControlsOverlay({super.key, required this.isLandscape});

  final bool isLandscape;

  @override
  ConsumerState<VirtualControlsOverlay> createState() =>
      _VirtualControlsOverlayState();
}

class _VirtualControlsOverlayState
    extends ConsumerState<VirtualControlsOverlay> {
  void _startRewinding() {
    final emulationSettings = ref.read(emulationSettingsProvider);
    if (!emulationSettings.rewindEnabled) return;

    ref.read(emulationStatusProvider.notifier).setRewinding(true);
    unawaitedLogged(
      frb_emulation.setRewinding(rewinding: true),
      message: 'setRewinding(true)',
      logger: 'nes_shell',
    );
  }

  void _stopRewinding() {
    ref.read(emulationStatusProvider.notifier).setRewinding(false);
    unawaitedLogged(
      frb_emulation.setRewinding(rewinding: false),
      message: 'setRewinding(false)',
      logger: 'nes_shell',
    );
  }

  void _startFastForwarding() {
    ref.read(emulationStatusProvider.notifier).setFastForwarding(true);
    unawaitedLogged(
      frb_emulation.setFastForwarding(fastForwarding: true),
      message: 'setFastForwarding(true)',
      logger: 'nes_shell',
    );
  }

  void _stopFastForwarding() {
    ref.read(emulationStatusProvider.notifier).setFastForwarding(false);
    unawaitedLogged(
      frb_emulation.setFastForwarding(fastForwarding: false),
      message: 'setFastForwarding(false)',
      logger: 'nes_shell',
    );
  }

  void _onButtonChanged(PadButton button, bool pressed, {required int pad}) {
    ref
        .read(nesInputMasksProvider.notifier)
        .setPressed(button, pressed, pad: pad);
  }

  void _onTurboChanged(PadButton button, bool enabled, {required int pad}) {
    ref
        .read(nesInputMasksProvider.notifier)
        .setTurboEnabled(button, enabled, pad: pad);
  }

  @override
  Widget build(BuildContext context) {
    final editor = ref.watch(virtualControlsEditorProvider);
    final isEditing = editor.enabled;
    final liveSettings = ref.watch(virtualControlsSettingsProvider);
    final settings = (isEditing ? editor.draft : null) ?? liveSettings;

    final inputState = ref.watch(inputSettingsProvider);
    final netplay = ref.watch(netplayProvider);
    final pad = netplay.isInRoom
        ? (netplay.status.playerIndex < 4 ? netplay.status.playerIndex : 0)
        : 0;

    if (!isEditing &&
        inputState.ports[0]!.device != InputDevice.virtualController) {
      return const SizedBox.shrink();
    }

    final safeInsets = MediaQuery.paddingOf(context);

    return LayoutBuilder(
      builder: (context, constraints) {
        final viewport = NesScreenView.computeViewportSize(
          constraints,
          integerScaling: ref.watch(videoSettingsProvider).integerScaling,
          aspectRatio: ref.watch(videoSettingsProvider).aspectRatio,
        );
        if (viewport == null) return const SizedBox.shrink();

        final available = constraints.biggest;
        final isLandscape = widget.isLandscape;

        final base = _basePositions(
          settings,
          isLandscape: isLandscape,
          available: available,
          safeInsets: safeInsets,
        );

        final dpadBaseSize = _dpadClusterSize(settings);
        final mainButtonBaseSize = _mainButtonHitboxSize(settings);
        final turboButtonBaseSize = _turboButtonHitboxSize(settings);
        final systemButtonBaseSize = _systemButtonHitboxSize(settings);
        final buttonsBaseSize = _abClusterSize(settings);

        final dpadScale = isLandscape
            ? settings.landscapeDpadScale
            : settings.portraitDpadScale;
        final buttonsScale = isLandscape
            ? settings.landscapeButtonsScale
            : settings.portraitButtonsScale;
        final systemScale = isLandscape
            ? settings.landscapeSystemScale
            : settings.portraitSystemScale;

        final dpadSize = Size(
          dpadBaseSize.width * dpadScale,
          dpadBaseSize.height * dpadScale,
        );
        final buttonsSize = Size(
          buttonsBaseSize.width * buttonsScale,
          buttonsBaseSize.height * buttonsScale,
        );

        final dpadOffset = isLandscape
            ? settings.landscapeDpadOffset
            : settings.portraitDpadOffset;
        final buttonsOffset = isLandscape
            ? settings.landscapeButtonsOffset
            : settings.portraitButtonsOffset;
        final systemOffset = isLandscape
            ? settings.landscapeSystemOffset
            : settings.portraitSystemOffset;

        final aOffset = isLandscape
            ? settings.landscapeAOffset
            : settings.portraitAOffset;
        final bOffset = isLandscape
            ? settings.landscapeBOffset
            : settings.portraitBOffset;
        final turboAOffset = isLandscape
            ? settings.landscapeTurboAOffset
            : settings.portraitTurboAOffset;
        final turboBOffset = isLandscape
            ? settings.landscapeTurboBOffset
            : settings.portraitTurboBOffset;
        final selectOffset = isLandscape
            ? settings.landscapeSelectOffset
            : settings.portraitSelectOffset;
        final startOffset = isLandscape
            ? settings.landscapeStartOffset
            : settings.portraitStartOffset;
        final rewindOffset = isLandscape
            ? settings.landscapeRewindOffset
            : settings.portraitRewindOffset;
        final fastForwardOffset = isLandscape
            ? settings.landscapeFastForwardOffset
            : settings.portraitFastForwardOffset;

        final aScale = isLandscape
            ? settings.landscapeAScale
            : settings.portraitAScale;
        final bScale = isLandscape
            ? settings.landscapeBScale
            : settings.portraitBScale;
        final turboAScale = isLandscape
            ? settings.landscapeTurboAScale
            : settings.portraitTurboAScale;
        final turboBScale = isLandscape
            ? settings.landscapeTurboBScale
            : settings.portraitTurboBScale;
        final selectScale = isLandscape
            ? settings.landscapeSelectScale
            : settings.portraitSelectScale;
        final startScale = isLandscape
            ? settings.landscapeStartScale
            : settings.portraitStartScale;
        final rewindScale = isLandscape
            ? settings.landscapeRewindScale
            : settings.portraitRewindScale;
        final fastForwardScale = isLandscape
            ? settings.landscapeFastForwardScale
            : settings.portraitFastForwardScale;

        const frameGap = 2.0;

        final dpadDisc = _dpadDiscDiameter(settings);
        final dpadFrameInsets = _frameInsetsFromHitbox(
          hitbox: dpadBaseSize,
          visual: Size.square(dpadDisc),
        );
        final mainFrameInsets = _frameInsetsFromHitbox(
          hitbox: mainButtonBaseSize,
          visual: Size.square(settings.buttonSize),
        );
        final turboFrameInsets = _frameInsetsFromHitbox(
          hitbox: turboButtonBaseSize,
          visual: Size.square(settings.buttonSize),
        );
        final systemVisualSize = _systemButtonVisualSize(settings);
        final systemFrameInsets = _frameInsetsFromHitbox(
          hitbox: systemButtonBaseSize,
          visual: systemVisualSize,
        );

        var dpadPos = base.dpad + dpadOffset;
        var buttonsGroupPos = base.buttons + buttonsOffset;

        dpadPos = _clampPositionForFrame(
          dpadPos,
          size: dpadSize,
          frameInsets: _scaleInsets(dpadFrameInsets, dpadScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );

        buttonsGroupPos = _clampPositionForFrame(
          buttonsGroupPos,
          size: buttonsSize,
          frameInsets: EdgeInsets.zero,
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );

        final buttonsLocal = _buttonsLocalOffsets(settings);

        final aTotalScale = buttonsScale * aScale;
        final bTotalScale = buttonsScale * bScale;
        final turboATotalScale = buttonsScale * turboAScale;
        final turboBTotalScale = buttonsScale * turboBScale;
        final selectTotalScale = systemScale * selectScale;
        final startTotalScale = systemScale * startScale;
        final rewindTotalScale = systemScale * rewindScale;
        final fastForwardTotalScale = systemScale * fastForwardScale;

        final aSize = Size(
          mainButtonBaseSize.width * aTotalScale,
          mainButtonBaseSize.height * aTotalScale,
        );
        final bSize = Size(
          mainButtonBaseSize.width * bTotalScale,
          mainButtonBaseSize.height * bTotalScale,
        );
        final turboASize = Size(
          turboButtonBaseSize.width * turboATotalScale,
          turboButtonBaseSize.height * turboATotalScale,
        );
        final turboBSize = Size(
          turboButtonBaseSize.width * turboBTotalScale,
          turboButtonBaseSize.height * turboBTotalScale,
        );
        final selectSize = Size(
          systemButtonBaseSize.width * selectTotalScale,
          systemButtonBaseSize.height * selectTotalScale,
        );
        final startSize = Size(
          systemButtonBaseSize.width * startTotalScale,
          systemButtonBaseSize.height * startTotalScale,
        );
        final rewindSize = Size(
          mainButtonBaseSize.width * rewindTotalScale,
          mainButtonBaseSize.height * rewindTotalScale,
        );
        final fastForwardSize = Size(
          mainButtonBaseSize.width * fastForwardTotalScale,
          mainButtonBaseSize.height * fastForwardTotalScale,
        );

        var aPos =
            buttonsGroupPos +
            buttonsLocal.a * buttonsScale +
            aOffset * buttonsScale;
        var bPos =
            buttonsGroupPos +
            buttonsLocal.b * buttonsScale +
            bOffset * buttonsScale;
        var turboAPos =
            buttonsGroupPos +
            buttonsLocal.turboA * buttonsScale +
            turboAOffset * buttonsScale;
        var turboBPos =
            buttonsGroupPos +
            buttonsLocal.turboB * buttonsScale +
            turboBOffset * buttonsScale;

        var selectPos = base.select + systemOffset + selectOffset * systemScale;
        var startPos = base.start + systemOffset + startOffset * systemScale;
        var rewindPos = base.rewind + systemOffset + rewindOffset * systemScale;
        var fastForwardPos =
            base.fastForward + systemOffset + fastForwardOffset * systemScale;

        aPos = _clampPositionForFrame(
          aPos,
          size: aSize,
          frameInsets: _scaleInsets(mainFrameInsets, aTotalScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );
        bPos = _clampPositionForFrame(
          bPos,
          size: bSize,
          frameInsets: _scaleInsets(mainFrameInsets, bTotalScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );
        turboAPos = _clampPositionForFrame(
          turboAPos,
          size: turboASize,
          frameInsets: _scaleInsets(turboFrameInsets, turboATotalScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );
        turboBPos = _clampPositionForFrame(
          turboBPos,
          size: turboBSize,
          frameInsets: _scaleInsets(turboFrameInsets, turboBTotalScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );
        selectPos = _clampPositionForFrame(
          selectPos,
          size: selectSize,
          frameInsets: _scaleInsets(systemFrameInsets, selectTotalScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );
        startPos = _clampPositionForFrame(
          startPos,
          size: startSize,
          frameInsets: _scaleInsets(systemFrameInsets, startTotalScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );
        rewindPos = _clampPositionForFrame(
          rewindPos,
          size: rewindSize,
          frameInsets: _scaleInsets(mainFrameInsets, rewindTotalScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );
        fastForwardPos = _clampPositionForFrame(
          fastForwardPos,
          size: fastForwardSize,
          frameInsets: _scaleInsets(mainFrameInsets, fastForwardTotalScale),
          frameGap: frameGap,
          available: available,
          safeInsets: safeInsets,
        );

        // Dark, translucent "chrome" that still reads on black sidebars.
        final chromeBase = const Color(
          0xFF2A303A,
        ).withValues(alpha: (settings.opacity * 0.70).clamp(0.0, 1.0));
        final chromeSurface = const Color(
          0xFF3A4352,
        ).withValues(alpha: (settings.opacity * 0.72).clamp(0.0, 1.0));

        return Stack(
          fit: StackFit.expand,
          clipBehavior: Clip.none,
          children: [
            if (isEditing && editor.gridSnapEnabled)
              Positioned.fill(
                child: IgnorePointer(
                  child: CustomPaint(
                    painter: _GridPainter(
                      spacing: editor.gridSpacing,
                      color: Colors.white.withValues(alpha: 0.10),
                    ),
                  ),
                ),
              ),
            Positioned(
              left: dpadPos.dx,
              top: dpadPos.dy,
              width: dpadSize.width,
              height: dpadSize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: dpadBaseSize,
                frameInsets: dpadFrameInsets,
                scale: dpadScale,
                topLeft: dpadPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.dpad,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _DpadCluster(
                    settings: settings,
                    baseColor: chromeBase,
                    surfaceColor: chromeSurface,
                    onButtonChanged: (btn, pressed) =>
                        _onButtonChanged(btn, pressed, pad: pad),
                  ),
                ),
              ),
            ),
            Positioned(
              left: buttonsGroupPos.dx,
              top: buttonsGroupPos.dy,
              width: buttonsSize.width,
              height: buttonsSize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: buttonsBaseSize,
                scale: buttonsScale,
                topLeft: buttonsGroupPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.buttonsGroup,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                centerHandle: Container(
                  width: 24,
                  height: 24,
                  decoration: BoxDecoration(
                    color: Colors.white.withValues(alpha: 0.20),
                    shape: BoxShape.circle,
                    border: Border.all(
                      color: Colors.white.withValues(alpha: 0.40),
                      width: 1.2,
                    ),
                  ),
                  child: const Icon(
                    Icons.drag_indicator,
                    color: Colors.white,
                    size: 16,
                  ),
                ),
                child: const SizedBox.shrink(),
              ),
            ),
            Positioned(
              left: selectPos.dx,
              top: selectPos.dy,
              width: selectSize.width,
              height: selectSize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: systemButtonBaseSize,
                frameInsets: systemFrameInsets,
                scale: selectTotalScale,
                topLeft: selectPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.select,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _SystemButton(
                    settings: settings,
                    baseColor: chromeBase,
                    label: 'SELECT',
                    button: PadButton.select,
                    onButtonChanged: (btn, pressed) =>
                        _onButtonChanged(btn, pressed, pad: pad),
                  ),
                ),
              ),
            ),
            Positioned(
              left: startPos.dx,
              top: startPos.dy,
              width: startSize.width,
              height: startSize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: systemButtonBaseSize,
                frameInsets: systemFrameInsets,
                scale: startTotalScale,
                topLeft: startPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.start,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _SystemButton(
                    settings: settings,
                    baseColor: chromeBase,
                    label: 'START',
                    button: PadButton.start,
                    onButtonChanged: (btn, pressed) =>
                        _onButtonChanged(btn, pressed, pad: pad),
                  ),
                ),
              ),
            ),
            Positioned(
              left: turboBPos.dx,
              top: turboBPos.dy,
              width: turboBSize.width,
              height: turboBSize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: turboButtonBaseSize,
                frameInsets: turboFrameInsets,
                scale: turboBTotalScale,
                topLeft: turboBPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.turboB,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _TurboButton(
                    settings: settings,
                    label: 'TB',
                    button: PadButton.b,
                    onTurboChanged: (btn, enabled) =>
                        _onTurboChanged(btn, enabled, pad: pad),
                  ),
                ),
              ),
            ),
            Positioned(
              left: turboAPos.dx,
              top: turboAPos.dy,
              width: turboASize.width,
              height: turboASize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: turboButtonBaseSize,
                frameInsets: turboFrameInsets,
                scale: turboATotalScale,
                topLeft: turboAPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.turboA,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _TurboButton(
                    settings: settings,
                    label: 'TA',
                    button: PadButton.a,
                    onTurboChanged: (btn, enabled) =>
                        _onTurboChanged(btn, enabled, pad: pad),
                  ),
                ),
              ),
            ),
            Positioned(
              left: bPos.dx,
              top: bPos.dy,
              width: bSize.width,
              height: bSize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: mainButtonBaseSize,
                frameInsets: mainFrameInsets,
                scale: bTotalScale,
                topLeft: bPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.b,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _MainButton(
                    settings: settings,
                    label: 'B',
                    button: PadButton.b,
                    onButtonChanged: (btn, pressed) =>
                        _onButtonChanged(btn, pressed, pad: pad),
                  ),
                ),
              ),
            ),
            Positioned(
              left: aPos.dx,
              top: aPos.dy,
              width: aSize.width,
              height: aSize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: mainButtonBaseSize,
                frameInsets: mainFrameInsets,
                scale: aTotalScale,
                topLeft: aPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.a,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _MainButton(
                    settings: settings,
                    label: 'A',
                    button: PadButton.a,
                    onButtonChanged: (btn, pressed) =>
                        _onButtonChanged(btn, pressed, pad: pad),
                  ),
                ),
              ),
            ),
            if (isEditing || ref.watch(emulationSettingsProvider).rewindEnabled)
              Positioned(
                left: rewindPos.dx,
                top: rewindPos.dy,
                width: rewindSize.width,
                height: rewindSize.height,
                child: _EditableCluster(
                  enabled: isEditing,
                  baseSize: mainButtonBaseSize,
                  frameInsets: mainFrameInsets,
                  scale: rewindTotalScale,
                  topLeft: rewindPos,
                  available: available,
                  safeInsets: safeInsets,
                  gridSnapEnabled: editor.gridSnapEnabled,
                  gridSpacing: editor.gridSpacing,
                  onTransform: (topLeft, scale) {
                    if (!isEditing) return;
                    ref
                        .read(virtualControlsEditorProvider.notifier)
                        .updateDraft(
                          (draft) => _applyElementTransform(
                            draft,
                            element: _VirtualControlElement.rewind,
                            isLandscape: isLandscape,
                            available: available,
                            topLeft: topLeft,
                            scale: scale,
                            safeInsets: safeInsets,
                          ),
                        );
                  },
                  child: IgnorePointer(
                    ignoring: isEditing,
                    child: _RewindButton(
                      settings: settings,
                      baseColor: chromeBase,
                      onRewindChanged: (pressed) {
                        if (pressed) {
                          _startRewinding();
                        } else {
                          _stopRewinding();
                        }
                      },
                    ),
                  ),
                ),
              ),
            Positioned(
              left: fastForwardPos.dx,
              top: fastForwardPos.dy,
              width: fastForwardSize.width,
              height: fastForwardSize.height,
              child: _EditableCluster(
                enabled: isEditing,
                baseSize: mainButtonBaseSize,
                frameInsets: mainFrameInsets,
                scale: fastForwardTotalScale,
                topLeft: fastForwardPos,
                available: available,
                safeInsets: safeInsets,
                gridSnapEnabled: editor.gridSnapEnabled,
                gridSpacing: editor.gridSpacing,
                onTransform: (topLeft, scale) {
                  if (!isEditing) return;
                  ref
                      .read(virtualControlsEditorProvider.notifier)
                      .updateDraft(
                        (draft) => _applyElementTransform(
                          draft,
                          element: _VirtualControlElement.fastForward,
                          isLandscape: isLandscape,
                          available: available,
                          topLeft: topLeft,
                          scale: scale,
                          safeInsets: safeInsets,
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _FastForwardButton(
                    settings: settings,
                    baseColor: chromeBase,
                    onFastForwardChanged: (pressed) {
                      if (pressed) {
                        _startFastForwarding();
                      } else {
                        _stopFastForwarding();
                      }
                    },
                  ),
                ),
              ),
            ),
          ],
        );
      },
    );
  }
}

enum _VirtualControlElement {
  dpad,
  a,
  b,
  turboA,
  turboB,
  select,
  start,
  rewind,
  fastForward,
  buttonsGroup,
}

const double _clusterScaleMin = 0.6;
const double _clusterScaleMax = 1.8;

VirtualControlsSettings _applyElementTransform(
  VirtualControlsSettings draft, {
  required _VirtualControlElement element,
  required bool isLandscape,
  required Size available,
  required Offset topLeft,
  required double scale,
  required EdgeInsets safeInsets,
}) {
  final nextTotalScale = scale
      .clamp(_clusterScaleMin, _clusterScaleMax)
      .toDouble();

  VirtualControlsSettings withScale;
  switch (element) {
    case _VirtualControlElement.dpad:
      withScale = isLandscape
          ? draft.copyWith(landscapeDpadScale: nextTotalScale)
          : draft.copyWith(portraitDpadScale: nextTotalScale);
      break;
    case _VirtualControlElement.a:
      final groupScale =
          (isLandscape
                  ? draft.landscapeButtonsScale
                  : draft.portraitButtonsScale)
              .clamp(_clusterScaleMin, _clusterScaleMax)
              .toDouble();
      final next = nextTotalScale / (groupScale == 0 ? 1 : groupScale);
      withScale = isLandscape
          ? draft.copyWith(landscapeAScale: next)
          : draft.copyWith(portraitAScale: next);
      break;
    case _VirtualControlElement.b:
      final groupScale =
          (isLandscape
                  ? draft.landscapeButtonsScale
                  : draft.portraitButtonsScale)
              .clamp(_clusterScaleMin, _clusterScaleMax)
              .toDouble();
      final next = nextTotalScale / (groupScale == 0 ? 1 : groupScale);
      withScale = isLandscape
          ? draft.copyWith(landscapeBScale: next)
          : draft.copyWith(portraitBScale: next);
      break;
    case _VirtualControlElement.turboA:
      final groupScale =
          (isLandscape
                  ? draft.landscapeButtonsScale
                  : draft.portraitButtonsScale)
              .clamp(_clusterScaleMin, _clusterScaleMax)
              .toDouble();
      final next = nextTotalScale / (groupScale == 0 ? 1 : groupScale);
      withScale = isLandscape
          ? draft.copyWith(landscapeTurboAScale: next)
          : draft.copyWith(portraitTurboAScale: next);
      break;
    case _VirtualControlElement.turboB:
      final groupScale =
          (isLandscape
                  ? draft.landscapeButtonsScale
                  : draft.portraitButtonsScale)
              .clamp(_clusterScaleMin, _clusterScaleMax)
              .toDouble();
      final next = nextTotalScale / (groupScale == 0 ? 1 : groupScale);
      withScale = isLandscape
          ? draft.copyWith(landscapeTurboBScale: next)
          : draft.copyWith(portraitTurboBScale: next);
      break;
    case _VirtualControlElement.select:
      final groupScale =
          (isLandscape ? draft.landscapeSystemScale : draft.portraitSystemScale)
              .clamp(_clusterScaleMin, _clusterScaleMax)
              .toDouble();
      final next = nextTotalScale / (groupScale == 0 ? 1 : groupScale);
      withScale = isLandscape
          ? draft.copyWith(landscapeSelectScale: next)
          : draft.copyWith(portraitSelectScale: next);
      break;
    case _VirtualControlElement.start:
      final groupScale =
          (isLandscape ? draft.landscapeSystemScale : draft.portraitSystemScale)
              .clamp(_clusterScaleMin, _clusterScaleMax)
              .toDouble();
      final next = nextTotalScale / (groupScale == 0 ? 1 : groupScale);
      withScale = isLandscape
          ? draft.copyWith(landscapeStartScale: next)
          : draft.copyWith(portraitStartScale: next);
      break;
    case _VirtualControlElement.rewind:
      final groupScale =
          (isLandscape ? draft.landscapeSystemScale : draft.portraitSystemScale)
              .clamp(_clusterScaleMin, _clusterScaleMax)
              .toDouble();
      final next = nextTotalScale / (groupScale == 0 ? 1 : groupScale);
      withScale = isLandscape
          ? draft.copyWith(landscapeRewindScale: next)
          : draft.copyWith(portraitRewindScale: next);
      break;
    case _VirtualControlElement.fastForward:
      final groupScale =
          (isLandscape ? draft.landscapeSystemScale : draft.portraitSystemScale)
              .clamp(_clusterScaleMin, _clusterScaleMax)
              .toDouble();
      final next = nextTotalScale / (groupScale == 0 ? 1 : groupScale);
      withScale = isLandscape
          ? draft.copyWith(landscapeFastForwardScale: next)
          : draft.copyWith(portraitFastForwardScale: next);
      break;
    case _VirtualControlElement.buttonsGroup:
      withScale = isLandscape
          ? draft.copyWith(landscapeButtonsScale: nextTotalScale)
          : draft.copyWith(portraitButtonsScale: nextTotalScale);
      break;
  }

  final base = _basePositions(
    withScale,
    isLandscape: isLandscape,
    available: available,
    safeInsets: safeInsets,
  );

  final buttonsOffset = isLandscape
      ? withScale.landscapeButtonsOffset
      : withScale.portraitButtonsOffset;
  final systemOffset = isLandscape
      ? withScale.landscapeSystemOffset
      : withScale.portraitSystemOffset;

  final buttonsScale = isLandscape
      ? withScale.landscapeButtonsScale
      : withScale.portraitButtonsScale;

  final buttonsLocal = _buttonsLocalOffsets(withScale);

  switch (element) {
    case _VirtualControlElement.dpad:
      return isLandscape
          ? withScale.copyWith(landscapeDpadOffset: topLeft - base.dpad)
          : withScale.copyWith(portraitDpadOffset: topLeft - base.dpad);
    case _VirtualControlElement.a:
      final baseline =
          base.buttons + buttonsOffset + buttonsLocal.a * buttonsScale;
      final newOffset =
          (topLeft - baseline) / (buttonsScale == 0 ? 1 : buttonsScale);
      return isLandscape
          ? withScale.copyWith(landscapeAOffset: newOffset)
          : withScale.copyWith(portraitAOffset: newOffset);
    case _VirtualControlElement.b:
      final baseline =
          base.buttons + buttonsOffset + buttonsLocal.b * buttonsScale;
      final newOffset =
          (topLeft - baseline) / (buttonsScale == 0 ? 1 : buttonsScale);
      return isLandscape
          ? withScale.copyWith(landscapeBOffset: newOffset)
          : withScale.copyWith(portraitBOffset: newOffset);
    case _VirtualControlElement.turboA:
      final baseline =
          base.buttons + buttonsOffset + buttonsLocal.turboA * buttonsScale;
      final newOffset =
          (topLeft - baseline) / (buttonsScale == 0 ? 1 : buttonsScale);
      return isLandscape
          ? withScale.copyWith(landscapeTurboAOffset: newOffset)
          : withScale.copyWith(portraitTurboAOffset: newOffset);
    case _VirtualControlElement.turboB:
      final baseline =
          base.buttons + buttonsOffset + buttonsLocal.turboB * buttonsScale;
      final newOffset =
          (topLeft - baseline) / (buttonsScale == 0 ? 1 : buttonsScale);
      return isLandscape
          ? withScale.copyWith(landscapeTurboBOffset: newOffset)
          : withScale.copyWith(portraitTurboBOffset: newOffset);
    case _VirtualControlElement.select:
      final baseline = base.select + systemOffset;
      final groupScale =
          (isLandscape ? draft.landscapeSystemScale : draft.portraitSystemScale)
              .clamp(0.1, 2.0);
      return isLandscape
          ? withScale.copyWith(
              landscapeSelectOffset: (topLeft - baseline) / groupScale,
            )
          : withScale.copyWith(
              portraitSelectOffset: (topLeft - baseline) / groupScale,
            );
    case _VirtualControlElement.start:
      final baseline = base.start + systemOffset;
      final groupScale =
          (isLandscape ? draft.landscapeSystemScale : draft.portraitSystemScale)
              .clamp(0.1, 2.0);
      return isLandscape
          ? withScale.copyWith(
              landscapeStartOffset: (topLeft - baseline) / groupScale,
            )
          : withScale.copyWith(
              portraitStartOffset: (topLeft - baseline) / groupScale,
            );
    case _VirtualControlElement.rewind:
      final baseline = base.rewind + systemOffset;
      final groupScale =
          (isLandscape ? draft.landscapeSystemScale : draft.portraitSystemScale)
              .clamp(0.1, 2.0);
      return isLandscape
          ? withScale.copyWith(
              landscapeRewindOffset: (topLeft - baseline) / groupScale,
            )
          : withScale.copyWith(
              portraitRewindOffset: (topLeft - baseline) / groupScale,
            );
    case _VirtualControlElement.fastForward:
      final baseline = base.fastForward + systemOffset;
      final groupScale =
          (isLandscape ? draft.landscapeSystemScale : draft.portraitSystemScale)
              .clamp(0.1, 2.0);
      return isLandscape
          ? withScale.copyWith(
              landscapeFastForwardOffset: (topLeft - baseline) / groupScale,
            )
          : withScale.copyWith(
              portraitFastForwardOffset: (topLeft - baseline) / groupScale,
            );
    case _VirtualControlElement.buttonsGroup:
      return isLandscape
          ? withScale.copyWith(landscapeButtonsOffset: topLeft - base.buttons)
          : withScale.copyWith(portraitButtonsOffset: topLeft - base.buttons);
  }
}

({
  Offset dpad,
  Offset buttons,
  Offset select,
  Offset start,
  Offset rewind,
  Offset fastForward,
})
_basePositions(
  VirtualControlsSettings settings, {
  required bool isLandscape,
  required Size available,
  required EdgeInsets safeInsets,
}) {
  final basePadding = 8.0;

  final dpadBaseSize = _dpadClusterSize(settings);
  final buttonsBaseSize = _abClusterSize(settings);
  final mainButtonBaseSize = _mainButtonHitboxSize(settings);
  final systemButtonBaseSize = _systemButtonHitboxSize(settings);

  final systemScale = isLandscape
      ? settings.landscapeSystemScale
      : settings.portraitSystemScale;

  final systemButtonSize = Size(
    systemButtonBaseSize.width * systemScale,
    systemButtonBaseSize.height * systemScale,
  );

  final rewindScale = isLandscape
      ? settings.landscapeRewindScale
      : settings.portraitRewindScale;
  final rewindSize = Size(
    mainButtonBaseSize.width * systemScale * rewindScale,
    mainButtonBaseSize.height * systemScale * rewindScale,
  );

  final fastForwardScale = isLandscape
      ? settings.landscapeFastForwardScale
      : settings.portraitFastForwardScale;
  final fastForwardSize = Size(
    mainButtonBaseSize.width * systemScale * fastForwardScale,
    mainButtonBaseSize.height * systemScale * fastForwardScale,
  );

  final verticalOffset = isLandscape ? 20.0 : 120.0;
  final dpadDisc = _dpadDiscDiameter(settings);
  final dpadInternalPad = (dpadBaseSize.width - dpadDisc) / 2;
  const abClusterPad = 8.0; // Match 'pad' in _abClusterSize
  final dpadAlignmentOffset = abClusterPad - dpadInternalPad;

  final buttonsPos = Offset(
    available.width - buttonsBaseSize.width - basePadding - safeInsets.right,
    available.height -
        buttonsBaseSize.height -
        basePadding -
        verticalOffset -
        safeInsets.bottom,
  );

  // Align D-Pad visually with the buttons cluster by centering them vertically relative to each other.
  final buttonsCenterY = buttonsPos.dy + buttonsBaseSize.height / 2;
  final dpadPos = Offset(
    basePadding + safeInsets.left + dpadAlignmentOffset,
    buttonsCenterY - dpadBaseSize.height / 2,
  );

  final Offset selectPos;
  final Offset startPos;
  final Offset rewindPos;
  final Offset fastForwardPos;

  // Editor's Save button (✅) is at top: safeInsets.top + 12, right: 12.
  // It's a small FAB with a standard size of 40x40.
  const fabSize = 40.0;
  final fabCenter = Offset(
    available.width - 12.0 - fabSize / 2.0,
    safeInsets.top + 12.0 + fabSize / 2.0,
  );

  if (isLandscape) {
    final y = available.height * 0.3;
    selectPos = Offset(basePadding, y);
    startPos = Offset(
      available.width - basePadding - systemButtonSize.width,
      y,
    );
    final rightMargin = available.width - safeInsets.right - basePadding;
    rewindPos = Offset(
      rightMargin - rewindSize.width,
      fabCenter.dy - rewindSize.height / 2.0,
    );
    // Align horizontally to the left of Rewind
    fastForwardPos = Offset(
      rewindPos.dx - fastForwardSize.width - 12.0,
      rewindPos.dy,
    );
  } else {
    final clusterBottomY =
        available.height - basePadding - verticalOffset - safeInsets.bottom;
    // Position system buttons at a fixed height above the cluster anchor area
    // to prevent them from jumping when the clusters are scaled.
    final y =
        clusterBottomY -
        180.0 - // Reference height for portrait cluster area
        systemButtonSize.height -
        basePadding;

    // Use base sizes (scale 1.0) as horizontal reference to prevent
    // system buttons from jumping horizontally when clusters are scaled.
    selectPos = Offset(
      (basePadding + safeInsets.left) +
          (dpadBaseSize.width - systemButtonSize.width) / 2,
      y,
    );
    startPos = Offset(
      (available.width -
              buttonsBaseSize.width -
              basePadding -
              safeInsets.right) +
          (buttonsBaseSize.width - systemButtonSize.width) / 2,
      y,
    );
    final rightMargin = available.width - safeInsets.right - basePadding;
    rewindPos = Offset(
      rightMargin - rewindSize.width,
      fabCenter.dy - rewindSize.height / 2.0,
    );
    // Align horizontally to the left of Rewind
    fastForwardPos = Offset(
      rewindPos.dx - fastForwardSize.width - 12.0,
      rewindPos.dy,
    );
  }

  return (
    dpad: dpadPos,
    buttons: buttonsPos,
    select: selectPos,
    start: startPos,
    rewind: rewindPos,
    fastForward: fastForwardPos,
  );
}

class _GridPainter extends CustomPainter {
  _GridPainter({required this.spacing, required this.color});

  final double spacing;
  final Color color;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = color
      ..strokeWidth = 1.0;

    final step = spacing.clamp(2.0, 256.0);

    for (double x = 0; x <= size.width; x += step) {
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }
    for (double y = 0; y <= size.height; y += step) {
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }
  }

  @override
  bool shouldRepaint(covariant _GridPainter oldDelegate) {
    return oldDelegate.spacing != spacing || oldDelegate.color != color;
  }
}

class _EditableCluster extends StatefulWidget {
  const _EditableCluster({
    required this.enabled,
    required this.baseSize,
    this.frameInsets = EdgeInsets.zero,
    required this.scale,
    required this.topLeft,
    required this.available,
    required this.safeInsets,
    required this.gridSnapEnabled,
    required this.gridSpacing,
    required this.onTransform,
    this.centerHandle,
    required this.child,
  });

  final bool enabled;
  final Size baseSize;
  final EdgeInsets frameInsets;
  final double scale;
  final Offset topLeft;
  final Size available;
  final EdgeInsets safeInsets;
  final bool gridSnapEnabled;
  final double gridSpacing;
  final void Function(Offset topLeft, double scale) onTransform;
  final Widget child;
  final Widget? centerHandle;

  @override
  State<_EditableCluster> createState() => _EditableClusterState();
}

class _EditableClusterState extends State<_EditableCluster> {
  static const double _resizeHandleSize = 32;
  static const double _frameGap = 2.0;

  bool _active = false;
  bool _resizeMode = false;
  Offset? _snappedCenter;
  Offset _rawCenter = Offset.zero;
  Offset _rawTopLeft = Offset.zero;
  double _startScale = 1.0;
  double _baseDiagonal = 1.0;
  Offset _diagonalDir = Offset.zero;
  double _rawDiagonal = 0.0;

  @override
  void didUpdateWidget(covariant _EditableCluster oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (_active) return;
    final size = Size(
      widget.baseSize.width * widget.scale,
      widget.baseSize.height * widget.scale,
    );
    _rawCenter = widget.topLeft + Offset(size.width / 2, size.height / 2);
  }

  void _onScaleStart(ScaleStartDetails details) {
    _active = true;
    final size = Size(
      widget.baseSize.width * widget.scale,
      widget.baseSize.height * widget.scale,
    );
    final scaledInsets = _scaleInsets(widget.frameInsets, widget.scale);
    _rawCenter = widget.topLeft + Offset(size.width / 2, size.height / 2);
    _rawTopLeft = widget.topLeft;
    _startScale = widget.scale;
    _snappedCenter = null;

    final minSide = math.min(size.width, size.height);
    final handleSize = math.min(_resizeHandleSize, minSide / 2);
    final right = (size.width - scaledInsets.right).clamp(0.0, size.width);
    final bottom = (size.height - scaledInsets.bottom).clamp(0.0, size.height);
    _resizeMode =
        widget.enabled &&
        details.pointerCount == 1 &&
        details.localFocalPoint.dx >= right - handleSize &&
        details.localFocalPoint.dy >= bottom - handleSize;

    final baseW = widget.baseSize.width;
    final baseH = widget.baseSize.height;
    _baseDiagonal = math.max(1e-6, math.sqrt(baseW * baseW + baseH * baseH));
    _diagonalDir = Offset(baseW / _baseDiagonal, baseH / _baseDiagonal);
    _rawDiagonal = _baseDiagonal * widget.scale;
  }

  Offset _snapCenter(Offset rawCenter) {
    if (!widget.gridSnapEnabled) {
      _snappedCenter = null;
      return rawCenter;
    }

    final step = widget.gridSpacing.clamp(4.0, 128.0);
    final nearest = Offset(
      (rawCenter.dx / step).roundToDouble() * step,
      (rawCenter.dy / step).roundToDouble() * step,
    );

    // Smooth snap: only snap when close enough; keep snapped with a small hysteresis
    // so it doesn't flicker on/off near the threshold.
    final snapThreshold = (step * 0.35).clamp(6.0, 24.0);
    final releaseThreshold = snapThreshold * 1.25;

    final snapped = _snappedCenter;
    if (snapped != null) {
      if ((rawCenter - snapped).distance <= releaseThreshold) {
        return snapped;
      }
      _snappedCenter = null;
      return rawCenter;
    }

    if ((rawCenter - nearest).distance <= snapThreshold) {
      _snappedCenter = nearest;
      return nearest;
    }

    return rawCenter;
  }

  void _onScaleUpdate(ScaleUpdateDetails details) {
    if (_resizeMode && details.pointerCount == 1) {
      final projected =
          details.focalPointDelta.dx * _diagonalDir.dx +
          details.focalPointDelta.dy * _diagonalDir.dy;
      final rawDiagonal = _rawDiagonal + projected;
      final rawScale = rawDiagonal / _baseDiagonal;

      final baseW = widget.baseSize.width;
      final baseH = widget.baseSize.height;
      final insetR = widget.frameInsets.right;
      final insetB = widget.frameInsets.bottom;
      final maxScaleFromBounds = math.min(
        baseW <= 0
            ? _clusterScaleMax
            : (widget.available.width -
                          widget.safeInsets.right -
                          _rawTopLeft.dx)
                      .clamp(0.0, double.infinity) /
                  math.max(1e-6, baseW - insetR),
        baseH <= 0
            ? _clusterScaleMax
            : (widget.available.height -
                          widget.safeInsets.bottom -
                          _rawTopLeft.dy)
                      .clamp(0.0, double.infinity) /
                  math.max(1e-6, baseH - insetB),
      );

      final nextScale = rawScale
          .clamp(
            _clusterScaleMin,
            math.max(
              _clusterScaleMin,
              math.min(_clusterScaleMax, maxScaleFromBounds),
            ),
          )
          .toDouble();
      _rawDiagonal = nextScale * _baseDiagonal;

      widget.onTransform(_rawTopLeft, nextScale);
      return;
    }

    final nextScale = (_startScale * details.scale)
        .clamp(_clusterScaleMin, _clusterScaleMax)
        .toDouble();

    var rawCenter = _rawCenter + details.focalPointDelta;
    final center = _snapCenter(rawCenter);

    final size = Size(
      widget.baseSize.width * nextScale,
      widget.baseSize.height * nextScale,
    );

    var topLeft = center - Offset(size.width / 2, size.height / 2);
    final clampedTopLeft = _clampPositionForFrame(
      topLeft,
      size: size,
      frameInsets: _scaleInsets(widget.frameInsets, nextScale),
      frameGap: _frameGap,
      available: widget.available,
      safeInsets: widget.safeInsets,
    );
    if (clampedTopLeft != topLeft) {
      rawCenter = clampedTopLeft + Offset(size.width / 2, size.height / 2);
    }
    _rawCenter = rawCenter;

    widget.onTransform(clampedTopLeft, nextScale);
  }

  void _onScaleEnd(ScaleEndDetails details) {
    _active = false;
    _resizeMode = false;
    _snappedCenter = null;
  }

  @override
  Widget build(BuildContext context) {
    final scaledInsets = _scaleInsets(widget.frameInsets, widget.scale);
    final framePadding = EdgeInsets.fromLTRB(
      (scaledInsets.left - _frameGap).clamp(0.0, double.infinity),
      (scaledInsets.top - _frameGap).clamp(0.0, double.infinity),
      (scaledInsets.right - _frameGap).clamp(0.0, double.infinity),
      (scaledInsets.bottom - _frameGap).clamp(0.0, double.infinity),
    );

    final handle = widget.enabled
        ? Padding(
            padding: framePadding,
            child: DecoratedBox(
              decoration: BoxDecoration(
                border: Border.all(
                  color: Colors.white.withValues(alpha: 0.35),
                  width: 1,
                ),
                borderRadius: BorderRadius.circular(12),
              ),
            ),
          )
        : null;

    final resizeHandle = widget.enabled
        ? Positioned(
            right: framePadding.right,
            bottom: framePadding.bottom,
            width: _resizeHandleSize,
            height: _resizeHandleSize,
            child: Center(
              child: DecoratedBox(
                decoration: BoxDecoration(
                  color: Colors.black.withValues(alpha: 0.22),
                  borderRadius: BorderRadius.circular(10),
                  border: Border.all(
                    color: Colors.white.withValues(alpha: 0.30),
                    width: 1,
                  ),
                ),
                child: Padding(
                  padding: const EdgeInsets.all(6),
                  child: Icon(
                    Icons.open_in_full,
                    size: 16,
                    color: Colors.white.withValues(alpha: 0.80),
                  ),
                ),
              ),
            ),
          )
        : null;

    return GestureDetector(
      behavior: HitTestBehavior.opaque,
      onScaleStart: widget.enabled ? _onScaleStart : null,
      onScaleUpdate: widget.enabled ? _onScaleUpdate : null,
      onScaleEnd: widget.enabled ? _onScaleEnd : null,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Transform.scale(
            scale: widget.scale,
            alignment: Alignment.topLeft,
            child: SizedBox(
              width: widget.baseSize.width,
              height: widget.baseSize.height,
              child: widget.child,
            ),
          ),
          if (handle != null)
            Positioned.fill(child: IgnorePointer(child: handle)),
          if (widget.centerHandle != null && widget.enabled)
            Center(child: IgnorePointer(child: widget.centerHandle!)),
          if (resizeHandle != null) resizeHandle,
        ],
      ),
    );
  }
}

EdgeInsets _scaleInsets(EdgeInsets value, double scale) {
  return EdgeInsets.fromLTRB(
    value.left * scale,
    value.top * scale,
    value.right * scale,
    value.bottom * scale,
  );
}

EdgeInsets _frameInsetsFromHitbox({
  required Size hitbox,
  required Size visual,
}) {
  return EdgeInsets.fromLTRB(
    math.max(0.0, (hitbox.width - visual.width) / 2),
    math.max(0.0, (hitbox.height - visual.height) / 2),
    math.max(0.0, (hitbox.width - visual.width) / 2),
    math.max(0.0, (hitbox.height - visual.height) / 2),
  );
}

Offset _clampPositionForFrame(
  Offset pos, {
  required Size size,
  required EdgeInsets frameInsets,
  double frameGap = 0.0,
  required Size available,
  required EdgeInsets safeInsets,
}) {
  final leftPad = (frameInsets.left - frameGap).clamp(0.0, double.infinity);
  final topPad = (frameInsets.top - frameGap).clamp(0.0, double.infinity);
  final rightPad = (frameInsets.right - frameGap).clamp(0.0, double.infinity);
  final bottomPad = (frameInsets.bottom - frameGap).clamp(0.0, double.infinity);

  // Clamp based on the visual frame, not the full hitbox, so users can move
  // controls flush to the screen edges even when the hitbox is larger.
  final minX = safeInsets.left - leftPad;
  final minY = safeInsets.top - topPad;
  final maxX = math.max(
    minX,
    available.width - safeInsets.right - (size.width - rightPad),
  );
  final maxY = math.max(
    minY,
    available.height - safeInsets.bottom - (size.height - bottomPad),
  );

  return Offset(
    pos.dx.clamp(minX, maxX).toDouble(),
    pos.dy.clamp(minY, maxY).toDouble(),
  );
}

Size _dpadClusterSize(VirtualControlsSettings settings) {
  final disc = _dpadDiscDiameter(settings);
  final hitbox = disc * settings.hitboxScale;
  return Size.square(hitbox);
}

double _dpadDiscDiameter(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final g = settings.gap;
  return s * 2.15 + g * 2;
}

Size _systemButtonVisualSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  return Size(s * 1.25, s * 0.55);
}

Size _abClusterSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final g = settings.gap * 2.5; // Increased gap for cross layout
  final mainHit = s * settings.hitboxScale;
  const pad = 8.0;
  return Size(pad + mainHit * 2 + g + pad, pad + mainHit * 2 + g + pad);
}

Size _mainButtonHitboxSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final main = s;
  return Size.square(main * settings.hitboxScale);
}

Size _turboButtonHitboxSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final turbo = s;
  return Size.square(turbo * settings.hitboxScale);
}

Size _systemButtonHitboxSize(VirtualControlsSettings settings) {
  final visual = _systemButtonVisualSize(settings);
  final w = visual.width;
  final h = visual.height;
  return Size(w * settings.hitboxScale, h * settings.hitboxScale);
}

({Offset turboB, Offset turboA, Offset b, Offset a}) _buttonsLocalOffsets(
  VirtualControlsSettings settings,
) {
  final g = settings.gap * 2.5; // Matches increased gap in _abClusterSize
  final mainHit = _mainButtonHitboxSize(settings).width;
  const pad = 8.0;

  final centerOffset = (mainHit + g) / 2;

  return (
    // TB is at top
    turboB: Offset(pad + centerOffset, pad),
    // TA is at right
    turboA: Offset(pad + mainHit + g, pad + centerOffset),
    // B is at left
    b: Offset(pad, pad + centerOffset),
    // A is at bottom
    a: Offset(pad + centerOffset, pad + mainHit + g),
  );
}

class _VirtualPressButton extends StatefulWidget {
  const _VirtualPressButton({
    required this.visualSize,
    required this.hitboxScale,
    required this.hapticsEnabled,
    required this.visualBuilder,
    required this.onPressedChanged,
  });

  final Size visualSize;
  final double hitboxScale;
  final bool hapticsEnabled;
  final Widget Function(bool pressed) visualBuilder;
  final ValueChanged<bool> onPressedChanged;

  @override
  State<_VirtualPressButton> createState() => _VirtualPressButtonState();
}

class _VirtualPressButtonState extends State<_VirtualPressButton> {
  int? _pointerId;
  bool _pressed = false;

  void _setPressed(bool value) {
    if (_pressed == value) return;
    setState(() => _pressed = value);
    widget.onPressedChanged(value);
    if (value && widget.hapticsEnabled) {
      HapticFeedback.lightImpact();
    }
  }

  @override
  void dispose() {
    if (_pressed) {
      widget.onPressedChanged(false);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final hitboxSize = Size(
      widget.visualSize.width * widget.hitboxScale,
      widget.visualSize.height * widget.hitboxScale,
    );
    return Listener(
      onPointerDown: (event) {
        if (_pointerId != null) return;
        _pointerId = event.pointer;
        _setPressed(true);
      },
      onPointerUp: (event) {
        if (_pointerId != event.pointer) return;
        _pointerId = null;
        _setPressed(false);
      },
      onPointerCancel: (event) {
        if (_pointerId != event.pointer) return;
        _pointerId = null;
        _setPressed(false);
      },
      child: AnimatedScale(
        duration: const Duration(milliseconds: 60),
        scale: _pressed ? 0.95 : 1,
        child: SizedBox(
          width: hitboxSize.width,
          height: hitboxSize.height,
          child: Center(
            child: SizedBox(
              width: widget.visualSize.width,
              height: widget.visualSize.height,
              child: widget.visualBuilder(_pressed),
            ),
          ),
        ),
      ),
    );
  }
}

class _VirtualToggleButton extends StatefulWidget {
  const _VirtualToggleButton({
    required this.visualSize,
    required this.hitboxScale,
    required this.hapticsEnabled,
    required this.visualBuilder,
    required this.onToggle,
  });

  final Size visualSize;
  final double hitboxScale;
  final bool hapticsEnabled;
  final Widget Function(bool pressed) visualBuilder;
  final ValueChanged<bool> onToggle;

  @override
  State<_VirtualToggleButton> createState() => _VirtualToggleButtonState();
}

class _VirtualToggleButtonState extends State<_VirtualToggleButton> {
  int? _pointerId;
  bool _pressed = false;

  @override
  void dispose() {
    if (_pressed) {
      widget.onToggle(false);
    }
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final hitboxSize = Size(
      widget.visualSize.width * widget.hitboxScale,
      widget.visualSize.height * widget.hitboxScale,
    );
    return Listener(
      onPointerDown: (event) {
        if (_pointerId != null) return;
        _pointerId = event.pointer;
        _pressed = true;
        widget.onToggle(true);
        if (widget.hapticsEnabled) {
          HapticFeedback.lightImpact();
        }
        setState(() {});
      },
      onPointerUp: (event) {
        if (_pointerId != event.pointer) return;
        _pointerId = null;
        _pressed = false;
        widget.onToggle(false);
        setState(() {});
      },
      onPointerCancel: (event) {
        if (_pointerId != event.pointer) return;
        _pointerId = null;
        _pressed = false;
        widget.onToggle(false);
        setState(() {});
      },
      child: AnimatedScale(
        duration: const Duration(milliseconds: 60),
        scale: _pressed ? 0.95 : 1,
        child: SizedBox(
          width: hitboxSize.width,
          height: hitboxSize.height,
          child: Center(
            child: SizedBox(
              width: widget.visualSize.width,
              height: widget.visualSize.height,
              child: widget.visualBuilder(_pressed),
            ),
          ),
        ),
      ),
    );
  }
}

class _DpadCluster extends StatelessWidget {
  const _DpadCluster({
    required this.settings,
    required this.baseColor,
    required this.surfaceColor,
    required this.onButtonChanged,
  });

  final VirtualControlsSettings settings;
  final Color baseColor;
  final Color surfaceColor;
  final void Function(PadButton button, bool pressed) onButtonChanged;

  @override
  Widget build(BuildContext context) {
    // Keep the D-pad smaller relative to the A/B cluster.
    final disc = _dpadDiscDiameter(settings);
    final hitbox = disc * settings.hitboxScale;
    return SizedBox(
      width: hitbox,
      height: hitbox,
      child: _VirtualDpad(
        discDiameter: disc,
        deadzoneRatio: settings.dpadDeadzoneRatio,
        boundaryDeadzoneRatio: settings.dpadBoundaryDeadzoneRatio,
        hapticsEnabled: settings.hapticsEnabled,
        baseColor: baseColor,
        surfaceColor: surfaceColor,
        onButtonsChanged: (next) {
          onButtonChanged(PadButton.up, next.contains(PadButton.up));
          onButtonChanged(PadButton.down, next.contains(PadButton.down));
          onButtonChanged(PadButton.left, next.contains(PadButton.left));
          onButtonChanged(PadButton.right, next.contains(PadButton.right));
        },
      ),
    );
  }
}

class _VirtualDpad extends StatefulWidget {
  const _VirtualDpad({
    required this.discDiameter,
    required this.deadzoneRatio,
    required this.boundaryDeadzoneRatio,
    required this.hapticsEnabled,
    required this.baseColor,
    required this.surfaceColor,
    required this.onButtonsChanged,
  });

  final double discDiameter;
  final double deadzoneRatio;
  final double boundaryDeadzoneRatio;
  final bool hapticsEnabled;
  final Color baseColor;
  final Color surfaceColor;
  final ValueChanged<Set<PadButton>> onButtonsChanged;

  @override
  State<_VirtualDpad> createState() => _VirtualDpadState();
}

class _VirtualDpadState extends State<_VirtualDpad> {
  int? _pointerId;
  Set<PadButton> _active = const {};

  @override
  void dispose() {
    if (_active.isNotEmpty) {
      widget.onButtonsChanged(const {});
    }
    super.dispose();
  }

  void _updateFromLocalOffset(Offset localPosition, Size hitboxSize) {
    final center = Offset(hitboxSize.width / 2, hitboxSize.height / 2);
    final delta = localPosition - center;
    final discRadius = widget.discDiameter / 2;
    final deadzone = discRadius * widget.deadzoneRatio.clamp(0.0, 0.9);
    final boundary = widget.boundaryDeadzoneRatio.clamp(0.25, 0.95);

    final dist = delta.distance;
    Set<PadButton> next;
    if (dist <= deadzone) {
      next = const {};
    } else {
      final dx = delta.dx;
      final dy = delta.dy;
      final ax = dx.abs();
      final ay = dy.abs();

      if (ax < 1e-6 && ay < 1e-6) {
        next = const {};
      } else {
        final horizontal = dx >= 0 ? PadButton.right : PadButton.left;
        final vertical = dy >= 0 ? PadButton.down : PadButton.up;

        // "Boundary deadzone" shrinks the diagonal region so cardinal directions
        // are easier to hit and less likely to accidentally include a neighbor.
        final ratio = ax < 1e-6 ? double.infinity : (ay / ax);
        if (ratio < boundary) {
          next = {horizontal};
        } else if (ratio > 1 / boundary) {
          next = {vertical};
        } else {
          next = {horizontal, vertical};
        }
      }
    }

    if (_setEquals(_active, next)) return;
    final hadAny = _active.isNotEmpty;
    final hasAny = next.isNotEmpty;
    _active = next;
    widget.onButtonsChanged(next);
    if (widget.hapticsEnabled && (!hadAny && hasAny)) {
      HapticFeedback.selectionClick();
    }
    setState(() {});
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final hitbox = constraints.biggest;
        final discSize = Size.square(widget.discDiameter);

        return Center(
          child: Listener(
            onPointerDown: (event) {
              if (_pointerId != null) return;
              _pointerId = event.pointer;
              _updateFromLocalOffset(event.localPosition, hitbox);
            },
            onPointerMove: (event) {
              if (_pointerId != event.pointer) return;
              _updateFromLocalOffset(event.localPosition, hitbox);
            },
            onPointerUp: (event) {
              if (_pointerId != event.pointer) return;
              _pointerId = null;
              if (_active.isNotEmpty) {
                _active = const {};
                widget.onButtonsChanged(const {});
                setState(() {});
              }
            },
            onPointerCancel: (event) {
              if (_pointerId != event.pointer) return;
              _pointerId = null;
              if (_active.isNotEmpty) {
                _active = const {};
                widget.onButtonsChanged(const {});
                setState(() {});
              }
            },
            child: SizedBox(
              width: hitbox.width,
              height: hitbox.height,
              child: Center(
                child: CustomPaint(
                  size: discSize,
                  painter: _DpadPainter(
                    baseColor: widget.baseColor,
                    surfaceColor: widget.surfaceColor,
                    active: _active,
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }
}

class _DpadPainter extends CustomPainter {
  const _DpadPainter({
    required this.baseColor,
    required this.surfaceColor,
    required this.active,
  });

  final Color baseColor;
  final Color surfaceColor;
  final Set<PadButton> active;

  @override
  void paint(Canvas canvas, Size size) {
    final center = Offset(size.width / 2, size.height / 2);
    final radius = math.min(size.width, size.height) / 2 * 0.98;
    final rect = Rect.fromCircle(center: center, radius: radius);

    double rad(double deg) => deg * math.pi / 180.0;

    final discBase = baseColor.withValues(
      alpha: (baseColor.a * 1.15).clamp(0.0, 1),
    );

    // Subtle outer glow so it reads on black sidebars.
    canvas.drawShadow(
      Path()..addOval(rect),
      Colors.white.withValues(alpha: 0.10),
      radius * 0.04,
      false,
    );

    final discPaint = Paint()
      ..shader = LinearGradient(
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
        colors: [
          Color.alphaBlend(Colors.white.withValues(alpha: 0.10), discBase),
          discBase,
          Color.alphaBlend(Colors.black.withValues(alpha: 0.20), discBase),
        ],
      ).createShader(rect);
    canvas.drawCircle(center, radius, discPaint);

    canvas.drawCircle(
      center,
      radius,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.4
        ..color = Colors.white.withValues(
          alpha: (discBase.a * 0.28).clamp(0.0, 1),
        ),
    );

    // Center circle radius relative to the outer radius.
    // Larger divisor => smaller center circle.
    const centerRadiusDivisor = 2.4;
    final centerRadius = radius / centerRadiusDivisor;
    final centerRect = Rect.fromCircle(center: center, radius: centerRadius);

    final sectionAngle = 88.0;
    final half = sectionAngle / 2.0;

    final normal = surfaceColor.withValues(
      alpha: (surfaceColor.a * 0.95).clamp(0.0, 1),
    );
    final pressed = Color.alphaBlend(
      Colors.white.withValues(alpha: 0.24),
      normal.withValues(alpha: (normal.a * 0.95).clamp(0.0, 1)),
    );

    final sectorPaint = Paint()..style = PaintingStyle.fill;

    void drawSector(double startDeg, bool isPressed) {
      sectorPaint.color = isPressed ? pressed : normal;
      final startRad = rad(startDeg);
      final sweepRad = rad(sectionAngle);
      final endRad = startRad + sweepRad;

      final outerStart = Offset(
        center.dx + radius * math.cos(startRad),
        center.dy + radius * math.sin(startRad),
      );
      final innerStart = Offset(
        center.dx + centerRadius * math.cos(startRad),
        center.dy + centerRadius * math.sin(startRad),
      );
      final innerEnd = Offset(
        center.dx + centerRadius * math.cos(endRad),
        center.dy + centerRadius * math.sin(endRad),
      );

      // Ring segment: avoids highlighting under the center circle.
      final path = Path()
        ..moveTo(innerStart.dx, innerStart.dy)
        ..lineTo(outerStart.dx, outerStart.dy)
        ..arcTo(rect, startRad, sweepRad, false)
        ..lineTo(innerEnd.dx, innerEnd.dy)
        ..arcTo(centerRect, endRad, -sweepRad, false)
        ..close();

      canvas.drawPath(path, sectorPaint);
    }

    drawSector(90 - half, active.contains(PadButton.down));
    drawSector(180 - half, active.contains(PadButton.left));
    drawSector(270 - half, active.contains(PadButton.up));
    drawSector(-half, active.contains(PadButton.right));
    canvas.drawCircle(
      center,
      centerRadius,
      Paint()
        ..shader = RadialGradient(
          colors: [
            Color.alphaBlend(
              Colors.white.withValues(alpha: 0.14),
              surfaceColor.withValues(alpha: (discBase.a * 1.25).clamp(0.0, 1)),
            ),
            Color.alphaBlend(
              Colors.black.withValues(alpha: 0.20),
              surfaceColor.withValues(alpha: (discBase.a * 1.25).clamp(0.0, 1)),
            ),
          ],
        ).createShader(centerRect),
    );
    canvas.drawCircle(
      center,
      centerRadius,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.1
        ..color = Colors.white.withValues(
          alpha: (discBase.a * 0.18).clamp(0.0, 1),
        ),
    );

    final crossSize = radius * 1.55;
    final crossScale =
        crossSize /
        math.max(_kDpadCrossBounds24.width, _kDpadCrossBounds24.height);
    final cross = _kDpadCrossPath24.transform(
      _translateScaleFromOrigin(
        origin: _kDpadCrossBounds24.center,
        scale: crossScale,
        translate: center,
      ),
    );
    canvas.drawShadow(
      cross,
      Colors.black.withValues(alpha: 0.35),
      radius * 0.02,
      true,
    );
    canvas.drawPath(
      cross,
      Paint()
        ..style = PaintingStyle.fill
        ..color = const Color(
          0xFF0B0D10,
        ).withValues(alpha: (discBase.a * 1.0).clamp(0.0, 1)),
    );
  }

  @override
  bool shouldRepaint(covariant _DpadPainter oldDelegate) {
    return oldDelegate.baseColor != baseColor ||
        oldDelegate.surfaceColor != surfaceColor ||
        oldDelegate.active != active;
  }
}

Float64List _translateScaleFromOrigin({
  required Offset origin,
  required double scale,
  required Offset translate,
}) {
  // Column-major 4x4 matrix for: translate(translate) * scale(scale) * translate(-origin).
  final tx = translate.dx - origin.dx * scale;
  final ty = translate.dy - origin.dy * scale;
  return Float64List.fromList([
    scale,
    0,
    0,
    0,
    0,
    scale,
    0,
    0,
    0,
    0,
    1,
    0,
    tx,
    ty,
    0,
    1,
  ]);
}

Path _buildDpadCrossPath24() {
  final path = Path();
  path
    ..moveTo(15, 7.5)
    ..lineTo(15, 2)
    ..lineTo(9, 2)
    ..lineTo(9, 7.5)
    ..lineTo(12, 10.5)
    ..close()
    ..moveTo(7.5, 9)
    ..lineTo(2, 9)
    ..lineTo(2, 15)
    ..lineTo(7.5, 15)
    ..lineTo(10.5, 12)
    ..close()
    ..moveTo(9, 16.5)
    ..lineTo(9, 22)
    ..lineTo(15, 22)
    ..lineTo(15, 16.5)
    ..lineTo(12, 13.5)
    ..close()
    ..moveTo(16.5, 9)
    ..lineTo(13.5, 12)
    ..lineTo(16.5, 15)
    ..lineTo(22, 15)
    ..lineTo(22, 9)
    ..close();

  return path;
}

bool _setEquals(Set<PadButton> a, Set<PadButton> b) {
  if (identical(a, b)) return true;
  if (a.length != b.length) return false;
  for (final v in a) {
    if (!b.contains(v)) return false;
  }
  return true;
}

Widget _roundVisual({
  required Color base,
  required Widget child,
  required bool pressed,
  Color? ringColor,
}) {
  final borderColor = (ringColor != null && pressed)
      ? ringColor
      : Colors.white.withValues(alpha: 0.18);

  return DecoratedBox(
    decoration: BoxDecoration(
      shape: BoxShape.circle,
      gradient: LinearGradient(
        begin: Alignment.topLeft,
        end: Alignment.bottomRight,
        colors: [
          Color.alphaBlend(Colors.white.withValues(alpha: 0.10), base),
          base.withValues(alpha: pressed ? (base.a * 0.80) : base.a),
          Color.alphaBlend(Colors.black.withValues(alpha: 0.22), base),
        ],
      ),
      boxShadow: [
        BoxShadow(
          color: Colors.white.withValues(alpha: 0.06),
          blurRadius: 10,
          offset: const Offset(0, 0),
        ),
        BoxShadow(
          color: Colors.black.withValues(alpha: 0.22),
          blurRadius: 10,
          offset: const Offset(0, 6),
        ),
      ],
      border: Border.all(
        color: borderColor,
        width: (ringColor != null && pressed) ? 2.0 : 1.6,
      ),
    ),
    child: Center(child: child),
  );
}

class _MainButton extends StatelessWidget {
  const _MainButton({
    required this.settings,
    required this.label,
    required this.button,
    required this.onButtonChanged,
  });

  final VirtualControlsSettings settings;
  final String label;
  final PadButton button;
  final void Function(PadButton button, bool pressed) onButtonChanged;

  @override
  Widget build(BuildContext context) {
    final s = settings.buttonSize;
    final main = s;

    final labelStyle = Theme.of(context).textTheme.titleMedium?.copyWith(
      color: Colors.white.withValues(alpha: 0.92),
      fontWeight: FontWeight.w900,
      letterSpacing: 0.2,
    );

    return _VirtualPressButton(
      visualSize: Size.square(main),
      hitboxScale: settings.hitboxScale,
      hapticsEnabled: settings.hapticsEnabled,
      visualBuilder: (pressed) => _roundVisual(
        base: const Color(
          0xFF272B33,
        ).withValues(alpha: (settings.opacity * 0.80).clamp(0.0, 1.0)),
        pressed: pressed,
        child: Text(label, style: labelStyle),
      ),
      onPressedChanged: (pressed) => onButtonChanged(button, pressed),
    );
  }
}

class _TurboButton extends StatelessWidget {
  const _TurboButton({
    required this.settings,
    required this.label,
    required this.button,
    required this.onTurboChanged,
  });

  final VirtualControlsSettings settings;
  final String label;
  final PadButton button;
  final void Function(PadButton button, bool enabled) onTurboChanged;

  @override
  Widget build(BuildContext context) {
    final s = settings.buttonSize;
    final turbo = s;

    final labelStyle = Theme.of(context).textTheme.titleMedium?.copyWith(
      color: Colors.white.withValues(alpha: 0.92),
      fontWeight: FontWeight.w900,
      letterSpacing: 0.2,
    );

    return _VirtualToggleButton(
      visualSize: Size.square(turbo),
      hitboxScale: settings.hitboxScale,
      hapticsEnabled: settings.hapticsEnabled,
      visualBuilder: (pressed) => _roundVisual(
        base: const Color(
          0xFF1D2128,
        ).withValues(alpha: (settings.opacity * 0.78).clamp(0.0, 1.0)),
        pressed: pressed,
        ringColor: const Color(
          0xFFFFC107,
        ).withValues(alpha: (settings.opacity * 0.75).clamp(0.0, 1.0)),
        child: Text(label, style: labelStyle),
      ),
      onToggle: (enabled) => onTurboChanged(button, enabled),
    );
  }
}

class _RewindButton extends StatelessWidget {
  const _RewindButton({
    required this.settings,
    required this.baseColor,
    required this.onRewindChanged,
  });

  final VirtualControlsSettings settings;
  final Color baseColor;
  final ValueChanged<bool> onRewindChanged;

  @override
  Widget build(BuildContext context) {
    return _VirtualPressButton(
      visualSize: Size.square(settings.buttonSize),
      hitboxScale: settings.hitboxScale,
      hapticsEnabled: settings.hapticsEnabled,
      onPressedChanged: onRewindChanged,
      visualBuilder: (pressed) {
        return Container(
          decoration: BoxDecoration(color: baseColor, shape: BoxShape.circle),
          child: Icon(
            Icons.history,
            color: Colors.white.withValues(alpha: pressed ? 0.9 : 0.7),
            size: settings.buttonSize * 0.6,
          ),
        );
      },
    );
  }
}

class _FastForwardButton extends StatelessWidget {
  const _FastForwardButton({
    required this.settings,
    required this.baseColor,
    required this.onFastForwardChanged,
  });

  final VirtualControlsSettings settings;
  final Color baseColor;
  final ValueChanged<bool> onFastForwardChanged;

  @override
  Widget build(BuildContext context) {
    return _VirtualPressButton(
      visualSize: Size.square(settings.buttonSize),
      hitboxScale: settings.hitboxScale,
      hapticsEnabled: settings.hapticsEnabled,
      onPressedChanged: onFastForwardChanged,
      visualBuilder: (pressed) {
        return Container(
          decoration: BoxDecoration(color: baseColor, shape: BoxShape.circle),
          child: Icon(
            Icons.fast_forward,
            color: Colors.white.withValues(alpha: pressed ? 0.9 : 0.7),
            size: settings.buttonSize * 0.6,
          ),
        );
      },
    );
  }
}

class _SystemButton extends StatelessWidget {
  const _SystemButton({
    required this.settings,
    required this.baseColor,
    required this.label,
    required this.button,
    required this.onButtonChanged,
  });

  final VirtualControlsSettings settings;
  final Color baseColor;
  final String label;
  final PadButton button;
  final void Function(PadButton button, bool pressed) onButtonChanged;

  @override
  Widget build(BuildContext context) {
    final s = settings.buttonSize;
    final h = s * 0.55;
    final w = s * 1.25;
    final textStyle = Theme.of(context).textTheme.labelLarge?.copyWith(
      color: Colors.white,
      fontWeight: FontWeight.w800,
      letterSpacing: 0.6,
    );

    Widget capsuleVisual(bool pressed) {
      final base = baseColor.withValues(alpha: pressed ? 0.55 : 0.65);
      return DecoratedBox(
        decoration: ShapeDecoration(
          shape: StadiumBorder(
            side: BorderSide(
              color: Colors.white.withValues(alpha: 0.16),
              width: 1,
            ),
          ),
          color: base,
          shadows: [
            BoxShadow(
              color: Colors.white.withValues(alpha: 0.05),
              blurRadius: 10,
              offset: const Offset(0, 0),
            ),
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.18),
              blurRadius: 10,
              offset: const Offset(0, 6),
            ),
          ],
        ),
        child: Center(child: Text(label, style: textStyle)),
      );
    }

    return _VirtualPressButton(
      visualSize: Size(w, h),
      hitboxScale: settings.hitboxScale,
      hapticsEnabled: settings.hapticsEnabled,
      visualBuilder: (pressed) => capsuleVisual(pressed),
      onPressedChanged: (pressed) => onButtonChanged(button, pressed),
    );
  }
}
