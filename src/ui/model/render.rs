use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

/// Render a search/filter bar with a blinking cursor when active.
pub fn render_search_bar(f: &mut Frame, area: Rect, query: &str, active: bool) {
    let (border_color, title) = if active {
        (Color::Yellow, " Search (Esc to stop typing, keep filter) ")
    } else {
        (Color::DarkGray, " Filter active (/ to edit, Esc to clear) ")
    };
    let text = Line::from(vec![
        Span::styled("/ ", Style::default().fg(Color::Yellow)),
        Span::styled(query, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
        if active { Span::styled("▌", Style::default().fg(Color::Yellow)) } else { Span::raw("") },
    ]);
    f.render_widget(
        Paragraph::new(text).block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(border_color)),
        ),
        area,
    );
}

/// Create a centered rectangle as a percentage of the parent area.
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
