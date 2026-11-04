# Contributing to Baytar

Thank you for helping improve Baytar. Keep contributions focused, reviewable and safe for veterinary records.

## Before you start

- Check the existing [issues](https://github.com/muhammedalikutbay/baytar/issues) for related work. For a substantial change, open an issue first to discuss its scope.
- Work in a focused branch based on `main`, named `task/<issue>` when an issue exists (for example, `task/bay-123`). Do not commit directly to `main`.
- Keep changes within the files and scope of the issue. Do not commit; the project maintainer reviews and integrates contributions.

## Make a change

- Write code, identifiers, comments and engineering documentation in English. All user-facing interface text belongs in `app/src/lib/i18n/tr.ts` so it can be translated consistently.
- Follow the existing style and architecture. The Rust core owns data access; keep the Tauri command layer thin and the UI independent of Tauri where possible.
- Use synthetic data only in tests, screenshots and examples. Never put real veterinary or owner information in Git, fixtures, logs or prompts.
- Preserve unknown legacy values and report them. Malformed imported data should not crash the application.
- Do not add telemetry, accounts, servers or runtime network dependencies.

## Data safety

- Never write to a PatientsCare database. Copy it first, including its `-wal` and `-shm` files when present, and read the copy.
- Back up the Baytar database before imports, restores or schema migrations. Run each migration in a transaction and verify database integrity before it commits.
- Do not hard-delete records from the UI. Use the existing soft-delete behavior (`deleted_at`) so historical references remain available.
- Treat import previews and reports as potentially containing personal data. Keep them local and do not put raw values in logs or test fixtures.

## Checks

From the repository root, run the Rust workspace tests:

```sh
cargo test --workspace
```

From `app/`, run the frontend check:

```sh
npm ci
npm run check
```

If your change affects the Tauri shell or packaging, build it on the target platform as well. On Windows, use PowerShell for Cargo commands; Git Bash's `link.exe` can shadow the MSVC linker. Do not terminate processes by image name.

## Commits

Use the format `type(scope): summary`, with a short, imperative summary. Common types include `feat`, `fix`, `docs`, `refactor` and `test`; use a focused scope such as `import`, `ui` or `backup` when useful.

Examples:

```text
fix(import): preserve unknown date values
docs(readme): clarify backup retention
```

## Issues and review

Include the problem, expected behavior and steps to reproduce in a bug report. Do not attach real databases or screenshots containing personal records. For a feature request, explain the user need and any data implications. In a change proposal, summarize what changed, list the checks you ran and note any known limitations.
