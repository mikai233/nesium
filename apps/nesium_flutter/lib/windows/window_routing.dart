import 'dart:convert';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../shell/nes_shell.dart';
import 'secondary_window.dart';

enum WindowKind { main, debugger, tools }

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
      }
    }
  } catch (_) {
    // Ignore malformed payloads, treat as main window.
  }
  return WindowKind.main;
}

class WindowRouter extends StatefulWidget {
  const WindowRouter({super.key});

  @override
  State<WindowRouter> createState() => _WindowRouterState();
}

class _WindowRouterState extends State<WindowRouter> {
  WindowKind? _kind;

  bool get _isDesktop =>
      !kIsWeb &&
      (defaultTargetPlatform == TargetPlatform.macOS ||
          defaultTargetPlatform == TargetPlatform.linux ||
          defaultTargetPlatform == TargetPlatform.windows);

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

    switch (kind) {
      case WindowKind.main:
        return const NesShell();
      case WindowKind.debugger:
        return SecondaryWindow(
          title: AppLocalizations.of(context)!.windowDebuggerTitle,
          child: const SecondaryDebuggerContent(),
        );
      case WindowKind.tools:
        return SecondaryWindow(
          title: AppLocalizations.of(context)!.windowToolsTitle,
          child: const SecondaryToolsContent(),
        );
    }
  }
}
