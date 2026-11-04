//! Shared domain logic for Baytar.

pub mod backup;
pub mod db;
pub mod legacy;
pub mod model;
pub mod repo;
pub mod text;

pub const APP_NAME: &str = "Baytar";

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    #[test]
    fn exposes_application_identity() {
        assert_eq!(super::APP_NAME, "Baytar");
        assert_eq!(super::version(), "0.2.0");
    }
}
