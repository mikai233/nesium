import 'dart:typed_data';

class NesState {
  const NesState({
    this.error,
    this.textureId,
    this.romHash,
    this.romName,
    this.romBytes,
  });

  final String? error;
  final int? textureId;
  final String? romHash;
  final String? romName;

  /// Cached ROM bytes for netplay late joiner sync
  final Uint8List? romBytes;

  factory NesState.initial() => const NesState();

  NesState copyWith({
    String? error,
    int? textureId,
    String? romHash,
    String? romName,
    Uint8List? romBytes,
    bool clearError = false,
    bool clearRomBytes = false,
  }) {
    return NesState(
      error: clearError ? null : (error ?? this.error),
      textureId: textureId ?? this.textureId,
      romHash: romHash ?? this.romHash,
      romName: romName ?? this.romName,
      romBytes: clearRomBytes ? null : (romBytes ?? this.romBytes),
    );
  }
}
