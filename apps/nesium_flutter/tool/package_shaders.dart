import 'dart:io';
import 'package:archive/archive_io.dart';
import 'package:path/path.dart' as path;

// Configuration
const List<String> kWhitelistCategories = [
  'crt',
  'ntsc',
  'pixel-art-scaling',
  'anti-aliasing',
  'sharpen',
  'scanlines',
  'vhs',
  'interpolation',
  'presets',
  'denoisers',
  'dithering',
  'glow',
  'film',
  'edge-smoothing',
];

const List<String> kBlacklistExtensions = [
  '.png',
  '.jpg',
  '.jpeg',
  '.gif',
  '.tga',
  '.bmp',
  '.mp3',
  '.wav',
  '.py',
  '.md',
  '.txt',
  '.yml',
  '.yaml',
  '.sh',
  '.ps1',
  '.bat',
  '.gitlab-ci.yml',
  '.gitignore',
  '.gitattributes',
  '.extracted',
  'makefile',
  'configure',
  'license',
  'copying',
];

// Global state
final Set<String> _copiedFiles = {};
String _sourceRoot = '';
String _destRoot = '';
final String sep = Platform.pathSeparator;

void main(List<String> args) async {
  String sourcePath;
  String destPath;
  String? zipPath;

  if (args.isEmpty) {
    // Default mode: Use local assets/shaders as source
    // Goal: Create optimized zip from the existing full set.
    final scriptFile = File(Platform.script.toFilePath());
    final toolDir = scriptFile.parent; // apps/nesium_flutter/tool
    final appRoot = toolDir.parent.path; // apps/nesium_flutter

    // Source: Existing assets/shaders folder (contains full repo?)
    sourcePath = joinPath(appRoot, joinPath('assets', 'shaders'));

    // Dest: Temporary build directory to assemble the clean pack
    destPath = joinPath(appRoot, joinPath('build', 'shaders_pkg'));

    // Zip: Output to assets/bundled/shaders.zip
    final bundledDir = Directory(
      joinPath(appRoot, joinPath('assets', 'bundled')),
    );
    if (!bundledDir.existsSync()) {
      bundledDir.createSync(recursive: true);
    }
    zipPath = joinPath(bundledDir.path, 'shaders.zip');

    logInfo('ℹ️ No arguments provided. Using defaults:');
    logInfo('   Source: $sourcePath');
    logInfo('   Dest:   $destPath');
    logInfo('   Zip:    $zipPath');
  } else if (args.length >= 2) {
    sourcePath = args[0];
    destPath = args[1];
    zipPath = args.length > 2 ? args[2] : null;
  } else {
    logInfo(
      'Usage: dart package_shaders.dart [path/to/slang-shaders-repo] [output-dir] [output-zip]',
    );
    logInfo(
      '   Or run without arguments to use defaults (requires slang-shaders in project root).',
    );
    exit(1);
  }

  _sourceRoot = Directory(sourcePath).absolute.path;
  _destRoot = Directory(destPath).absolute.path;
  String? zipOutput = zipPath != null ? File(zipPath).absolute.path : null;

  logInfo('📦 Packaging shaders from: $_sourceRoot');
  logInfo('📂 Output directory: $_destRoot');

  if (!Directory(_sourceRoot).existsSync()) {
    logError('❌ Source directory does not exist: $_sourceRoot');
    logInfo(
      '   Please clone https://github.com/libretro/slang-shaders into the project root.',
    );
    exit(1);
  }

  // Clean output directory
  final outputDir = Directory(_destRoot);
  if (outputDir.existsSync()) {
    outputDir.deleteSync(recursive: true);
  }
  outputDir.createSync(recursive: true);

  // 1. Process Whitelisted Categories
  for (final category in kWhitelistCategories) {
    final catDirPath = joinPath(_sourceRoot, category);
    final catDir = Directory(catDirPath);
    if (catDir.existsSync()) {
      await _processDirectory(catDir);
    } else {
      // Check for standalone files
      final catFile = File(catDirPath);
      if (catFile.existsSync()) {
        await _copyFile(catFile.path);
      }
    }
  }

  // Also process root files like stock.slang
  await _processDirectory(Directory(_sourceRoot), recursive: false);

  // 1.5 Clean up empty directories in the output
  _removeEmptyDirectories(Directory(_destRoot));

  logInfo(
    '✅ Shader packaging complete! ${_copiedFiles.length} files processed.',
  );

  // 2. Zip the result if requested
  if (zipOutput != null) {
    logInfo('🤐 Creating ZIP archive: $zipOutput');

    try {
      final encoder = ZipFileEncoder();
      encoder.create(zipOutput);

      // Add all files from _destRoot to the zip, but with relative paths
      // This mimics "tar -C _destRoot -cf ... ."
      final destDir = Directory(_destRoot);
      if (destDir.existsSync()) {
        await for (final entity in destDir.list(recursive: true)) {
          if (entity is File) {
            final relPath = path.relative(entity.path, from: _destRoot);
            encoder.addFile(entity, relPath);
          }
        }
      }

      encoder.close();
      logInfo('✨ ZIP created successfully.');

      await _generateMd5(zipOutput);
    } catch (e) {
      logError('⚠️ ZIP creation failed: $e');
    }
  }
}

Future<void> _generateMd5(String filePath) async {
  String? hash;
  try {
    if (Platform.isMacOS) {
      final result = await Process.run('md5', ['-q', filePath]);
      if (result.exitCode == 0) hash = result.stdout.toString().trim();
    } else if (Platform.isLinux) {
      final result = await Process.run('md5sum', [filePath]);
      if (result.exitCode == 0) {
        hash = result.stdout.toString().split(' ').first.trim();
      }
    } else if (Platform.isWindows) {
      final result = await Process.run('certutil', [
        '-hashfile',
        filePath,
        'MD5',
      ]);
      if (result.exitCode == 0) {
        final lines = result.stdout.toString().split('\n');
        // certutil output usually has the hash on the second line
        if (lines.length > 1) {
          hash = lines[1].trim().replaceAll(' ', '');
        }
      }
    }

    if (hash != null) {
      final md5File = File('$filePath.md5');
      await md5File.writeAsString(hash);
      logInfo('📝 MD5 hash generated: $hash');
    } else {
      logWarning(
        '⚠️ Could not generate MD5 hash for platform ${Platform.operatingSystem}',
      );
    }
  } catch (e) {
    logError('⚠️ Error generating MD5: $e');
  }
}

void _removeEmptyDirectories(Directory dir) {
  if (!dir.existsSync()) return;

  final entities = dir.listSync(recursive: false);
  for (final entity in entities) {
    if (entity is Directory) {
      _removeEmptyDirectories(entity);
    }
  }

  // After processing children, check if this directory is now empty
  if (dir.listSync(recursive: false).isEmpty) {
    // Don't delete the root destination directory
    if (dir.path != _destRoot) {
      dir.deleteSync();
    }
  }
}

// Simple path join helper
String joinPath(String part1, String part2) {
  if (part1.endsWith(sep)) return part1 + part2;
  return '$part1$sep$part2';
}

Future<void> _processDirectory(Directory dir, {bool recursive = true}) async {
  await for (final entity in dir.list(recursive: recursive)) {
    if (entity is File) {
      if (_shouldCopy(entity.path)) {
        await _copyFile(entity.path);

        if (entity.path.endsWith('.slangp')) {
          await _parseDependencies(entity);
        }
      }
    }
  }
}

bool _shouldCopy(String filePath) {
  final lowerPath = filePath.toLowerCase();
  for (final ext in kBlacklistExtensions) {
    if (lowerPath.endsWith(ext)) return false;
  }

  // Hidden files / git
  final filename = Uri.file(filePath).pathSegments.last;
  if (filename.startsWith('.')) return false;

  return true;
}

Future<void> _copyFile(String absSourcePath) async {
  // Normalize source path
  absSourcePath = File(absSourcePath).absolute.path;

  if (_copiedFiles.contains(absSourcePath)) return;

  // Verify it's inside source root (simple check)
  if (!absSourcePath.startsWith(_sourceRoot)) {
    logWarning('⚠️ Skipping external file: $absSourcePath');
    return;
  }

  final relativePath = absSourcePath.substring(_sourceRoot.length + 1);
  final destPath = joinPath(_destRoot, relativePath);

  final destFile = File(destPath);
  if (!destFile.parent.existsSync()) {
    destFile.parent.createSync(recursive: true);
  }

  await File(absSourcePath).copy(destPath);
  _copiedFiles.add(absSourcePath);
}

Future<void> _parseDependencies(File presetFile) async {
  try {
    final content = await presetFile.readAsString();
    final lines = content.split('\n');
    final presetDir = presetFile.parent.absolute.path;

    for (var line in lines) {
      line = line.trim();
      if (line.isEmpty) continue;

      if (line.startsWith('shader') && line.contains('=')) {
        final key = line.split('=')[0].trim();
        // Check if key is exactly "shader" + digits
        // This avoids matching "shaders = 3"
        if (!RegExp(r'^shader\d+$').hasMatch(key)) continue;

        final parts = line.split('=');
        if (parts.length >= 2) {
          var shaderPath = parts[1].trim();
          // Remove quotes if present
          shaderPath = shaderPath.replaceAll('"', '');

          // Resolve path (manual normalization)
          // Handle ../ references
          var targetPath = joinPath(presetDir, shaderPath);
          targetPath = File(targetPath).absolute.path; // Resolves .. natively

          // Fuzzy Resolution:
          // If the exact path doesn't exist, try resolving against parent directories.
          // This handles cases where presets are moved into subdirectories (e.g. crt/crt-effects)
          // but still reference headers as if they were in the parent (e.g. ../stock.slang).
          if (!File(targetPath).existsSync()) {
            // specific heuristic: try stripping one or two parent levels from the base?
            // Or simply prepend ../ to the shaderPath?

            // Try prepending ../ up to 3 times
            var candidate = shaderPath;
            for (var i = 0; i < 3; i++) {
              candidate = joinPath('..', candidate);
              var fuzzyPath = joinPath(presetDir, candidate);
              fuzzyPath = File(fuzzyPath).absolute.path;

              // Ensure we don't go outside source root (security)
              if (!fuzzyPath.startsWith(_sourceRoot)) break;

              if (File(fuzzyPath).existsSync()) {
                targetPath = fuzzyPath;
                break;
              }
            }
          }

          // Copy dependency
          if (File(targetPath).existsSync()) {
            if (!_copiedFiles.contains(targetPath)) {
              // print('  🔗 Pulling dependency: $shaderPath');
              await _copyFile(targetPath);

              if (shaderPath.endsWith('.slangp')) {
                await _parseDependencies(File(targetPath));
              }
            }
          } else {
            logWarning(
              '⚠️ Missing dependency: $shaderPath (in ${presetFile.path})',
            );
          }
        }
      }

      // Handle #reference
      if (line.startsWith('#reference') || line.startsWith('#include')) {
        final parts = line.split(' ');
        if (parts.length >= 2) {
          var refPath = parts[1].trim().replaceAll('"', '');
          var targetPath = joinPath(presetDir, refPath);
          targetPath = File(targetPath).absolute.path;

          if (File(targetPath).existsSync()) {
            if (!_copiedFiles.contains(targetPath)) {
              // print('  🔗 Pulling reference: $refPath');
              await _copyFile(targetPath);
              await _parseDependencies(File(targetPath));
            }
          }
        }
      }
    }
  } catch (e) {
    logError('  Example error parsing ${presetFile.path}: $e');
  }
}

// Logger shims to match project style
void logInfo(String message) => _log('INFO', message);
void logWarning(String message) => _log('WARNING', message);
void logError(String message) => _log('SEVERE', message);

void _log(String level, String message) {
  // Use stdout for INFO/WARNING, stderr for SEVERE
  final iosink = level == 'SEVERE' ? stderr : stdout;
  iosink.writeln('[$level] $message');
}
