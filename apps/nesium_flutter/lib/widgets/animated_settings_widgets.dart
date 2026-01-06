import 'package:flutter/material.dart';

/// Material You style animated settings container
/// Uses subtle background instead of heavy cards
class AnimatedSettingsCard extends StatefulWidget {
  const AnimatedSettingsCard({super.key, required this.child, this.index = 0});

  final Widget child;
  final int index;

  @override
  State<AnimatedSettingsCard> createState() => _AnimatedSettingsCardState();
}

class _AnimatedSettingsCardState extends State<AnimatedSettingsCard>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _fadeAnimation;
  late Animation<Offset> _slideAnimation;
  bool _isHovered = false;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 400),
      vsync: this,
    );

    _fadeAnimation = Tween<double>(
      begin: 0.0,
      end: 1.0,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOutCubic));

    _slideAnimation = Tween<Offset>(
      begin: const Offset(0, 0.05),
      end: Offset.zero,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeOutCubic));

    Future.delayed(Duration(milliseconds: 50 * widget.index), () {
      if (mounted) _controller.forward();
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return FadeTransition(
      opacity: _fadeAnimation,
      child: SlideTransition(
        position: _slideAnimation,
        child: MouseRegion(
          onEnter: (_) => setState(() => _isHovered = true),
          onExit: (_) => setState(() => _isHovered = false),
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 200),
            curve: Curves.easeOut,
            margin: const EdgeInsets.only(bottom: 12),
            decoration: BoxDecoration(
              color: colorScheme.surfaceContainerLow,
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: _isHovered
                    ? colorScheme.outline.withValues(alpha: 0.3)
                    : colorScheme.outline.withValues(alpha: 0.08),
                width: 1,
              ),
            ),
            child: ClipRRect(
              borderRadius: BorderRadius.circular(16),
              child: widget.child,
            ),
          ),
        ),
      ),
    );
  }
}

/// Animated slider tile with modern design
class AnimatedSliderTile extends StatelessWidget {
  const AnimatedSliderTile({
    super.key,
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.onChanged,
    required this.valueLabel,
    this.divisions,
  });

  final String label;
  final double value;
  final double min;
  final double max;
  final int? divisions;
  final ValueChanged<double> onChanged;
  final String valueLabel;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(
              child: Text(label, style: Theme.of(context).textTheme.bodyMedium),
            ),
            Text(
              valueLabel,
              style: Theme.of(context).textTheme.labelMedium?.copyWith(
                color: colorScheme.primary,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
        SliderTheme(
          data: SliderThemeData(
            activeTrackColor: colorScheme.primary,
            inactiveTrackColor: colorScheme.surfaceContainerHighest,
            thumbColor: colorScheme.primary,
            overlayColor: colorScheme.primary.withValues(alpha: 0.12),
            trackHeight: 6,
            thumbShape: const RoundSliderThumbShape(
              enabledThumbRadius: 10,
              elevation: 2,
            ),
            overlayShape: const RoundSliderOverlayShape(overlayRadius: 20),
          ),
          child: Slider(
            value: value.clamp(min, max),
            min: min,
            max: max,
            divisions: divisions,
            onChanged: onChanged,
          ),
        ),
      ],
    );
  }
}

/// Material You style section header
/// Simpler design with subtle typography emphasis and animated accent line
class AnimatedSectionHeader extends StatefulWidget {
  const AnimatedSectionHeader({
    super.key,
    required this.title,
    this.icon,
    this.delay = Duration.zero,
  });

  final String title;
  final IconData? icon;
  final Duration delay;

  @override
  State<AnimatedSectionHeader> createState() => _AnimatedSectionHeaderState();
}

class _AnimatedSectionHeaderState extends State<AnimatedSectionHeader>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _fadeAnimation;
  late Animation<double> _lineAnimation;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 600),
      vsync: this,
    );

    _fadeAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(
        parent: _controller,
        curve: const Interval(0.0, 0.4, curve: Curves.easeOut),
      ),
    );

    _lineAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(
        parent: _controller,
        curve: const Interval(0.2, 1.0, curve: Curves.easeOutCubic),
      ),
    );

    Future.delayed(widget.delay, () {
      if (mounted) _controller.forward();
    });
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return FadeTransition(
      opacity: _fadeAnimation,
      child: Padding(
        padding: const EdgeInsets.only(left: 4, bottom: 16, top: 12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                if (widget.icon != null) ...[
                  Icon(widget.icon, size: 20, color: colorScheme.primary),
                  const SizedBox(width: 8),
                ],
                Text(
                  widget.title,
                  style: theme.textTheme.titleMedium?.copyWith(
                    fontWeight: FontWeight.w600,
                    color: colorScheme.onSurface,
                    letterSpacing: 0.15,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 8),
            AnimatedBuilder(
              animation: _lineAnimation,
              builder: (context, child) {
                return Container(
                  height: 2,
                  width: 48 * _lineAnimation.value,
                  decoration: BoxDecoration(
                    gradient: LinearGradient(
                      colors: [
                        colorScheme.primary.withValues(alpha: 0.9),
                        colorScheme.primary.withValues(alpha: 0.0),
                      ],
                    ),
                    borderRadius: BorderRadius.circular(1),
                  ),
                );
              },
            ),
          ],
        ),
      ),
    );
  }
}

/// Animated expansion tile with smooth expand/collapse
class AnimatedExpansionTile extends StatefulWidget {
  const AnimatedExpansionTile({
    super.key,
    required this.title,
    required this.children,
    this.initiallyExpanded = false,
    this.leading,
  });

  final Widget title;
  final List<Widget> children;
  final bool initiallyExpanded;
  final Widget? leading;

  @override
  State<AnimatedExpansionTile> createState() => _AnimatedExpansionTileState();
}

class _AnimatedExpansionTileState extends State<AnimatedExpansionTile>
    with SingleTickerProviderStateMixin {
  late bool _isExpanded;
  late AnimationController _controller;
  late Animation<double> _iconRotation;
  late Animation<double> _heightFactor;

  @override
  void initState() {
    super.initState();
    _isExpanded = widget.initiallyExpanded;
    _controller = AnimationController(
      duration: const Duration(milliseconds: 300),
      vsync: this,
      value: _isExpanded ? 1.0 : 0.0,
    );

    _iconRotation = Tween<double>(
      begin: 0.0,
      end: 0.5,
    ).animate(CurvedAnimation(parent: _controller, curve: Curves.easeInOut));

    _heightFactor = CurvedAnimation(
      parent: _controller,
      curve: Curves.fastOutSlowIn,
    );
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  void _toggle() {
    setState(() {
      _isExpanded = !_isExpanded;
      if (_isExpanded) {
        _controller.forward();
      } else {
        _controller.reverse();
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        InkWell(
          onTap: _toggle,
          borderRadius: BorderRadius.circular(8),
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 12),
            child: Row(
              children: [
                if (widget.leading != null) ...[
                  widget.leading!,
                  const SizedBox(width: 16),
                ],
                Expanded(child: widget.title),
                RotationTransition(
                  turns: _iconRotation,
                  child: const Icon(Icons.expand_more),
                ),
              ],
            ),
          ),
        ),
        SizeTransition(
          sizeFactor: _heightFactor,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: widget.children,
          ),
        ),
      ],
    );
  }
}

/// Animated switch tile with state transition
class AnimatedSwitchTile extends StatelessWidget {
  const AnimatedSwitchTile({
    super.key,
    required this.value,
    required this.onChanged,
    required this.title,
    this.subtitle,
    this.leading,
  });

  final bool value;
  final ValueChanged<bool> onChanged;
  final Widget title;
  final Widget? subtitle;
  final Widget? leading;

  @override
  Widget build(BuildContext context) {
    return SwitchListTile(
      value: value,
      onChanged: onChanged,
      title: title,
      subtitle: subtitle != null
          ? AnimatedSwitcher(
              duration: const Duration(milliseconds: 200),
              transitionBuilder: (child, animation) {
                return FadeTransition(
                  opacity: animation,
                  child: SlideTransition(
                    position: Tween<Offset>(
                      begin: const Offset(0, -0.2),
                      end: Offset.zero,
                    ).animate(animation),
                    child: child,
                  ),
                );
              },
              child: subtitle,
            )
          : null,
      secondary: leading,
    );
  }
}
