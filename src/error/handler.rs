use derive_more::From;
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
pub struct ReceiverHandlerError;

#[derive(Error, Debug, From)]
pub enum AskError {
    #[error("{0}")]
    ReceiverClosed(ReceiverClosedError),
    #[error("{0}")]
    ReceiverHandlerError(ReceiverHandlerError),
}

#[doc(hidden)]
#[derive(Error, Debug)]
pub enum AskHandlerError<E>
where
    E: Debug,
{
    #[error("Send ok error: {0}")]
    SendOk(Box<dyn std::error::Error + Send + Sync>),
    #[error("Handle error: {0} and send error: {1}")]
    SendError(E, Box<dyn std::error::Error + Send + Sync>),
    #[error("Handle error: {0}")]
    Handle(E),
}

#[doc(hidden)]
#[derive(Error, Debug)]
#[error("Tell handle error: {0}")]
pub struct TellHandlerError<E>(#[source] pub E);

#[doc(hidden)]
#[derive(Error, Debug)]
pub enum BaseHandlerError<E>
where
    E: Debug,
{
    #[error("Ask handler error: {0}")]
    AskHandlerError(AskHandlerError<E>),
    #[error("Tell handler error: {0}")]
    TellHandlerError(TellHandlerError<E>),
}

impl<E> From<TellHandlerError<E>> for BaseHandlerError<E>
where
    E: Debug,
{
    fn from(value: TellHandlerError<E>) -> Self {
        BaseHandlerError::TellHandlerError(value)
    }
}

impl<E> From<AskHandlerError<E>> for BaseHandlerError<E>
where
    E: Debug,
{
    fn from(value: AskHandlerError<E>) -> Self {
        BaseHandlerError::AskHandlerError(value)
    }
}

#[derive(Error, Debug)]
#[error("Fatal error {0}")]
pub struct FatalError(#[source] Box<dyn std::error::Error + Send + Sync>);

#[derive(Debug)]
pub enum DefaultHandlerError {
    Fatal(FatalError),
}

impl fmt::Display for DefaultHandlerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DefaultHandlerError::Fatal(inner) => write!(f, "{inner}"),
        }
    }
}

impl<E> From<E> for DefaultHandlerError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        let error = FatalError(Box::new(value));

        DefaultHandlerError::Fatal(error)
    }
}
