#[macro_use]
extern crate optional_struct;
#[macro_use]
extern crate serde_with;

#[cfg(feature = "logic")]
pub mod gateway;
pub mod models;

#[cfg(feature = "logic")]
pub use gateway::GatewayClient;
