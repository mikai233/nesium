import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../l10n/app_localizations.dart';
import '../../../widgets/animated_dropdown_menu.dart';
import '../../../widgets/animated_settings_widgets.dart';
import '../language_settings.dart';
import '../theme_settings.dart';

class GeneralTab extends ConsumerWidget {
  const GeneralTab({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final language = ref.watch(appLanguageProvider);
    final languageController = ref.read(appLanguageProvider.notifier);
    final themeSettings = ref.watch(themeSettingsProvider);
    final themeController = ref.read(themeSettingsProvider.notifier);

    return ListView(
      padding: const EdgeInsets.all(20),
      children: [
        AnimatedSectionHeader(
          title: l10n.generalTitle,
          icon: Icons.settings,
          delay: const Duration(milliseconds: 50),
        ),
        AnimatedSettingsCard(
          index: 0,
          child: Column(
            children: [
              ListTile(
                title: Text(l10n.languageLabel),
                subtitle: Text(
                  switch (language) {
                    AppLanguage.system => l10n.languageSystem,
                    AppLanguage.english => l10n.languageEnglish,
                    AppLanguage.chineseSimplified =>
                      l10n.languageChineseSimplified,
                  },
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.primary,
                  ),
                ),
                trailing: SizedBox(
                  width: 180,
                  child: AnimatedDropdownMenu<AppLanguage>(
                    density: AnimatedDropdownMenuDensity.compact,
                    value: language,
                    entries: [
                      DropdownMenuEntry(
                        value: AppLanguage.system,
                        label: l10n.languageSystem,
                      ),
                      DropdownMenuEntry(
                        value: AppLanguage.english,
                        label: l10n.languageEnglish,
                      ),
                      DropdownMenuEntry(
                        value: AppLanguage.chineseSimplified,
                        label: l10n.languageChineseSimplified,
                      ),
                    ],
                    onSelected: (value) {
                      languageController.setLanguage(value);
                    },
                  ),
                ),
              ),
              const Divider(height: 1),
              ListTile(
                title: Text(l10n.themeLabel),
                subtitle: Text(
                  switch (themeSettings.mode) {
                    AppThemeMode.system => l10n.themeSystem,
                    AppThemeMode.light => l10n.themeLight,
                    AppThemeMode.dark => l10n.themeDark,
                  },
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.primary,
                  ),
                ),
                trailing: SizedBox(
                  width: 180,
                  child: AnimatedDropdownMenu<AppThemeMode>(
                    density: AnimatedDropdownMenuDensity.compact,
                    value: themeSettings.mode,
                    entries: [
                      DropdownMenuEntry(
                        value: AppThemeMode.system,
                        label: l10n.themeSystem,
                      ),
                      DropdownMenuEntry(
                        value: AppThemeMode.light,
                        label: l10n.themeLight,
                      ),
                      DropdownMenuEntry(
                        value: AppThemeMode.dark,
                        label: l10n.themeDark,
                      ),
                    ],
                    onSelected: themeController.setThemeMode,
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}
