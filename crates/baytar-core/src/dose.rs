//! Arithmetic on veterinarian-supplied values only; no clinical dose data.
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DoseInput {
    pub weight_kg: f64,
    pub mg_per_kg_min: f64,
    pub mg_per_kg_max: Option<f64>,
    pub concentration_mg_per_ml: Option<f64>,
    pub strength_mg_per_unit: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoseRange {
    pub min: f64,
    pub max: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum DoseWarning {
    VerySmallVolume,
    ImpracticalUnitCount,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DoseResult {
    pub mg: DoseRange,
    pub ml: Option<DoseRange>,
    pub units: Option<DoseRange>,
    pub warnings: Vec<DoseWarning>,
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum DoseError {
    #[error("{field}: {reason}")]
    Invalid {
        field: &'static str,
        reason: &'static str,
    },
    #[error("The result is outside the representable positive finite range")]
    OutOfRange,
}

impl DoseRange {
    fn map(&self, f: impl Fn(f64) -> f64) -> Self {
        Self {
            min: f(self.min),
            max: self.max.map(f),
        }
    }

    fn checked(self) -> Result<Self, DoseError> {
        if !self.min.is_finite()
            || self.min <= 0.0
            || self.max.is_some_and(|v| !v.is_finite() || v <= 0.0)
        {
            Err(DoseError::OutOfRange)
        } else {
            Ok(self)
        }
    }
}

impl DoseResult {
    /// A display-only copy. Never use rounded values as calculation inputs.
    /// Small positive values may display as zero; warnings retain full precision.
    pub fn rounded(&self) -> Self {
        fn round(value: f64, scale: f64) -> f64 {
            let scaled = value * scale;
            if scaled.is_finite() {
                scaled.round() / scale
            } else {
                value
            }
        }
        Self {
            mg: self.mg.map(|v| round(v, 100.0)),
            ml: self.ml.as_ref().map(|r| r.map(|v| round(v, 100.0))),
            units: self.units.as_ref().map(|r| r.map(|v| round(v, 4.0))),
            warnings: self.warnings.clone(),
        }
    }
}

pub fn calculate(input: &DoseInput) -> Result<DoseResult, DoseError> {
    for (field, value) in [
        ("weightKg", Some(input.weight_kg)),
        ("mgPerKgMin", Some(input.mg_per_kg_min)),
        ("mgPerKgMax", input.mg_per_kg_max),
        ("concentrationMgPerMl", input.concentration_mg_per_ml),
        ("strengthMgPerUnit", input.strength_mg_per_unit),
    ] {
        if value.is_some_and(|v| !v.is_finite() || v <= 0.0) {
            return Err(DoseError::Invalid {
                field,
                reason: "must be finite and greater than zero",
            });
        }
    }
    if input.weight_kg > 1500.0 {
        return Err(DoseError::Invalid {
            field: "weightKg",
            reason: "must be at most 1500 kg",
        });
    }
    if input.mg_per_kg_max.is_some_and(|v| v < input.mg_per_kg_min) {
        return Err(DoseError::Invalid {
            field: "mgPerKgMax",
            reason: "must be at least mgPerKgMin",
        });
    }
    let mg = DoseRange {
        min: input.weight_kg * input.mg_per_kg_min,
        max: input.mg_per_kg_max.map(|v| input.weight_kg * v),
    }
    .checked()?;
    let ml = input
        .concentration_mg_per_ml
        .map(|c| mg.map(|v| v / c).checked())
        .transpose()?;
    let units = input
        .strength_mg_per_unit
        .map(|s| mg.map(|v| v / s).checked())
        .transpose()?;
    let mut warnings = Vec::new();
    if ml.as_ref().is_some_and(|r| r.min < 0.05) {
        warnings.push(DoseWarning::VerySmallVolume);
    }
    // Quarter units are an arithmetic display convention, not permission to split a product.
    let impractical = |v: f64| {
        let fraction = v.fract();
        v < 0.25 || (fraction - (fraction * 4.0).round() / 4.0).abs() > 1e-9
    };
    if units
        .as_ref()
        .is_some_and(|r| impractical(r.min) || r.max.is_some_and(impractical))
    {
        warnings.push(DoseWarning::ImpracticalUnitCount);
    }
    Ok(DoseResult {
        mg,
        ml,
        units,
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> DoseInput {
        DoseInput {
            weight_kg: 5.0,
            mg_per_kg_min: 2.5,
            mg_per_kg_max: None,
            concentration_mg_per_ml: None,
            strength_mg_per_unit: None,
        }
    }
    #[test]
    fn mass_only() {
        let r = calculate(&input()).unwrap();
        assert_eq!(r.mg.min, 12.5);
        assert_eq!(r.mg.max, None);
        assert_eq!(r.ml, None);
        assert_eq!(r.units, None);
    }
    #[test]
    fn liquid() {
        let r = calculate(&DoseInput {
            concentration_mg_per_ml: Some(50.0),
            ..input()
        })
        .unwrap();
        assert_eq!(r.ml.unwrap().min, 0.25);
    }
    #[test]
    fn range_and_both_conversions() {
        let r = calculate(&DoseInput {
            mg_per_kg_max: Some(5.0),
            concentration_mg_per_ml: Some(50.0),
            strength_mg_per_unit: Some(25.0),
            ..input()
        })
        .unwrap();
        assert_eq!(r.mg.max, Some(25.0));
        assert_eq!(r.ml.unwrap().max, Some(0.5));
        assert_eq!(
            r.units.unwrap(),
            DoseRange {
                min: 0.5,
                max: Some(1.0)
            }
        );
        assert!(r.warnings.is_empty());
    }
    #[test]
    fn equal_bounds() {
        assert_eq!(
            calculate(&DoseInput {
                mg_per_kg_max: Some(2.5),
                ..input()
            })
            .unwrap()
            .mg
            .max,
            Some(12.5)
        );
    }
    #[test]
    fn inverted_range() {
        assert!(calculate(&DoseInput {
            mg_per_kg_max: Some(1.0),
            ..input()
        })
        .is_err());
    }
    #[test]
    fn invalid_weights() {
        for w in [
            0.0,
            -1.0,
            1500.01,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
        ] {
            assert!(calculate(&DoseInput {
                weight_kg: w,
                ..input()
            })
            .is_err());
        }
    }
    #[test]
    fn maximum_weight() {
        assert_eq!(
            calculate(&DoseInput {
                weight_kg: 1500.0,
                ..input()
            })
            .unwrap()
            .mg
            .min,
            3750.0
        );
    }
    #[test]
    fn tiny_weight() {
        assert_eq!(
            calculate(&DoseInput {
                weight_kg: 0.001,
                ..input()
            })
            .unwrap()
            .mg
            .min,
            0.0025
        );
    }
    #[test]
    fn invalid_minimum() {
        for v in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(calculate(&DoseInput {
                mg_per_kg_min: v,
                ..input()
            })
            .is_err());
        }
    }
    #[test]
    fn invalid_maximum() {
        for v in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(calculate(&DoseInput {
                mg_per_kg_max: Some(v),
                ..input()
            })
            .is_err());
        }
    }
    #[test]
    fn invalid_concentration() {
        for v in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(calculate(&DoseInput {
                concentration_mg_per_ml: Some(v),
                ..input()
            })
            .is_err());
        }
    }
    #[test]
    fn invalid_strength() {
        for v in [0.0, -1.0, f64::NAN, f64::INFINITY] {
            assert!(calculate(&DoseInput {
                strength_mg_per_unit: Some(v),
                ..input()
            })
            .is_err());
        }
    }
    #[test]
    fn small_volume_boundary() {
        for (c, warning) in [(250.0, false), (251.0, true)] {
            let r = calculate(&DoseInput {
                concentration_mg_per_ml: Some(c),
                ..input()
            })
            .unwrap();
            assert_eq!(r.warnings.contains(&DoseWarning::VerySmallVolume), warning);
        }
    }
    #[test]
    fn impractical_upper_unit_count() {
        let r = calculate(&DoseInput {
            mg_per_kg_max: Some(3.0),
            strength_mg_per_unit: Some(25.0),
            ..input()
        })
        .unwrap();
        assert_eq!(r.warnings, vec![DoseWarning::ImpracticalUnitCount]);
    }
    #[test]
    fn practical_quarters() {
        for strength in [50.0, 25.0, 12.5 / 0.75, 12.5] {
            assert!(calculate(&DoseInput {
                strength_mg_per_unit: Some(strength),
                ..input()
            })
            .unwrap()
            .warnings
            .is_empty());
        }
    }
    #[test]
    fn rounding_preserves_original_and_warnings() {
        let r = calculate(&DoseInput {
            weight_kg: 1.0,
            mg_per_kg_min: 1.23456,
            mg_per_kg_max: Some(2.6789),
            concentration_mg_per_ml: Some(100.0),
            strength_mg_per_unit: Some(2.0),
        })
        .unwrap();
        let d = r.rounded();
        assert_eq!(r.mg.min, 1.23456);
        assert_eq!(
            d.mg,
            DoseRange {
                min: 1.23,
                max: Some(2.68)
            }
        );
        assert_eq!(
            d.ml.unwrap(),
            DoseRange {
                min: 0.01,
                max: Some(0.03)
            }
        );
        assert_eq!(
            d.units.unwrap(),
            DoseRange {
                min: 0.5,
                max: Some(1.25)
            }
        );
        assert_eq!(d.warnings, r.warnings);
    }
    #[test]
    fn quarter_tie_rounds_up() {
        assert_eq!(
            calculate(&DoseInput {
                weight_kg: 1.0,
                mg_per_kg_min: 0.625,
                strength_mg_per_unit: Some(1.0),
                ..input()
            })
            .unwrap()
            .rounded()
            .units
            .unwrap()
            .min,
            0.75
        );
    }
    #[test]
    fn overflow_and_underflow_rejected() {
        assert_eq!(
            calculate(&DoseInput {
                mg_per_kg_min: f64::MAX,
                ..input()
            }),
            Err(DoseError::OutOfRange)
        );
        assert_eq!(
            calculate(&DoseInput {
                concentration_mg_per_ml: Some(f64::MIN_POSITIVE),
                ..input()
            }),
            Err(DoseError::OutOfRange)
        );
        assert_eq!(
            calculate(&DoseInput {
                weight_kg: f64::from_bits(1),
                mg_per_kg_min: 0.1,
                ..input()
            }),
            Err(DoseError::OutOfRange)
        );
    }
    #[test]
    fn rounding_large_finite_result_stays_finite() {
        let r = calculate(&DoseInput {
            weight_kg: 1.0,
            mg_per_kg_min: f64::MAX,
            ..input()
        })
        .unwrap();
        assert!(r.rounded().mg.min.is_finite());
    }
    #[test]
    fn large_integral_unit_count_does_not_overflow_fraction_check() {
        let r = calculate(&DoseInput {
            weight_kg: 1.0,
            mg_per_kg_min: f64::MAX,
            strength_mg_per_unit: Some(1.0),
            ..input()
        })
        .unwrap();
        assert!(r.warnings.is_empty());
    }
    #[test]
    fn tiny_units_warn_even_when_near_zero() {
        let r = calculate(&DoseInput {
            strength_mg_per_unit: Some(1e15),
            ..input()
        })
        .unwrap();
        assert!(r.warnings.contains(&DoseWarning::ImpracticalUnitCount));
    }
}
