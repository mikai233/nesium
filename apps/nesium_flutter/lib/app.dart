import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'domain/rom_hash_sync_provider.dart';
import 'features/settings/language_settings.dart';
import 'l10n/app_localizations.dart';
import 'windows/window_routing.dart';

class NesiumApp extends ConsumerWidget {
  const NesiumApp({super.key, required this.windowKind});

  final WindowKind windowKind;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final language = ref.watch(appLanguageProvider);
    ref.watch(romHashSyncProvider);
    return MaterialApp(
      onGenerateTitle: (context) {
        final l10n = AppLocalizations.of(context);
        if (l10n == null) return 'Nesium';

        switch (windowKind) {
          case WindowKind.debugger:
            return l10n.menuDebugger;
          case WindowKind.tools:
            return l10n.menuTools;
          case WindowKind.tilemap:
            return l10n.menuTilemapViewer;
          case WindowKind.tileViewer:
            return l10n.menuTileViewer;
          case WindowKind.spriteViewer:
            return l10n.menuSpriteViewer;
          case WindowKind.paletteViewer:
            return l10n.menuPaletteViewer;
          case WindowKind.main:
            return l10n.appName;
        }
      },
      locale: language.locale,
      localizationsDelegates: AppLocalizations.localizationsDelegates,
      supportedLocales: AppLocalizations.supportedLocales,
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blueGrey,
          brightness: Brightness.light,
        ).copyWith(surface: Colors.white),
        scaffoldBackgroundColor: Colors.white,
        useMaterial3: true,
        visualDensity: VisualDensity.adaptivePlatformDensity,
      ),
      home: const WindowRouter(),
    );
  }
}
