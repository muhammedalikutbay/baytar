# Baytar

Free, open-source (GPL-3.0), offline-first desktop app for independent veterinarians and small clinics:
owners, patients, visits, vaccinations, medications with a dose calculator, and an appointment book.
Runs on Windows, macOS and Linux. Data stays on the vet's computer. It replaces PatientsCare, a
Windows Forms app (tag `v0.1-winforms`, branch `legacy`) whose SQLite data must be imported without loss.

Planning notes (Turkish) live in the owner's Obsidian vault:
`C:\Users\Kutbay\Desktop\ObsidianVault\Projeler\PatientsCare\AI\02-plan.md`.

## Stack
- Tauri 2 shell, Rust backend, SvelteKit (static adapter) + Svelte 5 + TypeScript + Vite frontend in `app/`.
- Tailwind CSS v4 with design tokens (`docs/design-language.md`), bits-ui primitives, `@lucide/svelte` icons.
- Rust Cargo workspace: `crates/baytar-core` (pure logic, all data access), `crates/baytar-cli` (dev CLI),
  `app/src-tauri` (thin command layer over `baytar-core`).
- SQLite via `rusqlite` (`bundled`), schema versions with `PRAGMA user_version` and `rusqlite_migration`.

## Layout
```
crates/baytar-core/   domain, schema + migrations, backups, repositories, legacy import, dose calc
crates/baytar-cli/    inspect (aggregates only), import --dry-run, backup, migrate
app/                  SvelteKit frontend; app/src-tauri is the Tauri crate
docs/                 design-language.md, data-model.md, agent-brief.md
tools/                build and check scripts
```

## Rules
- Code, identifiers, comments, commit messages and docs in `docs/` are English. The UI text is Turkish and
  lives in one place (`app/src/lib/i18n/tr.ts`) so another language can be added later.
- Data safety comes first. Never write to a legacy PatientsCare database: copy it first. Every schema
  migration takes a backup first and runs in a transaction. Nothing is hard-deleted from the UI (`deleted_at`).
- Real veterinary data is personal data. It lives in `C:\Users\Kutbay\Desktop\Baytar-veri\` and never enters
  git, test fixtures, logs or AI prompts. Fixtures are synthetic.
- `main` is the only long-lived branch. Workers run in git worktrees (`C:\Users\Kutbay\Desktop\Baytar-wt\<issue>`,
  branch `task/<issue>`), make one commit, and touch only the files their task names. The orchestrator
  reviews, rebases and fast-forwards into `main`.
- Task tracking is Beads (`bd`, prefix `bay`). Only the orchestrator writes to Beads. Workers never run `bd`.
- Verification stays light but real: `cargo test` for logic, `npm run check` for the frontend, a build when
  the task touches the shell. Migrations, backups, legacy import and dose math always have tests.
- Build on Windows from PowerShell: Git Bash's `link.exe` shadows MSVC's. Never kill processes by image name.
- Log where it helps (`log` crate in Rust, `console` in the frontend). No telemetry, no accounts, no server.
- Unknown or malformed legacy values never crash anything: keep the raw value and report it.
- Design: plain and calm. White or black theme (light/dark/system). Turquoise and orange are accents only
  (text, icons, thin lines, small badges), never large fills. Use tokens, never raw hex in components.
