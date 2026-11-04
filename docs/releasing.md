# Releasing Baytar

Release builds run on GitHub Actions and create a **draft** GitHub release. They
produce a per-user Windows NSIS installer, a universal macOS app and DMG (Apple
Silicon and Intel, macOS 11+), and Linux AppImage and deb packages. Debian
dependencies use Tauri's defaults. OS code signing is not configured yet.

## Cut a release

1. Start from the reviewed `main` branch with a clean working tree. Choose `X.Y.Z`.
2. From the repository root, run:

   ```powershell
   powershell -ExecutionPolicy Bypass -File tools/bump-version.ps1 -Version X.Y.Z
   ```

   The script updates exactly these three version sources, relative to its own
   location, preserving their formatting:
   - `Cargo.toml`: `[workspace.package].version` (the Tauri crate inherits this).
   - `app/package.json`: `version`.
   - `app/src-tauri/tauri.conf.json`: `version`.

   Supply a numeric version such as `0.3.0`, without the `v` prefix. The script
   validates all three files before writing. It does not update lockfiles: refresh
   `app/package-lock.json` with `npm.cmd install --package-lock-only` in `app`.
   The Windows build in the next step refreshes workspace versions in `Cargo.lock`.
3. Run `npm.cmd ci`, `npm.cmd run check`, and
   `npm.cmd run tauri build -- --bundles nsis` in `app` on Windows. Review the
   version and lockfile changes, then commit them to `main` through the normal
   review process.
4. Tag the release commit and push the tag:

   ```powershell
   git tag vX.Y.Z
   git push origin vX.Y.Z
   ```

5. Watch the **Release** workflow. To retry manually, use **Run workflow** and
   select the existing `vX.Y.Z` tag, not a branch. All matrix jobs upload to the
   same draft. Matrix jobs run one at a time to protect shared draft creation and
   updater metadata merges. Concurrent runs for the same tag are also serialized.
6. Review the draft's Turkish and English notes, version, and assets from all
   three platforms. Test installation and launch on each supported OS, then
   publish the draft when ready. Publication is a manual maintainer step.

## Automatic updates and signing

The app uses Tauri's updater and process plugins. It checks
`https://github.com/muhammedalikutbay/baytar/releases/latest/download/latest.json`
once, a few seconds after startup, unless `update_check_enabled` is `"0"`.
Automatic checks fail silently (only a generic error is logged); Settings →
**Güncellemeler** → **Şimdi denetle** checks manually even when startup checks
are disabled, and shows whether the app is current, an update exists, or the
network check failed. Browser development does not check or install updates.

An available update shows a small shell banner with its version and up to 300
characters of release notes as plain text. **Sonra** dismisses it for this session;
Settings still offers the update. **Şimdi güncelle** first creates a `pre-update`
SQLite backup in the app's backup folder. A failed backup stops the update and
shows an error. After a successful backup, the app downloads and installs the
signed update, shows progress, and restarts. Windows uses the passive installer
with installer-managed restart; macOS/Linux relaunch after installation returns.
Pre-update backups are retained by the existing backup policy (only daily backups
are pruned), and can be restored through Settings.

The public key is embedded in `app/src-tauri/tauri.conf.json`. The matching private
key lives only on the maintainer's machine and in the GitHub Actions repository
secret `TAURI_SIGNING_PRIVATE_KEY`; never put it in Git, logs, or release assets.
The key has an empty password: leave `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` unset or
empty. Keep a secure maintainer-controlled backup of the private key. Future
updater releases must use the same key. Losing it means existing users must
reinstall once from an installer containing the replacement public key before
automatic updates can work again. Updater signatures are separate from
Windows/macOS code signing. See [Tauri's updater documentation](https://v2.tauri.app/plugin/updater/).

The workflow passes a temporary inline config override that enables
`bundle.createUpdaterArtifacts` only when the private-key secret is nonempty.
Without the secret it explicitly disables updater artifacts, so ordinary
unsigned installers still build successfully. An invalid key or password fails
the signed build rather than silently falling back. `includeUpdaterJson` stays
enabled in tauri-action; without signatures it skips uploading `latest.json`.
Usable signed updater metadata requires signed artifacts. The committed config
does not set `bundle.createUpdaterArtifacts`; the release workflow decides it.
Publish the draft with its signed artifacts and `latest.json` to make the update
available to installed apps. See the action's
[updater metadata implementation](https://github.com/tauri-apps/tauri-action/blob/v0/src/upload-version-json.ts).

## Installation notes

- Windows: run the `*-setup.exe`. The installer uses the current user's account,
  without an administrator prompt. For unsigned builds, SmartScreen may require
  **More info → Run anyway**.
- macOS: open the DMG and drag Baytar to Applications. For the unsigned app's
  first launch, **right-click → Open**.
- Linux: make the AppImage executable (`chmod +x Baytar*.AppImage`) and run it,
  or install the deb using the distribution's package manager.

The workflow contains bilingual release notes with the same instructions.
