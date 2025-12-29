import 'dart:async';

import 'package:flutter/material.dart';

import 'package:file_selector/file_selector.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/emulation.dart' as nes_emulation;
import 'package:path/path.dart' as p;

import '../../domain/nes_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../logging/app_logger.dart';
import '../../platform/platform_capabilities.dart';
import 'save_state_repository.dart';

class SaveStateDialog extends ConsumerWidget {
  const SaveStateDialog({
    super.key,
    required this.isSaving,
    this.isAutoSave = false,
  });

  final bool isSaving;
  final bool isAutoSave;

  static const int _numSlots = 10;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    String title = isSaving ? l10n.menuSaveState : l10n.menuLoadState;
    if (isAutoSave) {
      title = l10n.menuAutoSave;
    }
    final hasRom = ref.watch(
      nesControllerProvider.select((s) => s.romHash != null),
    );

    return AlertDialog(
      title: Text(title),
      content: SizedBox(
        width: 300,
        child: SingleChildScrollView(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              for (int i = 1; i <= _numSlots; i++)
                _buildSlotTile(
                  context,
                  ref,
                  isAutoSave ? i + 10 : i,
                  l10n,
                  enabled: hasRom,
                ),
              if (!isAutoSave) ...[
                const Divider(),
                ListTile(
                  enabled: hasRom,
                  leading: const Icon(Icons.folder_open),
                  title: Text(
                    isSaving
                        ? l10n.saveToExternalFile
                        : l10n.loadFromExternalFile,
                  ),
                  onTap: () => _handleExternalFile(context, ref, l10n),
                ),
              ],
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(l10n.cancel),
        ),
      ],
    );
  }

  Widget _buildSlotTile(
    BuildContext context,
    WidgetRef ref,
    int slotIndex,
    AppLocalizations l10n, {
    required bool enabled,
  }) {
    final slotMeta = ref.watch(saveStateRepositoryProvider);
    final timestamp = slotMeta[slotIndex];
    final hasData = timestamp != null;

    String subtitle;
    if (hasData) {
      subtitle = timestamp.toLocal().toString().split('.')[0];
    } else {
      subtitle = l10n.slotEmpty;
    }

    final bool isAutoSlot = slotIndex > 10;
    final int displayIndex = isAutoSlot ? slotIndex - 10 : slotIndex;
    final String labelPrefix = isAutoSlot ? l10n.autoSlotLabel : l10n.slotLabel;

    final bool canInteract = enabled && (isSaving || hasData);

    return ListTile(
      enabled: canInteract,
      leading: Icon(
        hasData
            ? (isAutoSlot ? Icons.history : Icons.save)
            : Icons.check_box_outline_blank,
      ),
      title: Text('$labelPrefix $displayIndex'),
      subtitle: Text(subtitle, style: Theme.of(context).textTheme.bodySmall),
      onTap: canInteract
          ? () => _handleSlot(context, ref, slotIndex, l10n)
          : null,
      trailing: hasData && isSaving && enabled && !isAutoSlot
          ? IconButton(
              icon: const Icon(Icons.delete),
              onPressed: () => _handleDeleteSlot(context, ref, slotIndex, l10n),
            )
          : null,
    );
  }

  Future<void> _handleSlot(
    BuildContext context,
    WidgetRef ref,
    int slotIndex,
    AppLocalizations l10n,
  ) async {
    final repository = ref.read(saveStateRepositoryProvider.notifier);

    try {
      if (isSaving) {
        final data = await nes_emulation.saveStateToMemory();
        await repository.saveState(slotIndex, data);
        if (context.mounted) {
          Navigator.of(context).pop();
          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(content: Text(l10n.stateSavedToSlot(slotIndex))),
          );
        }
      } else {
        if (!repository.hasSave(slotIndex)) {
          return;
        }
        final data = await repository.loadState(slotIndex);
        if (data != null) {
          await nes_emulation.loadStateFromMemory(data: data);
          if (context.mounted) {
            Navigator.of(context).pop();
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text(l10n.stateLoadedFromSlot(slotIndex))),
            );
          }
        }
      }
    } catch (e, st) {
      if (context.mounted) {
        Navigator.of(context).pop();
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              '${l10n.commandFailed(isSaving ? 'Save' : 'Load')}: $e',
            ),
          ),
        );
      }
      logError(
        e,
        stackTrace: st,
        message: 'Slot operation failed',
        logger: 'SaveStateDialog',
      );
    }
  }

  Future<void> _handleDeleteSlot(
    BuildContext context,
    WidgetRef ref,
    int slotIndex,
    AppLocalizations l10n,
  ) async {
    final repository = ref.read(saveStateRepositoryProvider.notifier);
    await repository.deleteState(slotIndex);
    if (context.mounted) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(l10n.slotCleared(slotIndex))));
    }
  }

  Future<void> _handleExternalFile(
    BuildContext context,
    WidgetRef ref,
    AppLocalizations l10n,
  ) async {
    Navigator.of(context).pop(); // Close dialog first

    const XTypeGroup typeGroup = XTypeGroup(
      label: 'Nesium State',
      extensions: <String>['nesium'],
    );

    try {
      if (isSaving) {
        String? path;
        final romName = ref.read(nesControllerProvider).romName ?? 'save';
        final suggestedName = '$romName.nesium';

        if (isNativeMobile) {
          // Android/iOS: getSaveLocation is not implemented.
          // Fallback: pick a directory and save with default name.
          final String? directoryPath = await getDirectoryPath(
            confirmButtonText: 'Save here',
          );
          if (directoryPath != null) {
            path = p.join(directoryPath, suggestedName);
          }
        } else {
          final FileSaveLocation? result = await getSaveLocation(
            acceptedTypeGroups: <XTypeGroup>[typeGroup],
            suggestedName: suggestedName,
          );
          path = result?.path;
        }

        if (path != null) {
          await nes_emulation.saveState(path: path);
          if (context.mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text(l10n.commandSucceeded('Save to file'))),
            );
          }
        }
      } else {
        final XFile? result = await openFile(
          acceptedTypeGroups: <XTypeGroup>[typeGroup],
        );

        if (result != null) {
          await nes_emulation.loadState(path: result.path);
          if (context.mounted) {
            ScaffoldMessenger.of(context).showSnackBar(
              SnackBar(content: Text(l10n.commandSucceeded('Load from file'))),
            );
          }
        }
      }
    } catch (e) {
      if (context.mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(
            content: Text(
              '${l10n.commandFailed(isSaving ? 'Save' : 'Load')}: $e',
            ),
          ),
        );
      }
    }
  }
}
