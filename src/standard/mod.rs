mod common {
  pub use standard::sleeper;
  pub use standard::delayed;
  pub use standard::delayed_series;
  pub use standard::instant;
  pub use standard::instant_series;
  pub use standard::buffered;
}

pub mod sleeper;
pub mod instant;
pub mod delayed;
pub mod instant_series;
pub mod delayed_series;
pub mod buffered;
