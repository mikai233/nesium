import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:url_launcher/url_launcher.dart';

import '../../l10n/app_localizations.dart';

class AboutPage extends StatefulWidget {
  const AboutPage({super.key});

  static const String repoUrl = 'https://github.com/mikai233/nesium';
  static const String webDemoUrl = 'https://mikai233.github.io/nesium/';

  @override
  State<AboutPage> createState() => _AboutPageState();
}

class _AboutPageState extends State<AboutPage>
    with SingleTickerProviderStateMixin {
  late final AnimationController _controller;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      vsync: this,
      duration: const Duration(milliseconds: 800),
    );
    _controller.forward();
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    // Helper to build animated items with staggering
    Widget animate(int index, Widget child) {
      // Stagger delay per item
      const stagger = 0.05;
      // Duration of each item's animation (relative to total duration)
      const duration = 0.5;

      final start = (index * stagger).clamp(0.0, 1.0 - duration);
      final end = start + duration;

      final animation = CurvedAnimation(
        parent: _controller,
        curve: Interval(start, end, curve: Curves.easeOutQuart),
      );

      return FadeTransition(
        opacity: animation,
        child: SlideTransition(
          position: Tween<Offset>(
            begin: const Offset(0.1, 0),
            end: Offset.zero,
          ).animate(animation),
          child: child,
        ),
      );
    }

    final children = [
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
        value: AboutPage.repoUrl,
        copiedLabel: l10n.lastErrorCopied,
      ),
      _LinkTile(
        label: l10n.aboutWebDemoLabel,
        value: AboutPage.webDemoUrl,
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
      const _ComponentTile(
        name: 'Gilrs',
        url: 'https://gitlab.com/gilrs-project/gilrs',
      ),
      const _ComponentTile(name: 'Hive', url: 'https://github.com/hivedb/hive'),
      const _ComponentTile(name: 'Tokio', url: 'https://tokio.rs'),
      const _ComponentTile(
        name: 'file_picker',
        url: 'https://github.com/miguelpruivo/flutter_file_picker',
      ),
      const SizedBox(height: 16),
      Text(
        l10n.aboutLicenseHeading,
        style: Theme.of(context).textTheme.titleMedium,
      ),
      const SizedBox(height: 8),
      Text(l10n.aboutLicenseBody),
    ];

    return Scaffold(
      appBar: AppBar(title: Text(l10n.aboutTitle)),
      body: ListView.builder(
        padding: const EdgeInsets.all(16),
        itemCount: children.length,
        itemBuilder: (context, index) {
          return animate(index, children[index]);
        },
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

  Future<void> _launch(BuildContext context) async {
    final uri = Uri.parse(value);
    if (await canLaunchUrl(uri)) {
      await launchUrl(uri, mode: LaunchMode.externalApplication);
    } else {
      if (context.mounted) {
        final l10n = AppLocalizations.of(context)!;
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text(l10n.aboutLaunchFailed(value))));
      }
    }
  }

  Future<void> _copy(BuildContext context) async {
    await Clipboard.setData(ClipboardData(text: value));
    if (!context.mounted) return;
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(copiedLabel)));
  }

  @override
  Widget build(BuildContext context) {
    return ListTile(
      contentPadding: EdgeInsets.zero,
      title: Text(label),
      subtitle: MouseRegion(
        cursor: SystemMouseCursors.click,
        child: GestureDetector(
          onTap: () => _launch(context),
          onLongPress: () => _copy(context),
          child: Text(
            value,
            style: TextStyle(
              color: Colors.blue[700],
              decoration: TextDecoration.underline,
              decorationColor: Colors.blue[700],
            ),
          ),
        ),
      ),
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
