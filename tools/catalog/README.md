# Built-in pet clinical catalog

Edit the UTF-8 JSON files in `crates/baytar-core/catalog/`. Each file has `version: 1`,
`sources` (`id`, `title`, `url`) and `items`. This is the source-file format version;
the generated seed requires **database schema v2**. Every item has all its fields,
explicit `null` for unknown/inapplicable values and valid `source_ids`. Source
references remain in JSON and are not database columns.

From the repository root, using Python 3 (standard library only):

```powershell
python tools/catalog/build.py
python tools/catalog/check_seed.py
```

`py -3` can replace `python`; if neither is on PATH, invoke an installed Python 3
executable directly with these script paths. No third-party Python packages are needed.
To verify two independent builds byte for byte in PowerShell:

```powershell
python tools/catalog/build.py
$first = (Get-FileHash crates/baytar-core/src/db/migrations/0003_catalog_seed.sql).Hash
python tools/catalog/build.py
$second = (Get-FileHash crates/baytar-core/src/db/migrations/0003_catalog_seed.sql).Hash
if ($first -ne $second) { throw 'Catalog generation changed between builds' }
python tools/catalog/check_seed.py
```

Integrate the JSON and generated `0003_catalog_seed.sql` together. Do not hand-edit
SQL. Generation uses fixed column/catalog order, names sorted with Python
`str.lower`, UTF-8 and LF endings, without timestamps or machine paths. Validation
rejects missing/unexpected fields, invalid sources, duplicate lowercase names,
invalid enums, nonpositive/nonfinite numbers, fractional vaccine timing, invalid
prescription flags, missing dose bounds and reversed bounds.
`prescription_required` uses `1`, `0`, or `null`, not JSON booleans.

## Pet scope and schema contract

Vaccine species are `cat, dog, bird, rabbit, rodent, ferret, reptile, other`, or
`null`. A null species is reserved for **dog-and-cat rabies**; it is not an
all-species wildcard. A cat-only rabies product retains `species: "cat"`.
Medicine `target_species` lists only `Köpek, Kedi, Tavşan, Kuş, Kemirgen, Gelincik,
Sürüngen`, separated by comma-space. The catalog contains no livestock-only
products or farm visit service. There is no `withdrawal_note` source/SQL column.
Eprinomectin, moxidectin and florfenicol remain only in actual pet combination
products (cat spot-ons and canine otic gel), not livestock formulations.

The round-two instructions supersede the older livestock species and withdrawal
column still described in `docs/v2-plan.md` in this worktree. The orchestrator must
merge the matching pet-only v2 schema before registering migration 0003.

The check builds a disposable in-memory DB from `0001_init.sql`, rebuilds the empty
vaccine table with the pet-only CHECK, and adds just the v2 seed columns/services
table. It does not depend on the parallel `0002_v2.sql` task. It verifies every SQL
value, repeat insertion, builtin flags, unpriced services, preservation of existing
user rows, replacement of soft-deleted names, quoting, SQLite integrity and foreign
keys. Rejection cases cover all four removed livestock species, mixed pet/livestock
medicine targets, the removed column and non-rabies null species. Clinical
regressions guard cat/dog fluralaner and meloxicam doses, opioid decimal units,
per-administration pimobendan, combinations and mg/g versus mg/ml. No clinic data
is opened. Migration registration and backup/transaction execution belong to the
orchestrator's migration-runner task.

## Clinical numbers and source interpretation

The vaccine schedule fields are editable **clinic starting defaults requested by
the product owner**, not a claim that every brand label or every WSAVA antigen
interval is identical. They cover a puppy/kitten primary course, not a catch-up
course for an adult. Core puppy series must finish at or after 16 weeks: three
four-weekly doses starting at six weeks do not achieve that, so notes explicitly
require adjustment/additional doses. Annual core reminders are local practice
defaults; WSAVA supports longer subsequent intervals for several core antigens.
L-containing dog combinations use the two-dose leptospirosis default, with a note
to complete the core-antigen series separately. Rabies notes state
`Yasal zorunluluk` and annual Turkish-practice reminders. Single-dose primary
courses have no primary interval; Puppy DP has no independent booster because it
is followed by the core course. Nobivac Puppy DP starts at six weeks; it is not the
different DP Plus product.

Product-specific exceptions are visible in notes: Biocan DHPPi+L's label interval
is 14–21 days rather than the requested 28-day default; Versifel FeLV starts at
nine weeks on its label rather than the eight-week class default. Versifel CVR
also requires its label age/primary course to be checked. FeLV combination rows
retain the requested three-dose cat-core default, while noting the two-dose FeLV
component. Coronavirus is retained as an editable catalog choice, with the WSAVA
recommendation against routine use stated explicitly. Rabbit and ferret schedules
use their own references, not WSAVA dog/cat guidance.

Medicine doses are **mg/kg per administration**. Notes identify route, frequency
and context; a loading dose, maintenance dose, daily total or infusion rate must
not be substituted for another. Brand rows use label values or a clearly named
standard clinical context; generic entries can use the requested textbook ranges.
Examples of deliberate distinctions:

- Baytril has its 5 mg/kg label dose; the generic dog enrofloxacin row has the
  requested 5–20 range and a cat retinal-toxicity warning.
- Cat Bravecto spot-on uses 40–94 mg/kg, versus 25–56 for dogs. Weight-band
  tablets/pipettes are selected from the label, not subdivided from a calculation.
- Cat oral meloxicam is 0.1 mg/kg initially then 0.05; the dog oral row is 0.2 then
  0.1. The shared injection row's numeric field is the dog dose and its note gives
  the separate feline protocol. Sedative premedication ranges are not monotherapy
  doses. Pimobendan's 0.2–0.3 is per dose, twice daily.
- Synulox/Kesium strength and dose both mean total amoxicillin plus clavulanate
  in a fixed 4:1 ratio. Other combinations list **each printed strength in notes**
  and leave ambiguous scalar strength/dose fields null (Cardalis, combination
  antiparasitics, ear products, Zoletil, Catosal).
- Isathal is 10 mg/g, not 10 mg/ml. IsoFlo is 100% w/w, not 100 mg/ml. Insulin
  and interferon use activity units. These values are not forced into mg/kg fields.
  Stock fluid concentrations do not define an infusion-volume prescription.

Antibiotics, NSAIDs, anaesthetics, sedatives, opioids, hormones, corticosteroids,
cardiac medicines and Apoquel/Atopica are flagged `1`. OTC Frontline Spot On,
Advantix and Seresto presentations and vitamin products are `0`. Prescription
antiparasitics retain `1`; supply status is not inferred merely from a spot-on
route. Uncertain flags remain null. These are catalog defaults for clinical
review, not a legal opinion on dispensing in every country.

Prefer Turkish manufacturer labels and Ministry records, then EMA/UPD and
European regulator product information; standard references support generic
clinical ranges. A European product record establishes identity, not current
Turkish stock or Turkish authorization. Manufacturer identifies the recognizable
product company; it is not necessarily the physical batch-release site or local
distributor. Generic entries have no manufacturer. Propoven and gabapentin stock
references are human presentations, explicitly not veterinary marketing claims.
Some older manufacturer URLs have been replaced with regulator leaflets; source
links may still require normal browser access when automated requests are blocked.

Services are editable procedure names without prices or VAT assumptions. Sources
establish procedure names, not a recommended protocol or fee. Kremasyon
yönlendirme records a referral, not an in-house cremation service.

## Round-two coverage and deliberate omissions

The catalog has **35 vaccines** (29 brands, 6 generics), **117 medicines**
(67 brand presentations, 50 generics), and **33 services**. The requested brand
family list contains more variants than fit a 35-vaccine target; representative
presentations were selected rather than treating similar names as interchangeable.

- Vaccine variants omitted to keep that target: Nobivac Ducat and RL; Vanguard
  Plus 5/L4 CV and R; standalone Eurican L4; Versican Plus DHPPi and DHPPi/L4;
  additional Canigen combinations. Their disease groups are covered by retained
  entries. Vanguard Plus 5/L4 has a Turkish Zoetis leaflet; no Turkish availability
  claim is made for the CV variant.
- [CaniLeish](https://www.ema.europa.eu/en/medicines/veterinary/EPAR/canileish)
  and [Fevaxyn Pentofel](https://www.ema.europa.eu/en/medicines/veterinary/EPAR/fevaxyn-pentofel)
  were omitted because their EU authorizations are withdrawn. No unspecified
  current Fevaxyn presentation was substituted.
- Medicine alternatives omitted: Enrotril (Baytril retained), Dexafort
  (Rapidexon retained) and Narketan (Ketasol-100 retained). Revolution Plus was
  not introduced as an EU/Turkish brand claim; the corresponding European
  Stronghold Plus presentation is included. Extra strengths/weight bands of
  retained lines are deliberately not enumerated.
- All livestock-only entries and livestock-only formulations were removed.
  No pet indication was inferred just to retain a former livestock row. Uncertain
  generic strengths, mg/animal prescriptions, infusion/activation-unit doses and
  ambiguous multi-ingredient mg/kg values remain null; null never means zero.

Numeric coverage (four clinical fields per vaccine/medicine; prescription flags
excluded):

| Catalog | Filled | Null | Detail of filled fields |
| --- | ---: | ---: | --- |
| Vaccines | 130 / 140 | 10 | age 35; primary doses 35; primary interval 26; booster 34 |
| Medicines | 210 / 468 | 258 | concentration 32; unit strength 34; dose minimum 72; dose maximum 72 |
| Total | **340 / 608** | **268** | Services have no source numeric fields |

Prescription flags are populated for 98/117 medicines; 19 remain null. Numeric
nulls include inapplicable fields (a tablet usually has no liquid concentration),
not only missing evidence. Build/check print the current counts so subsequent
edits can be reviewed without relying on this snapshot.
