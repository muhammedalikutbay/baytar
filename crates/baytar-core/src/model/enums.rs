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
    Other,
}

impl Species {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cat => "cat",
            Self::Dog => "dog",
            Self::Bird => "bird",
            Self::Rabbit => "rabbit",
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
    }
}
