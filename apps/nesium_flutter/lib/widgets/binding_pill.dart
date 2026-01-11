import 'package:flutter/material.dart';

class BindingPill extends StatefulWidget {
  final String label;
  final String buttonName;
  final bool isPressed;
  final bool isRemapping;
  final bool isConflicted;
  final String? conflictLabel;
  final bool isEnabled;
  final VoidCallback? onTap;
  final VoidCallback? onLongPress;
  final IconData? icon;

  const BindingPill({
    super.key,
    required this.label,
    required this.buttonName,
    required this.isPressed,
    required this.isRemapping,
    required this.isConflicted,
    this.isEnabled = true,
    this.conflictLabel,
    required this.onTap,
    this.onLongPress,
    this.icon,
  });

  @override
  State<BindingPill> createState() => _BindingPillState();
}

class _BindingPillState extends State<BindingPill> {
  bool _isHovered = false;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    final borderColor = !widget.isEnabled
        ? colorScheme.outlineVariant.withAlpha(50)
        : widget.isRemapping
        ? colorScheme.primary
        : widget.isConflicted
        ? colorScheme.error
        : widget.isPressed
        ? colorScheme.secondary
        : _isHovered
        ? colorScheme.primary.withAlpha(150)
        : colorScheme.outlineVariant.withAlpha(128);

    final backgroundColor = !widget.isEnabled
        ? colorScheme.surfaceContainerHighest.withAlpha(30)
        : widget.isRemapping
        ? colorScheme.primaryContainer
        : widget.isConflicted
        ? colorScheme.errorContainer
        : widget.isPressed
        ? colorScheme.secondaryContainer
        : _isHovered
        ? colorScheme.surfaceContainerHigh
        : colorScheme.surfaceContainerHighest.withAlpha(75);

    final textColor = !widget.isEnabled
        ? colorScheme.onSurface.withAlpha(80)
        : widget.isRemapping
        ? colorScheme.onPrimaryContainer
        : widget.isConflicted
        ? colorScheme.onErrorContainer
        : widget.isPressed
        ? colorScheme.onSecondaryContainer
        : colorScheme.onSurface;

    return MouseRegion(
      onEnter: (_) => setState(() => _isHovered = true),
      onExit: (_) => setState(() => _isHovered = false),
      cursor: SystemMouseCursors.click,
      child: AnimatedScale(
        scale: _isHovered ? 1.02 : 1.0,
        duration: const Duration(milliseconds: 150),
        curve: Curves.easeOutCubic,
        child: AnimatedContainer(
          duration: const Duration(milliseconds: 150),
          decoration: BoxDecoration(
            color: backgroundColor,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: borderColor, width: 1.5),
            boxShadow: _isHovered
                ? [
                    BoxShadow(
                      color: colorScheme.shadow.withAlpha(20),
                      blurRadius: 8,
                      offset: const Offset(0, 4),
                    ),
                  ]
                : [],
          ),
          child: Material(
            type: MaterialType.transparency,
            child: InkWell(
              onTap: widget.isEnabled ? widget.onTap : null,
              onLongPress: widget.isEnabled ? widget.onLongPress : null,
              borderRadius: BorderRadius.circular(12),
              splashColor: colorScheme.primary.withAlpha(30),
              hoverColor: Colors.transparent, // Handled by MouseRegion
              child: Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 12,
                  vertical: 8,
                ),
                child: Stack(
                  clipBehavior: Clip.none,
                  children: [
                    Row(
                      mainAxisSize: MainAxisSize.min,
                      children: [
                        if (widget.icon != null) ...[
                          Icon(
                            widget.icon,
                            size: 16,
                            color: textColor.withAlpha(180),
                          ),
                          const SizedBox(width: 8),
                        ],
                        Expanded(
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              if (widget.label.isNotEmpty)
                                Text(
                                  widget.label.toUpperCase(),
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: TextStyle(
                                    fontSize: 10,
                                    fontWeight: FontWeight.bold,
                                    color: textColor.withAlpha(150),
                                    letterSpacing: 0.5,
                                  ),
                                ),
                              const SizedBox(height: 4),
                              Container(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 6,
                                  vertical: 2,
                                ),
                                decoration: BoxDecoration(
                                  color: textColor.withAlpha(25),
                                  borderRadius: BorderRadius.circular(4),
                                  border: Border.all(
                                    color: textColor.withAlpha(50),
                                    width: 1,
                                  ),
                                ),
                                child: Text(
                                  widget.buttonName,
                                  maxLines: 1,
                                  overflow: TextOverflow.ellipsis,
                                  style: TextStyle(
                                    fontSize: 12,
                                    fontWeight: FontWeight.w800,
                                    color: textColor,
                                    height: 1.1,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ),
                    if (widget.isConflicted && widget.conflictLabel != null)
                      Positioned(
                        top: -4,
                        right: -4,
                        child: Container(
                          padding: const EdgeInsets.symmetric(
                            horizontal: 5,
                            vertical: 2,
                          ),
                          decoration: BoxDecoration(
                            color: colorScheme.error,
                            borderRadius: BorderRadius.circular(4),
                            boxShadow: [
                              BoxShadow(
                                color: colorScheme.shadow.withAlpha(40),
                                blurRadius: 4,
                                offset: const Offset(0, 2),
                              ),
                            ],
                          ),
                          child: Text(
                            widget.conflictLabel!,
                            style: TextStyle(
                              fontSize: 9,
                              fontWeight: FontWeight.w900,
                              color: colorScheme.onError,
                              height: 1.0,
                            ),
                          ),
                        ),
                      ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
