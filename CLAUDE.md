# LatticeQL — Agent Orientation

## Purpose

LatticeQL is a terminal-based tool for navigating relational datasets **without writing SQL**. Given a live database connection, it discovers the schema automatically and lets you build hierarchical views by declaring rules: "load these rows", "attach their related records", "drop nodes matching this criterion". The result is an interactive, collapsible tree rendered in the terminal.

The central insight is that relational data already has structure — foreign keys define parent-child relationships — and a good UI should make that structure explorable in real time rather than forcing the user to compose JOIN queries. LatticeQL also handles polymorphic (Rails-style) associations that lack formal FK constraints through user-defined **virtual foreign keys**.

## Tech Stack

- **Language**: Rust (async, Tokio)
- **TUI**: Ratatui 0.29 + Crossterm 0.28
- **Databases**: MySQL and SQLite3 via `sqlx`
- **Config**: Jsonnet (evaluated by `rs-jsonnet`)
- **CLI args**: Clap

## Source Layout

```
src/
  main.rs                — thin entry point: CLI args, terminal setup, draw/tick/suspend loop
  engine/
    core.rs              — Engine, DataNode tree, rule execution, DB queries
    paths.rs             — PathStep/TablePath types, IDDFS path-finding, edge discovery
  rules.rs               — rule parser/grammar, tokenizer, completion hints
  schema.rs              — schema discovery, virtual FK definitions
  config.rs              — config file loading and merging
  log.rs                 — thread-safe log queue
  connection_manager.rs  — multi-connection manager, merged schema
  db/
    mod.rs               — Database trait, Value/Row/TableInfo types
    sqlite.rs            — SQLite backend
    mysql.rs             — MySQL backend (uses information_schema)
  app/
    tui/                 — generic TUI toolkit (Command pattern for input handling)
      keys.rs            — FocusLoci, UserKeyEvent, from_key_event(), snapshot test
      control_panel.rs   — ControlPanel trait, dispatch()
      render.rs          — centered_rect(), render_search_bar() utilities
    data_playground/     — core app logic (owns engine, state, connections)
      mod.rs             — module declarations + re-exports
      types.rs           — DataPlayground, TickResult, Mode, ConfirmAction, PALETTE_COMMANDS
      state.rs           — AppState struct + methods
      module.rs          — new(), tick(), render(), handle_key(), helpers
      render.rs          — all Ratatui render functions
      key_handler.rs     — mode-based key dispatch (Normal, Query, CommandSearch, etc.)
      widget_dispatch.rs — overlay widget dispatch chain
      widgets/           — ErrorInfoWidget, ConfirmWidget
    column_manager/      — column visibility overlay
    connection_manager/  — connection manager overlay (tabs, add form, alias prompt)
    log_viewer/          — log viewer overlay
    manuals_manager/     — manuals browser overlay (list + viewer)
    query_rules_manager/ — rule reorder overlay (with undo/redo)
    virtual_fk_manager/  — virtual FK manager overlay (list + wizard form)
    model.rs             — SchemaNode, ColumnDef (DB-agnostic schema types)
manuals/                 — in-app help manuals (embedded at compile time)
samples/                 — example SQL files and pre-built SQLite databases
testing/                 — testing guides (KEYS_EXPERIENCE.md)
```

## Architecture

### Command Pattern for Input Handling

All keyboard input flows through a three-layer Command pattern:

1. **`from_key_event(key, focus)`** (`tui/keys.rs`) — translates raw crossterm `KeyEvent` into semantic `UserKeyEvent` based on `FocusLoci` (2D focus state: `InputFocus` × `EntityFocus`)
2. **`dispatch(ctrl_panel, event)`** (`tui/control_panel.rs`) — exhaustive match routes `UserKeyEvent` to `on_*()` trait methods
3. **`ControlPanel` impl** — each widget handles only the events it cares about (default no-ops for the rest)

### Widget/Overlay Pattern

Each overlay module follows the same structure:

```
app/<module_name>/
  mod.rs            — module declarations
  widget.rs         — widget struct (data), FocusLoci field
  control_panel.rs  — impl ControlPanel (input handling) + tests
  render.rs         — render function (display)
```

**Key principles:**
- Widget owns its state (cursor, search, focus)
- `focus_loci()` returns `FocusLoci` stored on the widget — single source of truth
- Control panel handles all input; widget is pure data
- Render reads widget state; only writes `viewport_height` for scroll clamping
- Side effects use action enums (e.g., `ConnManagerAction`, `VfkAction`) — the widget produces an action, the app layer performs async/IO work

### DataPlayground

`DataPlayground` is the top-level application struct owning `AppState`, `Engine`, and `ConnectionManager`. It provides:
- `new()` — async initialization (config, connections, schema)
- `tick()` — drain logs, poll events, handle keys → returns `TickResult`
- `render()` — delegates to the render module

`main.rs` is a thin shell (~70 lines): parse args, create playground, terminal setup, `loop { draw, tick, suspend }`, teardown.

## Key Concepts

**Rules** are the user's instructions to the engine. Three kinds:
- `Filter` — load rows matching a WHERE clause as root nodes
- `Relation` — follow FK links and attach related rows as children
- `Prune` — remove matching nodes from the tree in-memory

**DataNode** is a tree node: table, row data (`HashMap<String, Value>`), children. Flattened to `Vec<(depth, &DataNode)>` for rendering.

**Schema** is discovered at startup: table names, columns, FK constraints, virtual FKs. Path-finding (IDDFS, max depth 10) produces `TablePath` objects for JOIN-equivalent queries.

**Mode** drives core TUI branching: `Normal`, `Query`, `CommandPalette`, `CommandSearch`, `PathSelection`. Overlays are `Option<Widget>` fields on `AppState`, not Mode variants.

## Adding a New Overlay

1. Create `src/app/<name>/` with `mod.rs`, `widget.rs`, `control_panel.rs`, `render.rs`
2. Widget struct holds state + `focus: FocusLoci` + `closed: bool`
3. Implement `ControlPanel` trait — `focus_loci()` returns `self.focus`, handle relevant `on_*()` events
4. Add `Option<Widget>` field to `AppState`
5. Add dispatch block in `widget_dispatch.rs`
6. Add render call in `data_playground/render.rs`
7. Add command palette entry in `PALETTE_COMMANDS`

## Testing

Run `cargo test`. 138 tests across modules:
- Engine integration tests (SQLite-backed) in `engine/core.rs`
- Rule parser tests in `rules.rs`
- Schema tests in `schema.rs`
- Key mapping snapshot test in `tui/keys.rs`
- ControlPanel tests in each widget's `control_panel.rs`

## Config File

Searched upward from `cwd` then from `~/.latticeql/config.jsonnet`. Format is Jsonnet:

```jsonnet
{
  columns: {
    default: ["id", "name"],
    tables: { users: { default: ["id", "email"] } }
  },
  virtual_fks: [
    {
      from_table: "comments", type_column: "commentable_type",
      type_value: "Post", id_column: "commentable_id",
      to_table: "posts", to_column: "id"
    }
  ]
}
```

## Manuals

In-app documentation lives in `manuals/`. Access via command palette `:m`. Embedded at compile time via `include_str!`.
