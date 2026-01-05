import 'package:nesium_flutter/bridge/api/events.dart' as bridge;

/// Single source of truth for auxiliary texture IDs (defined in Rust).
class AuxTextureIdsCache {
  static Future<bridge.AuxTextureIds>? _future;

  static Future<bridge.AuxTextureIds> get() {
    return _future ??= bridge.auxTextureIds();
  }

  static void reset() {
    _future = null;
  }
}
