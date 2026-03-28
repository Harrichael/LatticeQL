use crossterm::event::{KeyCode, KeyEvent};

use super::widget::ColumnManagerWidget;
use crate::ui::model::control_panel::ControlPanel;

impl ControlPanel for ColumnManagerWidget {
    fn on_navigate_up(&mut self) {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    fn on_navigate_down(&mut self) {
        let filtered = self.filtered_indices();
        if self.cursor + 1 < filtered.len() {
            self.cursor += 1;
        }
    }

    fn on_move_item_up(&mut self) {
        let filtered = self.filtered_indices();
        if self.cursor > 0 && self.cursor < filtered.len() {
            self.items.swap(filtered[self.cursor], filtered[self.cursor - 1]);
            self.cursor -= 1;
        }
    }

    fn on_move_item_down(&mut self) {
        let filtered = self.filtered_indices();
        if self.cursor + 1 < filtered.len() {
            self.items.swap(filtered[self.cursor], filtered[self.cursor + 1]);
            self.cursor += 1;
        }
    }

    fn on_toggle_item(&mut self) {
        let filtered = self.filtered_indices();
        if self.cursor < filtered.len() {
            let idx = filtered[self.cursor];
            self.items[idx].enabled = !self.items[idx].enabled;
        }
    }

    fn on_remove(&mut self) {
        // x key in Editable context — same as toggle for column manager.
        self.on_toggle_item();
    }

    fn on_start_search(&mut self) {
        self.search_active = true;
    }

    fn on_back(&mut self) {
        if self.search_active {
            self.search_active = false;
        } else if !self.search.is_empty() {
            self.search.clear();
            self.scroll = 0;
            self.cursor = 0;
        } else {
            self.closed = true;
        }
    }

    fn on_confirm(&mut self) {
        self.confirmed = true;
        self.closed = true;
    }

    fn on_text_input(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.search.push(c);
                self.scroll = 0;
                self.cursor = 0;
            }
            KeyCode::Backspace => {
                self.search.pop();
                self.scroll = 0;
                self.cursor = 0;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::column_manager::service::ColumnManagerItem;

    fn items(names: &[&str]) -> Vec<ColumnManagerItem> {
        names
            .iter()
            .map(|n| ColumnManagerItem {
                name: n.to_string(),
                enabled: true,
            })
            .collect()
    }

    fn panel(names: &[&str]) -> ColumnManagerWidget {
        ColumnManagerWidget::new("test_table".into(), items(names))
    }

    #[test]
    fn navigate_clamps_to_bounds() {
        let mut p = panel(&["id", "name", "email"]);
        p.on_navigate_up(); // already at 0
        assert_eq!(p.cursor, 0);
        p.on_navigate_down();
        p.on_navigate_down();
        assert_eq!(p.cursor, 2);
        p.on_navigate_down(); // at end
        assert_eq!(p.cursor, 2);
    }

    #[test]
    fn navigate_respects_search_filter() {
        let mut p = panel(&["id", "name", "email"]);
        p.search = "a".into(); // matches "name"(1), "email"(2)
        p.on_navigate_down();
        assert_eq!(p.cursor, 1); // only 2 filtered items
        p.on_navigate_down();
        assert_eq!(p.cursor, 1); // can't go past end of filtered list
    }

    #[test]
    fn toggle_flips_enabled() {
        let mut p = panel(&["id", "name"]);
        assert!(p.items[0].enabled);
        p.on_toggle_item();
        assert!(!p.items[0].enabled);
        p.on_toggle_item();
        assert!(p.items[0].enabled);
    }

    #[test]
    fn remove_behaves_like_toggle() {
        let mut p = panel(&["id", "name"]);
        p.on_remove();
        assert!(!p.items[0].enabled);
    }

    #[test]
    fn toggle_targets_filtered_item() {
        let mut p = panel(&["id", "name", "email"]);
        p.search = "a".into(); // filtered: name(1), email(2)
        p.cursor = 1; // points to email in filtered view
        p.on_toggle_item();
        assert!(p.items[0].enabled); // id untouched
        assert!(p.items[1].enabled); // name untouched
        assert!(!p.items[2].enabled); // email toggled
    }

    #[test]
    fn move_item_reorders() {
        let mut p = panel(&["id", "name", "email"]);
        p.cursor = 1; // on "name"
        p.on_move_item_up();
        assert_eq!(p.items[0].name, "name");
        assert_eq!(p.items[1].name, "id");
        assert_eq!(p.cursor, 0);
    }

    #[test]
    fn move_item_down_reorders() {
        let mut p = panel(&["id", "name", "email"]);
        p.cursor = 1; // on "name"
        p.on_move_item_down();
        assert_eq!(p.items[1].name, "email");
        assert_eq!(p.items[2].name, "name");
        assert_eq!(p.cursor, 2);
    }

    #[test]
    fn three_level_back() {
        let mut p = panel(&["id"]);
        p.search_active = true;
        p.search = "foo".into();

        p.on_back(); // level 1: deactivate search
        assert!(!p.search_active);
        assert_eq!(p.search, "foo");
        assert!(!p.closed);

        p.on_back(); // level 2: clear search text
        assert!(p.search.is_empty());
        assert_eq!(p.cursor, 0);
        assert!(!p.closed);

        p.on_back(); // level 3: close
        assert!(p.closed);
    }

    #[test]
    fn confirm_sets_flags_and_results() {
        let mut p = panel(&["id", "name", "email"]);
        p.items[1].enabled = false; // disable "name"
        p.on_confirm();
        assert!(p.confirmed);
        assert!(p.closed);
        assert_eq!(p.visible_columns(), vec!["id", "email"]);
        assert_eq!(p.column_order(), vec!["id", "name", "email"]);
    }

    #[test]
    fn text_input_updates_search() {
        let mut p = panel(&["id", "name"]);
        p.search_active = true;

        let char_n = KeyEvent::new(KeyCode::Char('n'), crossterm::event::KeyModifiers::NONE);
        let backspace = KeyEvent::new(KeyCode::Backspace, crossterm::event::KeyModifiers::NONE);

        p.on_text_input(char_n);
        assert_eq!(p.search, "n");

        p.on_text_input(backspace);
        assert!(p.search.is_empty());
    }

    #[test]
    fn start_search_activates() {
        let mut p = panel(&["id"]);
        assert!(!p.search_active);
        p.on_start_search();
        assert!(p.search_active);
    }
}
