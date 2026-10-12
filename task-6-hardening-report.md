
## Fix round (review 56276b6)

- Detached HEAD commits are protected by a dedicated snapshot ref and recover after reflog expiry plus `git gc --prune=now`.
- Oplog cleanup parses `refs/gitstream/<namespace>/oplog/<id>/...` correctly and removes refs outside the retained 50 operation IDs.
- Linked-worktree namespaces use injective hex encoding of canonical worktree Git directories, avoiding basename sanitization collisions while keeping common-dir isolation.
- Restore preflight permits existing directories for gitlink (`160000`) entries; a focused temporary repository test covers this.
- `useOplog` consumes `OpSummary.eligible`, guards refresh responses against repository switches, and avoids refreshing after undo/redo when the repository changed. Added two Vitest tests for eligibility and response races.

Fix-round validation:

- `cargo test --manifest-path src-tauri/Cargo.toml hardening_ -- --test-threads=1` — 9/9 passing.
- `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1` — 120/120 passing.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings` — passing.
- `npm test -- --run` — 8 files, 59 tests passing.

UI follow-up: `OplogPanel.vue` now hides Undo for summary entries with `eligible: false`; `useOplog` continues to guard stale repository responses. `npm run build` passed after this adjustment.
