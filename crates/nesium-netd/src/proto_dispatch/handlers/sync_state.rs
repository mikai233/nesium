//! SyncStateHandler - handles SyncState messages from clients.
//!
//! Note: Clients send ProvideState, not SyncState.
//! SyncState is a server-to-client message.

use nesium_netproto::messages::session::SyncState;

use super::{Handler, HandlerContext};
use crate::proto_dispatch::error::HandlerResult;

/// Handler for SyncState messages (no-op, as clients should use ProvideState).
pub(crate) struct SyncStateHandler;

impl Handler<SyncState> for SyncStateHandler {
    async fn handle(&self, _ctx: &mut HandlerContext<'_>, _msg: SyncState) -> HandlerResult {
        // Clients send ProvideState, not SyncState. Use ProvideState for C->S state updates.
        Ok(())
    }
}
