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
    ///
    /// `op_offset` is `Some(byte_offset)` for errors raised by the WASM-operator
    /// dispatcher in the frontend, where the precise byte offset is known. It is
    /// `None` for errors raised by the LLVM-IR-to-PVM backend, where we have
    /// already lost the WASM byte offset and only know the function identity.
    ///
    /// The inner error is named `cause` (not `source`) deliberately: thiserror
    /// auto-treats a field named `source` as the chain link, which would make
    /// anyhow print the same message twice — once via this variant's Display
    /// (which interpolates `{cause}`) and once via walking `.source()`.
    /// Pattern-matching on the variant still gives direct access via `cause`.
    #[error("{cause} (in function #{func_idx} '{func_name}'{})", format_op_offset(op_offset.as_ref()))]
    Located {
        func_idx: usize,
        func_name: String,
        op_offset: Option<usize>,
        cause: Box<Error>,
    },

    /// Wrapper attaching adapter-merge context (which adapter element / module-side
    /// label) to an error raised before any LLVM IR exists. Adapter-merge errors
    /// have no function index or operator byte offset to attach — only a textual
    /// context like `"adapter func 3"` or `"main type section"`.
    ///
    /// See `Located` for why the inner error is named `cause` rather than `source`.
    #[error("{cause} (during adapter merge: {context})")]
    AdapterMerge { context: String, cause: Box<Error> },
}

fn format_op_offset(op_offset: Option<&usize>) -> String {
    match op_offset {
        Some(o) => format!(" at byte offset 0x{o:x}"),
        None => " during PVM lowering".to_string(),
    }
}

pub type Result<T> = std::result::Result<T, Error>;
