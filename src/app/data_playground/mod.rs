mod key_handler;
mod module;
mod widget_dispatch;
pub mod widgets;

use std::path::PathBuf;

use crate::connection_manager::ConnectionManager;
use crate::engine;
use crate::rules;
use crate::engine::Engine;
use crate::ui::app::AppState;

pub enum TickResult {
    Continue,
    Suspend,
    Quit,
}

pub struct DataPlayground {
    pub state: AppState,
    pub engine: Engine,
    pub conn_mgr: ConnectionManager,
    pending_paths: Option<(rules::Rule, Vec<engine::TablePath>)>,
    history_file: Option<PathBuf>,
}
