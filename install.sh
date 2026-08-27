#!/usr/bin/env bash
# Deploys a COPY to ~/.local/bin, never a symlink into target/: a `cargo clean`
# or a rebuild in progress must not be able to break the live command.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

# The only lines that differ between tool repos. PKG is the crate name and it
# keys the receipt, so it has to survive a binary rename -- otherwise the old
# receipt is orphaned and the old binary along with it.
PKG="latticeql"
BIN="lql"

INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
STATE_DIR="${XDG_STATE_HOME:-$HOME/.local/state}/$PKG"
RECEIPT="$STATE_DIR/receipt"

command -v cargo >/dev/null 2>&1 || {
  echo "error: cargo not found. install Rust: https://rustup.rs" >&2
  exit 1
}

# Not `cargo install --path`: that ignores Cargo.lock unless given --locked, so
# it can ship dependency versions nothing was ever tested against, and it keeps
# its own bookkeeping under ~/.cargo -- a second registry of truth, which is
# exactly how two copies of this binary at different commits once ended up on
# one PATH. -p pins the root package so a workspace member cannot be dragged in.
echo "==> building"
cargo build --release -p "$PKG"

# A rename leaves the old name live on PATH at a stale commit forever, with
# nothing remaining that would ever update it. The receipt remembers what the
# last install put down, so clean it up here rather than warn about it elsewhere.
if [ -f "$RECEIPT" ]; then
  old_bin="$(sed -n 's/^bin=//p' "$RECEIPT")"
  if [ -n "$old_bin" ] && [ "$old_bin" != "$INSTALL_DIR/$BIN" ] && [ -e "$old_bin" ]; then
    rm -f "$old_bin"
    echo "==> removed renamed binary: $old_bin"
  fi
fi

mkdir -p "$INSTALL_DIR" "$STATE_DIR"
install -m 0755 "target/release/$BIN" "$INSTALL_DIR/$BIN"

# Which clone deployed this? Any checkout anywhere can run this script and
# become the live version, so record the source. Best-effort, because it must
# keep working from a tarball with no .git. No timestamp, so re-running
# converges byte-for-byte; uninstall.sh and the rename cleanup above read it.
commit="$(git -C "$HERE" rev-parse --short HEAD 2>/dev/null || echo unknown)"
dirty=no
[ -n "$(git -C "$HERE" status --porcelain 2>/dev/null)" ] && dirty=yes
printf 'bin=%s\nsource=%s\ncommit=%s\ndirty=%s\n' \
  "$INSTALL_DIR/$BIN" "$HERE" "$commit" "$dirty" > "$RECEIPT"

echo "==> installed: $INSTALL_DIR/$BIN ($commit$([ "$dirty" = yes ] && echo -dirty))"

# Advise, never edit: this repo does not own anyone's shell rc. On a dev-setup
# machine the entry is already there; anywhere else the hint is the fix.
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo
    echo "note: $INSTALL_DIR is not on your PATH. Add to your shell rc:"
    echo "    export PATH=\"$INSTALL_DIR:\$PATH\""
    ;;
esac
