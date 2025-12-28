import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../../l10n/app_localizations.dart';

class AboutPage extends StatelessWidget {
  const AboutPage({super.key});

  static const String repoUrl = 'https://github.com/mikai233/nesium';
  static const String webDemoUrl = 'https://mikai233.github.io/nesium/';

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Scaffold(
      appBar: AppBar(title: Text(l10n.aboutTitle)),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(l10n.aboutLead, style: Theme.of(context).textTheme.titleMedium),
          const SizedBox(height: 8),
          Text(l10n.aboutIntro),
          const SizedBox(height: 16),
          Text(
            l10n.aboutLinksHeading,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          _LinkTile(
            label: l10n.aboutGitHubLabel,
            value: repoUrl,
            copiedLabel: l10n.lastErrorCopied,
          ),
          _LinkTile(
            label: l10n.aboutWebDemoLabel,
            value: webDemoUrl,
            copiedLabel: l10n.lastErrorCopied,
          ),
          const SizedBox(height: 16),
          Text(
            l10n.aboutComponentsHeading,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 4),
          Text(
            l10n.aboutComponentsHint,
            style: Theme.of(context).textTheme.bodySmall?.copyWith(
              color: Theme.of(
                context,
              ).colorScheme.onSurface.withValues(alpha: 0.75),
            ),
          ),
          const SizedBox(height: 8),
          const _ComponentTile(name: 'Flutter', url: 'https://flutter.dev'),
          const _ComponentTile(
            name: 'flutter_rust_bridge',
            url: 'https://github.com/fzyzcjy/flutter_rust_bridge',
          ),
          const _ComponentTile(name: 'Riverpod', url: 'https://riverpod.dev'),
          const _ComponentTile(
            name: 'wasm-pack',
            url: 'https://github.com/rustwasm/wasm-pack',
          ),
          const SizedBox(height: 16),
          Text(
            l10n.aboutLicenseHeading,
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          Text(l10n.aboutLicenseBody),
        ],
      ),
    );
  }
}

class _LinkTile extends StatelessWidget {
  const _LinkTile({
    required this.label,
    required this.value,
    required this.copiedLabel,
  });

  final String label;
  final String value;
  final String copiedLabel;

  Future<void> _copy(BuildContext context) async {
    await Clipboard.setData(ClipboardData(text: value));
    if (!context.mounted) return;
    ScaffoldMessenger.of(context).showSnackBar(
      SnackBar(
        content: Text(copiedLabel),
        behavior: SnackBarBehavior.floating,
        duration: const Duration(seconds: 2),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    return ListTile(
      contentPadding: EdgeInsets.zero,
      title: Text(label),
      subtitle: SelectableText(value),
      trailing: IconButton(
        tooltip: MaterialLocalizations.of(context).copyButtonLabel,
        onPressed: () => _copy(context),
        icon: const Icon(Icons.copy),
      ),
      onTap: () => _copy(context),
    );
  }
}

class _ComponentTile extends StatelessWidget {
  const _ComponentTile({required this.name, required this.url});

  final String name;
  final String url;

  @override
  Widget build(BuildContext context) {
    return _LinkTile(
      label: name,
      value: url,
      copiedLabel: AppLocalizations.of(context)!.lastErrorCopied,
    );
  }
}
