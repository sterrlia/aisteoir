use tokio::sync::{mpsc, oneshot};

use crate::{
    error::{CallError, ReceiverClosedError, ReceiverHandleError},
    supervision::{ActorMessage, CommandMessage},
};

#[derive(Debug)]
pub struct CallMessage<I, O>(pub I, pub oneshot::Sender<Result<O, ReceiverHandleError>>);
#[derive(Debug)]
pub struct TellMessage<I>(pub I);

#[derive(Clone)]
pub struct Sender<M> {
    pub tx: mpsc::Sender<ActorMessage<M>>,
}

impl<M> Sender<M>
where
    M: Send,
{
    async fn send(&self, msg: ActorMessage<M>) -> Result<(), ReceiverClosedError> {
        self.tx
            .send(msg)
            .await
            .map_err(|err| ReceiverClosedError(err.to_string()))
    }

    pub async fn command(&self, command: CommandMessage) -> Result<(), ReceiverClosedError> {
        let msg = ActorMessage::CommandMessage(command);

        self.send(msg).await
    }

    pub async fn tell<F, I>(&self, case: F, value: I) -> Result<(), ReceiverClosedError>
    where
        I: Send,
        F: Send,
        F: Fn(TellMessage<I>) -> M,
    {
        let msg = ActorMessage::ActorMessage(case(TellMessage(value)));

        self.send(msg).await
    }

    pub async fn call<F, I, O>(&self, case: F, value: I) -> Result<O, CallError>
    where
        I: Send,
        O: Send,
        F: Send,
        F: Fn(CallMessage<I, O>) -> M,
    {
        let (result_tx, result_rx) = oneshot::channel();
        let msg = ActorMessage::ActorMessage(case(CallMessage(value, result_tx)));

        self.send(msg).await.map_err(CallError::ReceiverClosed)?;

        result_rx
            .await
            .map_err(|err| CallError::ReceiverClosed(ReceiverClosedError(err.to_string())))?
            .map_err(CallError::ReceiverHandleError)
    }
}
