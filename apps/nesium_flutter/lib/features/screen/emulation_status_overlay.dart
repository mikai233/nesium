import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/emulation_status.dart';
import '../settings/emulation_settings.dart';

class EmulationStatusOverlay extends ConsumerWidget {
  const EmulationStatusOverlay({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final mode = ref.watch(
      emulationStatusProvider.select((s) => s.overlayMode),
    );
    final showOverlay = ref.watch(
      emulationSettingsProvider.select((s) => s.showEmulationStatusOverlay),
    );
    if (!showOverlay) return const SizedBox.shrink();

    if (mode == EmulationOverlayMode.none) return const SizedBox.shrink();

    return IgnorePointer(
      child: Stack(
        fit: StackFit.expand,
        children: [
          // We use a centered alignment for the main status indicator
          Center(
            child: AnimatedSwitcher(
              duration: const Duration(milliseconds: 200),
              // Use default linear curves here to ensure the source animation stays 0..1
              transitionBuilder: (child, animation) {
                // Fade must be strictly 0..1
                final fade = CurvedAnimation(
                  parent: animation,
                  curve: Curves.ease,
                );
                // Scale can overshoot 1.0 (easeOutBack) but should not go below 0.0
                // We use easeOutBack for entry (pop-in) and easeIn for exit (shrink-out)
                final scaleCurve = CurvedAnimation(
                  parent: animation,
                  curve: Curves.easeOutBack,
                  reverseCurve: Curves.easeIn,
                );
                final scale = Tween<double>(
                  begin: 0.8,
                  end: 1.0,
                ).animate(scaleCurve);
                return FadeTransition(
                  opacity: fade,
                  child: ScaleTransition(scale: scale, child: child),
                );
              },
              child: mode == EmulationOverlayMode.none
                  ? const SizedBox.shrink()
                  : _StatusIndicator(key: ValueKey(mode), mode: mode),
            ),
          ),
        ],
      ),
    );
  }
}

class _StatusIndicator extends StatelessWidget {
  const _StatusIndicator({super.key, required this.mode});

  final EmulationOverlayMode mode;

  @override
  Widget build(BuildContext context) {
    final color = Colors.white;

    final Widget icon = switch (mode) {
      EmulationOverlayMode.paused => Icon(
        Icons.pause_rounded,
        color: color,
        size: 48,
        shadows: const [Shadow(blurRadius: 10, color: Colors.black45)],
      ),
      EmulationOverlayMode.rewinding => const _AnimatedDirectionIcon(
        icon: Icons.fast_rewind_rounded,
        direction: -1,
      ),
      EmulationOverlayMode.fastForwarding => const _AnimatedDirectionIcon(
        icon: Icons.fast_forward_rounded,
        direction: 1,
      ),
      EmulationOverlayMode.none => const SizedBox.shrink(),
    };

    return ClipRRect(
      borderRadius: BorderRadius.circular(24),
      child: BackdropFilter(
        filter: ImageFilter.blur(sigmaX: 10, sigmaY: 10),
        child: Container(
          padding: const EdgeInsets.all(24),
          decoration: BoxDecoration(
            color: Colors.black.withValues(alpha: 0.4),
            borderRadius: BorderRadius.circular(24),
            border: Border.all(
              color: Colors.white.withValues(alpha: 0.1),
              width: 1.5,
            ),
            boxShadow: [
              BoxShadow(
                color: Colors.black.withValues(alpha: 0.2),
                blurRadius: 20,
                spreadRadius: 5,
              ),
            ],
          ),
          child: icon,
        ),
      ),
    );
  }
}

class _AnimatedDirectionIcon extends StatefulWidget {
  const _AnimatedDirectionIcon({required this.icon, required this.direction});

  final IconData icon;
  final double direction;

  @override
  State<_AnimatedDirectionIcon> createState() => _AnimatedDirectionIconState();
}

class _AnimatedDirectionIconState extends State<_AnimatedDirectionIcon>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller = AnimationController(
    vsync: this,
    duration: const Duration(milliseconds: 600),
  )..repeat();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    const iconSize = 48.0;
    const color = Colors.white;
    const shadow = Shadow(blurRadius: 8, color: Colors.black38);

    return SizedBox(
      width: iconSize * 1.5,
      height: iconSize,
      child: Stack(
        alignment: Alignment.center,
        children: [
          // Animated overlay
          AnimatedBuilder(
            animation: _controller,
            builder: (context, child) {
              final val = _controller.value;
              // 0 -> 1
              // Move from slightly left to center to slightly right (if forward)
              // Opacity peaks in center
              final shift = (val - 0.5) * 12.0 * widget.direction;
              final opacity =
                  1.0 - (2 * (val - 0.5)).abs(); // Triangle wave 0->1->0

              return Transform.translate(
                offset: Offset(shift, 0),
                child: Opacity(
                  opacity: opacity,
                  child: Icon(
                    widget.icon,
                    color: color,
                    size: iconSize,
                    shadows: const [shadow],
                  ),
                ),
              );
            },
          ),
        ],
      ),
    );
  }
}
