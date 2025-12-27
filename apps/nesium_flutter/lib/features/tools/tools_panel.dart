import 'package:flutter/material.dart';

import '../common/panel_placeholder.dart';
import '../../l10n/app_localizations.dart';

class ToolsPanel extends StatelessWidget {
  const ToolsPanel({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return PanelPlaceholder(
      title: l10n.menuTools,
      body: l10n.toolsPlaceholderBody,
    );
  }
}
