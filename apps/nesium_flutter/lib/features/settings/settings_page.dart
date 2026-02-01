import 'dart:async';

import 'package:flutter/material.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../l10n/app_localizations.dart';
import '../../platform/platform_capabilities.dart';
import '../controls/input_settings.dart';
import 'settings_utils.dart';
import 'video_settings.dart';

import 'tabs/general_tab.dart';
import 'tabs/input_tab.dart';
import 'tabs/video_tab.dart';
import 'tabs/emulation_tab.dart';
import 'tabs/server_tab.dart';
import '../../domain/nes_controller.dart';
import '../screen/floating_game_preview_state.dart';

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  StreamSubscription<InputCollision>? _collisionSubscription;
  bool _popInProgress = false;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: supportsTcp ? 5 : 4, vsync: this);

    _collisionSubscription = ref
        .read(inputSettingsProvider.notifier)
        .collisionStream
        .listen((collision) {
          if (!mounted) return;
          final l10n = AppLocalizations.of(context)!;
          final player = switch (collision.port) {
            0 => l10n.player1,
            1 => l10n.player2,
            2 => l10n.player3,
            3 => l10n.player4,
            _ => 'Player ${collision.port + 1}',
          };

          final action = SettingsUtils.actionLabel(l10n, collision.action);

          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(l10n.inputBindingConflictCleared(player, action)),
              duration: const Duration(seconds: 2),
            ),
          );
        });
  }

  @override
  void dispose() {
    _collisionSubscription?.cancel();
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _pickAndApplyCustomPalette(
    BuildContext context,
    VideoSettingsController controller,
  ) async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['pal'],
      withData: true,
      withReadStream: false,
    );
    final file = result?.files.single;
    if (file == null) return;

    final bytes = file.bytes;
    if (bytes == null) return;

    try {
      await controller.setCustomPalette(bytes, name: file.name);
    } catch (e) {
      if (!context.mounted) return;
      final l10n = AppLocalizations.of(context)!;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('${l10n.commandFailed(l10n.actionLoadPalette)}: $e'),
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final preview = ref.watch(floatingGamePreviewProvider);

    return PopScope(
      canPop: false,
      onPopInvokedWithResult: (didPop, result) async {
        if (didPop) return;
        if (_popInProgress) return;
        _popInProgress = true;

        if (preview.visible) {
          await ref.read(floatingGamePreviewProvider.notifier).hideAnimated();
        }

        if (context.mounted) {
          Navigator.of(context).pop(result);
        }
      },
      child: Scaffold(
        appBar: AppBar(
          title: Text(l10n.settingsTitle),
          bottom: TabBar(
            controller: _tabController,
            tabs: [
              Tab(
                icon: const Icon(Icons.public),
                text: l10n.settingsTabGeneral,
              ),
              Tab(
                icon: const Icon(Icons.videogame_asset),
                text: l10n.settingsTabInput,
              ),
              Tab(icon: const Icon(Icons.palette), text: l10n.settingsTabVideo),
              Tab(
                icon: const Icon(Icons.settings_applications),
                text: l10n.settingsTabEmulation,
              ),
              if (supportsTcp)
                Tab(
                  icon: const Icon(Icons.dns_rounded),
                  text: l10n.settingsTabServer,
                ),
            ],
          ),
          actions: [
            if (defaultTargetPlatform == TargetPlatform.android ||
                defaultTargetPlatform == TargetPlatform.iOS ||
                defaultTargetPlatform == TargetPlatform.linux)
              if (ref.watch(
                nesControllerProvider.select((s) => s.romHash != null),
              ))
                IconButton(
                  onPressed: () {
                    ref.read(floatingGamePreviewProvider.notifier).toggle();
                  },
                  icon: Icon(
                    preview.visible
                        ? Icons.fullscreen_exit
                        : Icons.picture_in_picture_alt,
                  ),
                  tooltip: l10n.settingsFloatingPreviewTooltip,
                ),
          ],
        ),
        body: TabBarView(
          controller: _tabController,
          children: [
            const GeneralTab(),
            const InputTab(),
            VideoTab(pickAndApplyCustomPalette: _pickAndApplyCustomPalette),
            const EmulationTab(),
            if (supportsTcp) const ServerTab(),
          ],
        ),
      ),
    );
  }
}
