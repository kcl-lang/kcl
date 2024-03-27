use crossbeam_channel::Sender;
use lsp_server::{ExtractError, Request};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::error::Error;

use crate::{
    error::LSPError,
    state::{LanguageServerSnapshot, LanguageServerState, Task},
    util::from_json,
};

pub(crate) struct NotificationDispatcher<'a> {
    state: &'a mut LanguageServerState,
    notification: Option<lsp_server::Notification>,
}

impl<'a> NotificationDispatcher<'a> {
    /// Constructs a new dispatcher for the specified request
    pub fn new(state: &'a mut LanguageServerState, notification: lsp_server::Notification) -> Self {
        NotificationDispatcher {
            state,
            notification: Some(notification),
        }
    }

    /// Try to dispatch the event as the given Notification type.
    pub fn on<N>(
        &mut self,
        handle_notification_fn: fn(&mut LanguageServerState, N::Params) -> anyhow::Result<()>,
    ) -> anyhow::Result<&mut Self>
    where
        N: lsp_types::notification::Notification + 'static,
        N::Params: DeserializeOwned + Send + 'static,
    {
        let notification = match self.notification.take() {
            Some(it) => it,
            None => return Ok(self),
        };
        let params = match notification.extract::<N::Params>(N::METHOD) {
            Ok(it) => it,
            Err(ExtractError::JsonError { method, error }) => {
                panic!("Invalid request\nMethod: {method}\n error: {error}",)
            }
            Err(ExtractError::MethodMismatch(notification)) => {
                self.notification = Some(notification);
                return Ok(self);
            }
        };
        handle_notification_fn(self.state, params)?;
        Ok(self)
    }

    /// Wraps-up the dispatcher. If the notification was not handled, log an error.
    pub fn finish(&mut self) {
        if let Some(notification) = &self.notification {
            if !notification.method.starts_with("$/") {
                log::error!("unhandled notification: {:?}", notification);
            }
        }
    }
}

/// A helper struct to ergonomically dispatch LSP requests to functions.
pub(crate) struct RequestDispatcher<'a> {
    state: &'a mut LanguageServerState,
    request: Option<lsp_server::Request>,
}

impl<'a> RequestDispatcher<'a> {
    /// Constructs a new dispatcher for the specified request
    pub fn new(state: &'a mut LanguageServerState, request: lsp_server::Request) -> Self {
        RequestDispatcher {
            state,
            request: Some(request),
        }
    }

    /// Try to dispatch the event as the given Request type on the current thread.
    pub fn on_sync<R>(
        &mut self,
        compute_response_fn: fn(&mut LanguageServerState, R::Params) -> anyhow::Result<R::Result>,
    ) -> anyhow::Result<&mut Self>
    where
        R: lsp_types::request::Request + 'static,
        R::Params: DeserializeOwned + 'static,
        R::Result: Serialize + 'static,
    {
        let (req, params) = match self.parse::<R>() {
            Some(it) => it,
            None => return Ok(self),
        };

        let result = compute_response_fn(self.state, params);
        let response = result_to_response::<R>(req.id, result);
        let _result = self.state.respond(response);
        Ok(self)
    }

    /// Try to dispatch the event as the given Request type on the thread pool.
    pub fn on<R>(
        &mut self,
        compute_response_fn: fn(
            LanguageServerSnapshot,
            R::Params,
            Sender<Task>,
        ) -> anyhow::Result<R::Result>,
    ) -> anyhow::Result<&mut Self>
    where
        R: lsp_types::request::Request + 'static,
        R::Params: DeserializeOwned + 'static + Send,
        R::Result: Serialize + 'static,
    {
        let (req, params) = match self.parse::<R>() {
            Some(it) => it,
            None => return Ok(self),
        };

        self.state.thread_pool.execute({
            let snapshot = self.state.snapshot();
            let sender = self.state.task_sender.clone();

            move || {
                let result = compute_response_fn(snapshot, params, sender.clone());
                match &result {
                    Err(e)
                        if e.downcast_ref::<LSPError>()
                            .map_or(false, |lsp_err| matches!(lsp_err, LSPError::Retry)) =>
                    {
                        sender.send(Task::Retry(req)).unwrap();
                    }
                    _ => {
                        sender
                            .send(Task::Response(result_to_response::<R>(req.id, result)))
                            .unwrap();
                    }
                }
            }
        });

        Ok(self)
    }

    /// Tries to parse the request as the specified type. If the request is of the specified type,
    /// the request is transferred and any subsequent call to this method will return None. If an
    /// error is encountered during parsing of the request parameters an error is send to the
    /// client.
    fn parse<R>(&mut self) -> Option<(Request, R::Params)>
    where
        R: lsp_types::request::Request + 'static,
        R::Params: DeserializeOwned + 'static,
    {
        let req = match &self.request {
            Some(req) if req.method == R::METHOD => self.request.take().unwrap(),
            _ => return None,
        };

        match from_json(R::METHOD, req.params.clone()) {
            Ok(params) => Some((req, params)),
            Err(err) => {
                let response = lsp_server::Response::new_err(
                    req.id,
                    lsp_server::ErrorCode::InvalidParams as i32,
                    err.to_string(),
                );
                let _result = self.state.respond(response);
                None
            }
        }
    }

    /// Wraps-up the dispatcher. If the request was not handled, report back that this is an
    /// unknown request.
    pub fn finish(&mut self) {
        if let Some(req) = self.request.take() {
            log::error!("unknown request: {:?}", req);
            let response = lsp_server::Response::new_err(
                req.id,
                lsp_server::ErrorCode::MethodNotFound as i32,
                "unknown request".to_string(),
            );
            let _result = self.state.respond(response);
        }
    }
}

/// Converts the specified results of an LSP request into an LSP response handling any errors that
/// may have occurred.
fn result_to_response<R>(
    id: lsp_server::RequestId,
    result: anyhow::Result<R::Result>,
) -> lsp_server::Response
where
    R: lsp_types::request::Request + 'static,
    R::Params: DeserializeOwned + 'static,
    R::Result: Serialize + 'static,
{
    match result {
        Ok(resp) => lsp_server::Response::new_ok(id, &resp),
        Err(e) => {
            if is_canceled(&*e) {
                lsp_server::Response::new_err(
                    id,
                    lsp_server::ErrorCode::ContentModified as i32,
                    "content modified".to_string(),
                )
            } else {
                lsp_server::Response::new_err(
                    id,
                    lsp_server::ErrorCode::InternalError as i32,
                    e.to_string(),
                )
            }
        }
    }
}

/// An error signifying a cancelled operation.
pub struct Canceled {
    // This is here so that you cannot construct a Canceled
    _private: (),
}

impl Canceled {
    #[allow(unused)]
    fn new() -> Self {
        Canceled { _private: () }
    }
    #[allow(unused)]
    pub fn throw() -> ! {
        // We use resume and not panic here to avoid running the panic
        // hook (that is, to avoid collecting and printing backtrace).
        std::panic::resume_unwind(Box::new(Canceled::new()))
    }
}

impl std::fmt::Display for Canceled {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt.write_str("canceled")
    }
}

impl std::fmt::Debug for Canceled {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "Canceled")
    }
}

impl std::error::Error for Canceled {}

/// Returns true if the specified error is of type [`Canceled`]
pub(crate) fn is_canceled(e: &(dyn Error + 'static)) -> bool {
    e.downcast_ref::<Canceled>().is_some()
}
