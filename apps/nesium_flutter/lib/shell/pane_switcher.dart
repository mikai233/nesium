import 'package:flutter/material.dart';

import '../domain/nes_state.dart';
import '../features/debugger/debugger_panel.dart';
import '../features/screen/nes_screen_view.dart';
import '../features/tools/tools_panel.dart';
import 'panes.dart';

class PaneSwitcher extends StatelessWidget {
  const PaneSwitcher({
    super.key,
    required this.state,
    required this.selectedPane,
  });

  final NesState state;
  final NesPane selectedPane;

  @override
  Widget build(BuildContext context) {
    switch (selectedPane) {
      case NesPane.console:
        return NesScreenView(error: state.error, textureId: state.textureId);
      case NesPane.debugger:
        return const DebuggerPanel();
      case NesPane.tools:
        return const ToolsPanel();
    }
  }
}
