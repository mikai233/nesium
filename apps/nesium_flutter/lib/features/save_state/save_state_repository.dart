import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_controller.dart';
import '../../persistence/app_storage.dart';

/// Manages save state data and metadata, providing reactive state for slot status.
/// Slots are scoped to the currently loaded ROM.
class SaveStateRepository extends Notifier<Map<int, DateTime?>> {
  @override
  Map<int, DateTime?> build() {
    final storage = ref.watch(appStorageProvider);
    final romHash = ref.watch(nesControllerProvider.select((s) => s.romHash));

    final results = <int, DateTime?>{};
    if (romHash == null) {
      for (int i = 1; i <= 20; i++) {
        // 1-10: manual, 11-20: auto
        results[i] = null;
      }
      return results;
    }

    for (int i = 1; i <= 20; i++) {
      final meta = storage.get(_metaKey(romHash, i));
      if (meta is int) {
        results[i] = DateTime.fromMillisecondsSinceEpoch(meta);
      } else {
        results[i] = null;
      }
    }
    return results;
  }

  static String _dataKey(String romHash, int index) =>
      'save_state_${romHash}_slot_$index';
  static String _metaKey(String romHash, int index) =>
      'save_state_${romHash}_meta_$index';

  bool hasSave(int index) => state[index] != null;

  DateTime? getTimestamp(int index) => state[index];

  Future<void> saveState(int index, Uint8List data) async {
    final romHash = ref.read(nesControllerProvider).romHash;
    if (romHash == null) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(_dataKey(romHash, index), data);
    final now = DateTime.now();
    await storage.put(_metaKey(romHash, index), now.millisecondsSinceEpoch);

    state = {...state, index: now};
  }

  Future<void> performAutoSave(Uint8List data) async {
    final romHash = ref.read(nesControllerProvider).romHash;
    if (romHash == null) return;

    final storage = ref.read(appStorageProvider);

    // Slots 11-20 are reserved for auto-save.
    // Rotate slots: shift all existing saves down by one (11->12, 12->13, ..., 19->20).
    // Slot 20 is discarded, and slot 11 receives the new save.
    // This ensures the newest save is always in slot 11.

    // Shift existing slots from 19 down to 11 (move 19->20, 18->19, ..., 11->12)
    for (int i = 19; i >= 11; i--) {
      final srcDataKey = _dataKey(romHash, i);
      final srcMetaKey = _metaKey(romHash, i);
      final dstDataKey = _dataKey(romHash, i + 1);
      final dstMetaKey = _metaKey(romHash, i + 1);

      final srcData = storage.get(srcDataKey);
      final srcMeta = storage.get(srcMetaKey);

      if (srcData != null && srcMeta != null) {
        // Move data from slot i to slot i+1
        await storage.put(dstDataKey, srcData);
        await storage.put(dstMetaKey, srcMeta);
      } else {
        // Clear destination slot if source is empty
        await storage.delete(dstDataKey);
        await storage.delete(dstMetaKey);
      }
    }

    // Save new data to slot 11
    await storage.put(_dataKey(romHash, 11), data);
    final now = DateTime.now();
    await storage.put(_metaKey(romHash, 11), now.millisecondsSinceEpoch);

    // Rebuild state for slots 11-20
    final newState = <int, DateTime?>{...state};
    for (int i = 11; i <= 20; i++) {
      final meta = storage.get(_metaKey(romHash, i));
      if (meta is int) {
        newState[i] = DateTime.fromMillisecondsSinceEpoch(meta);
      } else {
        newState[i] = null;
      }
    }
    state = newState;
  }

  Future<Uint8List?> loadState(int index) async {
    final romHash = ref.read(nesControllerProvider).romHash;
    if (romHash == null) return null;

    final storage = ref.read(appStorageProvider);
    final data = storage.get(_dataKey(romHash, index));
    if (data is Uint8List) {
      return data;
    } else if (data is List<int>) {
      return Uint8List.fromList(data);
    }
    return null;
  }

  Future<void> deleteState(int index) async {
    final romHash = ref.read(nesControllerProvider).romHash;
    if (romHash == null) return;

    final storage = ref.read(appStorageProvider);
    await storage.delete(_dataKey(romHash, index));
    await storage.delete(_metaKey(romHash, index));

    state = {...state, index: null};
  }
}

final saveStateRepositoryProvider =
    NotifierProvider<SaveStateRepository, Map<int, DateTime?>>(
      SaveStateRepository.new,
    );
