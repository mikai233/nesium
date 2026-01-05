import 'dart:async';

import 'package:flutter/material.dart';
import 'package:skeletonizer/skeletonizer.dart';

class ViewerSkeletonScope extends InheritedWidget {
  const ViewerSkeletonScope({
    super.key,
    required this.enabled,
    required super.child,
  });

  final bool enabled;

  static bool enabledOf(BuildContext context) {
    final scope = context
        .dependOnInheritedWidgetOfExactType<ViewerSkeletonScope>();
    return scope?.enabled ?? false;
  }

  @override
  bool updateShouldNotify(ViewerSkeletonScope oldWidget) {
    return oldWidget.enabled != enabled;
  }
}

class ViewerSkeletonizer extends StatefulWidget {
  const ViewerSkeletonizer({
    super.key,
    required this.enabled,
    required this.child,
    this.minEnabledDuration = const Duration(milliseconds: 450),
    this.switchAnimationDuration = const Duration(milliseconds: 300),
  });

  final bool enabled;
  final Widget child;
  final Duration minEnabledDuration;
  final Duration switchAnimationDuration;

  @override
  State<ViewerSkeletonizer> createState() => _ViewerSkeletonizerState();
}

class _ViewerSkeletonizerState extends State<ViewerSkeletonizer> {
  bool _effectiveEnabled = false;
  DateTime? _enabledAt;
  Timer? _disableTimer;

  @override
  void initState() {
    super.initState();
    _setDesiredEnabled(widget.enabled);
  }

  @override
  void didUpdateWidget(covariant ViewerSkeletonizer oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.enabled != widget.enabled ||
        oldWidget.minEnabledDuration != widget.minEnabledDuration) {
      _setDesiredEnabled(widget.enabled);
    }
  }

  @override
  void dispose() {
    _disableTimer?.cancel();
    super.dispose();
  }

  void _setDesiredEnabled(bool desired) {
    _disableTimer?.cancel();
    _disableTimer = null;

    if (desired) {
      _enabledAt = DateTime.now();
      if (_effectiveEnabled) return;
      setState(() => _effectiveEnabled = true);
      return;
    }

    if (!_effectiveEnabled) return;

    final enabledAt = _enabledAt;
    if (enabledAt == null) {
      setState(() => _effectiveEnabled = false);
      return;
    }

    final elapsed = DateTime.now().difference(enabledAt);
    final remaining = widget.minEnabledDuration - elapsed;
    if (remaining <= Duration.zero) {
      setState(() => _effectiveEnabled = false);
      return;
    }

    _disableTimer = Timer(remaining, () {
      if (!mounted) return;
      setState(() => _effectiveEnabled = false);
    });
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return Skeletonizer(
      enabled: _effectiveEnabled,
      ignorePointers: true,
      enableSwitchAnimation: true,
      switchAnimationConfig: SwitchAnimationConfig(
        duration: widget.switchAnimationDuration,
      ),
      containersColor: colorScheme.surfaceContainerHighest,
      child: ViewerSkeletonScope(
        enabled: _effectiveEnabled,
        child: widget.child,
      ),
    );
  }
}
