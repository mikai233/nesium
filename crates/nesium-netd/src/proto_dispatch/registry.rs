//! Handler registry for automatic message dispatch.
//!
//! This module provides a registry-based approach for routing messages to
//! their handlers based on message ID, with automatic decoding.

use std::collections::HashMap;

use nesium_netproto::msg_id::MsgId;

use super::error::HandlerResult;
use super::handlers::{ErasedHandler, HandlerContext};

/// Registry mapping MsgId to type-erased handler trait objects.
pub(crate) struct HandlerRegistry {
    handlers: HashMap<MsgId, Box<dyn ErasedHandler>>,
}

impl HandlerRegistry {
    /// Create a new empty registry.
    pub(crate) fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }

    /// Register a type-erased handler for a message ID.
    pub(crate) fn register(&mut self, msg_id: MsgId, handler: Box<dyn ErasedHandler>) {
        self.handlers.insert(msg_id, handler);
    }

    /// Dispatch a message to its registered handler.
    ///
    /// Returns `None` if no handler is registered for the message ID.
    pub(crate) async fn dispatch(
        &self,
        msg_id: MsgId,
        ctx: &mut HandlerContext<'_>,
        payload: &[u8],
    ) -> Option<HandlerResult> {
        match self.handlers.get(&msg_id) {
            Some(handler) => Some(handler.handle_erased(ctx, payload).await),
            None => None,
        }
    }

    /// Check if a handler is registered for a message ID.
    #[allow(dead_code)]
    pub(crate) fn has_handler(&self, msg_id: MsgId) -> bool {
        self.handlers.contains_key(&msg_id)
    }
}

impl Default for HandlerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro to register struct-based handlers into a registry.
///
/// Each handler is a struct implementing `Handler<M>` for a specific message type.
/// The macro wraps them in `TypedHandler` for type-erased storage.
///
/// # Example
/// ```ignore
/// let registry = register_handlers! {
///     Hello => HelloHandler,
///     PauseGame => PauseGameHandler,
/// };
/// ```
#[macro_export]
macro_rules! register_handlers {
    ($($msg_type:ty => $handler:expr),* $(,)?) => {{
        use $crate::proto_dispatch::registry::HandlerRegistry;
        use $crate::proto_dispatch::handlers::TypedHandler;

        let mut registry = HandlerRegistry::new();

        $(
            registry.register(
                <$msg_type as nesium_netproto::messages::Message>::msg_id(),
                Box::new(TypedHandler::<$msg_type, _>::new($handler)),
            );
        )*

        registry
    }};
}
