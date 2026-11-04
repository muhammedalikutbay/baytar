# Baytar v2 plan

v1 rebuilt the old app faithfully. v2 makes Baytar a full practice tool for independent vets and small clinics,
modelled on what paid practice-management systems offer (ezyVet, Provet Cloud, Digitail, Shepherd, Turkish
desktop programs), minus anything that needs a server or an official integration. Everything stays offline,
single-user, free.

Product owner feedback that drives this plan (2026-11-04):
1. Custom window bar inside the app design, no native Windows title bar.
2. Theme: a small two-state light/dark toggle in the sidebar instead of the large appearance block. First start
   follows the OS theme.
3. Built-in catalog of the vaccines and medicines commonly used in Turkish clinics.
4. Vaccines page: add a vaccination and plan a future one directly from there.
5. Do not stay bound to the old app; follow the logic of standard practice software.
6. Proper input validation and formatting: international phone input (default +90), email, numbers.
7. Notice and add what is missing; do not only fix reported issues.
8. More modern, premium look: subtle grid and gradients like Menura, clear visual hierarchy, colour that guides.
9. One logo everywhere: app, window, installer, `.exe`.
10. Review the font; use turquoise and orange more.

## 1. Visual identity (task: identity)

- **Font:** Plus Jakarta Sans Variable (`@fontsource-variable/plus-jakarta-sans`, offline, full Turkish
  coverage). Use `font-feature-settings: "tnum"` on numbers in tables, stats and money. Headings 600-700 with
  slight negative tracking; body 400-500 at 14 px.
- **Colour roles** (both themes, keep WCAG 4.5:1 for text):
  - Turquoise = primary. Primary buttons are filled turquoise (`--color-primary`, white text in light, near-black
    text in dark). Active nav item: turquoise soft background + turquoise icon + 3 px left bar. Links, focus ring,
    selected tabs, "today" markers, icon chips of clinical sections.
  - Orange = attention and secondary emphasis: due/overdue badges, debt amounts, low-stock and expiry warnings,
    icon chips of money/stock sections, one highlighted stat per dashboard.
  - Ink (black/white) stays the neutral base: page text, secondary buttons are surface + hairline.
  - Soft tints (`color-mix` 10-14 %) for chips, badges, card header strips. No large saturated fills except the
    primary button and the logo tile.
- **Background:** canvas with a faint 32 px grid (border colour, ~45 % opacity) masked by a radial gradient from
  the top, plus two very soft blurred glows (turquoise top-left, orange top-right, opacity <= 0.12 light,
  <= 0.18 dark). Content cards stay solid surface on top.
- **Hierarchy:** every page has a header row: title + one-line subtitle + the page's primary action on the right
  (turquoise). Cards have a header with an icon chip (tinted circle) + title + optional count badge + "see all"
  link. Stat tiles: large tabular number, label, small trend/hint line, coloured icon chip.
- **Logo:** a rounded-square tile with a turquoise gradient (#2DD4BF -> #0F766E, 135deg), a white paw print, and a
  small orange (#F59E0B -> #EA580C) plus sign inside the main pad. Files: `app/src-tauri/icons/app-icon.svg`
  (1024 artboard), regenerated icon set via `npx.cmd tauri icon app/src-tauri/icons/app-icon.svg`, the same SVG
  as an inline component `app/src/lib/components/brand/Logo.svelte` (sizes 20/28/40) and as favicon.
- **Window bar:** Windows and Linux: `decorations: false`; macOS: `titleBarStyle: "Overlay"` + `hiddenTitle: true`
  through `tauri.macos.conf.json` (native traffic lights stay). Custom 40 px bar spanning the window:
  logo + "Baytar" on the left over the sidebar, page breadcrumb, global search trigger (Ctrl+K) centred,
  window buttons on the right (minimise, maximise/restore, close; close turns red on hover). Whole bar is a drag
  region (`data-tauri-drag-region`), double-click toggles maximise. Capabilities: `core:window:allow-minimize`,
  `allow-toggle-maximize`, `allow-close`, `allow-start-dragging`, `allow-is-maximized`. In the browser mock the
  buttons render but do nothing.
- **Theme toggle:** compact 2-segment pill (sun / moon) at the sidebar bottom, 28 px high. No stored preference
  -> follow `prefers-color-scheme` live. After a click, store `light`/`dark` and stop following the OS. Settings
  page keeps a "Sistem temasını kullan" reset link.

## 2. Schema v2 (task: schema-v2, migration `0002_v2.sql`, user_version 2)

Money is stored as INTEGER minor units (kuruş) in `*_minor` columns. All new user-facing tables get the four
shared audit columns (`id`, `created_at`, `updated_at`, `deleted_at`) and STRICT, like v1.

**Species** (patients.species and vaccine_types.species CHECK, table rebuild with `PRAGMA defer_foreign_keys`):
`cat, dog, bird, rabbit, rodent, ferret, reptile, horse, cattle, sheep, goat, other`.

**Changed tables**
- `owners`: `phone` and `phone_alt` now hold E.164 (`+905551112233`). Migration converts `^0\d{10}$` to
  `+90` + 10 digits and `^\d{10}$` starting with 5 to `+90` + digits; anything else stays as is. New:
  `national_id TEXT` (TCKN or VKN), `city TEXT`, `district TEXT`.
- `patients`: new `alerts TEXT` (allergies, aggression, chronic conditions; shown as a warning banner).
- `visits`: new `heart_rate_bpm INTEGER`, `resp_rate_bpm INTEGER`, `body_condition INTEGER CHECK 1..9`.
- `vaccine_types`: new `is_builtin INTEGER 0/1 NOT NULL DEFAULT 0`, `manufacturer TEXT`, `diseases TEXT`,
  `primary_doses INTEGER`, `primary_interval_days INTEGER`, `min_age_weeks INTEGER`.
  `default_interval_days` keeps meaning "booster interval".
- `medications`: new `is_builtin INTEGER 0/1 NOT NULL DEFAULT 0`, `category TEXT CHECK` (`antibiotic, nsaid,
  analgesic, antiparasitic, anesthetic, sedative, corticosteroid, antiemetic, gastro, cardio, dermatologic,
  ophthalmic, otic, vitamin, fluid, hormone, antifungal, antiviral, other`), `manufacturer TEXT`,
  `target_species TEXT` (display text, e.g. "Köpek, Kedi"), `withdrawal_note TEXT` (food-animal withdrawal
  period), `prescription_required INTEGER 0/1/NULL`, `pack_unit TEXT`.
- `appointments`: new `vaccine_type_id INTEGER FK vaccine_types`, `vaccination_id INTEGER FK vaccinations`
  (set when a planned vaccination is given).

**New tables**
- `services`: `name` req, `category TEXT` (`exam, vaccination, surgery, lab, imaging, grooming, boarding,
  hospitalisation, other`), `price_minor INTEGER >= 0` nullable, `vat_rate INTEGER` (percent, nullable),
  `is_builtin`, `is_active`, `notes`.
- `inventory_items`: `name` req, `kind` CHECK (`medication, vaccine, supply, food, product, other`),
  `medication_id FK`, `vaccine_type_id FK`, `unit TEXT NOT NULL DEFAULT 'adet'`, `sale_price_minor`,
  `reorder_level REAL`, `is_active`, `notes`.
- `inventory_lots`: `item_id FK` req, `lot_no`, `expires_on` date, `quantity REAL NOT NULL DEFAULT 0`,
  `cost_minor`, `received_on` date, `notes`.
- `stock_movements`: `item_id FK` req, `lot_id FK`, `delta REAL NOT NULL` (non-zero), `reason` CHECK
  (`purchase, use, sale, adjust, expired, return`), `ref_table TEXT`, `ref_id INTEGER`, `moved_on` date req,
  `notes`. Lot quantity is kept in sync by the repository in the same transaction.
- `charges`: `owner_id FK` req, `patient_id FK`, `visit_id FK`, `service_id FK`, `inventory_item_id FK`,
  `description` req, `quantity REAL > 0`, `unit_price_minor INTEGER >= 0`, `discount_minor INTEGER >= 0 DEFAULT 0`,
  `vat_rate INTEGER`, `total_minor INTEGER` (= round(quantity * unit_price) - discount, computed by the repo),
  `charged_on` date req, `notes`.
- `payments`: `owner_id FK` req, `paid_on` date req, `amount_minor INTEGER > 0`, `method` CHECK (`cash, card,
  transfer, other`), `notes`.
- `attachments`: `patient_id FK` req, `visit_id FK`, `kind` CHECK (`photo, lab, imaging, document, other`),
  `title`, `file_name` req, `mime` req, `size_bytes INTEGER`, `sha256 TEXT`.
- `attachment_data`: `attachment_id INTEGER PRIMARY KEY FK attachments`, `data BLOB NOT NULL`. Files live inside
  the database so daily backups and restore stay one file. Limit 25 MB per file in the app.
- `visit_templates`: `name` req, `reason`, `findings`, `diagnosis`, `treatment`, `notes`.

Owner balance = sum of non-deleted `charges.total_minor` - sum of non-deleted `payments.amount_minor`.

The legacy importer must write E.164 phones from now on (`legacy::normalize`).

## 3. Catalog seed (task: catalog, migration `0003_catalog_seed.sql`)

Built-in rows (`is_builtin = 1`), inserted only where no non-deleted row with the same `lower(name)` exists,
so user entries are never duplicated or overwritten. Users can edit, deactivate or delete built-ins.
- Vaccines: about 40 entries covering dogs (combined DHPPi/L, leptospirosis, rabies, kennel cough, corona,
  leishmania), cats (FVRCP, FeLV, rabies, chlamydia), rabbits (RHD, myxomatosis), ferrets and the main food
  animal programmes in Turkey (foot-and-mouth, brucellosis, sheep and goat pox, PPR, enterotoxaemia, lumpy skin
  disease, anthrax, IBR/BVD, theileriosis). Brand names as sold in Turkey where known, plus generic entries.
- Medicines: about 120 entries across the categories above, active ingredient, form, typical concentration or
  strength, typical mg/kg range only where it is a standard textbook value, route, target species,
  prescription flag and withdrawal note for food-animal products. When unsure of a value, leave it NULL: the
  dose calculator already refuses to calculate without hekim-entered bounds.
- Services: about 25 common services without prices (examination, vaccination, microchip, neutering, dental
  scaling, blood test, x-ray, ultrasound, wound care, home visit, farm visit, boarding...).
- Source data lives in `crates/baytar-core/catalog/*.json` with a `sources` field; a small script
  (`tools/catalog/build.py`) generates the migration SQL so the data stays reviewable.

## 4. Features

| Task | Scope |
| --- | --- |
| inputs | `PhoneInput` (country select with flag + dial code, default TR, `libphonenumber-js` min metadata, as-you-type formatting, stores E.164, validates length/type), `EmailInput`, `NationalIdInput` (TCKN checksum / 10-digit VKN), `MicrochipInput` (15 digits ISO 11784, warns otherwise), `NumberInput` with unit suffix, min/max and Turkish decimal comma, `MoneyInput` (₺, kuruş), `DateInput`. Apply to every existing form. Display helpers format E.164 nationally for TR, internationally otherwise. WhatsApp links use E.164. |
| vaccines | Vaccines page gets "Aşı uygula" (pick patient via search, vaccine from catalog filtered by species, date, batch, next due auto-filled from catalog interval) and "Aşı planla" (patient, vaccine, date -> appointment kind `vaccine` with `vaccine_type_id`). Upcoming list merges planned appointments and `next_due_on` of last doses, deduplicated; each row has "Uygulandı" (creates vaccination, marks appointment done, links ids), "Ertele", "WhatsApp". Patient page vaccine tab shows a per-patient schedule based on catalog primary series and boosters. |
| billing | "Hesaplar" area: services price list page; charges added from a visit (services, medicines, vaccines with price) and standalone; payments; owner account statement (charges and payments over time, running balance); "Alacaklar" list of owners with positive balance, oldest debt age, WhatsApp debt reminder; daily cash summary. Money formatting tr-TR ₺. |
| inventory | "Stok" page: items, lots with expiry, quantity, reorder level; receive stock (purchase), adjust, write off expired; movements history; warnings for low stock and expiry within 60 days on the home page. Optional deduction when a vaccination or prescription line uses an inventory item. |
| clinical | Vitals on visits (HR, RR, BCS 1-9, temperature, weight), visit templates (pick to prefill), patient alerts banner, attachments and photos on patient and visit (stored in DB, preview images and PDFs, open externally via a temp copy), patient photo as avatar, deceased flow (mark deceased, hide from reminders). |
| documents | Clinic profile in settings (clinic name, vet name, diploma/registry no, address, phone, email, logo image). Print views with the clinic header: vaccination card, prescription, visit summary, account statement/receipt. Printed through a dedicated print route and `window.print()` (user can save as PDF). |
| reminders | "Hatırlatmalar" centre: due/overdue vaccines, tomorrow's appointments, debts, follow-ups; editable WhatsApp message templates with placeholders ({sahip}, {hasta}, {asi}, {tarih}, {tutar}, {klinik}); mark as contacted (stored so a reminder is not repeated the same day). |
| reports | Reports page: visits, new patients, revenue, collections, outstanding debt per month; species mix; top diagnoses and top used medicines; vaccine coverage. CSV export of owners, patients, visits, vaccinations, charges, payments. |
| home | Home page rebuilt on the new design: greeting + date, quick actions (Yeni sahip, Yeni muayene, Aşı uygula, Tahsilat al), today's agenda, due vaccines, alerts (debts, low stock, expiry), small stats. |
| lock | Optional app lock: 4-8 digit PIN (argon2 hash in settings), lock on start and after N idle minutes. Off by default. |

## 5. Order

Wave A (parallel): schema-v2 (Sonnet), identity (Sol high), catalog data (Sol medium), inputs (Sol medium).
Wave B (after schema merges): vaccines, billing, inventory, clinical, documents.
Wave C: reminders, reports, home, lock. Then orchestrator review, real-app end-to-end check, release 0.3.0.
