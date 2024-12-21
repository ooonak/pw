pub mod machine;
mod error;
mod utils;

/// Trait to access machine information.
pub trait Machine {
    fn bootid(&self) -> &str;
    fn mac(&self) -> u64;
}
