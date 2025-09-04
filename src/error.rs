use std::fmt::Debug;
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
pub enum DefaultActorError {
    #[error("Fatal: {0}")]
    Fatal(String),
}

