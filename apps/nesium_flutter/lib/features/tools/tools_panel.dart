import 'package:flutter/material.dart';

import '../common/panel_placeholder.dart';

class ToolsPanel extends StatelessWidget {
  const ToolsPanel({super.key});

  @override
  Widget build(BuildContext context) {
    return const PanelPlaceholder(
      title: 'Tools',
      body:
          'Recording/playback, input mapping, and cheats can share these widgets '
          'between desktop side panes and mobile bottom sheets.',
    );
  }
}
