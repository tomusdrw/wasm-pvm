#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("WASM parsing error: {0}")]
    WasmParse(#[from] wasmparser::BinaryReaderError),

    #[error("Unsupported WASM feature: {0}")]
    Unsupported(String),

    #[error("Float operations are not supported by PVM")]
    FloatNotSupported,

    #[error("No exported function found")]
    NoExportedFunction,

    #[error("Function index {0} not found")]
    FunctionNotFound(u32),

    #[error("Unresolved import: {0}")]
    UnresolvedImport(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
