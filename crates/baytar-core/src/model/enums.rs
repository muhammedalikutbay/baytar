//! Canonical enum values shared by SQLite and the frontend.

use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid {enum_name} value: {value}")]
pub struct ParseEnumError {
    pub enum_name: &'static str,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Species {
    Cat,
    Dog,
    Bird,
    Rabbit,
    Rodent,
    Ferret,
    Reptile,
    Other,
}

impl Species {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cat => "cat",
            Self::Dog => "dog",
            Self::Bird => "bird",
            Self::Rabbit => "rabbit",
            Self::Rodent => "rodent",
            Self::Ferret => "ferret",
            Self::Reptile => "reptile",
            Self::Other => "other",
        }
    }
}

impl FromStr for Species {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "cat" => Ok(Self::Cat),
            "dog" => Ok(Self::Dog),
            "bird" => Ok(Self::Bird),
            "rabbit" => Ok(Self::Rabbit),
            "rodent" => Ok(Self::Rodent),
            "ferret" => Ok(Self::Ferret),
            "reptile" => Ok(Self::Reptile),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError {
                enum_name: "Species",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Sex {
    Male,
    Female,
    Unknown,
}

impl Sex {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Male => "male",
            Self::Female => "female",
            Self::Unknown => "unknown",
        }
    }
}

impl FromStr for Sex {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "male" => Ok(Self::Male),
            "female" => Ok(Self::Female),
            "unknown" => Ok(Self::Unknown),
            _ => Err(ParseEnumError {
                enum_name: "Sex",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MedicationForm {
    Tablet,
    Capsule,
    Injection,
    Suspension,
    Drops,
    Ointment,
    Other,
}

impl MedicationForm {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tablet => "tablet",
            Self::Capsule => "capsule",
            Self::Injection => "injection",
            Self::Suspension => "suspension",
            Self::Drops => "drops",
            Self::Ointment => "ointment",
            Self::Other => "other",
        }
    }
}

impl FromStr for MedicationForm {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "tablet" => Ok(Self::Tablet),
            "capsule" => Ok(Self::Capsule),
            "injection" => Ok(Self::Injection),
            "suspension" => Ok(Self::Suspension),
            "drops" => Ok(Self::Drops),
            "ointment" => Ok(Self::Ointment),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError {
                enum_name: "MedicationForm",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppointmentKind {
    Visit,
    Vaccine,
    Reminder,
    Other,
}

impl AppointmentKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Visit => "visit",
            Self::Vaccine => "vaccine",
            Self::Reminder => "reminder",
            Self::Other => "other",
        }
    }
}

impl FromStr for AppointmentKind {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "visit" => Ok(Self::Visit),
            "vaccine" => Ok(Self::Vaccine),
            "reminder" => Ok(Self::Reminder),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError {
                enum_name: "AppointmentKind",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AppointmentStatus {
    Planned,
    Done,
    Cancelled,
}

impl AppointmentStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Done => "done",
            Self::Cancelled => "cancelled",
        }
    }
}

impl FromStr for AppointmentStatus {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "planned" => Ok(Self::Planned),
            "done" => Ok(Self::Done),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(ParseEnumError {
                enum_name: "AppointmentStatus",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ImportStatus {
    Previewed,
    Completed,
    Failed,
}

impl ImportStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Previewed => "previewed",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl FromStr for ImportStatus {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "previewed" => Ok(Self::Previewed),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(ParseEnumError {
                enum_name: "ImportStatus",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum MedicationCategory {
    Antibiotic,
    Nsaid,
    Analgesic,
    Antiparasitic,
    Anesthetic,
    Sedative,
    Corticosteroid,
    Antiemetic,
    Gastro,
    Cardio,
    Dermatologic,
    Ophthalmic,
    Otic,
    Vitamin,
    Fluid,
    Hormone,
    Antifungal,
    Antiviral,
    Other,
}

impl MedicationCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Antibiotic => "antibiotic",
            Self::Nsaid => "nsaid",
            Self::Analgesic => "analgesic",
            Self::Antiparasitic => "antiparasitic",
            Self::Anesthetic => "anesthetic",
            Self::Sedative => "sedative",
            Self::Corticosteroid => "corticosteroid",
            Self::Antiemetic => "antiemetic",
            Self::Gastro => "gastro",
            Self::Cardio => "cardio",
            Self::Dermatologic => "dermatologic",
            Self::Ophthalmic => "ophthalmic",
            Self::Otic => "otic",
            Self::Vitamin => "vitamin",
            Self::Fluid => "fluid",
            Self::Hormone => "hormone",
            Self::Antifungal => "antifungal",
            Self::Antiviral => "antiviral",
            Self::Other => "other",
        }
    }
}

impl FromStr for MedicationCategory {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "antibiotic" => Ok(Self::Antibiotic),
            "nsaid" => Ok(Self::Nsaid),
            "analgesic" => Ok(Self::Analgesic),
            "antiparasitic" => Ok(Self::Antiparasitic),
            "anesthetic" => Ok(Self::Anesthetic),
            "sedative" => Ok(Self::Sedative),
            "corticosteroid" => Ok(Self::Corticosteroid),
            "antiemetic" => Ok(Self::Antiemetic),
            "gastro" => Ok(Self::Gastro),
            "cardio" => Ok(Self::Cardio),
            "dermatologic" => Ok(Self::Dermatologic),
            "ophthalmic" => Ok(Self::Ophthalmic),
            "otic" => Ok(Self::Otic),
            "vitamin" => Ok(Self::Vitamin),
            "fluid" => Ok(Self::Fluid),
            "hormone" => Ok(Self::Hormone),
            "antifungal" => Ok(Self::Antifungal),
            "antiviral" => Ok(Self::Antiviral),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError {
                enum_name: "MedicationCategory",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ServiceCategory {
    Exam,
    Vaccination,
    Surgery,
    Lab,
    Imaging,
    Grooming,
    Boarding,
    Hospitalisation,
    Other,
}

impl ServiceCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exam => "exam",
            Self::Vaccination => "vaccination",
            Self::Surgery => "surgery",
            Self::Lab => "lab",
            Self::Imaging => "imaging",
            Self::Grooming => "grooming",
            Self::Boarding => "boarding",
            Self::Hospitalisation => "hospitalisation",
            Self::Other => "other",
        }
    }
}

impl FromStr for ServiceCategory {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "exam" => Ok(Self::Exam),
            "vaccination" => Ok(Self::Vaccination),
            "surgery" => Ok(Self::Surgery),
            "lab" => Ok(Self::Lab),
            "imaging" => Ok(Self::Imaging),
            "grooming" => Ok(Self::Grooming),
            "boarding" => Ok(Self::Boarding),
            "hospitalisation" => Ok(Self::Hospitalisation),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError {
                enum_name: "ServiceCategory",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InventoryKind {
    Medication,
    Vaccine,
    Supply,
    Food,
    Product,
    Other,
}

impl InventoryKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Medication => "medication",
            Self::Vaccine => "vaccine",
            Self::Supply => "supply",
            Self::Food => "food",
            Self::Product => "product",
            Self::Other => "other",
        }
    }
}

impl FromStr for InventoryKind {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "medication" => Ok(Self::Medication),
            "vaccine" => Ok(Self::Vaccine),
            "supply" => Ok(Self::Supply),
            "food" => Ok(Self::Food),
            "product" => Ok(Self::Product),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError {
                enum_name: "InventoryKind",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StockReason {
    Purchase,
    Use,
    Sale,
    Adjust,
    Expired,
    Return,
}

impl StockReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Purchase => "purchase",
            Self::Use => "use",
            Self::Sale => "sale",
            Self::Adjust => "adjust",
            Self::Expired => "expired",
            Self::Return => "return",
        }
    }
}

impl FromStr for StockReason {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "purchase" => Ok(Self::Purchase),
            "use" => Ok(Self::Use),
            "sale" => Ok(Self::Sale),
            "adjust" => Ok(Self::Adjust),
            "expired" => Ok(Self::Expired),
            "return" => Ok(Self::Return),
            _ => Err(ParseEnumError {
                enum_name: "StockReason",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PaymentMethod {
    Cash,
    Card,
    Transfer,
    Other,
}

impl PaymentMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cash => "cash",
            Self::Card => "card",
            Self::Transfer => "transfer",
            Self::Other => "other",
        }
    }
}

impl FromStr for PaymentMethod {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "cash" => Ok(Self::Cash),
            "card" => Ok(Self::Card),
            "transfer" => Ok(Self::Transfer),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError {
                enum_name: "PaymentMethod",
                value: value.to_owned(),
            }),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AttachmentKind {
    Photo,
    Lab,
    Imaging,
    Document,
    Other,
}

impl AttachmentKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Photo => "photo",
            Self::Lab => "lab",
            Self::Imaging => "imaging",
            Self::Document => "document",
            Self::Other => "other",
        }
    }
}

impl FromStr for AttachmentKind {
    type Err = ParseEnumError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "photo" => Ok(Self::Photo),
            "lab" => Ok(Self::Lab),
            "imaging" => Ok(Self::Imaging),
            "document" => Ok(Self::Document),
            "other" => Ok(Self::Other),
            _ => Err(ParseEnumError {
                enum_name: "AttachmentKind",
                value: value.to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check<T>(variants: &[T], names: &[&str])
    where
        T: std::fmt::Debug + PartialEq + FromStr + for<'de> Deserialize<'de>,
        T::Err: std::fmt::Debug,
    {
        for (variant, name) in variants.iter().zip(names) {
            assert_eq!(&name.parse::<T>().unwrap(), variant);
            let deserializer =
                serde::de::value::StrDeserializer::<serde::de::value::Error>::new(name);
            assert_eq!(&T::deserialize(deserializer).unwrap(), variant);
        }
        for invalid in ["", "CAT", "invalid", " cat "] {
            assert!(invalid.parse::<T>().is_err());
        }
    }

    #[test]
    fn enum_strings_round_trip_and_match_serde() {
        let variants = [
            Species::Cat,
            Species::Dog,
            Species::Bird,
            Species::Rabbit,
            Species::Rodent,
            Species::Ferret,
            Species::Reptile,
            Species::Other,
        ];
        check(&variants, &variants.map(Species::as_str));
        let variants = [Sex::Male, Sex::Female, Sex::Unknown];
        check(&variants, &variants.map(Sex::as_str));
        let variants = [
            MedicationForm::Tablet,
            MedicationForm::Capsule,
            MedicationForm::Injection,
            MedicationForm::Suspension,
            MedicationForm::Drops,
            MedicationForm::Ointment,
            MedicationForm::Other,
        ];
        check(&variants, &variants.map(MedicationForm::as_str));
        let variants = [
            AppointmentKind::Visit,
            AppointmentKind::Vaccine,
            AppointmentKind::Reminder,
            AppointmentKind::Other,
        ];
        check(&variants, &variants.map(AppointmentKind::as_str));
        let variants = [
            AppointmentStatus::Planned,
            AppointmentStatus::Done,
            AppointmentStatus::Cancelled,
        ];
        check(&variants, &variants.map(AppointmentStatus::as_str));
        let variants = [
            ImportStatus::Previewed,
            ImportStatus::Completed,
            ImportStatus::Failed,
        ];
        check(&variants, &variants.map(ImportStatus::as_str));
        let variants = [
            MedicationCategory::Antibiotic,
            MedicationCategory::Nsaid,
            MedicationCategory::Analgesic,
            MedicationCategory::Antiparasitic,
            MedicationCategory::Anesthetic,
            MedicationCategory::Sedative,
            MedicationCategory::Corticosteroid,
            MedicationCategory::Antiemetic,
            MedicationCategory::Gastro,
            MedicationCategory::Cardio,
            MedicationCategory::Dermatologic,
            MedicationCategory::Ophthalmic,
            MedicationCategory::Otic,
            MedicationCategory::Vitamin,
            MedicationCategory::Fluid,
            MedicationCategory::Hormone,
            MedicationCategory::Antifungal,
            MedicationCategory::Antiviral,
            MedicationCategory::Other,
        ];
        check(&variants, &variants.map(MedicationCategory::as_str));
        let variants = [
            ServiceCategory::Exam,
            ServiceCategory::Vaccination,
            ServiceCategory::Surgery,
            ServiceCategory::Lab,
            ServiceCategory::Imaging,
            ServiceCategory::Grooming,
            ServiceCategory::Boarding,
            ServiceCategory::Hospitalisation,
            ServiceCategory::Other,
        ];
        check(&variants, &variants.map(ServiceCategory::as_str));
        let variants = [
            InventoryKind::Medication,
            InventoryKind::Vaccine,
            InventoryKind::Supply,
            InventoryKind::Food,
            InventoryKind::Product,
            InventoryKind::Other,
        ];
        check(&variants, &variants.map(InventoryKind::as_str));
        let variants = [
            StockReason::Purchase,
            StockReason::Use,
            StockReason::Sale,
            StockReason::Adjust,
            StockReason::Expired,
            StockReason::Return,
        ];
        check(&variants, &variants.map(StockReason::as_str));
        let variants = [
            PaymentMethod::Cash,
            PaymentMethod::Card,
            PaymentMethod::Transfer,
            PaymentMethod::Other,
        ];
        check(&variants, &variants.map(PaymentMethod::as_str));
        let variants = [
            AttachmentKind::Photo,
            AttachmentKind::Lab,
            AttachmentKind::Imaging,
            AttachmentKind::Document,
            AttachmentKind::Other,
        ];
        check(&variants, &variants.map(AttachmentKind::as_str));
        let variants = [
            MedicationCategory::Antibiotic,
            MedicationCategory::Nsaid,
            MedicationCategory::Analgesic,
            MedicationCategory::Antiparasitic,
            MedicationCategory::Anesthetic,
            MedicationCategory::Sedative,
            MedicationCategory::Corticosteroid,
            MedicationCategory::Antiemetic,
            MedicationCategory::Gastro,
            MedicationCategory::Cardio,
            MedicationCategory::Dermatologic,
            MedicationCategory::Ophthalmic,
            MedicationCategory::Otic,
            MedicationCategory::Vitamin,
            MedicationCategory::Fluid,
            MedicationCategory::Hormone,
            MedicationCategory::Antifungal,
            MedicationCategory::Antiviral,
            MedicationCategory::Other,
        ];
        check(&variants, &variants.map(MedicationCategory::as_str));
        let variants = [
            ServiceCategory::Exam,
            ServiceCategory::Vaccination,
            ServiceCategory::Surgery,
            ServiceCategory::Lab,
            ServiceCategory::Imaging,
            ServiceCategory::Grooming,
            ServiceCategory::Boarding,
            ServiceCategory::Hospitalisation,
            ServiceCategory::Other,
        ];
        check(&variants, &variants.map(ServiceCategory::as_str));
        let variants = [
            InventoryKind::Medication,
            InventoryKind::Vaccine,
            InventoryKind::Supply,
            InventoryKind::Food,
            InventoryKind::Product,
            InventoryKind::Other,
        ];
        check(&variants, &variants.map(InventoryKind::as_str));
        let variants = [
            StockReason::Purchase,
            StockReason::Use,
            StockReason::Sale,
            StockReason::Adjust,
            StockReason::Expired,
            StockReason::Return,
        ];
        check(&variants, &variants.map(StockReason::as_str));
        let variants = [
            PaymentMethod::Cash,
            PaymentMethod::Card,
            PaymentMethod::Transfer,
            PaymentMethod::Other,
        ];
        check(&variants, &variants.map(PaymentMethod::as_str));
        let variants = [
            AttachmentKind::Photo,
            AttachmentKind::Lab,
            AttachmentKind::Imaging,
            AttachmentKind::Document,
            AttachmentKind::Other,
        ];
        check(&variants, &variants.map(AttachmentKind::as_str));
    }
}
