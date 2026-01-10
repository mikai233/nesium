import 'dart:async';
import 'dart:math' as math;

import 'package:animations/animations.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

enum AnimatedDropdownMenuDensity { regular, compact }

class AnimatedDropdownMenu<T> extends StatefulWidget {
  const AnimatedDropdownMenu({
    super.key,
    this.labelText,
    required this.value,
    required this.entries,
    required this.onSelected,
    this.helperText,
    this.enabled = true,
    this.density = AnimatedDropdownMenuDensity.regular,
    this.minMenuWidth,
    this.maxMenuHeight = 360,
    this.margin = 8,
  }) : assert(
         density != AnimatedDropdownMenuDensity.regular ||
             (labelText != null && labelText != ''),
         'labelText is required for regular density',
       );

  final String? labelText;
  final String? helperText;
  final bool enabled;
  final AnimatedDropdownMenuDensity density;
  final T value;
  final List<DropdownMenuEntry<T>> entries;
  final FutureOr<void> Function(T value) onSelected;
  final double? minMenuWidth;
  final double maxMenuHeight;
  final double margin;

  @override
  State<AnimatedDropdownMenu<T>> createState() =>
      _AnimatedDropdownMenuState<T>();
}

@immutable
class _SelectAnchorGeometry {
  const _SelectAnchorGeometry({
    required this.menuWidth,
    required this.maxHeight,
    required this.openAbove,
    required this.horizontalOffset,
  });

  final double menuWidth;
  final double maxHeight;
  final bool openAbove;
  final double horizontalOffset;

  @override
  bool operator ==(Object other) =>
      other is _SelectAnchorGeometry &&
      other.menuWidth == menuWidth &&
      other.maxHeight == maxHeight &&
      other.openAbove == openAbove &&
      other.horizontalOffset == horizontalOffset;

  @override
  int get hashCode =>
      Object.hash(menuWidth, maxHeight, openAbove, horizontalOffset);
}

class _NoTransitionModalConfiguration extends ModalConfiguration {
  const _NoTransitionModalConfiguration()
    : super(
        barrierColor: Colors.transparent,
        barrierDismissible: true,
        barrierLabel: 'Dismiss',
        transitionDuration: const Duration(milliseconds: 170),
        reverseTransitionDuration: const Duration(milliseconds: 120),
      );

  @override
  Widget transitionBuilder(
    BuildContext context,
    Animation<double> animation,
    Animation<double> secondaryAnimation,
    Widget child,
  ) {
    return child;
  }
}

class _AnimatedDropdownMenuState<T> extends State<AnimatedDropdownMenu<T>>
    with WidgetsBindingObserver {
  final LayerLink _link = LayerLink();
  final ValueNotifier<_SelectAnchorGeometry?> _geometry = ValueNotifier(null);
  bool _isOpen = false;

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _geometry.dispose();
    super.dispose();
  }

  @override
  void didChangeMetrics() {
    if (!_isOpen) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      _updateGeometry();
    });
  }

  DropdownMenuEntry<T> _selectedEntry() {
    for (final entry in widget.entries) {
      if (entry.value == widget.value) return entry;
    }
    return widget.entries.first;
  }

  InputDecorationTheme _decorationTheme(ColorScheme colorScheme) {
    final radius = switch (widget.density) {
      AnimatedDropdownMenuDensity.regular => 12.0,
      AnimatedDropdownMenuDensity.compact => 10.0,
    };
    final enabledBorder = OutlineInputBorder(
      borderRadius: BorderRadius.circular(radius),
      borderSide: BorderSide(
        color: colorScheme.outlineVariant.withValues(alpha: 0.7),
      ),
    );
    final focusedBorder = OutlineInputBorder(
      borderRadius: BorderRadius.circular(radius),
      borderSide: BorderSide(
        color: colorScheme.primary.withValues(alpha: 0.9),
        width: 1.2,
      ),
    );
    final padding = switch (widget.density) {
      AnimatedDropdownMenuDensity.regular => const EdgeInsets.fromLTRB(
        14,
        14,
        12,
        14,
      ),
      AnimatedDropdownMenuDensity.compact => const EdgeInsets.fromLTRB(
        12,
        10,
        10,
        10,
      ),
    };
    return InputDecorationTheme(
      filled: true,
      fillColor: colorScheme.surface,
      isDense: true,
      contentPadding: padding,
      border: enabledBorder,
      enabledBorder: enabledBorder,
      focusedBorder: focusedBorder,
    );
  }

  MenuStyle _menuStyle(ColorScheme colorScheme) {
    return MenuStyle(
      backgroundColor: WidgetStateProperty.all(colorScheme.surface),
      elevation: WidgetStateProperty.all(4),
      shape: WidgetStateProperty.all(
        RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      ),
    );
  }

  void _updateGeometry() {
    if (!mounted) return;
    final renderBox = context.findRenderObject() as RenderBox?;
    final overlay =
        Overlay.of(context).context.findRenderObject() as RenderBox?;
    if (renderBox == null || overlay == null) return;

    final size = renderBox.size;
    final topLeft = renderBox.localToGlobal(Offset.zero, ancestor: overlay);
    final bottomY = topLeft.dy + size.height;
    final margin = widget.margin;

    final menuWidth = math.max(size.width, widget.minMenuWidth ?? 0);

    final availableBelow = overlay.size.height - bottomY - margin;
    final availableAbove = topLeft.dy - margin;
    final openAbove =
        availableBelow < 220 && availableAbove > availableBelow + 64;

    final availableSpace = openAbove ? availableAbove : availableBelow;
    final maxHeight = math.max(
      0.0,
      math.min(widget.maxMenuHeight, availableSpace),
    );

    final horizontalOverflowRight =
        (topLeft.dx + menuWidth) - (overlay.size.width - margin);
    double shiftX = 0;
    if (horizontalOverflowRight > 0) {
      shiftX -= horizontalOverflowRight;
    }
    if (topLeft.dx + shiftX < margin) {
      shiftX += margin - (topLeft.dx + shiftX);
    }

    final next = _SelectAnchorGeometry(
      menuWidth: menuWidth,
      maxHeight: maxHeight,
      openAbove: openAbove,
      horizontalOffset: shiftX,
    );
    if (_geometry.value != next) _geometry.value = next;
  }

  Future<void> _openMenu() async {
    if (_isOpen || !widget.enabled) return;
    _updateGeometry();
    if (_geometry.value == null) return;

    setState(() => _isOpen = true);
    try {
      final result = await showModal<Object?>(
        context: context,
        configuration: const _NoTransitionModalConfiguration(),
        builder: (modalContext) {
          return _AnchoredSelectMenu<T>(
            link: _link,
            geometry: _geometry,
            entries: widget.entries,
            selectedValue: widget.value,
            menuStyle: _menuStyle(Theme.of(context).colorScheme),
          );
        },
      );
      if (!mounted) return;
      if (result is _DropdownMenuResult<T>) {
        final ret = widget.onSelected(result.value);
        if (ret is Future) unawaited(ret);
      }
    } finally {
      if (mounted) setState(() => _isOpen = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    if (_isOpen) {
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (!mounted) return;
        _updateGeometry();
      });
    }

    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final selected = _selectedEntry();

    final decoration = InputDecoration(
      labelText: widget.labelText,
      helperText: widget.helperText,
      enabled: widget.enabled,
      suffixIcon: AnimatedRotation(
        turns: _isOpen ? 0.5 : 0.0,
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOutCubic,
        child: Icon(
          Icons.expand_more_rounded,
          color: colorScheme.onSurfaceVariant,
        ),
      ),
    ).applyDefaults(_decorationTheme(colorScheme));

    return CompositedTransformTarget(
      link: _link,
      child: Material(
        color: Colors.transparent,
        child: InkWell(
          borderRadius: BorderRadius.circular(
            widget.density == AnimatedDropdownMenuDensity.compact ? 10 : 12,
          ),
          onTap: widget.enabled ? _openMenu : null,
          child: IgnorePointer(
            ignoring: true,
            child: InputDecorator(
              decoration: decoration,
              isFocused: _isOpen,
              child: DefaultTextStyle(
                style:
                    (widget.density == AnimatedDropdownMenuDensity.compact
                        ? theme.textTheme.bodyMedium
                        : theme.textTheme.bodyLarge) ??
                    const TextStyle(),
                child:
                    selected.labelWidget ??
                    Text(
                      selected.label,
                      overflow: TextOverflow.ellipsis,
                      maxLines: 1,
                    ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

class _AnchoredSelectMenu<T> extends StatelessWidget {
  const _AnchoredSelectMenu({
    required this.link,
    required this.geometry,
    required this.entries,
    required this.selectedValue,
    required this.menuStyle,
  });

  final LayerLink link;
  final ValueListenable<_SelectAnchorGeometry?> geometry;
  final List<DropdownMenuEntry<T>> entries;
  final T selectedValue;
  final MenuStyle menuStyle;

  Color _resolveColor(
    BuildContext context,
    WidgetStateProperty<Color?>? property,
  ) {
    return property?.resolve(const <WidgetState>{}) ??
        Theme.of(context).colorScheme.surface;
  }

  double _resolveElevation(WidgetStateProperty<double?>? property) {
    return property?.resolve(const <WidgetState>{}) ?? 4;
  }

  OutlinedBorder _resolveShape(WidgetStateProperty<OutlinedBorder?>? property) {
    return property?.resolve(const <WidgetState>{}) ??
        RoundedRectangleBorder(borderRadius: BorderRadius.circular(12));
  }

  @override
  Widget build(BuildContext context) {
    final route = ModalRoute.of(context);
    final animation = route?.animation;
    if (animation == null) return const SizedBox.shrink();

    final eased = CurvedAnimation(
      parent: animation,
      curve: Curves.easeOutCubic,
      reverseCurve: Curves.easeInCubic,
    );

    return ValueListenableBuilder(
      valueListenable: geometry,
      builder: (context, geo, _) {
        if (geo == null) return const SizedBox.shrink();

        final openAbove = geo.openAbove;
        final targetAnchor = openAbove
            ? Alignment.topLeft
            : Alignment.bottomLeft;
        final followerAnchor = openAbove
            ? Alignment.bottomLeft
            : Alignment.topLeft;
        final scaleAlignment = openAbove
            ? Alignment.bottomLeft
            : Alignment.topLeft;

        final background = _resolveColor(context, menuStyle.backgroundColor);
        final elevation = _resolveElevation(menuStyle.elevation);
        final shape = _resolveShape(menuStyle.shape);
        final colorScheme = Theme.of(context).colorScheme;

        return Align(
          alignment: Alignment.topLeft,
          child: CompositedTransformFollower(
            link: link,
            showWhenUnlinked: false,
            targetAnchor: targetAnchor,
            followerAnchor: followerAnchor,
            offset: Offset(geo.horizontalOffset, openAbove ? -8 : 8),
            child: FadeTransition(
              opacity: eased,
              child: ScaleTransition(
                alignment: scaleAlignment,
                scale: Tween<double>(begin: 0.98, end: 1).animate(eased),
                child: ConstrainedBox(
                  constraints: BoxConstraints.tightFor(
                    width: geo.menuWidth,
                  ).copyWith(maxHeight: geo.maxHeight),
                  child: Material(
                    color: background,
                    elevation: elevation,
                    shape: shape,
                    clipBehavior: Clip.antiAlias,
                    child: ListView.separated(
                      padding: const EdgeInsets.symmetric(
                        vertical: 6,
                        horizontal: 6,
                      ),
                      shrinkWrap: true,
                      itemCount: entries.length,
                      separatorBuilder: (context, index) =>
                          const SizedBox(height: 2),
                      itemBuilder: (context, index) {
                        final entry = entries[index];
                        final selected = entry.value == selectedValue;
                        final enabled = entry.enabled;
                        return InkWell(
                          borderRadius: BorderRadius.circular(10),
                          onTap: enabled
                              ? () => Navigator.of(
                                  context,
                                ).pop(_DropdownMenuResult(entry.value))
                              : null,
                          child: Padding(
                            padding: const EdgeInsets.symmetric(
                              horizontal: 10,
                              vertical: 10,
                            ),
                            child: Row(
                              children: [
                                if (entry.leadingIcon != null) ...[
                                  IconTheme(
                                    data: IconThemeData(
                                      size: 18,
                                      color: enabled
                                          ? colorScheme.onSurfaceVariant
                                          : colorScheme.onSurfaceVariant
                                                .withValues(alpha: 0.5),
                                    ),
                                    child: entry.leadingIcon!,
                                  ),
                                  const SizedBox(width: 10),
                                ],
                                Expanded(
                                  child: DefaultTextStyle(
                                    style:
                                        Theme.of(
                                          context,
                                        ).textTheme.bodyMedium?.copyWith(
                                          color: enabled
                                              ? colorScheme.onSurface
                                              : colorScheme.onSurface
                                                    .withValues(alpha: 0.5),
                                        ) ??
                                        const TextStyle(),
                                    child:
                                        entry.labelWidget ??
                                        Text(entry.label, maxLines: 2),
                                  ),
                                ),
                                if (selected) ...[
                                  const SizedBox(width: 10),
                                  Icon(
                                    Icons.check_rounded,
                                    size: 18,
                                    color: colorScheme.primary,
                                  ),
                                ],
                              ],
                            ),
                          ),
                        );
                      },
                    ),
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

@immutable
class _DropdownMenuResult<T> {
  const _DropdownMenuResult(this.value);
  final T value;
}
