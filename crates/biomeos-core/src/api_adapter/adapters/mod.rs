//! Primal-Specific API Adapters
//!
//! Each primal gets its own adapter that understands its unique API structure.

pub mod beardog;
pub mod nestgate;
pub mod songbird;
pub mod squirrel;
pub mod toadstool;

// Re-export for convenience
pub use beardog::BearDogAdapter;
pub use nestgate::NestGateAdapter;
pub use songbird::SongbirdAdapter;
pub use squirrel::SquirrelAdapter;
pub use toadstool::ToadStoolAdapter;
