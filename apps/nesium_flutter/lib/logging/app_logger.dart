import 'dart:async';
import 'dart:developer' as dev;

import 'package:flutter/foundation.dart';
import 'package:logging/logging.dart';

final Logger appLog = Logger('nesium_flutter');

void initLogging() {
  Logger.root.level = kDebugMode ? Level.ALL : Level.INFO;
  Logger.root.onRecord.listen((record) {
    final error = record.error;
    final stack = record.stackTrace;
    dev.log(
      record.message,
      time: record.time,
      name: record.loggerName,
      level: record.level.value,
      error: error,
      stackTrace: stack,
    );
  });
}

void logError(
  Object error, {
  StackTrace? stackTrace,
  String? message,
  String logger = 'nesium_flutter',
}) {
  Logger(logger).severe(message ?? error.toString(), error, stackTrace);
}

void logWarning(
  Object error, {
  StackTrace? stackTrace,
  String? message,
  String logger = 'nesium_flutter',
}) {
  Logger(logger).warning(message ?? error.toString(), error, stackTrace);
}

void logInfo(String message, {String logger = 'nesium_flutter'}) {
  Logger(logger).info(message);
}

void unawaitedLogged(
  Future<void> future, {
  String? message,
  String logger = 'nesium_flutter',
}) {
  unawaited(
    future.catchError((Object e, StackTrace st) {
      logError(e, stackTrace: st, message: message, logger: logger);
    }),
  );
}
