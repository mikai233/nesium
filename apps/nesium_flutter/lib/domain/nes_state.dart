class NesState {
  const NesState({required this.loading, this.error, this.textureId});

  final bool loading;
  final String? error;
  final int? textureId;

  factory NesState.initial() => const NesState(loading: true);

  NesState copyWith({
    bool? loading,
    String? error,
    int? textureId,
    bool clearError = false,
  }) {
    return NesState(
      loading: loading ?? this.loading,
      error: clearError ? null : (error ?? this.error),
      textureId: textureId ?? this.textureId,
    );
  }
}
