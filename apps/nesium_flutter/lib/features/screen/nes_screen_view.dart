import 'package:flutter/material.dart';

class NesScreenView extends StatelessWidget {
  const NesScreenView({
    super.key,
    required this.loading,
    required this.error,
    required this.textureId,
  });

  final bool loading;
  final String? error;
  final int? textureId;

  @override
  Widget build(BuildContext context) {
    if (loading) {
      return const Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            CircularProgressIndicator(),
            SizedBox(height: 12),
            Text('Initializing NES texture...'),
          ],
        ),
      );
    }

    if (error != null) {
      return Center(
        child: Column(
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
        ),
      );
    }

    if (textureId == null) {
      return const Center(child: Text('Texture ID is null (unexpected).'));
    }

    return LayoutBuilder(
      builder: (context, constraints) {
        const double nesWidth = 256;
        const double nesHeight = 240;
        if (constraints.maxWidth <= 0 || constraints.maxHeight <= 0) {
          return const SizedBox.shrink();
        }

        const double aspect = nesWidth / nesHeight;
        double width = constraints.maxWidth;
        double height = width / aspect;

        if (height > constraints.maxHeight) {
          height = constraints.maxHeight;
          width = height * aspect;
        }

        return Center(
          child: SizedBox(
            width: width,
            height: height,
            child: Texture(textureId: textureId!),
          ),
        );
      },
    );
  }
}
