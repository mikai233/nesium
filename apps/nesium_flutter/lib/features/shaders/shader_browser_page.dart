import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'shader_browser_controller.dart';
import '../settings/android_shader_settings.dart';

class ShaderBrowserPage extends ConsumerWidget {
  const ShaderBrowserPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(shaderBrowserProvider);
    final controller = ref.read(shaderBrowserProvider.notifier);

    return Scaffold(
      appBar: AppBar(
        title: Text(controller.currentPath ?? 'Shaders'),
        leading: controller.canGoBack
            ? IconButton(
                icon: const Icon(Icons.arrow_back),
                onPressed: controller.goBack,
              )
            : null,
      ),
      body: state.when(
        data: (nodes) {
          if (nodes.isEmpty) {
            return const Center(child: Text('No shaders found'));
          }
          return ListView.builder(
            itemCount: nodes.length,
            itemBuilder: (context, index) {
              final node = nodes[index];
              return ListTile(
                leading: Icon(node.isDirectory ? Icons.folder : Icons.brush),
                title: Text(node.name),
                onTap: () {
                  if (node.isDirectory) {
                    controller.enterDirectory(node.path);
                  } else {
                    ref
                        .read(androidShaderSettingsProvider.notifier)
                        .setPresetPath(node.path);
                    Navigator.of(context).pop();
                  }
                },
              );
            },
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, st) => Center(child: Text('Error: $e')),
      ),
    );
  }
}
