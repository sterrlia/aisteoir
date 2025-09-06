use std::fmt::{self, Debug};
use thiserror::Error;

#[derive(Error, Debug)]
#[error("Receiver closed: {0}")]
pub struct ReceiverClosedError(#[source] Box<dyn std::error::Error + Send + Sync>);

impl ReceiverClosedError {
    pub fn new(source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        Self(source)
    }
}

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
struct ActorInitErrorContainer(#[source] Box<dyn std::error::Error + Send + Sync>);
pub struct ActorInitError(ActorInitErrorContainer);

impl<E> From<E> for ActorInitError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        let container = ActorInitErrorContainer(Box::new(value));
        ActorInitError(container)
    }
}

impl fmt::Display for ActorInitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
#[error("Actor stop error: {0}")]
struct ActorStopErrorContainer(#[source] Box<dyn std::error::Error + Send + Sync>);
pub struct ActorStopError(ActorStopErrorContainer);

impl<E> From<E> for ActorStopError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        let container = ActorStopErrorContainer(Box::new(value));
        ActorStopError(container)
    }
}

impl fmt::Display for ActorStopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
#[error("Fatal error {0}")]
pub struct FatalError(#[source] Box<dyn std::error::Error + Send + Sync>);

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
        let error = FatalError(Box::new(value));

        DefaultHandleError::Fatal(error)
    }
}
