import 'package:flutter/material.dart';

import '../shell/web_shell.dart';
import 'window_types.dart';

Future<WindowKind> resolveWindowKind() async => WindowKind.main;

class WindowRouter extends StatelessWidget {
  const WindowRouter({super.key});

  @override
  Widget build(BuildContext context) => const WebShell();
}
