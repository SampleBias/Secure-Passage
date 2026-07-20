//! Events and connection state shared with the UI.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferKind {
    Send,
    Receive,
}

#[derive(Debug, Clone)]
pub enum AppEvent {
    Log {
        level: String,
        message: String,
    },
    VpnStatus {
        detected: bool,
        detail: String,
    },
    ConnectProgress {
        percent: u8,
        message: String,
    },
    Connected {
        address: String,
    },
    Disconnected,
    ConnectFailed {
        error: String,
    },
    ChatHosting {
        address: String,
        session_key: String,
    },
    ChatJoined {
        peer: String,
    },
    ChatMessage {
        sender_id: String,
        content: String,
        timestamp: f64,
        message_id: String,
    },
    ChatPeerLeft,
    ChatError {
        error: String,
    },
    TransferProgress {
        kind: TransferKind,
        file_name: String,
        bytes_done: u64,
        bytes_total: u64,
    },
    TransferComplete {
        kind: TransferKind,
        file_name: String,
        path: String,
    },
    TransferError {
        error: String,
    },
    FileHosting {
        address: String,
        session_key: String,
        file_name: String,
    },
}
