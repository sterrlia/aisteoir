use std::fmt::Debug;

use crate::{
    error::{CallHandleError, HandleError, ReceiverHandleError},
    messaging::{CallMessage, TellMessage},
};
use async_trait::async_trait;

pub struct BaseHandler {}

#[async_trait]
pub trait BaseHandlerTrait<A, S, M, R> {
    async fn _handle(actor: &A, state: &mut S, msg: M) -> R;
}

#[async_trait]
impl<A, S, I, O, E> BaseHandlerTrait<A, S, CallMessage<I, O>, Result<(), CallHandleError<E>>>
    for BaseHandler
where
    A: CallHandlerTrait<S, I, O, E> + Sync + Send + 'static,
    S: Send,
    I: Send + 'static,
    O: Send + 'static,
    E: Debug,
{
    async fn _handle(
        actor: &A,
        state: &mut S,
        msg: CallMessage<I, O>,
    ) -> Result<(), CallHandleError<E>> {
        let (request, tx) = (msg.0, msg.1);

        let result = actor.handle(state, request).await;

        match result {
            Ok(data) => tx.send(Ok(data)).map_err(|_| CallHandleError::SendOk),

            Err(err) => Err(match tx.send(Err(ReceiverHandleError)) {
                Ok(_) => CallHandleError::Handle(err),
                Err(_) => CallHandleError::SendError(err),
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
    E: Debug,
{
    async fn _handle(actor: &A, state: &mut S, msg: TellMessage<I>) -> Result<(), E> {
        actor.handle(state, msg.0).await
    }
}

#[async_trait]
pub trait CallHandlerTrait<S, I, O, E>
where
    S: Send,
    I: Send + 'static,
    O: Send + 'static,
    E: Debug,
{
    async fn _handle(
        &self,
        state: &mut S,
        msg: CallMessage<I, O>,
    ) -> Result<(), CallHandleError<E>> {
        let (request, tx) = (msg.0, msg.1);

        let result = self.handle(state, request).await;

        match result {
            Ok(data) => tx.send(Ok(data)).map_err(|_| CallHandleError::SendOk),

            Err(err) => Err(match tx.send(Err(ReceiverHandleError)) {
                Ok(_) => CallHandleError::Handle(err),
                Err(_) => CallHandleError::SendError(err),
            }),
        }
    }

    async fn handle(&self, state: &mut S, msg: I) -> Result<O, E>;
}

#[async_trait]
pub trait TellHandlerTrait<S, I, E>
where
    S: Send,
    I: Send + 'static,
{
    async fn _handle(&self, state: &mut S, msg: TellMessage<I>) -> Result<(), E> {
        self.handle(state, msg.0).await
    }

    async fn handle(&self, state: &mut S, msg: I) -> Result<(), E>;
}

#[async_trait]
pub trait ActorMessageHandlerTrait<S, M, E>
where
    E: Debug,
{
    async fn __handle(&self, state: &mut S, msg: M) -> Result<(), HandleError<E>>;
}
