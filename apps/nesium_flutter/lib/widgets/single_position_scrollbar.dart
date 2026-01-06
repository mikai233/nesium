import 'package:flutter/material.dart';

class SinglePositionScrollbar extends StatefulWidget {
  const SinglePositionScrollbar({
    super.key,
    required this.builder,
    this.thumbVisibility,
  });

  final Widget Function(BuildContext context, ScrollController controller)
  builder;
  final bool? thumbVisibility;

  @override
  State<SinglePositionScrollbar> createState() =>
      _SinglePositionScrollbarState();
}

class _SinglePositionScrollbarState extends State<SinglePositionScrollbar> {
  final ScrollController _controller = ScrollController();

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Scrollbar(
      controller: _controller,
      thumbVisibility: widget.thumbVisibility,
      child: widget.builder(context, _controller),
    );
  }
}
