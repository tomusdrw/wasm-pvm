#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(feature = "compiler")]
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

    /// Wrapper attaching the source WASM operator location to a translation error.
    /// Produced by the function-body translator so users can pinpoint which operator
    /// in which function caused an unsupported-feature failure.
    #[error("{source} (in function #{func_idx} '{func_name}' at byte offset 0x{op_offset:x})")]
    Located {
        func_idx: usize,
        func_name: String,
        op_offset: usize,
        #[source]
        source: Box<Error>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
