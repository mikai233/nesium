//! Message dispatch via registry-based handlers.
//!
//! Each handler is a struct implementing `Handler<M>` for a specific message type.
//! The `register_handlers!` macro creates a registry that automatically decodes
//! messages and dispatches to the appropriate handler.

use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::OnceLock;

use nesium_netproto::messages::Message;
use nesium_netproto::messages::input::InputBatch;
use nesium_netproto::messages::session::{
    ErrorCode, ErrorMsg, Hello, JoinRoom, LoadRom, P2PCreateRoom, P2PJoinRoom, P2PRequestFallback,
    PauseGame, ProvideState, QueryRoom, RejoinReady, RequestFallbackRelay, RequestState, ResetGame,
    RomLoaded, SetSyncMode, SwitchRole, SyncState,
};
use nesium_netproto::messages::sync::Ping;
use tracing::warn;

use super::error::{HandlerError, HandlerResult};
use super::registry::HandlerRegistry;
use crate::ConnCtx;
use crate::net::framing::PacketOwned;
use crate::net::inbound::ConnId;
use crate::net::outbound::send_msg;
use crate::room::state::{Room, RoomManager};

mod hello;
mod input_batch;
mod join_room;
mod load_rom;
mod p2p_create_room;
mod p2p_join_room;
mod p2p_request_fallback;
mod pause_game;
mod ping;
mod provide_state;
mod query_room;
mod rejoin_ready;
mod request_fallback_relay;
mod request_state;
mod reset_game;
mod rom_loaded;
mod set_sync_mode;
mod switch_role;
mod sync_state;

// ============================================================================
// Handler Trait System
// ============================================================================

/// Context passed to message handlers.
///
/// Contains all the information a handler might need to process a message.
pub(crate) struct HandlerContext<'a> {
    /// The connection context (outbound channels, client_id, etc.)
    pub(crate) conn_ctx: &'a mut ConnCtx,
    /// The connection ID for this client.
    pub(crate) conn_id: ConnId,
    /// The peer's socket address.
    pub(crate) peer: &'a SocketAddr,
    /// The room manager for accessing/modifying room state.
    pub(crate) room_mgr: &'a mut RoomManager,
    /// Room inactivity timeout in seconds (for Welcome message).
    pub(crate) room_idle_timeout_secs: u16,
}

impl HandlerContext<'_> {
    /// Get the assigned client ID or return an error.
    pub(crate) fn require_client_id(&self) -> Result<u32, HandlerError> {
        self.conn_ctx
            .assigned_client_id
            .ok_or_else(HandlerError::invalid_state)
    }

    /// Get the room the client is in or return an error.
    pub(crate) fn require_room(&self) -> Result<&Room, HandlerError> {
        let client_id = self.require_client_id()?;
        self.room_mgr
            .client_room(client_id)
            .ok_or_else(HandlerError::not_in_room)
    }

    /// Get the mutable room the client is in or return an error.
    pub(crate) fn require_room_mut(&mut self) -> Result<&mut Room, HandlerError> {
        let client_id = self.require_client_id()?;
        self.room_mgr
            .client_room_mut(client_id)
            .ok_or_else(HandlerError::not_in_room)
    }
}

/// Async trait for type-safe message handlers.
///
/// Each handler implements this trait for the specific message type it handles.
/// The handler receives the decoded message directly, without needing to
/// deserialize from bytes.
///
/// # Example
/// ```ignore
/// pub struct HelloHandler;
///
/// impl Handler<Hello> for HelloHandler {
///     async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: Hello) -> HandlerResult {
///         // msg is already the decoded Hello struct
///         // ...
///         Ok(())
///     }
/// }
/// ```
pub(crate) trait Handler<M: Message>: Send + Sync {
    /// Handle the incoming message.
    fn handle(
        &self,
        ctx: &mut HandlerContext<'_>,
        msg: M,
    ) -> impl Future<Output = HandlerResult> + Send;
}

/// Type-erased handler trait for dynamic dispatch.
///
/// This trait allows storing handlers of different message types in a single
/// registry and dispatching to them based on the message ID.
pub(crate) trait ErasedHandler: Send + Sync {
    /// Handle a message from raw bytes.
    ///
    /// This method decodes the message and dispatches to the typed handler.
    fn handle_erased<'a>(
        &'a self,
        ctx: &'a mut HandlerContext<'_>,
        payload: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = HandlerResult> + Send + 'a>>;
}

/// Wrapper to implement ErasedHandler for any Handler<M>
pub(crate) struct TypedHandler<M: Message, H: Handler<M>> {
    handler: H,
    _marker: std::marker::PhantomData<M>,
}

impl<M: Message, H: Handler<M>> TypedHandler<M, H> {
    pub(crate) fn new(handler: H) -> Self {
        Self {
            handler,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<M: Message + Sync, H: Handler<M>> ErasedHandler for TypedHandler<M, H> {
    fn handle_erased<'a>(
        &'a self,
        ctx: &'a mut HandlerContext<'_>,
        payload: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = HandlerResult> + Send + 'a>> {
        Box::pin(async move {
            let msg: M = match postcard::from_bytes(payload) {
                Ok(m) => m,
                Err(_) => return Err(HandlerError::bad_message()),
            };
            self.handler.handle(ctx, msg).await
        })
    }
}

// ============================================================================
// Registry and Dispatch
// ============================================================================

/// Global handler registry, initialized once.
static REGISTRY: OnceLock<HandlerRegistry> = OnceLock::new();

/// Build the handler registry with all message handlers.
fn build_registry() -> HandlerRegistry {
    crate::register_handlers! {
        Hello => hello::HelloHandler,
        JoinRoom => join_room::JoinRoomHandler,
        InputBatch => input_batch::InputBatchHandler,
        SwitchRole => switch_role::SwitchRoleHandler,
        LoadRom => load_rom::LoadRomHandler,
        RomLoaded => rom_loaded::RomLoadedHandler,
        PauseGame => pause_game::PauseGameHandler,
        ResetGame => reset_game::ResetGameHandler,
        RequestState => request_state::RequestStateHandler,
        SyncState => sync_state::SyncStateHandler,
        ProvideState => provide_state::ProvideStateHandler,
        RejoinReady => rejoin_ready::RejoinReadyHandler,
        QueryRoom => query_room::QueryRoomHandler,
        P2PCreateRoom => p2p_create_room::P2PCreateRoomHandler,
        P2PJoinRoom => p2p_join_room::P2PJoinRoomHandler,
        P2PRequestFallback => p2p_request_fallback::P2PRequestFallbackHandler,
        RequestFallbackRelay => request_fallback_relay::RequestFallbackRelayHandler,
        SetSyncMode => set_sync_mode::SetSyncModeHandler,
        Ping => ping::PingHandler,
    }
}

/// Get the global handler registry.
fn get_registry() -> &'static HandlerRegistry {
    REGISTRY.get_or_init(build_registry)
}

/// Sends an error response to the client.
pub(crate) async fn send_error_response(ctx: &mut ConnCtx, error: HandlerError) {
    let msg = ErrorMsg { code: error.code };
    let _ = send_msg(&ctx.outbound, &msg).await;
}

/// Dispatch an incoming packet to its registered handler.
///
/// Uses the global registry to find and invoke the appropriate handler
/// based on the message ID. The handler receives the decoded message directly.
pub(crate) async fn dispatch_packet(
    ctx: &mut ConnCtx,
    conn_id: ConnId,
    peer: &SocketAddr,
    packet: &PacketOwned,
    room_mgr: &mut RoomManager,
    room_idle_timeout_secs: u16,
) -> bool {
    let mut handler_ctx = HandlerContext {
        conn_ctx: ctx,
        conn_id,
        peer,
        room_mgr,
        room_idle_timeout_secs,
    };

    let result = get_registry()
        .dispatch(packet.msg_id(), &mut handler_ctx, &packet.payload)
        .await;

    match result {
        Some(Ok(())) => true,
        Some(Err(e)) => {
            let code = e.code;
            send_error_response(handler_ctx.conn_ctx, e).await;
            // Only disconnect if it's a protocol-level error like BadMessage.
            // Business errors (RoomNotFound, etc.) should not close the connection.
            code != ErrorCode::BadMessage
        }
        None => {
            warn!(
                conn_id,
                client_id = handler_ctx.conn_ctx.assigned_client_id.unwrap_or(0),
                msg_id = ?packet.msg_id(),
                payload_len = packet.payload.len(),
                %peer,
                "Unhandled message (will disconnect)"
            );
            false
        }
    }
}
