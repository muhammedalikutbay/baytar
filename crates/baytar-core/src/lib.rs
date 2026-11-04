//! Shared domain logic for Baytar.

pub mod backup;
pub mod db;
pub mod dose;
pub mod export;
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
        // The release script bumps the version; only its shape is checked here.
        let parts: Vec<&str> = super::version().split('.').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts.iter().all(|part| part.parse::<u32>().is_ok()));
    }
}
