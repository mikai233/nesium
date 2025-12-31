import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../bridge/api/events.dart';

/// Stream provider for debug state updates.
///
/// Subscribes when listened and unsubscribes when disposed.
final debugStateProvider = StreamProvider.autoDispose<DebugStateNotification>((
  ref,
) {
  // When disposed, unsubscribe to stop unnecessary computation
  ref.onDispose(() {
    unsubscribeDebugState();
  });

  return debugStateStream();
});
