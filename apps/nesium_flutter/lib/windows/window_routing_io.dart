import 'dart:convert';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import '../logging/app_logger.dart';
import '../platform/platform_capabilities.dart';
import '../shell/nes_shell.dart';
import '../features/settings/settings_page.dart';
import 'secondary_window.dart';
import 'window_types.dart';

String encodeWindowArguments(
  WindowKind kind, {
  String? languageCode,
  String? mainWindowId,
  Map<String, dynamic>? initialData,
}) {
  final Map<String, dynamic> args = {
    'lang': ?languageCode,
    'mainId': ?mainWindowId,
    'initialData': ?initialData,
  };

  switch (kind) {
    case WindowKind.main:
      args['route'] = 'main';
      break;
    case WindowKind.debugger:
      args['route'] = 'debugger';
      break;
    case WindowKind.tools:
      args['route'] = 'tools';
      break;
    case WindowKind.tilemap:
      args['route'] = 'tilemap';
      break;
    case WindowKind.tileViewer:
      args['route'] = 'tileViewer';
      break;
    case WindowKind.spriteViewer:
      args['route'] = 'spriteViewer';
      break;
    case WindowKind.paletteViewer:
      args['route'] = 'paletteViewer';
      break;
    case WindowKind.settings:
      args['route'] = 'settings';
      break;
  }
  return jsonEncode(args);
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
        case 'tileViewer':
          return WindowKind.tileViewer;
        case 'spriteViewer':
          return WindowKind.spriteViewer;
        case 'paletteViewer':
          return WindowKind.paletteViewer;
        case 'settings':
          return WindowKind.settings;
      }
    }
  } catch (e, st) {
    logWarning(
      e,
      stackTrace: st,
      message: 'Failed to parse window arguments',
      logger: 'window_routing',
    );
  }
  return WindowKind.main;
}

Future<String?> resolveMainWindowId() async {
  if (!isNativeDesktop) {
    return null;
  }

  try {
    final controller = await WindowController.fromCurrentEngine();
    final args = controller.arguments;
    if (args.isEmpty) return null;

    final data = jsonDecode(args);
    if (data is Map && data['mainId'] is String) {
      return data['mainId'] as String;
    }
  } catch (e, st) {
    logWarning(
      e,
      stackTrace: st,
      message: 'Failed to resolve main window ID',
      logger: 'window_routing',
    );
  }
  return null;
}

Future<Map<String, dynamic>?> resolveInitialData() async {
  if (!isNativeDesktop) {
    return null;
  }

  try {
    final controller = await WindowController.fromCurrentEngine();
    final args = controller.arguments;
    if (args.isEmpty) return null;

    final data = jsonDecode(args);
    if (data is Map && data['initialData'] is Map) {
      return Map<String, dynamic>.from(data['initialData'] as Map);
    }
  } catch (e, st) {
    logWarning(
      e,
      stackTrace: st,
      message: 'Failed to resolve initial data',
      logger: 'window_routing',
    );
  }
  return null;
}

Future<WindowKind> resolveWindowKind() async {
  if (!isNativeDesktop) {
    return WindowKind.main;
  }

  try {
    final controller = await WindowController.fromCurrentEngine();
    return _parseWindowKindFromArguments(controller.arguments);
  } catch (e, st) {
    logWarning(
      e,
      stackTrace: st,
      message: 'Failed to resolve window kind',
      logger: 'window_routing',
    );
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
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to resolve kind in WindowRouter',
        logger: 'window_routing',
      );
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
          kind: WindowKind.debugger,
          title: l10n.menuDebugger,
          child: const SecondaryDebuggerContent(),
        );
      case WindowKind.tools:
        return SecondaryWindow(
          kind: WindowKind.tools,
          title: l10n.menuTools,
          child: const SecondaryToolsContent(),
        );
      case WindowKind.tilemap:
        return SecondaryWindow(
          kind: WindowKind.tilemap,
          title: l10n.menuTilemapViewer,
          child: const SecondaryTilemapContent(),
        );
      case WindowKind.tileViewer:
        return SecondaryWindow(
          kind: WindowKind.tileViewer,
          title: l10n.menuTileViewer,
          child: const SecondaryTileViewerContent(),
        );
      case WindowKind.spriteViewer:
        return SecondaryWindow(
          kind: WindowKind.spriteViewer,
          title: l10n.menuSpriteViewer,
          child: const SecondarySpriteViewerContent(),
        );
      case WindowKind.paletteViewer:
        return SecondaryWindow(
          kind: WindowKind.paletteViewer,
          title: l10n.menuPaletteViewer,
          child: const SecondaryPaletteViewerContent(),
        );
      case WindowKind.settings:
        return SecondaryWindow(
          kind: WindowKind.settings,
          title: l10n.settingsTitle,
          child: const SettingsPage(),
        );
    }
  }
}
