use thiserror::Error;

#[derive(Debug, Error)]
pub enum IndexerError {
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),

    #[error("RPC connection failed for {chain}: {reason}")]
    RpcConnectionFailed { chain: String, reason: String },

    #[error("Block sync error on {chain} at block {block}: {reason}")]
    BlockSyncError { chain: String, block: u64, reason: String },

    #[error("Transaction decoding failed: {0}")]
    DecodeError(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Position tracking error: {0}")]
    PositionError(String),

    #[error("Protocol parser error for {protocol}: {reason}")]
    ParserError { protocol: String, reason: String },

    #[error("Token not recognized: {0}")]
    UnknownToken(String),

    #[error("Chain reorganization detected on {chain} at block {block}")]
    ReorgDetected { chain: String, block: u64 },

    #[error("Indexer not initialized")]
    NotInitialized,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}   