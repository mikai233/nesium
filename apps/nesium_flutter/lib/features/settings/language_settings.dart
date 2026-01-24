import 'package:flutter/material.dart';
import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../platform/platform_capabilities.dart';
import '../../windows/settings_sync.dart';
import 'package:desktop_multi_window/desktop_multi_window.dart';

enum AppLanguage { system, english, chineseSimplified }

extension AppLanguageX on AppLanguage {
  Locale? get locale => switch (this) {
    AppLanguage.system => null,
    AppLanguage.english => const Locale('en'),
    AppLanguage.chineseSimplified => const Locale('zh'),
  };

  String? get languageCode => switch (this) {
    AppLanguage.system => null,
    AppLanguage.english => 'en',
    AppLanguage.chineseSimplified => 'zh',
  };

  static AppLanguage fromLanguageCode(String? code) => switch (code) {
    'en' => AppLanguage.english,
    'zh' => AppLanguage.chineseSimplified,
    _ => AppLanguage.system,
  };
}

class LanguageSettingsController extends Notifier<AppLanguage> {
  @override
  AppLanguage build() {
    final stored = ref
        .read(appStorageProvider)
        .get<String>(StorageKeys.settingsLanguage);
    if (stored != null) {
      try {
        return AppLanguage.values.byName(stored);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'Failed to lookup language by name: $stored',
          logger: 'language_settings',
        );
      }
    }
    return AppLanguage.system;
  }

  Future<void> setLanguage(AppLanguage language) async {
    if (language == state) return;
    state = language;
    _persist(language);

    if (isNativeDesktop) {
      unawaited(
        SettingsSync.broadcast(group: 'language', payload: language.name),
      );
      final windows = await WindowController.getAll();
      for (final window in windows) {
        unawaited(
          window.invokeMethod<void>('setLanguage', language.languageCode),
        );
      }
    }
  }

  void applyIncomingLanguageFromWindow(String? code) {
    final language = AppLanguageX.fromLanguageCode(code);
    if (language == state) return;
    state = language;
  }

  void applySynced(String name) {
    try {
      final language = AppLanguage.values.byName(name);
      if (language == state) return;
      state = language;
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to apply synced language: $name',
        logger: 'language_settings',
      );
    }
  }

  void _persist(AppLanguage language) {
    unawaitedLogged(
      Future<void>.sync(
        () => ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsLanguage, language.name),
      ),
      message: 'Persist language',
      logger: 'language_settings',
    );
  }
}

final appLanguageProvider =
    NotifierProvider<LanguageSettingsController, AppLanguage>(
      LanguageSettingsController.new,
    );
