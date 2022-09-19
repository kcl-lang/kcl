use anyhow::Result;
use compiler_base_error::{diagnostic_handler::DiagnosticHandler, Diagnostic, DiagnosticStyle};
use compiler_base_span::SourceMap;
use std::sync::Arc;

/// Represents the data associated with a compilation
/// session for a single crate.
///
/// Note: TODO(zongz): This is a WIP structure.
/// Currently only contains the part related to error diagnostic displaying.
pub struct Session {
    pub sm: Arc<SourceMap>,
    pub diag_handler: Arc<DiagnosticHandler>,
}

impl Session {
    /// Construct a `Session`
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use compiler_base_session::Session;
    /// # use compiler_base_error::diagnostic_handler::DiagnosticHandler;
    /// # use std::path::PathBuf;
    /// # use compiler_base_span::FilePathMapping;
    /// # use compiler_base_span::SourceMap;
    /// # use std::sync::Arc;
    /// # use std::fs;
    /// 
    /// // 1. You should create a new `SourceMap` wrapped with `Arc`.
    /// let filename = fs::canonicalize(&PathBuf::from("./src/test_datas/code_snippet")).unwrap().display().to_string();
    /// let src = std::fs::read_to_string(filename.clone()).unwrap();
    /// let sm = Arc::new(SourceMap::new(FilePathMapping::empty()));
    /// sm.new_source_file(PathBuf::from(filename.clone()).into(), src.to_string());
    /// 
    /// // 2. You should create a new `DiagnosticHandler` wrapped with `Arc`.
    /// let diag_handler = Arc::new(DiagnosticHandler::new_with_template_dir("./src/test_datas/locales/en-US").unwrap());
    /// 
    /// // 3. Create `Session`
    /// let sess = Session::new(sm, diag_handler);
    /// 
    /// ```
    #[inline]
    pub fn new(sm: Arc<SourceMap>, diag_handler: Arc<DiagnosticHandler>) -> Self {
        Self { sm, diag_handler }
    }
}

/// Trait implemented by error types.
///
/// You can implement manually for error types as below.
///
/// # Example
///
/// ```rust
/// use anyhow::Result;
/// use compiler_base_error::components::Label;
/// use compiler_base_error::DiagnosticStyle;
/// use compiler_base_error::Diagnostic;
/// use compiler_base_session::Session;
/// use compiler_base_session::SessionDiagnostic;
///
/// // 1. Create your own error type.
/// struct MyError;
///
/// // 2. Implement trait `SessionDiagnostic` manually.
/// impl SessionDiagnostic for MyError {
///     fn into_diagnostic(self, sess: &Session) -> Result<Diagnostic<DiagnosticStyle>> {
///         let mut diag = Diagnostic::<DiagnosticStyle>::new();
///         // 1. Label Component
///         let label_component = Box::new(Label::Error("error".to_string()));
///         diag.append_component(label_component);
///         Ok(diag)
///     }
/// }
///
/// // 3. The diagnostic of MyError will display "error" on terminal.
/// // For more information about diagnositc displaying, see doc in `compiler_base_error`.
/// ```
///
/// Note:
/// TODO(zongz): `#[derive(SessionDiagnostic)]` is WIP, before that you need to manually implement this trait.
/// This should not be implemented manually. Instead, use `#[derive(SessionDiagnostic)]` in the future.
pub trait SessionDiagnostic {
    fn into_diagnostic(self, sess: &Session) -> Result<Diagnostic<DiagnosticStyle>>;
}
