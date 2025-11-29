import 'package:flutter/material.dart';

import '../common/panel_placeholder.dart';

class DebuggerPanel extends StatelessWidget {
  const DebuggerPanel({super.key});

  @override
  Widget build(BuildContext context) {
    return const PanelPlaceholder(
      title: 'Debugger',
      body:
          'Space reserved for CPU/PPU monitors, memory viewers, and OAM inspectors. '
          'The same widgets can live in a desktop side panel or a mobile sheet.',
    );
  }
}
