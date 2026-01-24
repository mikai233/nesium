import 'dart:convert';

import '../logging/app_logger.dart';

typedef JsonMap = Map<String, dynamic>;

JsonMap? coerceJsonMap(Object? raw, {String? storageKey}) {
  if (raw == null) return null;

  if (raw is! String) return null;
  try {
    final decoded = jsonDecode(raw);
    if (decoded is Map) {
      return Map<String, dynamic>.from(decoded);
    }
  } catch (e, st) {
    logWarning(
      e,
      stackTrace: st,
      message:
          'Failed to decode JSON map from storage'
          '${storageKey == null ? '' : ' (key=$storageKey)'}: '
          'len=${raw.length}, prefix=${_truncate(raw, 120)}',
      logger: 'storage_codec',
    );
  }
  return null;
}

final class StorageCodec<T> {
  const StorageCodec({required this.encode, required this.decode});

  final Object? Function(T value) encode;
  final T? Function(Object? raw) decode;
}

StorageCodec<JsonMap> jsonMapStringCodec({
  JsonMap? fallback,
  String? storageKey,
}) => StorageCodec<JsonMap>(
  encode: jsonEncode,
  decode: (raw) => coerceJsonMap(raw, storageKey: storageKey) ?? fallback,
);

StorageCodec<T> jsonModelStringCodec<T>({
  required T Function(JsonMap json) fromJson,
  required JsonMap Function(T value) toJson,
  String? storageKey,
}) {
  return StorageCodec<T>(
    encode: (value) => jsonEncode(toJson(value)),
    decode: (raw) {
      final map = coerceJsonMap(raw, storageKey: storageKey);
      if (map == null) return null;
      return fromJson(map);
    },
  );
}

String _truncate(String s, int max) {
  if (s.length <= max) return s;
  return '${s.substring(0, max)}…';
}
