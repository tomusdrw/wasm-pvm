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
    ///
    /// The inner error is named `cause` (not `source`) deliberately: thiserror
    /// auto-treats a field named `source` as the chain link, which would make
    /// anyhow print the same message twice — once via this variant's Display
    /// (which interpolates `{cause}`) and once via walking `.source()`.
    /// Pattern-matching on the variant still gives direct access via `cause`.
    #[error("{cause} (in function #{func_idx} '{func_name}' at byte offset 0x{op_offset:x})")]
    Located {
        func_idx: usize,
        func_name: String,
        op_offset: usize,
        cause: Box<Error>,
    },
}

pub type Result<T> = std::result::Result<T, Error>;
