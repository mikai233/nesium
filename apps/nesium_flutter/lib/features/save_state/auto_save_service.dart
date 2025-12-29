import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/emulation.dart' as nes_emulation;

import '../../domain/nes_controller.dart';
import '../settings/emulation_settings.dart';
import 'save_state_repository.dart';

/// Service that handles automatic saving of game state.
class AutoSaveService {
  AutoSaveService(this.ref) {
    _startTimer();
  }

  final Ref ref;
  Timer? _timer;

  void _startTimer() {
    _timer?.cancel();
    _timer = Timer.periodic(const Duration(seconds: 30), (timer) {
      _checkAndPerformSave();
    });
  }

  String? _lastRomHash;
  DateTime? _lastSaveTime;

  Future<void> _checkAndPerformSave() async {
    final settings = ref.read(emulationSettingsProvider);
    if (!settings.autoSaveEnabled) return;

    final romHash = ref.read(nesControllerProvider).romHash;
    if (romHash == null) {
      _lastRomHash = null;
      _lastSaveTime = null;
      return;
    }

    // Reset timer if we switched games
    if (romHash != _lastRomHash) {
      _lastRomHash = romHash;
      _lastSaveTime = DateTime.now();
      return;
    }

    final interval = Duration(minutes: settings.autoSaveIntervalInMinutes);
    final now = DateTime.now();

    if (_lastSaveTime == null || now.difference(_lastSaveTime!) >= interval) {
      try {
        final data = await nes_emulation.saveStateToMemory();
        await ref
            .read(saveStateRepositoryProvider.notifier)
            .performAutoSave(data);
        _lastSaveTime = now;
      } catch (_) {
        // Silent failure for background auto-save
      }
    }
  }

  void dispose() {
    _timer?.cancel();
  }
}

final autoSaveServiceProvider = Provider<AutoSaveService>((ref) {
  final service = AutoSaveService(ref);
  ref.onDispose(service.dispose);
  return service;
});
