# Worker brief

You are a worker on Baytar. Read `AGENTS.md` first; its rules bind you. The orchestrator gives you one task:
an issue ID, the files you may touch, and acceptance criteria.

## How to work
- You are inside a git worktree on branch `task/<issue>`. Work only there and only on the files your task names.
  If you must touch another file, keep the change minimal and say so in your report.
- Do not run `bd`, `git push`, `git rebase`, `git merge` or anything that changes other branches.
- Do not install global tools. Project dependencies via `cargo` and `npm` are fine.
- Run the checks your task lists (usually `cargo test -p baytar-core`, `npm run check` in `app/`). Fix what fails.
- Finish with exactly one commit on your branch: `<type>(<scope>): <summary>` in English, e.g.
  `feat(core): schema v1 and migration runner`. Add the line `Refs: <issue>`.
- If you are blocked, stop and explain why instead of guessing at data, credentials or product decisions.

## Report (your final message)
Plain text, short:
1. What you changed (files).
2. How you verified it (commands and results).
3. Anything the orchestrator must know: deviations, open questions, follow-up work.

## Quality bar
- Rust: `cargo fmt`, no `unwrap()` on user data paths, errors with `thiserror` in core, `anyhow` only in the CLI.
- TypeScript: strict, no `any` unless unavoidable; Svelte 5 runes.
- UI text in Turkish via `app/src/lib/i18n/tr.ts`; identifiers and comments in English.
- No real personal data anywhere. Synthetic fixtures only.
