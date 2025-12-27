import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'virtual_controls_settings.dart';

@immutable
class VirtualControlsEditorState {
  static const Object _draftNoChange = Object();

  const VirtualControlsEditorState({
    required this.enabled,
    required this.gridSnapEnabled,
    required this.gridSpacing,
    required this.draft,
  });

  final bool enabled;
  final bool gridSnapEnabled;
  final double gridSpacing;
  final VirtualControlsSettings? draft;

  VirtualControlsEditorState copyWith({
    bool? enabled,
    bool? gridSnapEnabled,
    double? gridSpacing,
    Object? draft = _draftNoChange,
  }) {
    return VirtualControlsEditorState(
      enabled: enabled ?? this.enabled,
      gridSnapEnabled: gridSnapEnabled ?? this.gridSnapEnabled,
      gridSpacing: gridSpacing ?? this.gridSpacing,
      draft: identical(draft, _draftNoChange)
          ? this.draft
          : draft as VirtualControlsSettings?,
    );
  }

  static const defaults = VirtualControlsEditorState(
    enabled: false,
    gridSnapEnabled: true,
    gridSpacing: 16,
    draft: null,
  );
}

class VirtualControlsEditorController
    extends Notifier<VirtualControlsEditorState> {
  @override
  VirtualControlsEditorState build() => VirtualControlsEditorState.defaults;

  void setEnabled(bool enabled) {
    if (enabled == state.enabled) return;

    if (enabled) {
      final current = ref.read(virtualControlsSettingsProvider);
      state = state.copyWith(enabled: true, draft: current);
      return;
    }

    final draft = state.draft;
    if (draft != null) {
      ref.read(virtualControlsSettingsProvider.notifier).replace(draft);
    }
    state = state.copyWith(enabled: false, draft: null);
  }

  void setGridSnapEnabled(bool enabled) {
    if (enabled == state.gridSnapEnabled) return;
    state = state.copyWith(gridSnapEnabled: enabled);
  }

  void setGridSpacing(double spacing) {
    final next = spacing.clamp(4, 64).toDouble();
    if (next == state.gridSpacing) return;
    state = state.copyWith(gridSpacing: next);
  }

  void updateDraft(
    VirtualControlsSettings Function(VirtualControlsSettings) update,
  ) {
    final draft = state.draft;
    if (!state.enabled || draft == null) return;
    state = state.copyWith(draft: update(draft));
  }
}

final virtualControlsEditorProvider =
    NotifierProvider<
      VirtualControlsEditorController,
      VirtualControlsEditorState
    >(VirtualControlsEditorController.new);
