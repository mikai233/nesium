import 'dart:async';
import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../persistence/storage_codec.dart';
import '../../persistence/storage_key.dart';

final StorageKey<ThemeSettings> _themeSettingsKey = StorageKey(
  StorageKeys.settingsTheme,
  jsonModelStringCodec<ThemeSettings>(
    fromJson: ThemeSettings.fromJson,
    toJson: (value) => value.toJson(),
    storageKey: StorageKeys.settingsTheme,
  ),
);

/// Theme mode options
enum AppThemeMode {
  system,
  light,
  dark;

  String toJson() => name;

  static AppThemeMode fromJson(String json) {
    return AppThemeMode.values.firstWhere(
      (e) => e.name == json,
      orElse: () => AppThemeMode.system,
    );
  }
}

/// Theme settings state
class ThemeSettings {
  const ThemeSettings({this.mode = AppThemeMode.system});

  final AppThemeMode mode;

  ThemeSettings copyWith({AppThemeMode? mode}) {
    return ThemeSettings(mode: mode ?? this.mode);
  }

  ThemeMode get themeMode => switch (mode) {
    AppThemeMode.system => ThemeMode.system,
    AppThemeMode.light => ThemeMode.light,
    AppThemeMode.dark => ThemeMode.dark,
  };

  Map<String, dynamic> toJson() => {'mode': mode.toJson()};

  factory ThemeSettings.fromJson(Map<String, dynamic> json) {
    return ThemeSettings(
      mode: AppThemeMode.fromJson(json['mode'] as String? ?? 'system'),
    );
  }
}

/// Theme settings controller
class ThemeSettingsController extends Notifier<ThemeSettings> {
  StreamSubscription<void>? _subscription;

  @override
  ThemeSettings build() {
    // Listen for storage changes
    final storage = ref.read(appStorageProvider);
    _subscription = storage.onKeyChanged.listen((event) {
      if (event.key == StorageKeys.settingsTheme) {
        final value = event.value;
        if (value is String) {
          applySynced(ThemeSettings.fromJson(jsonDecode(value)));
        }
      }
    });
    ref.onDispose(() => _subscription?.cancel());

    try {
      final stored = storage.read(_themeSettingsKey);
      if (stored != null) {
        return stored;
      }
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to load theme settings',
        logger: 'theme_settings',
      );
    }
    return const ThemeSettings();
  }

  Future<void> setThemeMode(AppThemeMode mode) async {
    if (mode == state.mode) return;
    state = state.copyWith(mode: mode);
    await _persist();
  }

  void applySynced(ThemeSettings next) {
    if (next.mode == state.mode) return;
    state = next;
  }

  Future<void> _persist() async {
    unawaitedLogged(
      Future<void>.sync(
        () => ref.read(appStorageProvider).write(_themeSettingsKey, state),
      ),
      message: 'Persist theme settings',
      logger: 'theme_settings',
    );
  }
}

/// Provider for theme settings
final themeSettingsProvider =
    NotifierProvider<ThemeSettingsController, ThemeSettings>(
      ThemeSettingsController.new,
    );
