use lsp_server::ExtractError;
use serde::de::DeserializeOwned;

use crate::state::LanguageServerState;

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
