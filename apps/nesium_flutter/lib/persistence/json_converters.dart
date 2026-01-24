import 'package:flutter/services.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

class LogicalKeyboardKeyConverter
    implements JsonConverter<LogicalKeyboardKey, int> {
  const LogicalKeyboardKeyConverter();

  @override
  LogicalKeyboardKey fromJson(int json) => LogicalKeyboardKey(json);

  @override
  int toJson(LogicalKeyboardKey object) => object.keyId;
}

class LogicalKeyboardKeyNullableConverter
    implements JsonConverter<LogicalKeyboardKey?, int?> {
  const LogicalKeyboardKeyNullableConverter();

  @override
  LogicalKeyboardKey? fromJson(int? json) =>
      json != null ? LogicalKeyboardKey(json) : null;

  @override
  int? toJson(LogicalKeyboardKey? object) => object?.keyId;
}
