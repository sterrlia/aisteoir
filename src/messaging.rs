use std::time::{Duration, Instant};

use async_trait::async_trait;

use crate::{
    error::handler::{AskError, ReceiverClosedError, ReceiverHandlerError},
    supervision::{ActorMessage, CommandMessage},
};

#[doc(hidden)]
#[derive(Debug)]
pub struct AskMessage<I, O> {
    pub request: I,
    pub tx: oneshot::Sender<Result<O, ReceiverHandlerError>>,
}

#[doc(hidden)]
#[derive(Debug)]
pub struct TellMessage<I>(pub I);

pub trait MessageRequest<M> {
    fn get_case() -> fn(Self) -> M;
}

pub struct Sender<M> {
    tx: async_channel::Sender<ActorMessage<M>>,
}
pub struct MessageSender<M> {
    tx: async_channel::Sender<ActorMessage<M>>,
}
pub struct CommandSender<M> {
    tx: async_channel::Sender<ActorMessage<M>>,
}

pub fn bounded_channel<M>(mailbox_size: usize) -> (Sender<M>, Receiver<M>) {
    let (tx, rx) = async_channel::bounded::<ActorMessage<M>>(mailbox_size);

    (Sender { tx }, Receiver { rx })
}

pub fn unbounded_channel<M>() -> (Sender<M>, Receiver<M>) {
    let (tx, rx) = async_channel::unbounded::<ActorMessage<M>>();

    (Sender { tx }, Receiver { rx })
}

pub struct Receiver<M> {
    pub rx: async_channel::Receiver<ActorMessage<M>>,
}

impl<M> Sender<M>
where
    M: Send + Sync + 'static,
{
    pub fn into_message_sender(self) -> MessageSender<M> {
        MessageSender { tx: self.tx }
    }

    pub fn split(self) -> (MessageSender<M>, CommandSender<M>) {
        (
            MessageSender {
                tx: self.tx.clone(),
            },
            CommandSender { tx: self.tx },
        )
    }

    pub async fn command(&self, command: CommandMessage) -> Result<(), ReceiverClosedError> {
        send_command(self, command).await
    }

    pub async fn tell<I>(&self, value: I) -> Result<(), ReceiverClosedError>
    where
        I: Send,
        TellMessage<I>: MessageRequest<M>,
    {
        send_tell(self, value, None).await
    }

    pub async fn tell_with_ttl<I>(&self, value: I, ttl: Duration) -> Result<(), ReceiverClosedError>
    where
        I: Send,
        TellMessage<I>: MessageRequest<M>,
    {
        send_tell(self, value, Some(ttl)).await
    }

    pub async fn ask<I, O>(&self, value: I) -> Result<O, AskError>
    where
        I: Send,
        AskMessage<I, O>: MessageRequest<M>,
        O: Send,
    {
        send_ask(self, value, None).await
    }
}

impl<M> MessageSender<M>
where
    M: Send + Sync + 'static,
{
    pub async fn tell<I>(&self, value: I) -> Result<(), ReceiverClosedError>
    where
        I: Send,
        TellMessage<I>: MessageRequest<M>,
    {
        send_tell(self, value, None).await
    }

    pub async fn tell_with_ttl<I>(&self, value: I, ttl: Duration) -> Result<(), ReceiverClosedError>
    where
        I: Send,
        TellMessage<I>: MessageRequest<M>,
    {
        send_tell(self, value, Some(ttl)).await
    }

    pub async fn ask<I, O>(&self, value: I) -> Result<O, AskError>
    where
        I: Send,
        AskMessage<I, O>: MessageRequest<M>,
        O: Send,
    {
        send_ask(self, value, None).await
    }
}

impl<M> CommandSender<M>
where
    M: Send + Sync + 'static,
{
    pub async fn command(&self, command: CommandMessage) -> Result<(), ReceiverClosedError> {
        send_command(self, command).await
    }
}

async fn send_command<SE, M>(tx: &SE, command: CommandMessage) -> Result<(), ReceiverClosedError>
where
    M: Send + Sync + 'static,
    SE: Send + Sync + AbstractSenderTrait<M>,
{
    let msg = ActorMessage::CommandMessage(command);

    tx.send(msg).await?;

    Ok(())
}

async fn send_tell<SE, M, I>(
    tx: &SE,
    value: I,
    ttl: Option<Duration>,
) -> Result<(), ReceiverClosedError>
where
    SE: Send + Sync + AbstractSenderTrait<M>,
    I: Send,
    TellMessage<I>: MessageRequest<M>,
    M: Send + Sync + 'static,
{
    let tell_message = TellMessage(value);
    let case = TellMessage::get_case();
    let sent_at = Instant::now();
    let msg = ActorMessage::RegularMessage {
        msg: case(tell_message),
        sent_at,
        ttl,
    };

    tx.send(msg).await
}

async fn send_ask<SE, M, I, O>(tx: &SE, value: I, ttl: Option<Duration>) -> Result<O, AskError>
where
    SE: Send + Sync + AbstractSenderTrait<M>,
    I: Send,
    AskMessage<I, O>: MessageRequest<M>,
    O: Send,
    M: Send + Sync + 'static,
{
    let (result_tx, result_rx) = oneshot::channel();
    let call_message = AskMessage {
        request: value,
        tx: result_tx,
    };
    let case = AskMessage::get_case();
    let sent_at = Instant::now();
    let msg = ActorMessage::RegularMessage {
        msg: case(call_message),
        sent_at,
        ttl,
    };

    tx.send(msg).await.map_err(AskError::ReceiverClosed)?;

    result_rx
        .await
        .map_err(|err| AskError::ReceiverClosed(ReceiverClosedError::new(Box::new(err))))?
        .map_err(AskError::ReceiverHandlerError)
}

#[async_trait]
trait AbstractSenderTrait<M>
where
    M: Send + Sync + 'static,
{
    fn get_tx(&self) -> &async_channel::Sender<ActorMessage<M>>;

    async fn send(&self, msg: ActorMessage<M>) -> Result<(), ReceiverClosedError> {
        let tx = self.get_tx();

        tx.send(msg)
            .await
            .map_err(|err| ReceiverClosedError::new(Box::new(err)))
    }
}

impl<M> AbstractSenderTrait<M> for CommandSender<M>
where
    M: Send + Sync + 'static,
{
    fn get_tx(&self) -> &async_channel::Sender<ActorMessage<M>> {
        &self.tx
    }
}

impl<M> AbstractSenderTrait<M> for MessageSender<M>
where
    M: Send + Sync + 'static,
{
    fn get_tx(&self) -> &async_channel::Sender<ActorMessage<M>> {
        &self.tx
    }
}

impl<M> AbstractSenderTrait<M> for Sender<M>
where
    M: Send + Sync + 'static,
{
    fn get_tx(&self) -> &async_channel::Sender<ActorMessage<M>> {
        &self.tx
    }
}

impl<M> Clone for Sender<M> {
    fn clone(&self) -> Sender<M> {
        Sender {
            tx: self.tx.clone(),
        }
    }
}

impl<M> Clone for MessageSender<M> {
    fn clone(&self) -> MessageSender<M> {
        MessageSender {
            tx: self.tx.clone(),
        }
    }
}

impl<M> Clone for CommandSender<M> {
    fn clone(&self) -> CommandSender<M> {
        CommandSender {
            tx: self.tx.clone(),
        }
    }
}
