import 'package:flutter/material.dart';

import '../common/panel_placeholder.dart';
import '../../l10n/app_localizations.dart';

class DebuggerPanel extends StatelessWidget {
  const DebuggerPanel({super.key});

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return PanelPlaceholder(
      title: l10n.menuDebugger,
      body: l10n.debuggerPlaceholderBody,
    );
  }
}
