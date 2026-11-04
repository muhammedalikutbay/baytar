//! Plain row models for schema v2.
//!
//! Dates, times, UTC timestamps, and stored JSON remain strings in these transport
//! models. The app validates their documented formats before writing. SQLite 0/1
//! flags are represented as bool, and SQL NULL as Option. Field names serialize
//! as camelCase; enum values use the canonical lowercase database spellings.

mod enums;

pub use enums::{
    AppointmentKind, AppointmentStatus, AttachmentKind, ImportStatus, InventoryKind,
    MedicationCategory, MedicationForm, ParseEnumError, PaymentMethod, ServiceCategory, Sex,
    Species, StockReason,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Owner {
    pub id: i64,
    pub full_name: String,
    pub phone: Option<String>,
    pub phone_raw: Option<String>,
    pub phone_alt: Option<String>,
    pub email: Option<String>,
    pub address: Option<String>,
    pub notes: Option<String>,
    pub national_id: Option<String>,
    pub city: Option<String>,
    pub district: Option<String>,
    pub search_text: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patient {
    pub id: i64,
    pub owner_id: i64,
    pub name: String,
    pub species: Species,
    pub species_note: Option<String>,
    pub breed: Option<String>,
    pub sex: Sex,
    pub neutered: Option<bool>,
    pub birth_date: Option<String>,
    pub birth_date_is_estimate: bool,
    pub color: Option<String>,
    pub microchip: Option<String>,
    pub notes: Option<String>,
    pub alerts: Option<String>,
    pub deceased_on: Option<String>,
    pub search_text: String,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Visit {
    pub id: i64,
    pub patient_id: i64,
    pub visit_on: String,
    pub visit_time: Option<String>,
    pub reason: Option<String>,
    pub findings: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub weight_kg: Option<f64>,
    pub temperature_c: Option<f64>,
    pub heart_rate_bpm: Option<i64>,
    pub resp_rate_bpm: Option<i64>,
    pub body_condition: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VaccineType {
    pub id: i64,
    pub name: String,
    pub species: Option<Species>,
    pub default_interval_days: Option<i64>,
    pub notes: Option<String>,
    pub is_active: bool,
    pub is_builtin: bool,
    pub manufacturer: Option<String>,
    pub diseases: Option<String>,
    pub primary_doses: Option<i64>,
    pub primary_interval_days: Option<i64>,
    pub min_age_weeks: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Vaccination {
    pub id: i64,
    pub patient_id: i64,
    pub vaccine_type_id: Option<i64>,
    pub vaccine_name: String,
    pub given_on: String,
    pub next_due_on: Option<String>,
    pub batch_no: Option<String>,
    pub visit_id: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Medication {
    pub id: i64,
    pub name: String,
    pub active_ingredient: Option<String>,
    pub form: Option<MedicationForm>,
    pub concentration_mg_per_ml: Option<f64>,
    pub strength_mg_per_unit: Option<f64>,
    pub dose_mg_per_kg_min: Option<f64>,
    pub dose_mg_per_kg_max: Option<f64>,
    pub route: Option<String>,
    pub notes: Option<String>,
    pub is_builtin: bool,
    pub category: Option<MedicationCategory>,
    pub manufacturer: Option<String>,
    pub target_species: Option<String>,
    pub prescription_required: Option<bool>,
    pub pack_unit: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PrescriptionItem {
    pub id: i64,
    pub visit_id: i64,
    pub medication_id: Option<i64>,
    pub medication_name: String,
    pub dose_mg: Option<f64>,
    pub dose_ml: Option<f64>,
    pub dose_text: Option<String>,
    pub frequency: Option<String>,
    pub duration_days: Option<i64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Appointment {
    pub id: i64,
    pub patient_id: Option<i64>,
    pub owner_id: Option<i64>,
    pub title: String,
    pub starts_on: String,
    pub starts_time: Option<String>,
    pub duration_min: Option<i64>,
    pub kind: AppointmentKind,
    pub status: AppointmentStatus,
    pub series_id: Option<String>,
    pub notes: Option<String>,
    pub vaccine_type_id: Option<i64>,
    pub vaccination_id: Option<i64>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportRun {
    pub id: i64,
    pub source_name: Option<String>,
    pub source_sha256: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub status: ImportStatus,
    pub report_json: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyMap {
    pub id: i64,
    pub import_run_id: i64,
    pub source_sha256: String,
    pub source_table: String,
    pub source_id: i64,
    pub target_table: String,
    pub target_id: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub id: i64,
    pub name: String,
    pub category: ServiceCategory,
    /// Minor units (kurus).
    pub price_minor: Option<i64>,
    /// Percent.
    pub vat_rate: Option<i64>,
    pub is_builtin: bool,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryItem {
    pub id: i64,
    pub name: String,
    pub kind: InventoryKind,
    pub medication_id: Option<i64>,
    pub vaccine_type_id: Option<i64>,
    pub unit: String,
    pub sale_price_minor: Option<i64>,
    pub reorder_level: Option<f64>,
    pub is_active: bool,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InventoryLot {
    pub id: i64,
    pub item_id: i64,
    pub lot_no: Option<String>,
    pub expires_on: Option<String>,
    pub quantity: f64,
    pub cost_minor: Option<i64>,
    pub received_on: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StockMovement {
    pub id: i64,
    pub item_id: i64,
    pub lot_id: Option<i64>,
    /// Signed quantity change; never zero.
    pub delta: f64,
    pub reason: StockReason,
    pub ref_table: Option<String>,
    pub ref_id: Option<i64>,
    pub moved_on: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Charge {
    pub id: i64,
    pub owner_id: i64,
    pub patient_id: Option<i64>,
    pub visit_id: Option<i64>,
    pub service_id: Option<i64>,
    pub inventory_item_id: Option<i64>,
    pub description: String,
    pub quantity: f64,
    pub unit_price_minor: i64,
    pub discount_minor: i64,
    pub vat_rate: Option<i64>,
    /// round(quantity * unit price) - discount, computed by the repository.
    pub total_minor: i64,
    pub charged_on: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub id: i64,
    pub owner_id: i64,
    pub paid_on: String,
    pub amount_minor: i64,
    pub method: PaymentMethod,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

/// Attachment metadata. The file bytes live in `attachment_data` and are loaded separately.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    pub id: i64,
    pub patient_id: i64,
    pub visit_id: Option<i64>,
    pub kind: AttachmentKind,
    pub title: Option<String>,
    pub file_name: String,
    pub mime: String,
    pub size_bytes: Option<i64>,
    pub sha256: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VisitTemplate {
    pub id: i64,
    pub name: String,
    pub reason: Option<String>,
    pub findings: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment: Option<String>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
}
