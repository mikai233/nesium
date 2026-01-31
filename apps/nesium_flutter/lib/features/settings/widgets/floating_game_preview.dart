import 'dart:math' as math;
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../screen/nes_screen_view.dart';
import '../../../domain/nes_controller.dart';
import '../../../l10n/app_localizations.dart';

class FloatingGamePreview extends ConsumerStatefulWidget {
  const FloatingGamePreview({
    super.key,
    required this.visible,
    required this.offset,
    required this.onOffsetChanged,
    required this.onClose,
  });

  final bool visible;
  final Offset offset;
  final ValueChanged<Offset> onOffsetChanged;
  final VoidCallback onClose;

  @override
  ConsumerState<FloatingGamePreview> createState() =>
      _FloatingGamePreviewState();
}

class _FloatingGamePreviewState extends ConsumerState<FloatingGamePreview>
    with SingleTickerProviderStateMixin {
  bool _isMinimized = false;
  late final AnimationController _entryController;
  late final Animation<double> _entryAnimation;

  Offset? _dragStartOffset;
  Offset? _dragStartPointer;

  @override
  void initState() {
    super.initState();
    _entryController = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 400),
    );
    _entryAnimation = CurvedAnimation(
      parent: _entryController,
      curve: Curves.easeOutBack,
    );

    _entryController.addListener(_onAnimationStatusChanged);

    if (widget.visible) {
      _entryController.forward();
    }
  }

  void _onAnimationStatusChanged() {
    // Rebuild when animation reaches 0 to effectively remove from tree via build()
    if (_entryController.value == 0 || _entryController.value == 1.0) {
      if (mounted) setState(() {});
    }
  }

  @override
  void didUpdateWidget(FloatingGamePreview oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.visible && !oldWidget.visible) {
      _entryController.forward();
    } else if (!widget.visible && oldWidget.visible) {
      _entryController.reverse();
    }
  }

  @override
  void dispose() {
    _entryController.removeListener(_onAnimationStatusChanged);
    _entryController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final status = _entryController.status;
    final isClosed = !widget.visible && status == AnimationStatus.dismissed;

    if (isClosed) {
      return const SizedBox.shrink();
    }

    final nesState = ref.watch(nesControllerProvider);
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    if (nesState.romHash == null) {
      return const SizedBox.shrink();
    }

    final screenSize = MediaQuery.sizeOf(context);
    final windowWidth = (_isMinimized ? 120.0 : 240.0) + 2.0;
    const headerHeight = 38.0;
    const contentHeight = 225.0;
    final windowHeight =
        (_isMinimized ? headerHeight : (headerHeight + contentHeight)) + 2.0;

    // Ensure the offset is always within screen bounds (e.g. after rotation)
    final maxDx = math.max(0.0, screenSize.width - windowWidth);
    final maxDy = math.max(0.0, screenSize.height - windowHeight);
    final clampedOffset = Offset(
      widget.offset.dx.clamp(0.0, maxDx),
      widget.offset.dy.clamp(0.0, maxDy),
    );

    if (clampedOffset != widget.offset) {
      // Use addPostFrameCallback to avoid updating state during build
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) {
          widget.onOffsetChanged(clampedOffset);
        }
      });
    }

    return Transform.translate(
      offset: clampedOffset,
      child: Align(
        alignment: Alignment.topLeft,
        child: ScaleTransition(
          scale: _entryAnimation,
          child: GestureDetector(
            onPanStart: (details) {
              _dragStartOffset = widget.offset;
              _dragStartPointer = details.globalPosition;
            },
            onPanUpdate: (details) {
              if (_dragStartOffset == null || _dragStartPointer == null) return;

              final pointerDelta = details.globalPosition - _dragStartPointer!;
              var newOffset = _dragStartOffset! + pointerDelta;

              // Clamp X/Y with safety for small screen sizes
              final panMaxDx = math.max(0.0, screenSize.width - windowWidth);
              final panMaxDy = math.max(0.0, screenSize.height - windowHeight);

              newOffset = Offset(
                newOffset.dx.clamp(0.0, panMaxDx),
                newOffset.dy.clamp(0.0, panMaxDy),
              );

              widget.onOffsetChanged(newOffset);
            },
            onPanEnd: (_) {
              _dragStartOffset = null;
              _dragStartPointer = null;
            },
            onPanCancel: () {
              _dragStartOffset = null;
              _dragStartPointer = null;
            },
            child: Material(
              elevation: 8,
              borderRadius: BorderRadius.circular(12),
              clipBehavior: Clip.antiAlias,
              color: theme.colorScheme.surface,
              child: AnimatedContainer(
                duration: const Duration(milliseconds: 250),
                curve: Curves.easeInOut,
                width: windowWidth,
                height: windowHeight,
                clipBehavior: Clip.hardEdge,
                decoration: BoxDecoration(
                  border: Border.all(
                    color: theme.colorScheme.outlineVariant,
                    width: 1,
                  ),
                  borderRadius: BorderRadius.circular(12),
                ),
                child: Stack(
                  children: [
                    _buildHeader(l10n, theme),
                    Positioned(
                      top: headerHeight,
                      left: 0,
                      width: 240,
                      height: contentHeight,
                      child: RepaintBoundary(
                        child: NesScreenView(
                          key: const ValueKey('floating_preview_game'),
                          textureId: nesState.textureId,
                          error: nesState.error,
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

  Widget _buildHeader(AppLocalizations l10n, ThemeData theme) {
    return Container(
      height: 38,
      padding: const EdgeInsets.symmetric(horizontal: 8),
      color: theme.colorScheme.secondaryContainer.withAlpha(128),
      child: Row(
        children: [
          Icon(
            Icons.drag_handle,
            size: 18,
            color: theme.colorScheme.onSecondaryContainer,
          ),
          if (!_isMinimized) ...[
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                l10n.appName,
                overflow: TextOverflow.ellipsis,
                style: theme.textTheme.labelSmall?.copyWith(
                  fontWeight: FontWeight.bold,
                  color: theme.colorScheme.onSecondaryContainer,
                ),
              ),
            ),
          ] else
            const Spacer(),
          InkResponse(
            onTap: () {
              setState(() {
                _isMinimized = !_isMinimized;
              });
            },
            radius: 20,
            child: Icon(
              _isMinimized ? Icons.expand_more : Icons.expand_less,
              size: 24,
            ),
          ),
          const SizedBox(width: 16),
          InkResponse(
            onTap: () {
              widget.onClose();
            },
            radius: 20,
            child: const Icon(Icons.close, size: 18),
          ),
        ],
      ),
    );
  }
}
