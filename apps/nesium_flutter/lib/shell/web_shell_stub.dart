import 'package:flutter/material.dart';

class WebShell extends StatelessWidget {
  const WebShell({super.key});

  @override
  Widget build(BuildContext context) {
    return const Scaffold(
      body: Center(child: Text('WebShell is only available on Web.')),
    );
  }
}
