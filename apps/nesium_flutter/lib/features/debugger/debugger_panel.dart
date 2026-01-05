import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/api/events.dart';
import '../../l10n/app_localizations.dart';
import 'debug_state_provider.dart';

class DebuggerPanel extends ConsumerWidget {
  const DebuggerPanel({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final asyncState = ref.watch(debugStateProvider);

    return asyncState.when(
      data: (state) => _DebugStateView(state: state),
      loading: () => const _EmptyStateView(),
      error: (error, stack) => _ErrorStateView(error: error),
    );
  }
}

/// Shown when no ROM is running (waiting for debug stream data).
class _EmptyStateView extends StatelessWidget {
  const _EmptyStateView();

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            Icons.videogame_asset_off_outlined,
            size: 64,
            color: theme.colorScheme.outline,
          ),
          const SizedBox(height: 16),
          Text(
            l10n.debuggerNoRomTitle,
            style: theme.textTheme.titleMedium?.copyWith(
              color: theme.colorScheme.outline,
            ),
          ),
          const SizedBox(height: 8),
          Text(
            l10n.debuggerNoRomSubtitle,
            style: theme.textTheme.bodySmall?.copyWith(
              color: theme.colorScheme.outline,
            ),
          ),
        ],
      ),
    );
  }
}

/// Shown when there's an error subscribing to the debug stream.
class _ErrorStateView extends StatelessWidget {
  final Object error;

  const _ErrorStateView({required this.error});

  @override
  Widget build(BuildContext context) {
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.error_outline, size: 48, color: Colors.red),
          const SizedBox(height: 16),
          Text('Error: $error', textAlign: TextAlign.center),
        ],
      ),
    );
  }
}

class _DebugStateView extends StatelessWidget {
  final DebugStateNotification state;

  const _DebugStateView({required this.state});

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;
    final monoStyle = theme.textTheme.bodyMedium?.copyWith(
      fontFamily: 'monospace',
      fontFeatures: const [FontFeature.tabularFigures()],
    );

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // CPU Registers Section
          const _SectionHeader(title: 'CPU Registers'),
          const SizedBox(height: 8),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      _RegisterCell(
                        label: 'PC',
                        value: _hex16(state.cpuPc),
                        style: monoStyle,
                      ),
                      _RegisterCell(
                        label: 'A',
                        value: _hex8(state.cpuA),
                        style: monoStyle,
                      ),
                      _RegisterCell(
                        label: 'X',
                        value: _hex8(state.cpuX),
                        style: monoStyle,
                      ),
                      _RegisterCell(
                        label: 'Y',
                        value: _hex8(state.cpuY),
                        style: monoStyle,
                      ),
                      _RegisterCell(
                        label: 'SP',
                        value: _hex8(state.cpuSp),
                        style: monoStyle,
                      ),
                    ],
                  ),
                  const Divider(height: 24),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      Tooltip(
                        message: l10n.debuggerCpuStatusTooltip,
                        child: _StatusRegisterCell(
                          label: 'P (Status)',
                          rawValue: state.cpuStatus,
                          style: monoStyle,
                        ),
                      ),
                      _RegisterCell(
                        label: 'Cycle',
                        value: state.cpuCycle.toString(),
                        style: monoStyle,
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),

          const SizedBox(height: 24),

          // PPU State Section
          const _SectionHeader(title: 'PPU State'),
          const SizedBox(height: 8),
          Card(
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Column(
                children: [
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      Tooltip(
                        message: l10n.debuggerScanlineTooltip,
                        child: _RegisterCell(
                          label: 'Scanline',
                          value: state.ppuScanline.toString(),
                          style: monoStyle,
                        ),
                      ),
                      _RegisterCell(
                        label: 'Cycle',
                        value: state.ppuCycle.toString(),
                        style: monoStyle,
                      ),
                      _RegisterCell(
                        label: 'Frame',
                        value: state.ppuFrame.toString(),
                        style: monoStyle,
                      ),
                    ],
                  ),
                  const Divider(height: 24),
                  Row(
                    mainAxisAlignment: MainAxisAlignment.spaceEvenly,
                    children: [
                      Tooltip(
                        message: l10n.debuggerPpuCtrlTooltip,
                        child: _PpuRegisterCell(
                          label: 'CTRL',
                          rawValue: state.ppuCtrl,
                          flagNames: const [
                            'V',
                            'P',
                            'H',
                            'B',
                            'S',
                            'I',
                            'N',
                            'N',
                          ],
                          style: monoStyle,
                        ),
                      ),
                      Tooltip(
                        message: l10n.debuggerPpuMaskTooltip,
                        child: _PpuRegisterCell(
                          label: 'MASK',
                          rawValue: state.ppuMask,
                          flagNames: const [
                            'B',
                            'G',
                            'R',
                            's',
                            'b',
                            'M',
                            'm',
                            'g',
                          ],
                          style: monoStyle,
                        ),
                      ),
                      Tooltip(
                        message: l10n.debuggerPpuStatusTooltip,
                        child: _PpuRegisterCell(
                          label: 'STATUS',
                          rawValue: state.ppuStatus,
                          flagNames: const [
                            'V',
                            'S',
                            'O',
                            '-',
                            '-',
                            '-',
                            '-',
                            '-',
                          ],
                          style: monoStyle,
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
        ],
      ),
    );
  }

  String _hex8(int value) =>
      '\$${value.toRadixString(16).toUpperCase().padLeft(2, '0')}';
  String _hex16(int value) =>
      '\$${value.toRadixString(16).toUpperCase().padLeft(4, '0')}';
}

class _SectionHeader extends StatelessWidget {
  final String title;

  const _SectionHeader({required this.title});

  @override
  Widget build(BuildContext context) {
    return Text(
      title,
      style: Theme.of(
        context,
      ).textTheme.titleMedium?.copyWith(fontWeight: FontWeight.bold),
    );
  }
}

class _RegisterCell extends StatelessWidget {
  final String label;
  final String value;
  final TextStyle? style;

  const _RegisterCell({required this.label, required this.value, this.style});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text(
          label,
          style: Theme.of(context).textTheme.labelSmall?.copyWith(
            color: Theme.of(context).colorScheme.primary,
          ),
        ),
        const SizedBox(height: 4),
        Text(value, style: style),
      ],
    );
  }
}

/// CPU Status register cell with flag breakdown.
class _StatusRegisterCell extends StatelessWidget {
  final String label;
  final int rawValue;
  final TextStyle? style;

  const _StatusRegisterCell({
    required this.label,
    required this.rawValue,
    this.style,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final hexValue =
        '\$${rawValue.toRadixString(16).toUpperCase().padLeft(2, '0')}';
    final flags = _formatFlags(rawValue);

    return Column(
      children: [
        Text(
          label,
          style: theme.textTheme.labelSmall?.copyWith(
            color: theme.colorScheme.primary,
          ),
        ),
        const SizedBox(height: 4),
        Text(flags, style: style),
        const SizedBox(height: 2),
        Text(
          hexValue,
          style: theme.textTheme.bodySmall?.copyWith(
            fontFamily: 'monospace',
            color: theme.colorScheme.outline,
          ),
        ),
      ],
    );
  }

  String _formatFlags(int status) {
    // NV-BDIZC format (bit 5 is always 1, shown as -)
    final n = (status >> 7) & 1;
    final v = (status >> 6) & 1;
    final b = (status >> 4) & 1;
    final d = (status >> 3) & 1;
    final i = (status >> 2) & 1;
    final z = (status >> 1) & 1;
    final c = status & 1;
    return '${n == 1 ? 'N' : 'n'}${v == 1 ? 'V' : 'v'}-${b == 1 ? 'B' : 'b'}${d == 1 ? 'D' : 'd'}${i == 1 ? 'I' : 'i'}${z == 1 ? 'Z' : 'z'}${c == 1 ? 'C' : 'c'}';
  }
}

/// PPU register cell with flag breakdown.
class _PpuRegisterCell extends StatelessWidget {
  final String label;
  final int rawValue;
  final List<String> flagNames;
  final TextStyle? style;

  const _PpuRegisterCell({
    required this.label,
    required this.rawValue,
    required this.flagNames,
    this.style,
  });

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final hexValue =
        '\$${rawValue.toRadixString(16).toUpperCase().padLeft(2, '0')}';
    final flags = _formatFlags(rawValue);

    return Column(
      children: [
        Text(
          label,
          style: theme.textTheme.labelSmall?.copyWith(
            color: theme.colorScheme.primary,
          ),
        ),
        const SizedBox(height: 4),
        Text(flags, style: style),
        const SizedBox(height: 2),
        Text(
          hexValue,
          style: theme.textTheme.bodySmall?.copyWith(
            fontFamily: 'monospace',
            color: theme.colorScheme.outline,
          ),
        ),
      ],
    );
  }

  String _formatFlags(int value) {
    final buffer = StringBuffer();
    for (int i = 7; i >= 0; i--) {
      final bit = (value >> i) & 1;
      final name = flagNames[7 - i];
      if (name == '-') {
        buffer.write('-');
      } else {
        buffer.write(bit == 1 ? name.toUpperCase() : name.toLowerCase());
      }
    }
    return buffer.toString();
  }
}
