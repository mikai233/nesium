import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'nes_controller.dart';

/// Polls the Rust runtime for current ROM hash and keeps `nesControllerProvider`
/// in sync across multi-window engines.
///
/// Desktop multi-window creates separate Flutter engines, so Riverpod state is
/// not shared. The runtime itself is shared (Rust static), so polling is a
/// simple way to synchronize window UI state.
final romHashSyncProvider = Provider<void>((ref) {
  if (kIsWeb) return;

  final controller = ref.read(nesControllerProvider.notifier);

  Future<void> tick() async {
    await controller.refreshRomHash();
  }

  unawaited(tick());
  final timer = Timer.periodic(const Duration(seconds: 1), (_) {
    unawaited(tick());
  });
  ref.onDispose(timer.cancel);
});
