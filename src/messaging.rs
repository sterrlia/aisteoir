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

pub fn create_channel<M>(mailbox_size: usize) -> (Sender<M>, Receiver<M>) {
    let (tx, rx) = mpsc::channel::<ActorMessage<M>>(mailbox_size);

    (Sender { tx }, Receiver { rx })
}

pub struct Receiver<M> {
    pub rx: mpsc::Receiver<ActorMessage<M>>,
}

pub trait MessageRequest<M> {
    fn get_case() -> fn(Self) -> M;
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

    pub async fn tell<I>(&self, value: I) -> Result<(), ReceiverClosedError>
    where
        I: Send,
        TellMessage<I>: MessageRequest<M>,
    {
        let tell_message = TellMessage(value);
        let case = TellMessage::get_case();
        let msg = ActorMessage::ActorMessage(case(tell_message));

        self.send(msg).await
    }

    pub async fn call<I, O>(&self, value: I) -> Result<O, CallError>
    where
        I: Send,
        CallMessage<I, O>: MessageRequest<M>,
        O: Send,
    {
        let (result_tx, result_rx) = oneshot::channel();
        let call_message = CallMessage(value, result_tx);
        let case = CallMessage::get_case();
        let msg = ActorMessage::ActorMessage(case(call_message));

        self.send(msg).await.map_err(CallError::ReceiverClosed)?;

        result_rx
            .await
            .map_err(|err| CallError::ReceiverClosed(ReceiverClosedError(err.to_string())))?
            .map_err(CallError::ReceiverHandleError)
    }
}
