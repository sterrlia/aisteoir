use std::fmt::{Debug, Display};

use crate::{
    error::handler::{AskHandlerError, BaseHandlerError, ReceiverHandlerError},
    messaging::{AskMessage, TellMessage},
};
use async_trait::async_trait;

#[doc(hidden)]
pub struct BaseHandler;

#[doc(hidden)]
#[async_trait]
pub trait BaseHandlerTrait<A, S, M, R> {
    async fn _handle(actor: &A, state: &mut S, msg: M) -> R;
}

#[async_trait]
impl<A, S, I, O, E> BaseHandlerTrait<A, S, AskMessage<I, O>, Result<(), AskHandlerError<E>>>
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
    ) -> Result<(), AskHandlerError<E>> {
        let result = actor.handle(state, msg.request).await;

        match result {
            Ok(data) => msg
                .tx
                .send(Ok(data))
                .map_err(|send_error| AskHandlerError::SendOk(Box::new(send_error))),

            Err(err) => Err(match msg.tx.send(Err(ReceiverHandlerError)) {
                Ok(_) => AskHandlerError::Handle(err),
                Err(send_error) => AskHandlerError::SendError(err, Box::new(send_error)),
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
    async fn __handle(&self, state: &mut S, msg: M) -> Result<(), BaseHandlerError<E>>;
}
