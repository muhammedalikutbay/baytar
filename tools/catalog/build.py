#!/usr/bin/env python3
"""Validate the reviewable catalogs and generate the schema-v2 seed (stdlib only)."""

import json
import math
from pathlib import Path
from urllib.parse import urlsplit

ROOT = Path(__file__).resolve().parents[2]
CATALOG = ROOT / "crates/baytar-core/catalog"
OUTPUT = ROOT / "crates/baytar-core/src/db/migrations/0003_catalog_seed.sql"
SPECIES = set("cat dog bird rabbit rodent ferret reptile other".split())
PET_SPECIES_TEXT = set("Köpek Kedi Tavşan Kuş Kemirgen Gelincik Sürüngen".split())
MEDICATION_CATEGORIES = set(
    "antibiotic nsaid analgesic antiparasitic anesthetic sedative corticosteroid "
    "antiemetic gastro cardio dermatologic ophthalmic otic vitamin fluid hormone "
    "antifungal antiviral other".split()
)
SERVICE_CATEGORIES = set(
    "exam vaccination surgery lab imaging grooming boarding hospitalisation other".split()
)
FORMS = set("tablet capsule injection suspension drops ointment other".split())
VACCINE_NUMBERS = (
    "min_age_weeks", "primary_doses", "primary_interval_days", "default_interval_days",
)
MEDICATION_NUMBERS = (
    "concentration_mg_per_ml", "strength_mg_per_unit",
    "dose_mg_per_kg_min", "dose_mg_per_kg_max",
)
FIELDS = {
    "vaccines": (
        "name", "species", "manufacturer", "diseases", *VACCINE_NUMBERS, "notes",
    ),
    "medications": (
        "name", "active_ingredient", "category", "form", *MEDICATION_NUMBERS,
        "route", "target_species", "manufacturer", "prescription_required",
        "pack_unit", "notes",
    ),
    "services": ("name", "category", "notes"),
}
TABLES = {"vaccines": "vaccine_types", "medications": "medications", "services": "services"}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def exact_fields(value, fields, context):
    require(isinstance(value, dict), f"{context}: expected an object")
    require(set(value) == set(fields), f"{context}: expected fields {sorted(fields)}")


def nonempty_text(value):
    return isinstance(value, str) and bool(value.strip()) and "\x00" not in value


def validate(data, kind):
    """Return validated data; never silently drop unknown fields or source references."""
    require(kind in FIELDS, f"Unknown catalog: {kind}")
    exact_fields(data, ("version", "sources", "items"), kind)
    require(type(data["version"]) is int and data["version"] == 1, f"{kind}: version must be 1")
    require(isinstance(data["sources"], list) and data["sources"], f"{kind}: sources required")
    source_ids = set()
    for source in data["sources"]:
        exact_fields(source, ("id", "title", "url"), f"{kind} source")
        require(all(nonempty_text(v) for v in source.values()), f"{kind}: invalid source text")
        require(source["id"] not in source_ids, f"{kind}: duplicate source id {source['id']}")
        url = urlsplit(source["url"])
        require(url.scheme in ("https", "http") and bool(url.netloc), f"{kind}: invalid source URL")
        source_ids.add(source["id"])
    require(isinstance(data["items"], list) and data["items"], f"{kind}: items required")
    names = set()
    numbers = VACCINE_NUMBERS if kind == "vaccines" else MEDICATION_NUMBERS if kind == "medications" else ()
    for index, item in enumerate(data["items"], 1):
        context = f"{kind} item {index}"
        exact_fields(item, (*FIELDS[kind], "source_ids"), context)
        require(nonempty_text(item["name"]), f"{context}: name required")
        require(item["name"] == item["name"].strip(), f"{context}: name has surrounding whitespace")
        name = item["name"].lower()
        require(name not in names, f"{context}: duplicate lower(name) {name}")
        names.add(name)
        refs = item["source_ids"]
        require(isinstance(refs, list) and refs and all(nonempty_text(r) for r in refs),
                f"{context}: source_ids must be a nonempty string list")
        require(len(refs) == len(set(refs)) and set(refs) <= source_ids,
                f"{context}: duplicate or unknown source_ids")
        for field in FIELDS[kind]:
            value = item[field]
            if field in numbers:
                require(value is None or (
                    type(value) in (int, float) and math.isfinite(value) and value > 0
                ), f"{context}: {field} must be a positive finite number or null")
                if kind == "vaccines":
                    require(value is None or type(value) is int,
                            f"{context}: {field} must be an integer or null")
            elif field == "prescription_required":
                require(value is None or (type(value) is int and value in (0, 1)),
                        f"{context}: prescription_required must be 0, 1 or null")
            else:
                require(value is None or nonempty_text(value), f"{context}: invalid {field} text")
        if kind == "vaccines":
            require(item["species"] is None or item["species"] in SPECIES, f"{context}: invalid species")
            require(item["species"] is not None or item["diseases"] == "Kuduz",
                    f"{context}: null species is reserved for dog/cat rabies")
        elif kind == "medications":
            require(item["category"] in MEDICATION_CATEGORIES, f"{context}: invalid category")
            require(item["form"] in FORMS, f"{context}: invalid form")
            targets = item["target_species"]
            require(targets is not None and all(part in PET_SPECIES_TEXT for part in targets.split(", ")),
                    f"{context}: target_species must list Turkish pet species separated by comma-space")
            low, high = (item[f"dose_mg_per_kg_{bound}"] for bound in ("min", "max"))
            require((low is None) == (high is None), f"{context}: dose bounds must both be supplied or null")
            require(low is None or high is None or low <= high, f"{context}: dose min exceeds max")
        else:
            require(item["category"] in SERVICE_CATEGORIES, f"{context}: invalid category")
    return data


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        require(key not in result, f"Duplicate JSON key: {key}")
        result[key] = value
    return result


def load_catalogs():
    return {
        kind: validate(json.loads((CATALOG / f"{kind}.json").read_text(encoding="utf-8"),
                                  object_pairs_hook=unique_object), kind)
        for kind in FIELDS
    }


def literal(value):
    if value is None:
        return "NULL"
    if isinstance(value, str):
        return "'" + value.replace("'", "''") + "'"
    require(type(value) in (int, float) and math.isfinite(value), "Invalid SQL numeric literal")
    return str(value)


def render(catalogs):
    lines = [
        "-- Generated by tools/catalog/build.py; do not edit by hand.",
        "-- Requires schema v2. Source references are retained in catalog/*.json.",
        "-- Existing non-deleted names are preserved; deleted names may be seeded again.",
        "",
    ]
    for kind, fields in FIELDS.items():
        table = TABLES[kind]
        columns = (*fields, "is_builtin", *(("is_active",) if kind == "services" else ()))
        lines.append(f"-- {kind}")
        for item in sorted(catalogs[kind]["items"], key=lambda row: row["name"].lower()):
            values = [literal(item[field]) for field in fields] + ["1"]
            if kind == "services":
                values.append("1")
            lines.extend([
                f"INSERT INTO {table} ({', '.join(columns)})",
                f"SELECT {', '.join(values)}",
                f"WHERE NOT EXISTS (SELECT 1 FROM {table} "
                f"WHERE lower(name) = lower({literal(item['name'])}) AND deleted_at IS NULL);",
                "",
            ])
    return "\n".join(lines).encode("utf-8")


def report(catalogs):
    for kind, data in catalogs.items():
        fields = VACCINE_NUMBERS if kind == "vaccines" else MEDICATION_NUMBERS if kind == "medications" else ()
        total = len(data["items"]) * len(fields)
        nulls = sum(item[field] is None for item in data["items"] for field in fields)
        print(f"{kind}: {len(data['items'])} items; {total - nulls}/{total} clinical numeric fields filled; {nulls} null")
        if fields:
            print("  " + ", ".join(f"{field}={sum(item[field] is None for item in data['items'])} null"
                                    for field in fields))
    flags = catalogs["medications"]["items"]
    print(f"prescription_required: {sum(item['prescription_required'] is None for item in flags)}"
          f"/{len(flags)} flags null (excluded from clinical numeric counts)")


def main():
    try:
        catalogs = load_catalogs()
        OUTPUT.write_bytes(render(catalogs))
        report(catalogs)
        print(f"Generated {OUTPUT.relative_to(ROOT)}")
    except (ValueError, OSError) as error:
        raise SystemExit(f"Catalog build failed: {error}") from error


if __name__ == "__main__":
    main()
