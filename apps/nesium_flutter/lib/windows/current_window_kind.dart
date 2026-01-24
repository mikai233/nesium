import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'window_types.dart';

/// The kind of the current window (main/settings/debugger/...).
///
/// Desktop multi-window creates a separate Flutter engine per window, so each
/// engine needs to know its own role. `main.dart` overrides this provider based
/// on startup arguments.
final currentWindowKindProvider = Provider<WindowKind>(
  (ref) => WindowKind.main,
);
