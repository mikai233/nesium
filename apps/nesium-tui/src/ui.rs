mod widget;

use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Paragraph,
};

use self::widget::NesFrameWidget;
use crate::app::App;

pub fn draw(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Header
            Constraint::Min(0),    // Game
            Constraint::Length(1), // Footer
        ])
        .split(f.size());

    // Header
    let title = Line::from(vec![
        Span::styled(
            " Nesium TUI ",
            Style::default().bg(Color::Blue).fg(Color::White).bold(),
        ),
        Span::raw(format!(
            " | ROM: {} | FPS: {}",
            app.rom_name(),
            app.current_fps()
        )),
    ]);
    f.render_widget(
        Paragraph::new(title).alignment(Alignment::Center),
        chunks[0],
    );

    // Game Area
    if let Some(rt) = app.runtime() {
        let handle = rt.handle();
        if let Some(frame_handle) = handle.frame_handle() {
            // Render the game frame
            let area = chunks[1];
            let game_widget = NesFrameWidget::new(frame_handle.clone());
            f.render_widget(game_widget, area);
        }
    }

    // Footer
    let help = Line::from(vec![
        Span::styled("Q/Esc", Style::default().bold()),
        Span::raw(": Quit | "),
        Span::styled("R", Style::default().bold()),
        Span::raw(": Reset | "),
        Span::styled("Arrows", Style::default().bold()),
        Span::raw(": D-Pad | "),
        Span::styled("Z/X", Style::default().bold()),
        Span::raw(": A/B | "),
        Span::styled("Space/Enter", Style::default().bold()),
        Span::raw(": Sel/Start"),
    ]);
    f.render_widget(
        Paragraph::new(help)
            .alignment(Alignment::Center)
            .bg(Color::DarkGray),
        chunks[2],
    );
}
