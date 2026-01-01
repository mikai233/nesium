import 'dart:convert';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../platform/platform_capabilities.dart';
import '../shell/nes_shell.dart';
import 'secondary_window.dart';
import 'window_types.dart';

String encodeWindowArguments(WindowKind kind, {String? languageCode}) {
  switch (kind) {
    case WindowKind.main:
      return jsonEncode({
        'route': 'main',
        if (languageCode != null) 'lang': languageCode,
      });
    case WindowKind.debugger:
      return jsonEncode({
        'route': 'debugger',
        if (languageCode != null) 'lang': languageCode,
      });
    case WindowKind.tools:
      return jsonEncode({
        'route': 'tools',
        if (languageCode != null) 'lang': languageCode,
      });
    case WindowKind.tilemap:
      return jsonEncode({
        'route': 'tilemap',
        if (languageCode != null) 'lang': languageCode,
      });
  }
}

WindowKind _parseWindowKindFromArguments(String? arguments) {
  if (arguments == null || arguments.isEmpty) {
    return WindowKind.main;
  }

  try {
    final data = jsonDecode(arguments);
    if (data is Map && data['route'] is String) {
      switch (data['route'] as String) {
        case 'debugger':
          return WindowKind.debugger;
        case 'tools':
          return WindowKind.tools;
        case 'tilemap':
          return WindowKind.tilemap;
      }
    }
  } catch (_) {
    // Ignore malformed payloads, treat as main window.
  }
  return WindowKind.main;
}

Future<WindowKind> resolveWindowKind() async {
  if (!isNativeDesktop) {
    return WindowKind.main;
  }

  try {
    final controller = await WindowController.fromCurrentEngine();
    return _parseWindowKindFromArguments(controller.arguments);
  } catch (_) {
    return WindowKind.main;
  }
}

class WindowRouter extends StatefulWidget {
  const WindowRouter({super.key});

  @override
  State<WindowRouter> createState() => _WindowRouterState();
}

class _WindowRouterState extends State<WindowRouter> {
  WindowKind? _kind;
  bool get _isDesktop => isNativeDesktop;

  @override
  void initState() {
    super.initState();
    _resolveKind();
  }

  Future<void> _resolveKind() async {
    if (!_isDesktop) {
      setState(() => _kind = WindowKind.main);
      return;
    }

    try {
      final controller = await WindowController.fromCurrentEngine();
      final kind = _parseWindowKindFromArguments(controller.arguments);
      if (!mounted) return;
      setState(() => _kind = kind);
    } catch (_) {
      if (!mounted) return;
      setState(() => _kind = WindowKind.main);
    }
  }

  @override
  Widget build(BuildContext context) {
    final kind = _kind;
    if (kind == null) {
      return const Material(child: Center(child: CircularProgressIndicator()));
    }

    final l10n = AppLocalizations.of(context)!;
    switch (kind) {
      case WindowKind.main:
        return const NesShell();
      case WindowKind.debugger:
        return SecondaryWindow(
          title: l10n.menuDebugger,
          child: const SecondaryDebuggerContent(),
        );
      case WindowKind.tools:
        return SecondaryWindow(
          title: l10n.menuTools,
          child: const SecondaryToolsContent(),
        );
      case WindowKind.tilemap:
        return SecondaryWindow(
          title: l10n.menuTilemapViewer,
          child: const SecondaryTilemapContent(),
        );
    }
  }
}
