# ui-check

Drives the real Baytar window (not the browser mock) through WebView2's remote debugging port and the Chrome
DevTools Protocol. Windows only, Node 24+, no npm packages. Adapted from the R6Koc tool of the same name.

## Run

```powershell
# build once: in app/, npm run tauri build -- --debug --no-bundle
python tools/ui-check/make-legacy-db.py <scratch>/legacy-synthetic.db   # synthetic PatientsCare file
node tools/ui-check/run.mjs --scenario e2e-import --exe <target>/debug/baytar-app.exe
```

Options: `--scenario <name>` (file in `scenarios/`), `--port` (default 9333), `--exe <path>` (otherwise
`npm run tauri dev`), `--startup-timeout <seconds>`. Exit code 0 = pass, 1 = a step failed (`failure.png` saved).

The tool isolates the WebView profile, but the app still uses the normal app data folder
(`%APPDATA%\com.kutbay.baytar`). Run scenarios that import or write data only against an empty folder on a
development machine, never on a machine with real records. The `e2e-import` scenario expects an empty folder and
the synthetic file path written in the scenario; edit the path for your machine.

Step types: `wait` (`selector`/`text`), `click`, `eval` (`expr`, optional `expect`, promises awaited), `screenshot`,
`reload`, `localStorage.clear`. Output goes to `tools/ui-check/out/` (gitignored).
