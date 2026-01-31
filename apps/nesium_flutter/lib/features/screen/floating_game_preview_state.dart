import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

@immutable
class FloatingGamePreviewState {
  const FloatingGamePreviewState({required this.visible, required this.offset});

  final bool visible;
  final Offset offset;

  FloatingGamePreviewState copyWith({bool? visible, Offset? offset}) {
    return FloatingGamePreviewState(
      visible: visible ?? this.visible,
      offset: offset ?? this.offset,
    );
  }
}

class FloatingGamePreviewController extends Notifier<FloatingGamePreviewState> {
  @override
  FloatingGamePreviewState build() {
    return const FloatingGamePreviewState(
      visible: false,
      offset: Offset(20, 150),
    );
  }

  void toggle() {
    if (!state.visible) {
      state = state.copyWith(visible: true, offset: const Offset(20, 150));
    } else {
      state = state.copyWith(visible: false);
    }
  }

  void hide() => state = state.copyWith(visible: false);

  void show() =>
      state = state.copyWith(visible: true, offset: const Offset(20, 150));

  void setOffset(Offset offset) => state = state.copyWith(offset: offset);
}

final floatingGamePreviewProvider =
    NotifierProvider<FloatingGamePreviewController, FloatingGamePreviewState>(
      FloatingGamePreviewController.new,
    );
