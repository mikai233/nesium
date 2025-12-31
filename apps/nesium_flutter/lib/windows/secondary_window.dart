import 'package:flutter/material.dart';

import '../features/debugger/debugger_panel.dart';
import '../features/tools/tools_panel.dart';

class SecondaryWindow extends StatelessWidget {
  const SecondaryWindow({super.key, required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: SafeArea(
        child: Padding(padding: const EdgeInsets.all(16), child: child),
      ),
    );
  }
}

class SecondaryDebuggerContent extends StatelessWidget {
  const SecondaryDebuggerContent({super.key});

  @override
  Widget build(BuildContext context) {
    return const DebuggerPanel();
  }
}

class SecondaryToolsContent extends StatelessWidget {
  const SecondaryToolsContent({super.key});

  @override
  Widget build(BuildContext context) {
    return const ToolsPanel();
  }
}
