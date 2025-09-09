use async_channel::RecvError;
use derive_more::From;
use std::fmt::{self, Debug};
use thiserror::Error;

#[doc(hidden)]
#[derive(Error, Debug, From)]
pub enum ActorRuntimeError {
    #[error("Init error: {0}")]
    Init(ActorInitFailure),
    #[error("Receive message error: {0}")]
    Receive(RecvError),
    #[error("Stop hook error: {0}")]
    Stop(ActorStopFailure),
    #[error("Error while processing handler error: {0}")]
    HandleError(ActorHandleErrorFailure),
}

#[derive(Error, Debug)]
#[error("Actor init error: {0}")]
struct ActorInitFailureContainer(#[source] Box<dyn std::error::Error + Send + Sync>);
#[derive(Debug)]
pub struct ActorInitFailure(ActorInitFailureContainer);
impl<E> From<E> for ActorInitFailure
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        let container = ActorInitFailureContainer(Box::new(value));
        ActorInitFailure(container)
    }
}
impl fmt::Display for ActorInitFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
#[error("Actor stop error: {0}")]
struct ActorStopFailureContainer(#[source] Box<dyn std::error::Error + Send + Sync>);
#[derive(Debug)]
pub struct ActorStopFailure(ActorStopFailureContainer);
impl<E> From<E> for ActorStopFailure
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        let container = ActorStopFailureContainer(Box::new(value));
        ActorStopFailure(container)
    }
}
impl fmt::Display for ActorStopFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Error, Debug)]
#[error("Actor handle error error: {0}")]
struct ActorHandleErrorFailureContainer(#[source] Box<dyn std::error::Error + Send + Sync>);
#[derive(Debug)]
pub struct ActorHandleErrorFailure(ActorHandleErrorFailureContainer);
impl<E> From<E> for ActorHandleErrorFailure
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        let container = ActorHandleErrorFailureContainer(Box::new(value));
        ActorHandleErrorFailure(container)
    }
}
impl fmt::Display for ActorHandleErrorFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
