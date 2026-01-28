import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../features/screen/nes_screen_view.dart';
import '../features/settings/video_settings.dart';
import '../domain/nes_state.dart';
import '../domain/nes_controller.dart';
import '../features/save_state/save_state_repository.dart';
import 'nes_actions.dart';
import 'nes_menu_bar.dart';
import 'nes_menu_model.dart';

class DesktopShell extends ConsumerWidget {
  const DesktopShell({super.key, required this.state, required this.actions});

  final NesState state;
  final NesActions actions;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final slotStates = ref.watch(saveStateRepositoryProvider);
    final hasRom = ref.watch(
      nesControllerProvider.select((s) => s.romHash != null),
    );

    final screenVerticalOffset = ref.watch(
      videoSettingsProvider.select((s) => s.screenVerticalOffset),
    );

    return Scaffold(
      body: MediaQuery.removePadding(
        context: context,
        removeLeft: true,
        removeRight: true,
        removeBottom: true,
        child: Column(
          children: [
            NesMenuBar(
              actions: actions,
              sections: NesMenus.desktopMenuSections(),
              slotStates: slotStates,
              hasRom: hasRom,
            ),
            Expanded(
              child: NesScreenView(
                error: state.error,
                textureId: state.textureId,
                screenVerticalOffset: screenVerticalOffset,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
