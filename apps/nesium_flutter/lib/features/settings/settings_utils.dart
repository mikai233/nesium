import '../../l10n/app_localizations.dart';
import '../controls/input_settings.dart';
import 'input_settings_types.dart';

class SettingsUtils {
  static String presetLabel(AppLocalizations l10n, KeyboardPreset preset) =>
      switch (preset) {
        KeyboardPreset.none => l10n.keyboardPresetNone,
        KeyboardPreset.nesStandard => l10n.keyboardPresetNesStandard,
        KeyboardPreset.fightStick => l10n.keyboardPresetFightStick,
        KeyboardPreset.arcadeLayout => l10n.keyboardPresetArcadeLayout,
        KeyboardPreset.custom => l10n.keyboardPresetCustom,
      };

  static String actionLabel(AppLocalizations l10n, dynamic action) {
    if (action is NesButtonAction) {
      return switch (action) {
        NesButtonAction.a => l10n.inputButtonA,
        NesButtonAction.b => l10n.inputButtonB,
        NesButtonAction.select => l10n.inputButtonSelect,
        NesButtonAction.start => l10n.inputButtonStart,
        NesButtonAction.up => l10n.inputButtonUp,
        NesButtonAction.down => l10n.inputButtonDown,
        NesButtonAction.left => l10n.inputButtonLeft,
        NesButtonAction.right => l10n.inputButtonRight,
        NesButtonAction.turboA => l10n.inputButtonTurboA,
        NesButtonAction.turboB => l10n.inputButtonTurboB,
        NesButtonAction.rewind => l10n.inputButtonRewind,
        NesButtonAction.fastForward => l10n.inputButtonFastForward,
        NesButtonAction.saveState => l10n.inputButtonSaveState,
        NesButtonAction.loadState => l10n.inputButtonLoadState,
        NesButtonAction.pause => l10n.inputButtonPause,
        NesButtonAction.fullScreen => l10n.keyboardActionFullScreen,
      };
    } else if (action is KeyboardBindingAction) {
      return switch (action) {
        KeyboardBindingAction.up => l10n.keyboardActionUp,
        KeyboardBindingAction.down => l10n.keyboardActionDown,
        KeyboardBindingAction.left => l10n.keyboardActionLeft,
        KeyboardBindingAction.right => l10n.keyboardActionRight,
        KeyboardBindingAction.a => l10n.keyboardActionA,
        KeyboardBindingAction.b => l10n.keyboardActionB,
        KeyboardBindingAction.select => l10n.keyboardActionSelect,
        KeyboardBindingAction.start => l10n.keyboardActionStart,
        KeyboardBindingAction.turboA => l10n.keyboardActionTurboA,
        KeyboardBindingAction.turboB => l10n.keyboardActionTurboB,
        KeyboardBindingAction.rewind => l10n.keyboardActionRewind,
        KeyboardBindingAction.fastForward => l10n.keyboardActionFastForward,
        KeyboardBindingAction.saveState => l10n.keyboardActionSaveState,
        KeyboardBindingAction.loadState => l10n.keyboardActionLoadState,
        KeyboardBindingAction.pause => l10n.keyboardActionPause,
        KeyboardBindingAction.fullScreen => l10n.keyboardActionFullScreen,
      };
    }
    return 'Unknown';
  }
}
