# Changelog

All notable changes to Baytar. Versions follow [Semantic Versioning](https://semver.org/); before 1.0 a minor
version may change the database schema (Baytar migrates it automatically and backs it up first).

## 0.4.0 — 2026-11-26

First public release of the rewrite.

### Added
- Patient picker that lists owners with their pets: everything is expanded when the practice is small, owners
  collapse behind an arrow when it grows, and an optional search filters as you type. Used for visits,
  vaccinations, appointments, charges and payments.
- Type-to-search lists for vaccines, medicines and services from the built-in catalog.
- Time field that opens a quarter-hour list when clicked and accepts quick typing (`930` → 09:30).
- "New patient" on the home and patients pages; after a new owner is saved Baytar offers to add their pets.
- Patient page tabs: clinical history, vaccines (schedule per vaccine, give and plan), appointments, files;
  an "upcoming" summary and a labelled weight chart.
- Agenda month and list views; appointment titles are filled in from the type and the patient.
- Medicines page filters (category, species, built-in, active) and a dose calculator that can take the
  patient's latest weight.
- Settings sections with their own navigation, a visible clinic logo upload (the logo replaces the app logo
  inside Baytar), and WhatsApp modes (automatic, desktop app, WhatsApp Web) with a test message.
- `BAYTAR_DATA_DIR` to keep the data folder somewhere else.

### Changed
- Larger labels and more compact fields with icons; required fields are explained on every form.
- The owner page shows the pets first; the window bar no longer repeats the page name.
- Phone numbers are shown as `+90 555 111 22 33`.

### Fixed
- Searching by an owner's name now returns all of the owner's pets.
- The visit history no longer mixes in future appointments.

## 0.3.0 — 2026-11-04 (not published)

Practice features: accounts and receivables, stock with lots and expiry dates, reminders centre, reports with
CSV export, printable documents, vitals and visit templates, attachments, built-in pet catalog (35 vaccines,
117 medicines, 33 services), optional PIN lock, new visual identity and custom window bar. Pets only.

## 0.2.0 — 2026-11-04 (not published)

The Tauri and Rust rewrite of PatientsCare: owners, patients, visits, vaccinations, medications with a dose
calculator, agenda, daily backups and lossless import from the old Windows Forms app.

## 0.1 — 2025

PatientsCare, the original Windows Forms app (tag `v0.1-winforms`).
