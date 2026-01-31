import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../features/screen/nes_screen_view.dart';
import '../features/settings/video_settings.dart';
import '../domain/nes_state.dart';
import '../domain/nes_controller.dart';
import '../features/save_state/save_state_repository.dart';
import '../platform/platform_capabilities.dart';
import 'nes_actions.dart';
import 'nes_menu_bar.dart';
import 'nes_menu_model.dart';

class DesktopShell extends ConsumerStatefulWidget {
  const DesktopShell({super.key, required this.state, required this.actions});

  final NesState state;
  final NesActions actions;

  @override
  ConsumerState<DesktopShell> createState() => _DesktopShellState();
}

class _DesktopShellState extends ConsumerState<DesktopShell> {
  bool _menuVisible = false;

  @override
  Widget build(BuildContext context) {
    final slotStates = ref.watch(saveStateRepositoryProvider);
    final hasRom = ref.watch(
      nesControllerProvider.select((s) => s.romHash != null),
    );

    final videoSettings = ref.watch(videoSettingsProvider);
    final screenVerticalOffset = videoSettings.screenVerticalOffset;
    final isFullScreen = videoSettings.fullScreen;
    final bool hideMenuBar = isFullScreen && !isLinux;

    const double menuHeight = 28;

    return Scaffold(
      body: MediaQuery.removePadding(
        context: context,
        removeLeft: true,
        removeRight: true,
        removeBottom: true,
        child: MouseRegion(
          onHover: (event) {
            if (!hideMenuBar) return;
            // Show menu if mouse is within top 40px
            final bool nearTop = event.localPosition.dy < 40;
            if (nearTop != _menuVisible) {
              setState(() => _menuVisible = nearTop);
            }
          },
          child: Stack(
            children: [
              // Main Content
              Positioned.fill(
                child: Padding(
                  padding: EdgeInsets.only(top: hideMenuBar ? 0 : menuHeight),
                  child: NesScreenView(
                    error: widget.state.error,
                    textureId: widget.state.textureId,
                    screenVerticalOffset: screenVerticalOffset,
                  ),
                ),
              ),

              // Menu Bar (Overlay)
              AnimatedPositioned(
                duration: const Duration(milliseconds: 200),
                curve: Curves.easeInOut,
                top: (hideMenuBar && !_menuVisible) ? -menuHeight : 0,
                left: 0,
                right: 0,
                height: menuHeight,
                child: MouseRegion(
                  onEnter: (_) {
                    if (hideMenuBar) setState(() => _menuVisible = true);
                  },
                  onExit: (_) {
                    if (hideMenuBar) setState(() => _menuVisible = false);
                  },
                  child: NesMenuBar(
                    actions: widget.actions,
                    sections: NesMenus.desktopMenuSections(),
                    slotStates: slotStates,
                    hasRom: hasRom,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
