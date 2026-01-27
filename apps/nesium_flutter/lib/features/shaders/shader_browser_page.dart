import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../l10n/app_localizations.dart';
import 'shader_browser_controller.dart';
import '../settings/android_shader_settings.dart';
import '../settings/macos_shader_settings.dart';
import '../settings/windows_shader_settings.dart';
import 'package:flutter/foundation.dart';

class ShaderBrowserPage extends ConsumerStatefulWidget {
  const ShaderBrowserPage({super.key});

  @override
  ConsumerState<ShaderBrowserPage> createState() => _ShaderBrowserPageState();
}

class _ShaderBrowserPageState extends ConsumerState<ShaderBrowserPage> {
  final _searchController = TextEditingController();
  bool _isSearching = false;

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  void _startSearch() {
    setState(() {
      _isSearching = true;
    });
  }

  void _stopSearch() {
    setState(() {
      _isSearching = false;
      _searchController.clear();
    });
    ref.read(shaderBrowserProvider.notifier).search('');
  }

  @override
  Widget build(BuildContext context) {
    final state = ref.watch(shaderBrowserProvider);
    final controller = ref.read(shaderBrowserProvider.notifier);

    final l10n = AppLocalizations.of(context)!;

    return Scaffold(
      appBar: AppBar(
        title: _isSearching
            ? TextField(
                controller: _searchController,
                autofocus: true,
                decoration: InputDecoration(
                  hintText:
                      l10n.shaderBrowserTitle, // Using title as hint for now
                  border: InputBorder.none,
                ),
                style: const TextStyle(
                  color: Colors.black,
                ), // Ensure visibility
                onChanged: (value) {
                  controller.search(value);
                },
              )
            : Text(controller.currentPath ?? l10n.shaderBrowserTitle),
        leading: _isSearching
            ? IconButton(
                icon: const Icon(Icons.arrow_back),
                onPressed: _stopSearch,
              )
            : (controller.canGoBack
                  ? IconButton(
                      icon: const Icon(Icons.arrow_back),
                      onPressed: controller.goBack,
                    )
                  : const BackButton()), // Default back button
        actions: [
          if (!_isSearching)
            IconButton(icon: const Icon(Icons.search), onPressed: _startSearch),
          if (_isSearching)
            IconButton(
              icon: const Icon(Icons.clear),
              onPressed: () {
                _searchController.clear();
                controller.search('');
              },
            ),
        ],
      ),
      body: state.when(
        data: (nodes) {
          if (nodes.isEmpty) {
            return Center(child: Text(l10n.shaderBrowserNoShaders));
          }
          return ListView.builder(
            itemCount: nodes.length,
            itemBuilder: (context, index) {
              final node = nodes[index];
              return ListTile(
                leading: Icon(node.isDirectory ? Icons.folder : Icons.brush),
                title: Text(node.name),
                subtitle: _isSearching && !node.isDirectory
                    ? Text(node.path) // Show full path in search mode
                    : null,
                onTap: () {
                  if (node.isDirectory) {
                    // If searching, entering a directory clears search context
                    // This behavior might need tweaking, but sticking to simple for now
                    if (_isSearching) _stopSearch();
                    controller.enterDirectory(node.path);
                  } else {
                    if (!kIsWeb &&
                        defaultTargetPlatform == TargetPlatform.windows) {
                      ref
                          .read(windowsShaderSettingsProvider.notifier)
                          .setPresetPath(node.path);
                    } else if (!kIsWeb &&
                        defaultTargetPlatform == TargetPlatform.android) {
                      ref
                          .read(androidShaderSettingsProvider.notifier)
                          .setPresetPath(node.path);
                    } else if (!kIsWeb &&
                        defaultTargetPlatform == TargetPlatform.macOS) {
                      ref
                          .read(macosShaderSettingsProvider.notifier)
                          .setPresetPath(node.path);
                    }
                    Navigator.of(context).pop();
                  }
                },
              );
            },
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (e, st) =>
            Center(child: Text(l10n.shaderBrowserError(e.toString()))),
      ),
    );
  }
}
