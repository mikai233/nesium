import 'dart:io';

import 'package:path/path.dart' as p;

Future<void> main(List<String> arguments) async {
  final wasmPack = await _which('wasm-pack');
  if (wasmPack == null) {
    stderr.writeln(
      'wasm-pack not found in PATH.\n'
      'Install it first: https://rustwasm.github.io/wasm-pack/installer/',
    );
    exit(1);
  }

  final scriptFile = File.fromUri(Platform.script);
  final appDir = scriptFile.parent.parent.absolute;
  final repoRoot = appDir.parent.parent.absolute;

  final wasmCrateDir = p.join(repoRoot.path, 'crates', 'nesium-wasm');
  final outDir = p.join(appDir.path, 'web', 'nes', 'pkg');
  Directory(outDir).createSync(recursive: true);

  stdout.writeln('[wasm] wasm-pack build --release');
  final code = await _runStreaming(
    executable: wasmPack,
    args: [
      'build',
      wasmCrateDir,
      '--target',
      'web',
      '--out-dir',
      outDir,
      '--release',
    ],
    workingDirectory: repoRoot.path,
  );
  if (code != 0) {
    stderr.writeln('[wasm] wasm-pack failed with exit code $code');
    exit(code);
  }
}

Future<int> _runStreaming({
  required String executable,
  required List<String> args,
  required String workingDirectory,
}) async {
  final proc = await Process.start(
    executable,
    args,
    workingDirectory: workingDirectory,
    mode: ProcessStartMode.inheritStdio,
    runInShell: true,
  );
  return proc.exitCode;
}

Future<String?> _which(String name) async {
  final result = await Process.run(Platform.isWindows ? 'where' : 'which', [
    name,
  ], runInShell: true);
  if (result.exitCode != 0) return null;
  final out = (result.stdout as String).trim();
  if (out.isEmpty) return null;
  return out.split(RegExp(r'\r?\n')).first.trim();
}
