import 'dart:async';
import 'dart:convert';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../platform/platform_capabilities.dart';

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
  bool _initialized = false;
  bool _suppressBroadcast = false;
  WindowController? _windowController;

  @override
  AppLanguage build() {
    if (!_initialized) {
      _initialized = true;
      scheduleMicrotask(_init);
    }
    return AppLanguage.system;
  }

  void setLanguage(AppLanguage language) {
    if (language == state) return;
    state = language;
    unawaited(_broadcastLanguage(language));
  }

  bool get _supportsWindowMessaging => isNativeDesktop;

  Future<void> _init() async {
    if (!_supportsWindowMessaging) return;

    try {
      final controller = await WindowController.fromCurrentEngine();
      _windowController = controller;

      final args = controller.arguments;
      if (args.isNotEmpty) {
        try {
          final decoded = jsonDecode(args);
          if (decoded is Map && decoded['lang'] is String) {
            _applyIncomingLanguage(decoded['lang'] as String);
          }
        } catch (_) {}
      }

      await controller.setWindowMethodHandler((call) async {
        if (call.method != 'setLanguage') return null;
        final arg = call.arguments;
        if (arg == null) {
          _applyIncomingLanguage(null);
          return null;
        }
        if (arg is String) {
          _applyIncomingLanguage(arg);
        }
        return null;
      });
    } catch (_) {}
  }

  void _applyIncomingLanguage(String? languageCode) {
    final next = AppLanguageX.fromLanguageCode(languageCode);
    if (next == state) return;
    _suppressBroadcast = true;
    state = next;
    _suppressBroadcast = false;
  }

  Future<void> _broadcastLanguage(AppLanguage language) async {
    if (_suppressBroadcast) return;
    if (!_supportsWindowMessaging) return;

    try {
      final currentId = _windowController?.windowId;
      final windows = await WindowController.getAll();
      for (final window in windows) {
        if (currentId != null && window.windowId == currentId) continue;
        unawaited(
          window.invokeMethod<void>('setLanguage', language.languageCode),
        );
      }
    } catch (_) {}
  }
}

final appLanguageProvider =
    NotifierProvider<LanguageSettingsController, AppLanguage>(
      LanguageSettingsController.new,
    );
