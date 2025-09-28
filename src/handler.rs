use std::fmt::{Debug, Display};

use crate::{
    error::handler::{AskHandlerError, BaseHandlerError, ReceiverHandlerError, TellHandlerError},
    messaging::{AskMessage, TellMessage},
};
use async_trait::async_trait;

#[doc(hidden)]
pub struct BaseHandler;

#[doc(hidden)]
#[async_trait]
pub trait BaseHandlerTrait<A, M, R> {
    async fn _handle(actor: &A, msg: M) -> R;
}

#[async_trait]
impl<A, I, O, E> BaseHandlerTrait<A, AskMessage<I, O>, Result<(), AskHandlerError<E>>>
    for BaseHandler
where
    A: AskHandlerTrait<I, O, E> + Sync + Send + 'static,
    I: Send + 'static,
    O: Send + Sync + 'static,
    E: Display + Debug,
{
    async fn _handle(actor: &A, msg: AskMessage<I, O>) -> Result<(), AskHandlerError<E>> {
        let result = actor.handle(msg.request).await;

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
impl<A, I, E> BaseHandlerTrait<A, TellMessage<I>, Result<(), TellHandlerError<E>>> for BaseHandler
where
    A: TellHandlerTrait<I, E> + Sync + Send + 'static,
    I: Send + 'static,
    E: Display + Debug,
{
    async fn _handle(actor: &A, msg: TellMessage<I>) -> Result<(), TellHandlerError<E>> {
        actor.handle(msg.0).await.map_err(TellHandlerError)
    }
}

#[async_trait]
pub trait AskHandlerTrait<I, O, E>
where
    I: Send + 'static,
    O: Send + 'static,
    E: Display + Debug,
{
    async fn handle(&self, msg: I) -> Result<O, E>;
}

#[async_trait]
pub trait TellHandlerTrait<I, E>
where
    I: Send + 'static,
{
    async fn handle(&self, msg: I) -> Result<(), E>;
}

#[doc(hidden)]
#[async_trait]
pub trait ActorMessageHandlerTrait<M, E>
where
    E: Display + Debug,
{
    async fn __handle(&self, msg: M) -> Result<(), BaseHandlerError<E>>;
}
