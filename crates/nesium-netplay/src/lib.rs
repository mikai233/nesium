//! NES Netplay Client Library
//!
//! This crate provides TCP-based netplay support for the NES emulator.
//! It implements a lockstep model with input buffering, similar to Mesen2.
//!
//! # Architecture
//!
//! - [`session`]: Session state machine and input queue management
//! - [`tcp_client`]: Async TCP client for server communication
//! - [`input_provider`]: Interface for injecting network inputs into the NES runtime
//! - [`handler`]: Session handler for protocol message processing
//! - [`error`]: Error types

pub mod error;
pub mod handler;
pub mod input_provider;
pub mod session;
pub mod tcp_client;

// Re-export commonly used types
pub use error::NetplayError;
pub use handler::{NetplayCommand, NetplayConfig, NetplayEvent, SessionHandler};
pub use input_provider::{NetplayInputProvider, SharedInputProvider, create_input_provider};
pub use nesium_netproto::constants::SPECTATOR_PLAYER_INDEX;
pub use session::{NetplaySession, SessionState};
pub use tcp_client::{
    TcpClientEvent, TcpClientHandle, connect, connect_auto, connect_auto_pinned, connect_quic,
    connect_quic_pinned,
};
