use std::fmt::{self, Debug};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Receiver closed: {0}")]
pub struct ReceiverClosedError(pub String);

#[derive(Error, Debug)]
#[error("Receiver handle error")]
pub struct ReceiverHandleError;

#[derive(Error, Debug)]
pub enum CallError {
    #[error("{0}")]
    ReceiverClosed(ReceiverClosedError),
    #[error("{0}")]
    ReceiverHandleError(ReceiverHandleError),
}

#[doc(hidden)]
#[derive(Error, Debug)]
pub enum CallHandleError<E>
where
    E: Debug,
{
    #[error("Send ok error")]
    SendOk,
    #[error("Handle error: {0}; and error response error")]
    SendError(E),
    #[error("Handle error: {0}")]
    Handle(E),
}

#[doc(hidden)]
#[derive(Error, Debug)]
pub enum HandleError<E>
where
    E: Debug,
{
    CallHandleError(CallHandleError<E>),
    TellHandleError(E),
}

impl<E> From<E> for HandleError<E>
where
    E: Debug,
{
    fn from(value: E) -> Self {
        HandleError::TellHandleError(value)
    }
}

impl<E> From<CallHandleError<E>> for HandleError<E>
where
    E: Debug,
{
    fn from(value: CallHandleError<E>) -> Self {
        HandleError::CallHandleError(value)
    }
}

#[derive(Error, Debug)]
#[error("Actor init error: {0}")]
pub struct ActorInitError(String);

#[derive(Error, Debug)]
#[error("Actor stop error: {0}")]
pub struct ActorStopError(String);

#[derive(Error, Debug)]
#[error("Fatal error {source}")]
pub struct FatalError {
    #[source]
    source: Box<dyn std::error::Error + Send + Sync>
}

#[derive(Debug)]
pub enum DefaultHandleError {
    Fatal(FatalError),
}

impl fmt::Display for DefaultHandleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefaultHandleError::Fatal(inner) => write!(f, "{inner}"),
        }
    }
}

impl<E> From<E> for DefaultHandleError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        let error = FatalError {
            source: Box::new(value)
        };

        DefaultHandleError::Fatal(error)
    }
}
