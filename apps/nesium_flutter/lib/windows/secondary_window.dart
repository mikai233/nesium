import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../features/debugger/debugger_panel.dart';
import '../features/debugger/tilemap_viewer.dart';
import '../features/tools/tools_panel.dart';
import '../platform/platform_capabilities.dart';

class SecondaryWindow extends StatefulWidget {
  const SecondaryWindow({super.key, required this.child, required this.title});

  final Widget child;
  final String title;

  @override
  State<SecondaryWindow> createState() => _SecondaryWindowState();
}

class _SecondaryWindowState extends State<SecondaryWindow> {
  @override
  void initState() {
    super.initState();
    if (isNativeDesktop) {
      _setNativeTitle();
    }
  }

  Future<void> _setNativeTitle() async {
    try {
      const channel = MethodChannel('nesium/window');
      await channel.invokeMethod('setWindowTitle', widget.title);
    } catch (_) {}
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(body: widget.child);
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

class SecondaryTilemapContent extends StatelessWidget {
  const SecondaryTilemapContent({super.key});

  @override
  Widget build(BuildContext context) {
    return const TilemapViewer();
  }
}
