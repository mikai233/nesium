import 'package:flutter_riverpod/flutter_riverpod.dart';

class LaunchArgs {
  const LaunchArgs({this.romPath, this.luaScriptPath});

  final String? romPath;
  final String? luaScriptPath;
}

final launchArgsProvider = Provider<LaunchArgs>((ref) => const LaunchArgs());
