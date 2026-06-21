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

      // Ensure fresh zip
      final zipFile = File(zipOutput);
      if (zipFile.existsSync()) {
        try {
          zipFile.deleteSync();
        } catch (e) {
          logWarning('⚠️ Could not delete old ZIP: $e');
        }
      }

      encoder.create(zipOutput);

      // Add all files from _destRoot to the zip, but with relative paths
      final destDir = Directory(_destRoot);
      if (destDir.existsSync()) {
        final List<FileSystemEntity> entities = destDir.listSync(
          recursive: true,
        );
        logInfo('📦 Zipping ${_copiedFiles.length} files...');
        for (final entity in entities) {
          if (entity is File) {
            final relPath = path.relative(entity.path, from: _destRoot);
            // Ensure ZIP entries use forward slashes for cross-platform compatibility
            final zipEntryPath = relPath.replaceAll('\\', '/');
            await encoder.addFile(entity, zipEntryPath);
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

        final ext = path.extension(entity.path).toLowerCase();
        if (ext == '.slangp' ||
            ext == '.slang' ||
            ext == '.inc' ||
            ext == '.h') {
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
  absSourcePath = path.normalize(File(absSourcePath).absolute.path);
  final normalizedRoot = path.normalize(_sourceRoot);

  if (_copiedFiles.contains(absSourcePath)) return;

  // Verify it's inside source root
  if (!absSourcePath.toLowerCase().startsWith(normalizedRoot.toLowerCase())) {
    logWarning(
      '⚠️ Skipping external file: $absSourcePath (Root: $normalizedRoot)',
    );
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

Future<void> _parseDependencies(File file) async {
  try {
    final content = await file.readAsString();
    final lines = content.split('\n');
    final fileDir = file.parent.absolute.path;
    final ext = path.extension(file.path).toLowerCase();

    for (var line in lines) {
      line = line.trim();
      if (line.isEmpty) continue;

      // Skip comments in .slangp files
      if (ext == '.slangp' && line.startsWith('#')) {
        // Handle #reference / #include / #import (Slang or Slangp)
        final referenceMatch = RegExp(
          r'^#(reference|include|import)\s+"?([^"]+)"?',
        ).firstMatch(line);
        if (referenceMatch != null) {
          final refPath = referenceMatch.group(2)!;
          await _resolveAndCopy(fileDir, refPath, file.path);
        }
        continue;
      }

      // Handle Key=Value pairs (Presets)
      if (ext == '.slangp' && line.contains('=')) {
        final parts = line.split('=');
        if (parts.length >= 2) {
          var value = parts.sublist(1).join('=').trim().replaceAll('"', '');

          if (value.isEmpty) continue;

          // Check if value looks like a relative path to a dependency
          final valExt = path.extension(value).toLowerCase();
          final isDependencyExt = [
            '.slang',
            '.slangp',
            '.inc',
            '.h',
            '.png',
            '.jpg',
            '.jpeg',
            '.tga',
            '.bmp',
          ].contains(valExt);

          if (isDependencyExt) {
            await _resolveAndCopy(fileDir, value, file.path);
          }
        }
      }

      // Handle #include / #import in .slang / .h / .inc files
      if (ext != '.slangp' && line.startsWith('#')) {
        final match = RegExp(
          r'^#(include|import)\s+["<]([^">]+)[">]',
        ).firstMatch(line);
        if (match != null) {
          final refPath = match.group(2)!;
          await _resolveAndCopy(fileDir, refPath, file.path);
        }
      }
    }
  } catch (e) {
    // Avoid crashing on binary or malformed files
    // logError('  Error parsing ${file.path}: $e');
  }
}

Future<void> _resolveAndCopy(
  String baseDir,
  String relativePath,
  String sourceFile,
) async {
  // Normalize path
  var targetPath = path.normalize(path.join(baseDir, relativePath));
  targetPath = File(targetPath).absolute.path;

  // Handle "rooted" paths (e.g. shaders/...) assuming common parent structure
  if (!File(targetPath).existsSync() && relativePath.startsWith('shaders/')) {
    final potentialRootPath = path.normalize(
      path.join(_sourceRoot, '..', relativePath),
    );
    if (File(potentialRootPath).existsSync()) {
      targetPath = File(potentialRootPath).absolute.path;
    }
  }

  // Fuzzy Resolution (matches existing heuristic, but more aggressive)
  if (!File(targetPath).existsSync()) {
    var candidate = relativePath;
    // Try going up to 4 levels to find shifted directories
    for (var i = 0; i < 4; i++) {
      candidate = path.join('..', candidate);
      var fuzzyPath = path.normalize(path.join(baseDir, candidate));
      fuzzyPath = File(fuzzyPath).absolute.path;

      if (!fuzzyPath.toLowerCase().startsWith(_sourceRoot.toLowerCase()) &&
          !fuzzyPath.toLowerCase().startsWith(
            path.dirname(_sourceRoot).toLowerCase(),
          )) {
        break;
      }

      if (File(fuzzyPath).existsSync()) {
        targetPath = fuzzyPath;
        break;
      }
    }
  }

  // Final check: if it's still missing, try looking for the filename inside the source root
  if (!File(targetPath).existsSync()) {
    final fileName = path.basename(relativePath);
    // This is expensive, so only do it for actual potential paths
    if (relativePath.contains('/') || relativePath.contains('\\')) {
      final results = _findFileGlobally(_sourceRoot, fileName);
      if (results.isNotEmpty) {
        // If multiple matches, we might pick the wrong one, but usually it's better than nothing
        // Or we could try to find the best match by comparing the end of the path
        targetPath = results.first;
      }
    }
  }

  // Copy and Recurse
  if (File(targetPath).existsSync()) {
    if (!_copiedFiles.contains(targetPath)) {
      await _copyFile(targetPath);

      final ext = path.extension(targetPath).toLowerCase();
      if (ext == '.slangp' || ext == '.slang' || ext == '.inc' || ext == '.h') {
        await _parseDependencies(File(targetPath));
      }
    }
  } else {
    // Don't warn for obviously non-path strings that might have matched our simple regex
    if (relativePath.contains('/') || relativePath.contains('\\')) {
      logWarning('⚠️ Missing dependency: $relativePath (in $sourceFile)');
    }
  }
}

List<String> _findFileGlobally(String root, String fileName) {
  final List<String> matches = [];
  final dir = Directory(root);
  if (!dir.existsSync()) return matches;

  try {
    for (final entity in dir.listSync(recursive: true, followLinks: false)) {
      if (entity is File && path.basename(entity.path) == fileName) {
        matches.add(entity.absolute.path);
      }
    }
  } catch (e) {
    // Ignore errors during listing
  }
  return matches;
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
