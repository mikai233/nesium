import 'package:flutter/material.dart';

import 'shell/nes_shell.dart';
import 'windows/secondary_window.dart';

class NesiumApp extends StatelessWidget {
  const NesiumApp({super.key, this.windowKind});

  final String? windowKind;

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Nesium',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blueGrey,
          brightness: Brightness.light,
        ).copyWith(surface: Colors.white),
        scaffoldBackgroundColor: Colors.white,
        useMaterial3: true,
        visualDensity: VisualDensity.adaptivePlatformDensity,
      ),
      home: _buildHome(),
    );
  }

  Widget _buildHome() {
    switch (windowKind) {
      case 'debugger':
        return const SecondaryWindow(
          title: 'Nesium Debugger',
          child: SecondaryDebuggerContent(),
        );
      case 'tools':
        return const SecondaryWindow(
          title: 'Nesium Tools',
          child: SecondaryToolsContent(),
        );
      default:
        return const NesShell();
    }
  }
}
