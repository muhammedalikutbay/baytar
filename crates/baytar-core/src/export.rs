//! CSV export of the clinic's tables for Turkish Excel: UTF-8 with a BOM, `;` separator,
//! CRLF line ends, ISO dates, decimal comma. Soft-deleted rows are skipped. Row contents are
//! only ever written to the returned bytes; nothing here logs them.
use crate::repo::{invalid, Result};
use rusqlite::{types::Value, Connection};
use serde::{Deserialize, Serialize};

pub const BOM: &str = "\u{feff}";
pub const SEPARATOR: char = ';';

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportTable {
    Owners,
    Patients,
    Visits,
    Vaccinations,
    Prescriptions,
    Charges,
    Payments,
    Inventory,
}

impl ExportTable {
    pub const ALL: [ExportTable; 8] = [
        ExportTable::Owners,
        ExportTable::Patients,
        ExportTable::Visits,
        ExportTable::Vaccinations,
        ExportTable::Prescriptions,
        ExportTable::Charges,
        ExportTable::Payments,
        ExportTable::Inventory,
    ];

    /// File name stem, also the identifier used by the UI.
    pub fn stem(self) -> &'static str {
        match self {
            Self::Owners => "owners",
            Self::Patients => "patients",
            Self::Visits => "visits",
            Self::Vaccinations => "vaccinations",
            Self::Prescriptions => "prescriptions",
            Self::Charges => "charges",
            Self::Payments => "payments",
            Self::Inventory => "inventory",
        }
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Text,
    /// Integer minor units rendered as a decimal with a comma.
    Money,
}

struct Spec {
    columns: &'static [(&'static str, Kind)],
    sql: &'static str,
}

const T: Kind = Kind::Text;
const M: Kind = Kind::Money;

fn spec(table: ExportTable) -> Spec {
    match table {
        ExportTable::Owners => Spec {
            columns: &[
                ("No", T),
                ("Ad soyad", T),
                ("Telefon", T),
                ("Telefon 2", T),
                ("E-posta", T),
                ("Adres", T),
                ("İl", T),
                ("İlçe", T),
                ("Not", T),
                ("Kayıt tarihi", T),
            ],
            sql: "SELECT id, full_name, phone, phone_alt, email, address, city, district, notes, \
                  substr(created_at, 1, 10) FROM owners WHERE deleted_at IS NULL ORDER BY id",
        },
        ExportTable::Patients => Spec {
            columns: &[
                ("No", T),
                ("Sahip", T),
                ("Ad", T),
                ("Tür", T),
                ("Irk", T),
                ("Cinsiyet", T),
                ("Kısır", T),
                ("Doğum tarihi", T),
                ("Renk", T),
                ("Mikroçip", T),
                ("Ölüm tarihi", T),
                ("Not", T),
            ],
            sql: "SELECT p.id, o.full_name, p.name, p.species, p.breed, p.sex, p.neutered, \
                  p.birth_date, p.color, p.microchip, p.deceased_on, p.notes \
                  FROM patients p LEFT JOIN owners o ON o.id = p.owner_id \
                  WHERE p.deleted_at IS NULL ORDER BY p.id",
        },
        ExportTable::Visits => Spec {
            columns: &[
                ("No", T),
                ("Tarih", T),
                ("Saat", T),
                ("Hasta", T),
                ("Sahip", T),
                ("Şikayet", T),
                ("Bulgular", T),
                ("Tanı", T),
                ("Tedavi", T),
                ("Kilo (kg)", T),
                ("Ateş (°C)", T),
                ("Not", T),
            ],
            sql:
                "SELECT v.id, v.visit_on, v.visit_time, p.name, o.full_name, v.reason, v.findings, \
                  v.diagnosis, v.treatment, v.weight_kg, v.temperature_c, v.notes \
                  FROM visits v LEFT JOIN patients p ON p.id = v.patient_id \
                  LEFT JOIN owners o ON o.id = p.owner_id \
                  WHERE v.deleted_at IS NULL ORDER BY v.visit_on, v.id",
        },
        ExportTable::Vaccinations => Spec {
            columns: &[
                ("No", T),
                ("Uygulama tarihi", T),
                ("Sonraki tarih", T),
                ("Aşı", T),
                ("Seri no", T),
                ("Hasta", T),
                ("Sahip", T),
                ("Not", T),
            ],
            sql: "SELECT x.id, x.given_on, x.next_due_on, x.vaccine_name, x.batch_no, p.name, \
                  o.full_name, x.notes \
                  FROM vaccinations x LEFT JOIN patients p ON p.id = x.patient_id \
                  LEFT JOIN owners o ON o.id = p.owner_id \
                  WHERE x.deleted_at IS NULL ORDER BY x.given_on, x.id",
        },
        ExportTable::Prescriptions => Spec {
            columns: &[
                ("No", T),
                ("Muayene tarihi", T),
                ("Hasta", T),
                ("Sahip", T),
                ("İlaç", T),
                ("Doz (mg)", T),
                ("Doz (ml)", T),
                ("Doz notu", T),
                ("Sıklık", T),
                ("Süre (gün)", T),
                ("Not", T),
            ],
            sql: "SELECT i.id, v.visit_on, p.name, o.full_name, i.medication_name, i.dose_mg, \
                  i.dose_ml, i.dose_text, i.frequency, i.duration_days, i.notes \
                  FROM prescription_items i LEFT JOIN visits v ON v.id = i.visit_id \
                  LEFT JOIN patients p ON p.id = v.patient_id \
                  LEFT JOIN owners o ON o.id = p.owner_id \
                  WHERE i.deleted_at IS NULL ORDER BY v.visit_on, i.id",
        },
        ExportTable::Charges => Spec {
            columns: &[
                ("No", T),
                ("Tarih", T),
                ("Sahip", T),
                ("Hasta", T),
                ("Açıklama", T),
                ("Miktar", T),
                ("Birim fiyat", M),
                ("İndirim", M),
                ("KDV %", T),
                ("Toplam", M),
                ("Not", T),
            ],
            sql: "SELECT c.id, c.charged_on, o.full_name, p.name, c.description, c.quantity, \
                  c.unit_price_minor, c.discount_minor, c.vat_rate, c.total_minor, c.notes \
                  FROM charges c LEFT JOIN owners o ON o.id = c.owner_id \
                  LEFT JOIN patients p ON p.id = c.patient_id \
                  WHERE c.deleted_at IS NULL ORDER BY c.charged_on, c.id",
        },
        ExportTable::Payments => Spec {
            columns: &[
                ("No", T),
                ("Tarih", T),
                ("Sahip", T),
                ("Tutar", M),
                ("Yöntem", T),
                ("Not", T),
            ],
            sql: "SELECT y.id, y.paid_on, o.full_name, y.amount_minor, y.method, y.notes \
                  FROM payments y LEFT JOIN owners o ON o.id = y.owner_id \
                  WHERE y.deleted_at IS NULL ORDER BY y.paid_on, y.id",
        },
        ExportTable::Inventory => Spec {
            columns: &[
                ("No", T),
                ("Ad", T),
                ("Tür", T),
                ("Birim", T),
                ("Stok", T),
                ("Satış fiyatı", M),
                ("Sipariş seviyesi", T),
                ("Aktif", T),
                ("Not", T),
            ],
            sql: "SELECT i.id, i.name, i.kind, i.unit, \
                  (SELECT COALESCE(SUM(l.quantity), 0) FROM inventory_lots l \
                      WHERE l.item_id = i.id AND l.deleted_at IS NULL), \
                  i.sale_price_minor, i.reorder_level, i.is_active, i.notes \
                  FROM inventory_items i WHERE i.deleted_at IS NULL ORDER BY i.name, i.id",
        },
    }
}

/// Integer minor units as a decimal with a comma and two digits: `1250` -> `12,50`.
pub fn money(minor: i64) -> String {
    let sign = if minor < 0 { "-" } else { "" };
    let abs = minor.unsigned_abs();
    format!("{sign}{},{:02}", abs / 100, abs % 100)
}

fn real(value: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }
    format!("{value}").replace('.', ",")
}

/// Quotes a field when it holds the separator, a quote or a line break (quotes are doubled).
/// A text starting with `=` or `@` gets a leading apostrophe so Excel never runs it as a formula.
pub fn csv_field(value: &str) -> String {
    let guarded;
    let value = if value.starts_with(['=', '@', '\t']) {
        guarded = format!("'{value}");
        guarded.as_str()
    } else {
        value
    };
    if value.contains([SEPARATOR, '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_owned()
    }
}

pub fn csv_line<I, S>(fields: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut line = fields
        .into_iter()
        .map(|f| csv_field(f.as_ref()))
        .collect::<Vec<_>>()
        .join(&SEPARATOR.to_string());
    line.push_str("\r\n");
    line
}

fn cell(value: Value, kind: Kind) -> String {
    match (value, kind) {
        (Value::Null | Value::Blob(_), _) => String::new(),
        (Value::Integer(i), Kind::Money) => money(i),
        (Value::Integer(i), Kind::Text) => i.to_string(),
        (Value::Real(r), _) => real(r),
        (Value::Text(t), _) => t,
    }
}

/// Whole table as UTF-8 bytes with a BOM. Reads the connection only.
pub fn export_csv(conn: &Connection, table: ExportTable) -> Result<Vec<u8>> {
    let spec = spec(table);
    let mut out = String::from(BOM);
    out.push_str(&csv_line(spec.columns.iter().map(|(name, _)| *name)));
    let mut stmt = conn.prepare(spec.sql)?;
    if stmt.column_count() != spec.columns.len() {
        return Err(invalid("table", "export definition is inconsistent"));
    }
    let mut rows = stmt.query([])?;
    while let Some(row) = rows.next()? {
        let mut fields = Vec::with_capacity(spec.columns.len());
        for (index, (_, kind)) in spec.columns.iter().enumerate() {
            fields.push(cell(row.get::<_, Value>(index)?, *kind));
        }
        out.push_str(&csv_line(fields));
    }
    Ok(out.into_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::Database;

    #[test]
    fn escapes_quotes_separators_and_newlines() {
        assert_eq!(csv_field("plain"), "plain");
        assert_eq!(csv_field("a;b"), "\"a;b\"");
        assert_eq!(csv_field("say \"hi\""), "\"say \"\"hi\"\"\"");
        assert_eq!(csv_field("line1\nline2"), "\"line1\nline2\"");
        assert_eq!(csv_field("cr\rlf"), "\"cr\rlf\"");
        assert_eq!(csv_field("Çağrı; \"Şimşek\""), "\"Çağrı; \"\"Şimşek\"\"\"");
        assert_eq!(csv_field("a,b"), "a,b");
        assert_eq!(csv_field(""), "");
        assert_eq!(csv_line(["a", "b;c", ""]), "a;\"b;c\";\r\n");
    }

    #[test]
    fn formula_prefixes_are_neutralised_but_phones_and_negatives_survive() {
        assert_eq!(csv_field("=1+1"), "'=1+1");
        assert_eq!(csv_field("@cmd"), "'@cmd");
        assert_eq!(csv_field("+905321112233"), "+905321112233");
        assert_eq!(csv_field("-5"), "-5");
    }

    #[test]
    fn money_uses_decimal_comma() {
        assert_eq!(money(0), "0,00");
        assert_eq!(money(5), "0,05");
        assert_eq!(money(1250), "12,50");
        assert_eq!(money(123456789), "1234567,89");
        assert_eq!(money(-5), "-0,05");
        assert_eq!(money(-1250), "-12,50");
        assert_eq!(money(i64::MIN), "-92233720368547758,08");
        assert_eq!(real(4.1), "4,1");
        assert_eq!(real(f64::NAN), "");
    }

    fn seeded() -> Database {
        let db = Database::open_in_memory().unwrap();
        db.conn()
            .execute_batch(
                "INSERT INTO owners(id, full_name, phone, notes) VALUES
                    (1, 'Çağrı Şimşek', '+905321112233', 'Not; \"quoted\"
second line'),
                    (2, 'Deleted Owner', NULL, NULL);
                 UPDATE owners SET deleted_at = '2026-01-01T00:00:00Z' WHERE id = 2;
                 INSERT INTO patients(id, owner_id, name, species, neutered, birth_date)
                    VALUES (1, 1, 'Pamuk', 'cat', 1, '2022-04-12');
                 INSERT INTO visits(id, patient_id, visit_on, weight_kg, diagnosis)
                    VALUES (1, 1, '2026-10-24', 4.1, '=cmd');
                 INSERT INTO vaccinations(patient_id, vaccine_name, given_on, next_due_on)
                    VALUES (1, 'Kuduz', '2026-10-24', '2027-10-24');
                 INSERT INTO prescription_items(visit_id, medication_name, dose_mg, duration_days)
                    VALUES (1, 'Amoksisilin', 25.5, 7);
                 INSERT INTO charges(owner_id, patient_id, description, quantity, unit_price_minor, discount_minor, total_minor, charged_on)
                    VALUES (1, 1, 'Muayene', 1, 125050, 50, 125000, '2026-10-24');
                 INSERT INTO payments(owner_id, paid_on, amount_minor, method)
                    VALUES (1, '2026-10-25', 99905, 'card');
                 INSERT INTO inventory_items(id, name, unit, sale_price_minor) VALUES (1, 'Mama', 'kg', 10);
                 INSERT INTO inventory_lots(item_id, quantity) VALUES (1, 2.5), (1, 1);",
            )
            .unwrap();
        db
    }

    fn text(db: &Database, table: ExportTable) -> String {
        String::from_utf8(export_csv(db.conn(), table).unwrap()).unwrap()
    }

    #[test]
    fn owners_export_has_bom_header_and_skips_deleted_rows() {
        let csv = text(&seeded(), ExportTable::Owners);
        assert!(csv.starts_with("\u{feff}No;Ad soyad;"));
        assert!(csv.contains("1;Çağrı Şimşek;+905321112233;"));
        assert!(csv.contains("\"Not; \"\"quoted\"\"\nsecond line\""));
        assert!(!csv.contains("Deleted Owner"));
        assert!(csv.ends_with("\r\n"));
    }

    #[test]
    fn every_table_exports_with_a_matching_column_count() {
        let db = seeded();
        for table in ExportTable::ALL {
            let csv = text(&db, table);
            assert!(csv.starts_with(BOM), "{table:?}");
            assert!(csv.lines().count() >= 2, "{table:?} has a data row");
        }
        let visits = text(&db, ExportTable::Visits);
        assert!(visits.contains(";4,1;"));
        assert!(visits.contains(";'=cmd;"));
        let charges = text(&db, ExportTable::Charges);
        assert!(charges.contains(";Muayene;1;1250,50;0,50;;1250,00;"));
        let payments = text(&db, ExportTable::Payments);
        assert!(payments.contains(";999,05;card;"));
        assert!(text(&db, ExportTable::Inventory).contains(";Mama;other;kg;3,5;0,10;"));
        assert!(text(&db, ExportTable::Prescriptions).contains(";Amoksisilin;25,5;;;;7;"));
    }

    #[test]
    fn empty_database_exports_only_the_header() {
        let db = Database::open_in_memory().unwrap();
        for table in ExportTable::ALL {
            assert_eq!(text(&db, table).matches("\r\n").count(), 1, "{table:?}");
        }
    }
}
