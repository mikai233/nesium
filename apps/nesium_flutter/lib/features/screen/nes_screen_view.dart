import 'package:flutter/material.dart';
import 'dart:math' as math;

class NesScreenView extends StatelessWidget {
  const NesScreenView({super.key, this.error, required this.textureId});

  final String? error;
  final int? textureId;

  static const double nesWidth = 256;
  static const double nesHeight = 240;

  static Size? computeViewportSize(BoxConstraints constraints) {
    if (constraints.maxWidth <= 0 || constraints.maxHeight <= 0) {
      return null;
    }

    final scale = math.min(
      constraints.maxWidth / nesWidth,
      constraints.maxHeight / nesHeight,
    );
    final finalScale = scale < 1.0 ? 1.0 : scale;
    return Size(nesWidth * finalScale, nesHeight * finalScale);
  }

  @override
  Widget build(BuildContext context) {
    Widget content;
    if (error != null) {
      content = Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          const Icon(Icons.error_outline, color: Colors.red),
          const SizedBox(height: 8),
          Text(
            'Failed to create texture',
            style: Theme.of(
              context,
            ).textTheme.titleMedium?.copyWith(color: Colors.red),
          ),
          const SizedBox(height: 4),
          Text(error!, textAlign: TextAlign.center),
        ],
      );
    } else if (textureId == null) {
      content = const SizedBox.shrink();
    } else {
      content = LayoutBuilder(
        builder: (context, constraints) {
          final viewport = computeViewportSize(constraints);
          if (viewport == null) return const SizedBox.shrink();

          return SizedBox(
            width: viewport.width,
            height: viewport.height,
            child: Texture(
              textureId: textureId!,
              filterQuality: FilterQuality.none, // nearest-neighbor scaling
            ),
          );
        },
      );
    }

    return Container(
      color: Colors.black,
      alignment: Alignment.center,
      child: content,
    );
  }
}
