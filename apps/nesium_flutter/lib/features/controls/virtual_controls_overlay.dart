import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/lib.dart' show PadButton;
import 'package:flutter/services.dart';

import '../../domain/nes_input_masks.dart';
import '../screen/nes_screen_view.dart';
import 'input_settings.dart';
import 'virtual_controls_editor.dart';
import 'virtual_controls_settings.dart';

class VirtualControlsOverlay extends ConsumerWidget {
  const VirtualControlsOverlay({super.key, required this.isLandscape});

  final bool isLandscape;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final editor = ref.watch(virtualControlsEditorProvider);
    final isEditing = editor.enabled;
    final liveSettings = ref.watch(virtualControlsSettingsProvider);
    final settings = (isEditing ? editor.draft : null) ?? liveSettings;

    final inputSettings = ref.watch(inputSettingsProvider);
    if (!isEditing && inputSettings.device != InputDevice.virtualController) {
      return const SizedBox.shrink();
    }

    final safeInsets = MediaQuery.paddingOf(context);
    final input = ref.read(nesInputMasksProvider.notifier);

    return LayoutBuilder(
      builder: (context, constraints) {
        final viewport = NesScreenView.computeViewportSize(constraints);
        if (viewport == null) return const SizedBox.shrink();

        final available = constraints.biggest;
        final basePadding = 8.0;

        final base = _basePositions(
          settings,
          isLandscape: isLandscape,
          available: available,
        );

        final dpadBaseSize = _dpadClusterSize(settings);
        final buttonsGroupBaseSize = _abClusterSize(settings);
        final systemGroupBaseSize = _systemClusterSize(settings);

        final mainButtonBaseSize = _mainButtonHitboxSize(settings);
        final turboButtonBaseSize = _turboButtonHitboxSize(settings);
        final systemButtonBaseSize = _systemButtonHitboxSize(settings);

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
        final buttonsGroupSize = Size(
          buttonsGroupBaseSize.width * buttonsScale,
          buttonsGroupBaseSize.height * buttonsScale,
        );
        final systemGroupSize = Size(
          systemGroupBaseSize.width * systemScale,
          systemGroupBaseSize.height * systemScale,
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

        var dpadPos = base.dpad + dpadOffset;
        final buttonsGroupPos = base.buttons + buttonsOffset;
        var systemGroupPos = base.system + systemOffset;

        dpadPos = _clampPosition(
          dpadPos,
          size: dpadSize,
          available: available,
          safeInsets: safeInsets,
        );

        // Avoid placing Select/Start directly under the D-pad/buttons group.
        //
        // When the user has explicitly positioned Select/Start (or when editing),
        // do not auto-shift them.
        if (!isEditing &&
            systemOffset == Offset.zero &&
            selectOffset == Offset.zero &&
            startOffset == Offset.zero) {
          final systemRect = systemGroupPos & systemGroupSize;
          final leftRect = dpadPos & dpadSize;
          final rightRect = buttonsGroupPos & buttonsGroupSize;
          if (systemRect.overlaps(leftRect) || systemRect.overlaps(rightRect)) {
            systemGroupPos = systemGroupPos.translate(
              0,
              -(systemGroupSize.height + basePadding),
            );
          }
        }

        final buttonsLocal = _buttonsLocalOffsets(settings);
        final systemLocal = _systemLocalOffsets(settings);

        final aTotalScale = buttonsScale * aScale;
        final bTotalScale = buttonsScale * bScale;
        final turboATotalScale = buttonsScale * turboAScale;
        final turboBTotalScale = buttonsScale * turboBScale;
        final selectTotalScale = systemScale * selectScale;
        final startTotalScale = systemScale * startScale;

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

        var aPos = buttonsGroupPos + buttonsLocal.a * buttonsScale + aOffset;
        var bPos = buttonsGroupPos + buttonsLocal.b * buttonsScale + bOffset;
        var turboAPos =
            buttonsGroupPos + buttonsLocal.turboA * buttonsScale + turboAOffset;
        var turboBPos =
            buttonsGroupPos + buttonsLocal.turboB * buttonsScale + turboBOffset;
        var selectPos =
            systemGroupPos + systemLocal.select * systemScale + selectOffset;
        var startPos =
            systemGroupPos + systemLocal.start * systemScale + startOffset;

        aPos = _clampPosition(
          aPos,
          size: aSize,
          available: available,
          safeInsets: safeInsets,
        );
        bPos = _clampPosition(
          bPos,
          size: bSize,
          available: available,
          safeInsets: safeInsets,
        );
        turboAPos = _clampPosition(
          turboAPos,
          size: turboASize,
          available: available,
          safeInsets: safeInsets,
        );
        turboBPos = _clampPosition(
          turboBPos,
          size: turboBSize,
          available: available,
          safeInsets: safeInsets,
        );
        selectPos = _clampPosition(
          selectPos,
          size: selectSize,
          available: available,
          safeInsets: safeInsets,
        );
        startPos = _clampPosition(
          startPos,
          size: startSize,
          available: available,
          safeInsets: safeInsets,
        );

        final chromeBase = const Color(
          0xFFB0B0B0,
        ).withValues(alpha: (settings.opacity * 0.55).clamp(0.0, 1.0));
        final chromeSurface = const Color(
          0xFF2B2B2B,
        ).withValues(alpha: (settings.opacity * 0.80).clamp(0.0, 1.0));

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
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _DpadCluster(
                    settings: settings,
                    baseColor: chromeBase,
                    surfaceColor: chromeSurface,
                    onButtonChanged: input.setPressed,
                  ),
                ),
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
                    onButtonChanged: input.setPressed,
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
                    onButtonChanged: input.setPressed,
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
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _TurboButton(
                    settings: settings,
                    label: 'TB',
                    button: PadButton.b,
                    onTurboChanged: input.setTurboEnabled,
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
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _TurboButton(
                    settings: settings,
                    label: 'TA',
                    button: PadButton.a,
                    onTurboChanged: input.setTurboEnabled,
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
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _MainButton(
                    settings: settings,
                    label: 'B',
                    button: PadButton.b,
                    onButtonChanged: input.setPressed,
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
                        ),
                      );
                },
                child: IgnorePointer(
                  ignoring: isEditing,
                  child: _MainButton(
                    settings: settings,
                    label: 'A',
                    button: PadButton.a,
                    onButtonChanged: input.setPressed,
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

enum _VirtualControlElement { dpad, a, b, turboA, turboB, select, start }

const double _clusterScaleMin = 0.6;
const double _clusterScaleMax = 1.8;

VirtualControlsSettings _applyElementTransform(
  VirtualControlsSettings draft, {
  required _VirtualControlElement element,
  required bool isLandscape,
  required Size available,
  required Offset topLeft,
  required double scale,
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
  }

  final base = _basePositions(
    withScale,
    isLandscape: isLandscape,
    available: available,
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
  final systemScale = isLandscape
      ? withScale.landscapeSystemScale
      : withScale.portraitSystemScale;

  final buttonsLocal = _buttonsLocalOffsets(withScale);
  final systemLocal = _systemLocalOffsets(withScale);

  switch (element) {
    case _VirtualControlElement.dpad:
      return isLandscape
          ? withScale.copyWith(landscapeDpadOffset: topLeft - base.dpad)
          : withScale.copyWith(portraitDpadOffset: topLeft - base.dpad);
    case _VirtualControlElement.a:
      final baseline =
          base.buttons + buttonsOffset + buttonsLocal.a * buttonsScale;
      return isLandscape
          ? withScale.copyWith(landscapeAOffset: topLeft - baseline)
          : withScale.copyWith(portraitAOffset: topLeft - baseline);
    case _VirtualControlElement.b:
      final baseline =
          base.buttons + buttonsOffset + buttonsLocal.b * buttonsScale;
      return isLandscape
          ? withScale.copyWith(landscapeBOffset: topLeft - baseline)
          : withScale.copyWith(portraitBOffset: topLeft - baseline);
    case _VirtualControlElement.turboA:
      final baseline =
          base.buttons + buttonsOffset + buttonsLocal.turboA * buttonsScale;
      return isLandscape
          ? withScale.copyWith(landscapeTurboAOffset: topLeft - baseline)
          : withScale.copyWith(portraitTurboAOffset: topLeft - baseline);
    case _VirtualControlElement.turboB:
      final baseline =
          base.buttons + buttonsOffset + buttonsLocal.turboB * buttonsScale;
      return isLandscape
          ? withScale.copyWith(landscapeTurboBOffset: topLeft - baseline)
          : withScale.copyWith(portraitTurboBOffset: topLeft - baseline);
    case _VirtualControlElement.select:
      final baseline =
          base.system + systemOffset + systemLocal.select * systemScale;
      return isLandscape
          ? withScale.copyWith(landscapeSelectOffset: topLeft - baseline)
          : withScale.copyWith(portraitSelectOffset: topLeft - baseline);
    case _VirtualControlElement.start:
      final baseline =
          base.system + systemOffset + systemLocal.start * systemScale;
      return isLandscape
          ? withScale.copyWith(landscapeStartOffset: topLeft - baseline)
          : withScale.copyWith(portraitStartOffset: topLeft - baseline);
  }
}

({Offset dpad, Offset buttons, Offset system}) _basePositions(
  VirtualControlsSettings settings, {
  required bool isLandscape,
  required Size available,
}) {
  final basePadding = 8.0;

  final dpadBaseSize = _dpadClusterSize(settings);
  final buttonsBaseSize = _abClusterSize(settings);
  final systemBaseSize = _systemClusterSize(settings);

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
  final systemSize = Size(
    systemBaseSize.width * systemScale,
    systemBaseSize.height * systemScale,
  );

  final dpadPos = Offset(
    basePadding,
    available.height - dpadSize.height - basePadding,
  );
  final buttonsPos = Offset(
    available.width - buttonsSize.width - basePadding,
    available.height - buttonsSize.height - basePadding,
  );
  final systemPos = Offset(
    (available.width - systemSize.width) / 2,
    available.height - systemSize.height - basePadding,
  );

  return (dpad: dpadPos, buttons: buttonsPos, system: systemPos);
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
    required this.scale,
    required this.topLeft,
    required this.available,
    required this.safeInsets,
    required this.gridSnapEnabled,
    required this.gridSpacing,
    required this.onTransform,
    required this.child,
  });

  final bool enabled;
  final Size baseSize;
  final double scale;
  final Offset topLeft;
  final Size available;
  final EdgeInsets safeInsets;
  final bool gridSnapEnabled;
  final double gridSpacing;
  final void Function(Offset topLeft, double scale) onTransform;
  final Widget child;

  @override
  State<_EditableCluster> createState() => _EditableClusterState();
}

class _EditableClusterState extends State<_EditableCluster> {
  static const double _resizeHandleSize = 32;

  bool _active = false;
  bool _resizeMode = false;
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
    _rawCenter = widget.topLeft + Offset(size.width / 2, size.height / 2);
    _rawTopLeft = widget.topLeft;
    _startScale = widget.scale;

    final minSide = math.min(size.width, size.height);
    final handleSize = math.min(_resizeHandleSize, minSide / 2);
    _resizeMode =
        widget.enabled &&
        details.pointerCount == 1 &&
        details.localFocalPoint.dx >= size.width - handleSize &&
        details.localFocalPoint.dy >= size.height - handleSize;

    final baseW = widget.baseSize.width;
    final baseH = widget.baseSize.height;
    _baseDiagonal = math.max(1e-6, math.sqrt(baseW * baseW + baseH * baseH));
    _diagonalDir = Offset(baseW / _baseDiagonal, baseH / _baseDiagonal);
    _rawDiagonal = _baseDiagonal * widget.scale;
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
      final maxScaleFromBounds = math.min(
        baseW <= 0
            ? _clusterScaleMax
            : (widget.available.width -
                          widget.safeInsets.right -
                          _rawTopLeft.dx)
                      .clamp(0.0, double.infinity) /
                  baseW,
        baseH <= 0
            ? _clusterScaleMax
            : (widget.available.height -
                          widget.safeInsets.bottom -
                          _rawTopLeft.dy)
                      .clamp(0.0, double.infinity) /
                  baseH,
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
    var center = rawCenter;
    if (widget.gridSnapEnabled) {
      final step = widget.gridSpacing.clamp(4.0, 128.0);
      center = Offset(
        (center.dx / step).roundToDouble() * step,
        (center.dy / step).roundToDouble() * step,
      );
    }

    final size = Size(
      widget.baseSize.width * nextScale,
      widget.baseSize.height * nextScale,
    );

    var topLeft = center - Offset(size.width / 2, size.height / 2);
    final clampedTopLeft = _clampPosition(
      topLeft,
      size: size,
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
  }

  @override
  Widget build(BuildContext context) {
    final handle = widget.enabled
        ? Container(
            decoration: BoxDecoration(
              border: Border.all(
                color: Colors.white.withValues(alpha: 0.25),
                width: 1,
              ),
              borderRadius: BorderRadius.circular(12),
            ),
          )
        : null;

    final resizeHandle = widget.enabled
        ? Positioned(
            right: 0,
            bottom: 0,
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
          if (handle != null) Positioned.fill(child: handle),
          if (resizeHandle != null) resizeHandle,
        ],
      ),
    );
  }
}

Offset _clampPosition(
  Offset pos, {
  required Size size,
  required Size available,
  required EdgeInsets safeInsets,
}) {
  final minX = safeInsets.left;
  final minY = safeInsets.top;
  final maxX = math.max(minX, available.width - size.width - safeInsets.right);
  final maxY = math.max(
    minY,
    available.height - size.height - safeInsets.bottom,
  );

  return Offset(
    pos.dx.clamp(minX, maxX).toDouble(),
    pos.dy.clamp(minY, maxY).toDouble(),
  );
}

Size _dpadClusterSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final g = settings.gap;
  final disc = s * 3 + g * 2;
  final hitbox = disc * settings.hitboxScale;
  return Size.square(hitbox);
}

Size _abClusterSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final g = settings.gap;
  final main = s * 1.12;
  final turbo = s * 0.84;
  final mainHit = main * settings.hitboxScale;
  final turboHit = turbo * settings.hitboxScale;
  final dx = mainHit * 0.10;
  final dy = mainHit * 0.10;
  const pad = 8.0;
  return Size(
    pad + mainHit + g + mainHit + dx + pad,
    pad + turboHit + g + mainHit + dy + pad,
  );
}

Size _systemClusterSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final g = settings.gap;
  final h = s * 0.55;
  final w = s * 1.25;
  final hitboxW = w * settings.hitboxScale;
  final hitboxH = h * settings.hitboxScale;
  const pad = 6.0;
  return Size(pad + hitboxW * 2 + g + pad, pad + hitboxH + pad);
}

Size _mainButtonHitboxSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final main = s * 1.12;
  return Size.square(main * settings.hitboxScale);
}

Size _turboButtonHitboxSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final turbo = s * 0.84;
  return Size.square(turbo * settings.hitboxScale);
}

Size _systemButtonHitboxSize(VirtualControlsSettings settings) {
  final s = settings.buttonSize;
  final h = s * 0.55;
  final w = s * 1.25;
  return Size(w * settings.hitboxScale, h * settings.hitboxScale);
}

({Offset turboB, Offset turboA, Offset b, Offset a}) _buttonsLocalOffsets(
  VirtualControlsSettings settings,
) {
  final g = settings.gap;
  final mainHit = _mainButtonHitboxSize(settings).width;
  final turboHit = _turboButtonHitboxSize(settings).width;
  final dx = mainHit * 0.10;
  final dy = mainHit * 0.10;
  const pad = 8.0;
  return (
    turboB: const Offset(pad, pad),
    turboA: Offset(pad + mainHit + g + dx, pad),
    b: Offset(pad, pad + turboHit + g + dy),
    a: Offset(pad + mainHit + g + dx, pad + turboHit + g),
  );
}

({Offset select, Offset start}) _systemLocalOffsets(
  VirtualControlsSettings settings,
) {
  final g = settings.gap;
  final buttonW = _systemButtonHitboxSize(settings).width;
  const pad = 6.0;
  return (
    select: const Offset(pad, pad),
    start: Offset(pad + buttonW + g, pad),
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
    final s = settings.buttonSize;
    final g = settings.gap;
    final disc = s * 3 + g * 2;
    final hitbox = disc * settings.hitboxScale;
    return SizedBox(
      width: hitbox,
      height: hitbox,
      child: _VirtualDpad(
        discDiameter: disc,
        deadzoneRatio: settings.dpadDeadzoneRatio,
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
    required this.hapticsEnabled,
    required this.baseColor,
    required this.surfaceColor,
    required this.onButtonsChanged,
  });

  final double discDiameter;
  final double deadzoneRatio;
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

    final dist = delta.distance;
    Set<PadButton> next;
    if (dist <= deadzone) {
      next = const {};
    } else {
      final angle = math.atan2(-delta.dy, delta.dx);
      final sector = ((angle / (math.pi / 4)).round() + 8) % 8;
      switch (sector) {
        case 0:
          next = {PadButton.right};
          break;
        case 1:
          next = {PadButton.right, PadButton.up};
          break;
        case 2:
          next = {PadButton.up};
          break;
        case 3:
          next = {PadButton.up, PadButton.left};
          break;
        case 4:
          next = {PadButton.left};
          break;
        case 5:
          next = {PadButton.left, PadButton.down};
          break;
        case 6:
          next = {PadButton.down};
          break;
        case 7:
          next = {PadButton.down, PadButton.right};
          break;
        default:
          next = const {};
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
    final radius = math.min(size.width, size.height) / 2;

    // Typical emulator overlays use a soft, translucent base with a darker cross.
    final disc = baseColor.withValues(
      alpha: (baseColor.a * 1.25).clamp(0.0, 1),
    );
    final cross = surfaceColor.withValues(
      alpha: (surfaceColor.a * 1.10).clamp(0.0, 1),
    );

    canvas.drawCircle(center, radius * 0.98, Paint()..color = disc);
    canvas.drawCircle(
      center,
      radius * 0.98,
      Paint()
        ..style = PaintingStyle.stroke
        ..strokeWidth = 1.0
        ..color = Colors.white.withValues(alpha: disc.a * 0.18),
    );

    final plusThickness = radius * 0.62;
    final plusLength = radius * 1.30;
    final plusRadius = Radius.circular(radius * 0.12);

    final vertical = RRect.fromRectAndRadius(
      Rect.fromCenter(center: center, width: plusThickness, height: plusLength),
      plusRadius,
    );
    final horizontal = RRect.fromRectAndRadius(
      Rect.fromCenter(center: center, width: plusLength, height: plusThickness),
      plusRadius,
    );
    final plusPath = Path()
      ..addRRect(vertical)
      ..addRRect(horizontal);

    canvas.drawShadow(plusPath, Colors.black.withValues(alpha: 0.18), 5, true);
    canvas.drawPath(plusPath, Paint()..color = cross);

    final plusBorder = Paint()
      ..color = Colors.white.withValues(alpha: disc.a * 0.10)
      ..style = PaintingStyle.stroke
      ..strokeWidth = 1.0;
    canvas.drawPath(plusPath, plusBorder);

    final centerRect = Rect.fromCircle(center: center, radius: radius * 0.16);
    canvas.drawCircle(
      center,
      radius * 0.16,
      Paint()
        ..shader = RadialGradient(
          colors: [
            Colors.black.withValues(alpha: cross.a * 0.28),
            Colors.transparent,
          ],
        ).createShader(centerRect),
    );

    final activePaint = Paint()..style = PaintingStyle.fill;
    final armHighlight = Colors.white.withValues(alpha: disc.a * 0.12);
    final arrowNormal = Colors.white.withValues(alpha: disc.a * 0.28);
    final arrowActive = Colors.white.withValues(alpha: disc.a * 0.65);

    if (active.contains(PadButton.up)) {
      activePaint.color = armHighlight;
      canvas.save();
      canvas.clipPath(plusPath);
      canvas.drawRect(
        Rect.fromLTRB(
          center.dx - plusThickness / 2,
          center.dy - plusLength / 2,
          center.dx + plusThickness / 2,
          center.dy - plusThickness / 2,
        ),
        activePaint,
      );
      canvas.restore();
    }
    if (active.contains(PadButton.down)) {
      activePaint.color = armHighlight;
      canvas.save();
      canvas.clipPath(plusPath);
      canvas.drawRect(
        Rect.fromLTRB(
          center.dx - plusThickness / 2,
          center.dy + plusThickness / 2,
          center.dx + plusThickness / 2,
          center.dy + plusLength / 2,
        ),
        activePaint,
      );
      canvas.restore();
    }
    if (active.contains(PadButton.left)) {
      activePaint.color = armHighlight;
      canvas.save();
      canvas.clipPath(plusPath);
      canvas.drawRect(
        Rect.fromLTRB(
          center.dx - plusLength / 2,
          center.dy - plusThickness / 2,
          center.dx - plusThickness / 2,
          center.dy + plusThickness / 2,
        ),
        activePaint,
      );
      canvas.restore();
    }
    if (active.contains(PadButton.right)) {
      activePaint.color = armHighlight;
      canvas.save();
      canvas.clipPath(plusPath);
      canvas.drawRect(
        Rect.fromLTRB(
          center.dx + plusThickness / 2,
          center.dy - plusThickness / 2,
          center.dx + plusLength / 2,
          center.dy + plusThickness / 2,
        ),
        activePaint,
      );
      canvas.restore();
    }

    void drawArrow(Offset tip, Offset left, Offset right, bool isActive) {
      final path = Path()
        ..moveTo(tip.dx, tip.dy)
        ..lineTo(left.dx, left.dy)
        ..lineTo(right.dx, right.dy)
        ..close();
      canvas.drawPath(
        path,
        Paint()
          ..color = isActive ? arrowActive : arrowNormal
          ..style = PaintingStyle.fill,
      );
    }

    final arrowTipOffset = radius * 0.60;
    final arrowBaseOffset = radius * 0.34;
    final arrowHalfWidth = radius * 0.16;
    drawArrow(
      center.translate(0, -arrowTipOffset),
      center.translate(-arrowHalfWidth, -arrowBaseOffset),
      center.translate(arrowHalfWidth, -arrowBaseOffset),
      active.contains(PadButton.up),
    );
    drawArrow(
      center.translate(0, arrowTipOffset),
      center.translate(arrowHalfWidth, arrowBaseOffset),
      center.translate(-arrowHalfWidth, arrowBaseOffset),
      active.contains(PadButton.down),
    );
    drawArrow(
      center.translate(-arrowTipOffset, 0),
      center.translate(-arrowBaseOffset, -arrowHalfWidth),
      center.translate(-arrowBaseOffset, arrowHalfWidth),
      active.contains(PadButton.left),
    );
    drawArrow(
      center.translate(arrowTipOffset, 0),
      center.translate(arrowBaseOffset, arrowHalfWidth),
      center.translate(arrowBaseOffset, -arrowHalfWidth),
      active.contains(PadButton.right),
    );
  }

  @override
  bool shouldRepaint(covariant _DpadPainter oldDelegate) {
    return oldDelegate.baseColor != baseColor ||
        oldDelegate.surfaceColor != surfaceColor ||
        oldDelegate.active != active;
  }
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
          color: Colors.black.withValues(alpha: 0.22),
          blurRadius: 10,
          offset: const Offset(0, 6),
        ),
      ],
      border: Border.all(
        color: (ringColor != null && pressed)
            ? ringColor
            : Colors.black.withValues(alpha: 0.22),
        width: (ringColor != null && pressed) ? 2.0 : 1.5,
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
    final main = s * 1.12;

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
        base: const Color(0xFFD32F2F).withValues(alpha: settings.opacity),
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
    final turbo = s * 0.84;

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
        base: const Color(0xFF3D3D3D).withValues(alpha: settings.opacity),
        pressed: pressed,
        ringColor: const Color(
          0xFFFFC107,
        ).withValues(alpha: (settings.opacity * 0.9).clamp(0.0, 1.0)),
        child: Text(label, style: labelStyle),
      ),
      onToggle: (enabled) => onTurboChanged(button, enabled),
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
              color: Colors.black.withValues(alpha: 0.18),
              width: 1,
            ),
          ),
          color: base,
          shadows: [
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
