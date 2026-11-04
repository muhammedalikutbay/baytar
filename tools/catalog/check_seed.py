#!/usr/bin/env python3
"""Exercise the generated SQL against an isolated, minimal schema-v2 SQLite DB."""

import copy
import hashlib
import sqlite3
import sys

sys.dont_write_bytecode = True
import build


def scratch_db():
    db = sqlite3.connect(":memory:")
    db.executescript((build.ROOT / "crates/baytar-core/src/db/migrations/0001_init.sql").read_text(encoding="utf-8"))
    # Empty v1 table rebuild expands the CHECK without relying on parallel migration 0002.
    db.executescript("""
        CREATE TABLE vaccine_types_v2 (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            species TEXT CHECK (species IN (
                'cat','dog','bird','rabbit','rodent','ferret','reptile',
                'other'
            )),
            default_interval_days INTEGER,
            notes TEXT,
            is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            deleted_at TEXT
        ) STRICT;
        DROP TABLE vaccine_types;
        ALTER TABLE vaccine_types_v2 RENAME TO vaccine_types;
        CREATE UNIQUE INDEX vaccine_types_name_idx ON vaccine_types(lower(name)) WHERE deleted_at IS NULL;
        ALTER TABLE vaccine_types ADD COLUMN is_builtin INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0,1));
        ALTER TABLE vaccine_types ADD COLUMN manufacturer TEXT;
        ALTER TABLE vaccine_types ADD COLUMN diseases TEXT;
        ALTER TABLE vaccine_types ADD COLUMN primary_doses INTEGER;
        ALTER TABLE vaccine_types ADD COLUMN primary_interval_days INTEGER;
        ALTER TABLE vaccine_types ADD COLUMN min_age_weeks INTEGER;
        ALTER TABLE medications ADD COLUMN is_builtin INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0,1));
        ALTER TABLE medications ADD COLUMN category TEXT CHECK (category IN (
            'antibiotic','nsaid','analgesic','antiparasitic','anesthetic','sedative',
            'corticosteroid','antiemetic','gastro','cardio','dermatologic','ophthalmic',
            'otic','vitamin','fluid','hormone','antifungal','antiviral','other'
        ));
        ALTER TABLE medications ADD COLUMN manufacturer TEXT;
        ALTER TABLE medications ADD COLUMN target_species TEXT;
        ALTER TABLE medications ADD COLUMN prescription_required INTEGER CHECK (prescription_required IN (0,1));
        ALTER TABLE medications ADD COLUMN pack_unit TEXT;
        CREATE TABLE services (
            id INTEGER PRIMARY KEY,
            name TEXT NOT NULL,
            category TEXT CHECK (category IN (
                'exam','vaccination','surgery','lab','imaging','grooming','boarding','hospitalisation','other'
            )),
            price_minor INTEGER CHECK (price_minor >= 0),
            vat_rate INTEGER,
            is_builtin INTEGER NOT NULL DEFAULT 0 CHECK (is_builtin IN (0,1)),
            is_active INTEGER NOT NULL DEFAULT 1 CHECK (is_active IN (0,1)),
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
            deleted_at TEXT
        ) STRICT;
        PRAGMA foreign_keys = ON;
    """)
    return db


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def counts(db):
    return {kind: db.execute(f"SELECT count(*) FROM {table}").fetchone()[0]
            for kind, table in build.TABLES.items()}


def check_validation(catalogs):
    cases = [
        ("vaccines", "species", "invalid"),
        ("vaccines", "species", "cattle"),
        ("vaccines", "species", "horse"),
        ("vaccines", "species", "sheep"),
        ("vaccines", "species", "goat"),
        ("vaccines", "primary_doses", 1.5),
        ("vaccines", "min_age_weeks", 0),
        ("medications", "category", "invalid"),
        ("medications", "form", "invalid"),
        ("medications", "target_species", "Köpek, Sığır"),
        ("medications", "target_species", "At"),
        ("medications", "concentration_mg_per_ml", float("inf")),
        ("medications", "strength_mg_per_unit", -1),
        ("medications", "prescription_required", True),
        ("services", "category", "invalid"),
        ("services", "source_ids", ["unknown"]),
    ]
    invalid = []
    for kind, field, value in cases:
        data = copy.deepcopy(catalogs[kind])
        data["items"][0][field] = value
        invalid.append((kind, data))
    for kind in build.FIELDS:
        data = copy.deepcopy(catalogs[kind])
        del data["items"][0]["notes"]
        invalid.append((kind, data))
        data = copy.deepcopy(catalogs[kind])
        data["items"].append(copy.deepcopy(data["items"][0]))
        data["items"][-1]["name"] = data["items"][0]["name"].translate(
            str.maketrans("abcdefghijklmnopqrstuvwxyz", "ABCDEFGHIJKLMNOPQRSTUVWXYZ")
        )
        invalid.append((kind, data))
    data = copy.deepcopy(catalogs["medications"])
    data["items"][0]["withdrawal_note"] = "Removed field"
    invalid.append(("medications", data))
    data = copy.deepcopy(catalogs["vaccines"])
    data["items"][0].update(species=None, diseases="Parvovirüs")
    invalid.append(("vaccines", data))
    data = copy.deepcopy(catalogs["medications"])
    data["items"][0].update(dose_mg_per_kg_min=10, dose_mg_per_kg_max=1)
    invalid.append(("medications", data))
    data = copy.deepcopy(catalogs["medications"])
    data["items"][0].update(dose_mg_per_kg_min=1, dose_mg_per_kg_max=None)
    invalid.append(("medications", data))
    for kind, data in invalid:
        try:
            build.validate(data, kind)
        except ValueError:
            continue
        raise AssertionError(f"Invalid {kind} fixture was accepted")
    print(f"Validation: rejected {len(invalid)} malformed synthetic catalogs")


def check_clinical_regressions(catalogs):
    """Guard unit and species distinctions that can silently change calculated doses."""
    medications = {row["name"]: row for row in catalogs["medications"]["items"]}
    examples = [
        ("Synulox 50 mg tablet", "strength_mg_per_unit", 50),
        ("Convenia 80 mg/ml enjeksiyon", "concentration_mg_per_ml", 80),
        ("Bravecto 250 mg spot-on (kedi)", "dose_mg_per_kg_min", 40),
        ("Bravecto 500 mg spot-on (köpek)", "dose_mg_per_kg_min", 25),
        ("Metacam 0,5 mg/ml oral süspansiyon (kedi)", "dose_mg_per_kg_min", 0.05),
        ("Vetmedin 2,5 mg kapsül", "dose_mg_per_kg_min", 0.2),
        ("Vetergesic 0,3 mg/ml enjeksiyon", "concentration_mg_per_ml", 0.3),
        ("IsoFlo %100 inhalasyon sıvısı", "concentration_mg_per_ml", None),
        ("Isathal 10 mg/g göz damlası", "concentration_mg_per_ml", None),
        ("Cardalis 2,5 mg/20 mg tablet", "dose_mg_per_kg_min", None),
    ]
    for name, field, expected in examples:
        require(medications[name][field] == expected, f"Clinical unit/species regression: {name} {field}")
    for row in medications.values():
        if row["category"] in {
            "antibiotic", "nsaid", "anesthetic", "sedative", "hormone", "corticosteroid", "cardio",
        }:
            require(row["prescription_required"] == 1, f"Missing prescription flag: {row['name']}")
    print("Clinical regressions: species-specific doses, combination and non-mg units preserved")


def main():
    catalogs = build.load_catalogs()
    require(30 <= len(catalogs["vaccines"]["items"]) <= 35, "Expected 30-35 pet vaccines")
    require(100 <= len(catalogs["medications"]["items"]) <= 120, "Expected about 110 pet medicines")
    require(not any(item["name"] == "Çiftlik ziyareti" for item in catalogs["services"]["items"]),
            "Farm service is outside the pet catalog")
    check_clinical_regressions(catalogs)
    require({item["category"] for item in catalogs["medications"]["items"]}
            == build.MEDICATION_CATEGORIES, "Medication category coverage is incomplete")
    first = build.render(catalogs)
    require(first == build.render(catalogs), "Generation is not deterministic")
    require(first == build.OUTPUT.read_bytes(), "Generated SQL is stale; run build.py")
    sql = first.decode("utf-8")
    with scratch_db() as db:
        for species in ("horse", "cattle", "sheep", "goat"):
            try:
                db.execute("INSERT INTO vaccine_types (name, species) VALUES (?, ?)",
                           ("Synthetic rejected species", species))
            except sqlite3.IntegrityError:
                continue
            raise AssertionError(f"Non-pet species accepted by scratch schema: {species}")
        require("withdrawal_note" not in {row[1] for row in db.execute("PRAGMA table_info(medications)")},
                "Scratch schema must omit the removed medication column")
        db.executescript(sql)
        once = counts(db)
        require(once == {kind: len(data["items"]) for kind, data in catalogs.items()}, "Seed counts differ")
        snapshot = {table: db.execute(f"SELECT * FROM {table} ORDER BY id").fetchall()
                    for table in build.TABLES.values()}
        db.executescript(sql)
        require(counts(db) == once, "Second seed changed counts")
        for kind, table in build.TABLES.items():
            require(db.execute(f"SELECT * FROM {table} ORDER BY id").fetchall() == snapshot[table],
                    f"Second seed changed {table}")
            require(db.execute(f"SELECT count(*) FROM {table} WHERE is_builtin = 1").fetchone()[0] == once[kind],
                    f"Missing builtin flags in {table}")
            for item in catalogs[kind]["items"]:
                fields = build.FIELDS[kind]
                actual = db.execute(f"SELECT {', '.join(fields)} FROM {table} WHERE name = ?",
                                    (item["name"],)).fetchone()
                require(actual == tuple(item[field] for field in fields), f"SQL value mismatch: {item['name']}")
        require(db.execute("SELECT count(*) FROM services WHERE price_minor IS NOT NULL OR vat_rate IS NOT NULL").fetchone()[0] == 0,
                "Services unexpectedly contain pricing")
        require(db.execute("PRAGMA integrity_check").fetchone()[0] == "ok", "SQLite integrity check failed")
        require(not db.execute("PRAGMA foreign_key_check").fetchall(), "Foreign key check failed")
        print(f"Seed twice: {once}; counts and rows unchanged")
    with scratch_db() as db:
        # Existing active user rows win; a deleted name does not block a replacement builtin.
        for kind, table in build.TABLES.items():
            name = catalogs[kind]["items"][0]["name"]
            # SQLite lower is ASCII-only; this has exactly its case conversion semantics.
            alternate_case = db.execute("SELECT upper(?)", (name,)).fetchone()[0]
            db.execute(f"INSERT INTO {table} (name, notes) VALUES (?, ?)", (alternate_case, "User edited"))
            deleted_name = catalogs[kind]["items"][1]["name"]
            db.execute(f"INSERT INTO {table} (name, deleted_at) VALUES (?, ?)", (deleted_name, "2026-01-01"))
        db.commit()
        db.executescript(sql)
        db.executescript(sql)
        for kind, table in build.TABLES.items():
            require(db.execute(f"SELECT notes, is_builtin FROM {table} WHERE notes = 'User edited'").fetchone()
                    == ("User edited", 0), f"User entry overwritten in {table}")
            require(db.execute(f"SELECT count(*) FROM {table} WHERE deleted_at IS NULL").fetchone()[0]
                    == len(catalogs[kind]["items"]), f"Conflict/deletion handling failed in {table}")
        print("Existing user entries preserved; soft-deleted names can be seeded again")
    # Exercise quoting with synthetic data, independent of catalog spelling.
    synthetic = copy.deepcopy(catalogs)
    synthetic["services"]["items"][0].update(name="Hekim'in \"özel\" işlemi", notes="İlk satır\nİkinci 'satır'")
    with scratch_db() as db:
        db.executescript(build.render(synthetic).decode("utf-8"))
        row = synthetic["services"]["items"][0]
        require(db.execute("SELECT notes FROM services WHERE name = ?", (row["name"],)).fetchone()[0] == row["notes"],
                "SQL quoting failed")
    check_validation(catalogs)
    build.report(catalogs)
    print(f"SQL SHA-256: {hashlib.sha256(first).hexdigest()}")


if __name__ == "__main__":
    main()
