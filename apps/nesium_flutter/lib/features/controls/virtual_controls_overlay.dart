import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/src/rust/lib.dart' show PadButton;
import 'package:flutter/services.dart';

import '../../domain/nes_input_masks.dart';
import '../screen/nes_screen_view.dart';
import 'input_settings.dart';
import 'virtual_controls_settings.dart';

class VirtualControlsOverlay extends ConsumerWidget {
  const VirtualControlsOverlay({super.key, required this.isLandscape});

  final bool isLandscape;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(virtualControlsSettingsProvider);
    final inputSettings = ref.watch(inputSettingsProvider);
    if (inputSettings.device != InputDevice.virtualController) {
      return const SizedBox.shrink();
    }

    final safeInsets = MediaQuery.paddingOf(context);
    final input = ref.read(nesInputMasksProvider.notifier);

    return LayoutBuilder(
      builder: (context, constraints) {
        final viewport = NesScreenView.computeViewportSize(constraints);
        if (viewport == null) return const SizedBox.shrink();

        final available = constraints.biggest;
        final viewportLeft = (available.width - viewport.width) / 2;
        final viewportTop = (available.height - viewport.height) / 2;
        final bottomMargin = available.height - (viewportTop + viewport.height);
        final leftMargin = viewportLeft;
        final rightMargin = available.width - (viewportLeft + viewport.width);

        final dpadSize = _dpadClusterSize(settings);
        final buttonsSize = _abClusterSize(settings);
        final systemSize = _systemClusterSize(settings);

        final basePadding = 8.0;

        Offset dpadPos;
        Offset buttonsPos;

        if (bottomMargin >=
            math.max(dpadSize.height, buttonsSize.height) +
                basePadding * 2 +
                safeInsets.bottom) {
          dpadPos = Offset(
            basePadding,
            available.height - dpadSize.height - basePadding,
          );
          buttonsPos = Offset(
            available.width - buttonsSize.width - basePadding,
            available.height - buttonsSize.height - basePadding,
          );
        } else if (leftMargin >= dpadSize.width + basePadding * 2 &&
            rightMargin >= buttonsSize.width + basePadding * 2) {
          dpadPos = Offset(
            basePadding,
            available.height - dpadSize.height - basePadding,
          );
          buttonsPos = Offset(
            available.width - buttonsSize.width - basePadding,
            available.height - buttonsSize.height - basePadding,
          );
        } else {
          dpadPos = Offset(
            basePadding,
            available.height - dpadSize.height - basePadding,
          );
          buttonsPos = Offset(
            available.width - buttonsSize.width - basePadding,
            available.height - buttonsSize.height - basePadding,
          );
        }

        var systemPos = Offset(
          (available.width - systemSize.width) / 2,
          available.height - systemSize.height - basePadding,
        );

        final dpadOffset = isLandscape
            ? settings.landscapeDpadOffset
            : settings.portraitDpadOffset;
        final buttonsOffset = isLandscape
            ? settings.landscapeButtonsOffset
            : settings.portraitButtonsOffset;

        dpadPos += dpadOffset;
        buttonsPos += buttonsOffset;

        dpadPos = _clampPosition(
          dpadPos,
          size: dpadSize,
          available: available,
          safeInsets: safeInsets,
        );
        buttonsPos = _clampPosition(
          buttonsPos,
          size: buttonsSize,
          available: available,
          safeInsets: safeInsets,
        );
        systemPos = _clampPosition(
          systemPos,
          size: systemSize,
          available: available,
          safeInsets: safeInsets,
        );

        // Avoid placing Select/Start directly under the D-pad/A-B clusters.
        final dpadRect = systemPos & systemSize;
        final leftRect = dpadPos & dpadSize;
        final rightRect = buttonsPos & buttonsSize;
        if (dpadRect.overlaps(leftRect) || dpadRect.overlaps(rightRect)) {
          systemPos = systemPos.translate(
            0,
            -(systemSize.height + basePadding),
          );
          systemPos = _clampPosition(
            systemPos,
            size: systemSize,
            available: available,
            safeInsets: safeInsets,
          );
        }

        final chromeBase = const Color(
          0xFFB0B0B0,
        ).withValues(alpha: (settings.opacity * 0.55).clamp(0.0, 1.0));
        final chromeSurface = const Color(
          0xFF2B2B2B,
        ).withValues(alpha: (settings.opacity * 0.80).clamp(0.0, 1.0));

        return Stack(
          fit: StackFit.expand,
          children: [
            Positioned(
              left: dpadPos.dx,
              top: dpadPos.dy,
              width: dpadSize.width,
              height: dpadSize.height,
              child: _DpadCluster(
                settings: settings,
                baseColor: chromeBase,
                surfaceColor: chromeSurface,
                onButtonChanged: input.setPressed,
              ),
            ),
            Positioned(
              left: systemPos.dx,
              top: systemPos.dy,
              width: systemSize.width,
              height: systemSize.height,
              child: _SystemCluster(
                settings: settings,
                baseColor: chromeBase,
                onButtonChanged: input.setPressed,
              ),
            ),
            Positioned(
              left: buttonsPos.dx,
              top: buttonsPos.dy,
              width: buttonsSize.width,
              height: buttonsSize.height,
              child: _ButtonsCluster(
                settings: settings,
                onButtonChanged: input.setPressed,
                onTurboChanged: input.setTurboEnabled,
              ),
            ),
          ],
        );
      },
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

class _ButtonsCluster extends StatelessWidget {
  const _ButtonsCluster({
    required this.settings,
    required this.onButtonChanged,
    required this.onTurboChanged,
  });

  final VirtualControlsSettings settings;
  final void Function(PadButton button, bool pressed) onButtonChanged;
  final void Function(PadButton button, bool enabled) onTurboChanged;

  @override
  Widget build(BuildContext context) {
    final s = settings.buttonSize;
    final g = settings.gap;
    final main = s * 1.12;
    final turbo = s * 0.84;
    final mainHit = main * settings.hitboxScale;
    final turboHit = turbo * settings.hitboxScale;
    final dx = mainHit * 0.10;
    final dy = mainHit * 0.10;
    const pad = 8.0;

    final labelStyle = Theme.of(context).textTheme.titleMedium?.copyWith(
      color: Colors.white.withValues(alpha: 0.92),
      fontWeight: FontWeight.w900,
      letterSpacing: 0.2,
    );

    Widget roundVisual({
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

    Widget button(String label, PadButton button, {required Color color}) {
      return _VirtualPressButton(
        visualSize: Size.square(main),
        hitboxScale: settings.hitboxScale,
        hapticsEnabled: settings.hapticsEnabled,
        visualBuilder: (pressed) => roundVisual(
          base: color.withValues(alpha: settings.opacity),
          pressed: pressed,
          child: Text(label, style: labelStyle),
        ),
        onPressedChanged: (pressed) => onButtonChanged(button, pressed),
      );
    }

    Widget turboButton(String label, PadButton button, {required Color color}) {
      return _VirtualToggleButton(
        visualSize: Size.square(turbo),
        hitboxScale: settings.hitboxScale,
        hapticsEnabled: settings.hapticsEnabled,
        visualBuilder: (pressed) => roundVisual(
          base: color.withValues(alpha: settings.opacity),
          pressed: pressed,
          ringColor: const Color(
            0xFFFFC107,
          ).withValues(alpha: (settings.opacity * 0.9).clamp(0.0, 1.0)),
          child: Text(label, style: labelStyle),
        ),
        onToggle: (enabled) => onTurboChanged(button, enabled),
      );
    }

    return SizedBox(
      width: pad + mainHit + g + mainHit + dx + pad,
      height: pad + turboHit + g + mainHit + dy + pad,
      child: Stack(
        clipBehavior: Clip.none,
        children: [
          Positioned(
            left: pad,
            top: pad,
            child: turboButton(
              'TB',
              PadButton.b,
              color: const Color(0xFF3D3D3D),
            ),
          ),
          Positioned(
            left: pad + mainHit + g + dx,
            top: pad,
            child: turboButton(
              'TA',
              PadButton.a,
              color: const Color(0xFF3D3D3D),
            ),
          ),
          Positioned(
            left: pad,
            top: pad + turboHit + g + dy,
            child: button('B', PadButton.b, color: const Color(0xFFD32F2F)),
          ),
          Positioned(
            left: pad + mainHit + g + dx,
            top: pad + turboHit + g,
            child: button('A', PadButton.a, color: const Color(0xFFD32F2F)),
          ),
        ],
      ),
    );
  }
}

class _SystemCluster extends StatelessWidget {
  const _SystemCluster({
    required this.settings,
    required this.baseColor,
    required this.onButtonChanged,
  });

  final VirtualControlsSettings settings;
  final Color baseColor;
  final void Function(PadButton button, bool pressed) onButtonChanged;

  @override
  Widget build(BuildContext context) {
    final s = settings.buttonSize;
    final g = settings.gap;
    final h = s * 0.55;
    final w = s * 1.25;
    final textStyle = Theme.of(context).textTheme.labelLarge?.copyWith(
      color: Colors.white,
      fontWeight: FontWeight.w800,
      letterSpacing: 0.6,
    );

    Widget capsuleVisual(bool pressed, String label) {
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

    Widget button(String label, PadButton button) {
      return _VirtualPressButton(
        visualSize: Size(w, h),
        hitboxScale: settings.hitboxScale,
        hapticsEnabled: settings.hapticsEnabled,
        visualBuilder: (pressed) => capsuleVisual(pressed, label),
        onPressedChanged: (pressed) => onButtonChanged(button, pressed),
      );
    }

    final hitboxW = w * settings.hitboxScale;
    final hitboxH = h * settings.hitboxScale;
    const pad = 6.0;

    return SizedBox(
      width: pad + hitboxW * 2 + g + pad,
      height: pad + hitboxH + pad,
      child: Padding(
        padding: const EdgeInsets.all(pad),
        child: Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            button('SELECT', PadButton.select),
            SizedBox(width: g),
            button('START', PadButton.start),
          ],
        ),
      ),
    );
  }
}
