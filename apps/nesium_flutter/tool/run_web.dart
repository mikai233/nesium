import 'dart:io';

import 'package:path/path.dart' as p;

Future<void> main(List<String> arguments) async {
  var forceWasm = false;
  var skipWasm = false;
  var useCache = false;
  var wasmRelease = true;

  final forwarded = <String>[];
  for (final arg in arguments) {
    switch (arg) {
      case '--force-wasm':
        forceWasm = true;
        break;
      case '--skip-wasm':
        skipWasm = true;
        break;
      case '--use-cache':
        useCache = true;
        break;
      case '--wasm-dev':
        wasmRelease = false;
        break;
      case '--wasm-release':
        wasmRelease = true;
        break;
      default:
        forwarded.add(arg);
        break;
    }
  }

  final scriptFile = File.fromUri(Platform.script);
  final appDir = scriptFile.parent.parent.absolute;
  final repoRoot = appDir.parent.parent.absolute;

  if (!skipWasm) {
    await _ensureWasmPackBuilt(
      repoRoot: repoRoot.path,
      appDir: appDir.path,
      force: forceWasm || !useCache,
      release: wasmRelease,
    );
  }

  final flutterArgs = <String>['run', ...forwarded];
  final hasDeviceArg =
      forwarded.contains('-d') ||
      forwarded.contains('--device-id') ||
      forwarded.any((a) => a.startsWith('-d=') || a.startsWith('--device-id='));
  if (!hasDeviceArg) {
    flutterArgs.insertAll(1, const ['-d', 'chrome']);
  }

  final exitCode = await _runStreaming(
    executable: 'flutter',
    args: flutterArgs,
    workingDirectory: appDir.path,
  );
  exit(exitCode);
}

Future<void> _ensureWasmPackBuilt({
  required String repoRoot,
  required String appDir,
  required bool force,
  required bool release,
}) async {
  final wasmPack = await _which('wasm-pack');
  if (wasmPack == null) {
    stderr.writeln(
      'wasm-pack not found in PATH.\n'
      'Install it first: https://rustwasm.github.io/wasm-pack/installer/',
    );
    exit(1);
  }

  final wasmCrateDir = p.join(repoRoot, 'crates', 'nesium-wasm');
  final outDir = p.join(appDir, 'web', 'nes', 'pkg');
  final marker = File(p.join(outDir, 'nesium_wasm.js'));

  DateTime newestSource = DateTime(1970);
  if (!force) {
    newestSource = _latestMtimeSync([
      p.join(wasmCrateDir, 'Cargo.toml'),
      p.join(wasmCrateDir, 'src'),
      p.join(repoRoot, 'Cargo.lock'),
      p.join(repoRoot, 'crates', 'nesium-core', 'Cargo.toml'),
      p.join(repoRoot, 'crates', 'nesium-core', 'src'),
    ]);
  }

  final markerMtime = marker.existsSync()
      ? marker.lastModifiedSync()
      : DateTime(1970);
  final needsBuild =
      force || !marker.existsSync() || markerMtime.isBefore(newestSource);

  if (!needsBuild) {
    stdout.writeln('[wasm] up to date');
    return;
  }

  stdout.writeln('[wasm] building (wasm-pack)…');
  Directory(outDir).createSync(recursive: true);

  final code = await _runStreaming(
    executable: wasmPack,
    args: [
      'build',
      wasmCrateDir,
      '--target',
      'web',
      '--out-dir',
      outDir,
      if (release) '--release',
    ],
    workingDirectory: repoRoot,
  );
  if (code != 0) {
    stderr.writeln('[wasm] wasm-pack failed with exit code $code');
    exit(code);
  }
}

DateTime _latestMtimeSync(List<String> paths) {
  DateTime latest = DateTime(1970);

  for (final path in paths) {
    final type = FileSystemEntity.typeSync(path, followLinks: false);
    switch (type) {
      case FileSystemEntityType.file:
        final m = File(path).lastModifiedSync();
        if (m.isAfter(latest)) latest = m;
        break;
      case FileSystemEntityType.directory:
        final dir = Directory(path);
        if (!dir.existsSync()) break;
        for (final entity in dir.listSync(
          recursive: true,
          followLinks: false,
        )) {
          if (entity is! File) continue;
          final m = entity.lastModifiedSync();
          if (m.isAfter(latest)) latest = m;
        }
        break;
      case FileSystemEntityType.link:
      case FileSystemEntityType.notFound:
      case FileSystemEntityType.pipe:
      case FileSystemEntityType.unixDomainSock:
        break;
    }
  }

  return latest;
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
