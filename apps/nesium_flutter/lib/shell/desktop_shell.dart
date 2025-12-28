import 'package:flutter/material.dart';

import '../features/screen/nes_screen_view.dart';
import '../domain/nes_state.dart';
import 'nes_actions.dart';
import 'nes_menu_bar.dart';
import 'nes_menu_model.dart';

class DesktopShell extends StatelessWidget {
  const DesktopShell({super.key, required this.state, required this.actions});

  final NesState state;
  final NesActions actions;

  @override
  Widget build(BuildContext context) {
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
              sections: NesMenus.desktopMenuSections,
            ),
            Expanded(
              child: NesScreenView(
                error: state.error,
                textureId: state.textureId,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
