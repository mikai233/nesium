import 'dart:async';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

@immutable
class FloatingGamePreviewState {
  const FloatingGamePreviewState({
    required this.visible,
    required this.offset,
    this.isHiding = false,
  });

  final bool visible;
  final Offset offset;
  final bool isHiding;

  FloatingGamePreviewState copyWith({
    bool? visible,
    Offset? offset,
    bool? isHiding,
  }) {
    return FloatingGamePreviewState(
      visible: visible ?? this.visible,
      offset: offset ?? this.offset,
      isHiding: isHiding ?? this.isHiding,
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
      state = state.copyWith(
        visible: true,
        offset: const Offset(20, 150),
        isHiding: false,
      );
    } else {
      state = state.copyWith(visible: false, isHiding: false);
    }
  }

  void hide() => state = state.copyWith(visible: false, isHiding: false);

  void show() => state = state.copyWith(
    visible: true,
    offset: const Offset(20, 150),
    isHiding: false,
  );

  void setOffset(Offset offset) => state = state.copyWith(offset: offset);

  Completer<void>? _hideCompleter;

  Future<void> hideAnimated() async {
    if (!state.visible || state.isHiding) return;
    _hideCompleter = Completer<void>();
    state = state.copyWith(isHiding: true);
    return _hideCompleter!.future;
  }

  void confirmHidden() {
    state = state.copyWith(visible: false, isHiding: false);
    _hideCompleter?.complete();
    _hideCompleter = null;
  }
}

final floatingGamePreviewProvider =
    NotifierProvider<FloatingGamePreviewController, FloatingGamePreviewState>(
      FloatingGamePreviewController.new,
    );
