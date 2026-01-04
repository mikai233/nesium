import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;

import '../logging/app_logger.dart';
import '../shell/nes_actions.dart';
import '../domain/nes_texture_service.dart';
import '../platform/window_manager_shim.dart';
import '../platform/desktop_window_manager.dart';

import '../features/debugger/debugger_panel.dart';
import '../features/debugger/tile_viewer.dart';
import '../features/debugger/tilemap_viewer.dart';
import '../features/debugger/sprite_viewer.dart';
import '../features/tools/tools_panel.dart';
import '../platform/platform_capabilities.dart';
import 'window_types.dart';

class SecondaryWindow extends StatefulWidget {
  const SecondaryWindow({
    super.key,
    required this.kind,
    required this.child,
    required this.title,
  });

  final WindowKind kind;
  final Widget child;
  final String title;

  @override
  State<SecondaryWindow> createState() => _SecondaryWindowState();
}

class _SecondaryWindowState extends State<SecondaryWindow> with WindowListener {
  final NesTextureService _textureService = NesTextureService();
  bool _didHandleClose = false;

  @override
  void initState() {
    super.initState();
    if (isNativeDesktop) {
      _setNativeTitle();
      windowManager.addListener(this);
    }
  }

  Future<void> _cleanupOnWindowClose() async {
    if (_didHandleClose) return;
    _didHandleClose = true;

    Future<void> bestEffort(Future<void> Function() fn) async {
      try {
        await fn();
      } catch (_) {}
    }

    switch (widget.kind) {
      case WindowKind.tilemap:
        await bestEffort(bridge.unsubscribeTilemapTexture);
        await bestEffort(() => _textureService.pauseAuxTexture(1));
        await bestEffort(() => _textureService.disposeAuxTexture(1));
        break;
      case WindowKind.tileViewer:
        await bestEffort(bridge.unsubscribeChrState);
        await bestEffort(() => _textureService.pauseAuxTexture(2));
        await bestEffort(() => _textureService.disposeAuxTexture(2));
        break;
      case WindowKind.spriteViewer:
        await bestEffort(bridge.unsubscribeSpriteState);
        await bestEffort(() => _textureService.pauseAuxTexture(3));
        await bestEffort(() => _textureService.pauseAuxTexture(4));
        await bestEffort(() => _textureService.disposeAuxTexture(3));
        await bestEffort(() => _textureService.disposeAuxTexture(4));
        break;
      case WindowKind.debugger:
        await bestEffort(bridge.unsubscribeDebugState);
        break;
      case WindowKind.tools:
      case WindowKind.main:
        break;
    }
  }

  Future<void> _setNativeTitle() async {
    unawaitedLogged(
      windowManager.setTitle(widget.title),
      logger: 'secondary_window',
      message: 'Failed to set window title',
    );
  }

  @override
  void didUpdateWidget(covariant SecondaryWindow oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (!isNativeDesktop) return;
    if (oldWidget.title != widget.title) {
      _setNativeTitle();
    }
  }

  @override
  void onWindowClose() {
    appLog.info('SecondaryWindow onWindowClose kind=${widget.kind}');
    unawaitedLogged(
      _cleanupOnWindowClose(),
      logger: 'secondary_window',
      message: 'Failed to cleanup on window close',
    );
  }

  @override
  void dispose() {
    if (isNativeDesktop) {
      windowManager.removeListener(this);
    }
    super.dispose();
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
    return ProviderScope(
      overrides: [
        nesActionsProvider.overrideWithValue(
          NesActions(
            openTilemapViewer: () async {
              await DesktopWindowManager().openTilemapWindow();
            },
            openTileViewer: () async {
              await DesktopWindowManager().openTileViewerWindow();
            },
            openSpriteViewer: () async {
              await DesktopWindowManager().openSpriteViewerWindow();
            },
          ),
        ),
      ],
      child: const ToolsPanel(),
    );
  }
}

class SecondaryTilemapContent extends StatelessWidget {
  const SecondaryTilemapContent({super.key});

  @override
  Widget build(BuildContext context) {
    return const TilemapViewer();
  }
}

class SecondaryTileViewerContent extends StatelessWidget {
  const SecondaryTileViewerContent({super.key});

  @override
  Widget build(BuildContext context) {
    return const TileViewer();
  }
}

class SecondarySpriteViewerContent extends StatelessWidget {
  const SecondarySpriteViewerContent({super.key});

  @override
  Widget build(BuildContext context) {
    return const SpriteViewer();
  }
}
