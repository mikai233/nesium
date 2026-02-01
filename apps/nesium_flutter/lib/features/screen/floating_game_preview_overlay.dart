import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../settings/widgets/floating_game_preview.dart';
import 'floating_game_preview_state.dart';

class FloatingGamePreviewOverlay extends ConsumerWidget {
  const FloatingGamePreviewOverlay({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final isSupportedPlatform =
        !kIsWeb &&
        (defaultTargetPlatform == TargetPlatform.android ||
            defaultTargetPlatform == TargetPlatform.iOS ||
            defaultTargetPlatform == TargetPlatform.linux);
    if (!isSupportedPlatform) return const SizedBox.shrink();

    final preview = ref.watch(floatingGamePreviewProvider);

    return FloatingGamePreview(
      visible: preview.visible,
      offset: preview.offset,
      onOffsetChanged: (newOffset) {
        ref.read(floatingGamePreviewProvider.notifier).setOffset(newOffset);
      },
      onClose: () => ref.read(floatingGamePreviewProvider.notifier).hide(),
    );
  }
}
