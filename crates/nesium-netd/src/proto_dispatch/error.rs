//! Handler error types for unified error responses.

use nesium_netproto::messages::session::ErrorCode;

/// Error returned by handlers to trigger automatic error response to client.
#[derive(Debug)]
pub struct HandlerError {
    pub code: ErrorCode,
}

impl HandlerError {
    pub fn bad_message() -> Self {
        Self {
            code: ErrorCode::BadMessage,
        }
    }

    pub fn room_not_found() -> Self {
        Self {
            code: ErrorCode::RoomNotFound,
        }
    }

    pub fn already_in_room() -> Self {
        Self {
            code: ErrorCode::AlreadyInRoom,
        }
    }

    pub fn not_in_room() -> Self {
        Self {
            code: ErrorCode::NotInRoom,
        }
    }

    pub fn permission_denied() -> Self {
        Self {
            code: ErrorCode::PermissionDenied,
        }
    }

    pub fn game_already_started() -> Self {
        Self {
            code: ErrorCode::GameAlreadyStarted,
        }
    }

    pub fn invalid_state() -> Self {
        Self {
            code: ErrorCode::InvalidState,
        }
    }

    pub fn host_not_available() -> Self {
        Self {
            code: ErrorCode::HostNotAvailable,
        }
    }
}

/// Convenient Result type for handlers.
pub type HandlerResult = Result<(), HandlerError>;
