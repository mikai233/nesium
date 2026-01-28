import 'package:flutter/material.dart';

import '../shell/web_shell.dart';
import 'window_types.dart';

Future<WindowKind> resolveWindowKind() async => WindowKind.main;
Future<String?> resolveMainWindowId() async => null;
Future<Map<String, dynamic>?> resolveInitialData() async => null;

class WindowRouter extends StatelessWidget {
  const WindowRouter({super.key});

  @override
  Widget build(BuildContext context) => const WebShell();
}
