use std::fmt::{Debug, Display};

use crate::{
    error::{AskHandleError, HandleError, ReceiverHandleError},
    logging,
    messaging::{AskMessage, TellMessage},
};
use async_trait::async_trait;

#[doc(hidden)]
pub struct BaseHandler {}

#[doc(hidden)]
#[async_trait]
pub trait BaseHandlerTrait<A, S, M, R> {
    async fn _handle(actor: &A, state: &mut S, msg: M) -> R;
}

#[async_trait]
impl<A, S, I, O, E> BaseHandlerTrait<A, S, AskMessage<I, O>, Result<(), AskHandleError<E>>>
    for BaseHandler
where
    A: AskHandlerTrait<S, I, O, E> + Sync + Send + 'static,
    S: Send,
    I: Send + 'static,
    O: Send + Sync + 'static,
    E: Display + Debug,
{
    async fn _handle(
        actor: &A,
        state: &mut S,
        msg: AskMessage<I, O>,
    ) -> Result<(), AskHandleError<E>> {
        let result = actor.handle(state, msg.request).await;

        match result {
            Ok(data) => msg
                .tx
                .send(Ok(data))
                .map_err(|send_error| AskHandleError::SendOk(Box::new(send_error))),

            Err(err) => Err(match msg.tx.send(Err(ReceiverHandleError)) {
                Ok(_) => AskHandleError::Handle(err),
                Err(send_error) => {
                    logging::error(format!("Ask result send error: {send_error}"));

                    AskHandleError::SendError(err)
                }
            }),
        }
    }
}

#[async_trait]
impl<A, S, I, E> BaseHandlerTrait<A, S, TellMessage<I>, Result<(), E>> for BaseHandler
where
    A: TellHandlerTrait<S, I, E> + Sync + Send + 'static,
    S: Send,
    I: Send + 'static,
    E: Display + Debug,
{
    async fn _handle(actor: &A, state: &mut S, msg: TellMessage<I>) -> Result<(), E> {
        actor.handle(state, msg.0).await
    }
}

#[async_trait]
pub trait AskHandlerTrait<S, I, O, E>
where
    S: Send,
    I: Send + 'static,
    O: Send + 'static,
    E: Display + Debug,
{
    async fn handle(&self, state: &mut S, msg: I) -> Result<O, E>;
}

#[async_trait]
pub trait TellHandlerTrait<S, I, E>
where
    S: Send,
    I: Send + 'static,
{
    async fn handle(&self, state: &mut S, msg: I) -> Result<(), E>;
}

#[doc(hidden)]
#[async_trait]
pub trait ActorMessageHandlerTrait<S, M, E>
where
    E: Display + Debug,
{
    async fn __handle(&self, state: &mut S, msg: M) -> Result<(), HandleError<E>>;
}
