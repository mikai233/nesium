class NesState {
  const NesState({this.error, this.textureId});

  final String? error;
  final int? textureId;

  factory NesState.initial() => const NesState();

  NesState copyWith({String? error, int? textureId, bool clearError = false}) {
    return NesState(
      error: clearError ? null : (error ?? this.error),
      textureId: textureId ?? this.textureId,
    );
  }
}
