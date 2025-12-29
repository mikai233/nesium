class NesState {
  const NesState({this.error, this.textureId, this.romHash, this.romName});

  final String? error;
  final int? textureId;
  final String? romHash;
  final String? romName;

  factory NesState.initial() => const NesState();

  NesState copyWith({
    String? error,
    int? textureId,
    String? romHash,
    String? romName,
    bool clearError = false,
  }) {
    return NesState(
      error: clearError ? null : (error ?? this.error),
      textureId: textureId ?? this.textureId,
      romHash: romHash ?? this.romHash,
      romName: romName ?? this.romName,
    );
  }
}
