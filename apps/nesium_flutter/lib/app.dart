import 'package:flutter/material.dart';

import 'windows/window_routing.dart';

class NesiumApp extends StatelessWidget {
  const NesiumApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'Nesium',
      theme: ThemeData(
        colorScheme: ColorScheme.fromSeed(
          seedColor: Colors.blueGrey,
          brightness: Brightness.light,
        ).copyWith(surface: Colors.white),
        scaffoldBackgroundColor: Colors.white,
        useMaterial3: true,
        visualDensity: VisualDensity.adaptivePlatformDensity,
      ),
      home: const WindowRouter(),
    );
  }
}
