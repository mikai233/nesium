import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../l10n/app_localizations.dart';
import '../../shader_parameter_provider.dart';
import 'shader_parameters_page.dart';

class ShaderSettingsCard extends ConsumerWidget {
  const ShaderSettingsCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final paramsAsync = ref.watch(shaderParametersProvider);

    if (!paramsAsync.hasValue || (paramsAsync.value?.isEmpty ?? true)) {
      return const SizedBox.shrink();
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const SizedBox(height: 1),
        ListTile(
          contentPadding: const EdgeInsets.symmetric(horizontal: 12),
          leading: const Icon(Icons.tune),
          title: Text(l10n.videoShaderParametersTitle),
          subtitle: Text(l10n.videoShaderParametersSubtitle),
          trailing: const Icon(Icons.navigate_next),
          onTap: () {
            Navigator.of(context).push(
              MaterialPageRoute(
                builder: (context) => const ShaderParametersPage(),
              ),
            );
          },
        ),
        const SizedBox(height: 12),
      ],
    );
  }
}
