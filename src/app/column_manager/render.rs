use ratatui::{
    layout::Rect,
    style::{Color, Style},
    widgets::{Block, Borders, Clear, List, ListItem},
    Frame,
};

use super::widget::ColumnManagerWidget;
use crate::ui::model::render::{centered_rect, render_search_bar};

pub fn render(f: &mut Frame, panel: &mut ColumnManagerWidget) {
    let area = centered_rect(50, 70, f.area());
    f.render_widget(Clear, area);

    // Reserve last row for search bar
    let has_search = panel.search_active || !panel.search.is_empty();
    let list_area = if has_search {
        Rect { height: area.height.saturating_sub(3), ..area }
    } else {
        area
    };

    let inner_height = list_area.height.saturating_sub(2) as usize;

    // Apply search filter — cursor is index into filtered list
    let q = panel.search.to_lowercase();
    let filtered: Vec<(usize, &super::service::ColumnManagerItem)> = panel.items.iter()
        .enumerate()
        .filter(|(_, it)| q.is_empty() || it.name.to_lowercase().contains(&q))
        .collect();

    // Clamp scroll: scroll only when cursor leaves visible window
    if panel.cursor < panel.scroll {
        panel.scroll = panel.cursor;
    } else if panel.cursor >= panel.scroll + inner_height {
        panel.scroll = panel.cursor + 1 - inner_height;
    }
    let offset = panel.scroll;

    let list_items: Vec<ListItem> = filtered
        .iter()
        .enumerate()
        .skip(offset)
        .take(inner_height)
        .map(|(fi, (_, col))| {
            let marker = if col.enabled { "[x]" } else { "[ ]" };
            let item = ListItem::new(format!("{} {}", marker, col.name));
            if fi == panel.cursor {
                item.style(Style::default().bg(Color::Green).fg(Color::Black))
            } else {
                item
            }
        })
        .collect();

    let match_info = if !panel.search.is_empty() {
        format!("  ({} matches)", filtered.len())
    } else {
        String::new()
    };
    let reorder_hint = if panel.search.is_empty() { "  u/d reorder" } else { "" };
    let list = List::new(list_items).block(
        Block::default()
            .title(format!(
                " Columns for '{}'{} (↑↓ nav · space/x toggle{}· /search · Enter apply · Esc) ",
                panel.table, match_info, reorder_hint
            ))
            .borders(Borders::ALL),
    );
    f.render_widget(list, list_area);

    // Search bar
    if has_search {
        let search_area = Rect { y: list_area.y + list_area.height, height: 3, ..area };
        render_search_bar(f, search_area, &panel.search, panel.search_active);
    }
}
