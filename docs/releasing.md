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

## Optional updater signatures

Set repository Actions secrets `TAURI_SIGNING_PRIVATE_KEY` and, if the key is
password-protected, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`. Keep the private key
outside Git and back it up securely; future updater releases must use the same
key. Generate a key with Tauri's CLI (`npm.cmd run tauri signer generate -- --help`
shows options). These signatures are separate from Windows/macOS code signing.

The workflow passes a temporary inline config override that enables
`bundle.createUpdaterArtifacts` only when the private-key secret is nonempty.
Without the secret it explicitly disables updater artifacts, so ordinary
unsigned installers still build successfully. An invalid key or password fails
the signed build rather than silently falling back. `includeUpdaterJson` stays
enabled in tauri-action; without signatures it skips uploading `latest.json`.
Usable signed updater metadata requires signed artifacts. No updater plugin,
endpoint, or public-key config is added yet. See the action's
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
