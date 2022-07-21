// mod formatter;
mod pendant;
mod shader;
pub mod styled_buffer;
mod snippet;


#[derive(Copy, PartialEq, Eq, Clone, Hash, Debug)]
pub enum Level {
    Bug,
    DelayedBug,
    Fatal,
    Error {
        /// If this error comes from a lint, don't abort compilation even when abort_if_errors() is called.
        lint: bool,
    },
    /// This [`LintExpectationId`] is used for expected lint diagnostics, which should
    /// also emit a warning due to the `force-warn` flag. In all other cases this should
    /// be `None`.
    Warning,
    Note,
    /// A note that is only emitted once.
    OnceNote,
    Help,
    FailureNote,
    Allow,
    Expect,
}

pub enum PendantTyep{
    Header,
    Label,
    CodeCtx
}
