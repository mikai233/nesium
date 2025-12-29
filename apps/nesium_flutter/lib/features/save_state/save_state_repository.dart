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
      for (int i = 1; i <= 10; i++) {
        results[i] = null;
      }
      return results;
    }

    for (int i = 1; i <= 10; i++) {
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
