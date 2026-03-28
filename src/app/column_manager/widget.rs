use super::service::ColumnManagerItem;

/// Temporary TUI overlay state for the column manager interaction.
/// Created by `ColumnManager::open_widget()`, lives while the overlay is open.
pub struct ColumnManagerWidget {
    pub table: String,
    pub items: Vec<ColumnManagerItem>,
    pub cursor: usize,
    pub search: String,
    pub search_active: bool,
    pub scroll: usize,
    pub confirmed: bool,
    pub closed: bool,
}

impl ColumnManagerWidget {
    pub fn new(table: String, items: Vec<ColumnManagerItem>) -> Self {
        Self {
            table,
            items,
            cursor: 0,
            search: String::new(),
            search_active: false,
            scroll: 0,
            confirmed: false,
            closed: false,
        }
    }

    /// Indices into `items` that match the current search filter.
    pub(crate) fn filtered_indices(&self) -> Vec<usize> {
        if self.search.is_empty() {
            (0..self.items.len()).collect()
        } else {
            let query = self.search.to_lowercase();
            self.items
                .iter()
                .enumerate()
                .filter(|(_, item)| item.name.to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect()
        }
    }

    /// The enabled column names in current order (result after confirm).
    pub fn visible_columns(&self) -> Vec<String> {
        self.items
            .iter()
            .filter(|i| i.enabled)
            .map(|i| i.name.clone())
            .collect()
    }

    /// All column names in current order (result after confirm).
    pub fn column_order(&self) -> Vec<String> {
        self.items.iter().map(|i| i.name.clone()).collect()
    }
}
