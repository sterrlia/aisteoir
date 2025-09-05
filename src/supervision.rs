use std::fmt::{Debug, Display};

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::{
    error::{
        ActorInitError, ActorStopError, CallError, DefaultActorError, HandleError,
        ReceiverClosedError,
    },
    handler::ActorMessageHandlerTrait,
    logging,
    messaging::Receiver,
};

pub enum CommandMessage {
    StopActor,
    ForceStopActor,
    RestartActor,
}

pub enum ActorMessage<M> {
    CommandMessage(CommandMessage),
    ActorMessage(M),
}

#[async_trait]
pub trait ActorTrait<S>
where
    S: Send + 'static,
{
    async fn init(&self) -> Result<S, ActorInitError>;
    async fn on_stop(&self, mut state: S) -> Result<(), ActorStopError>;
}

pub fn start_actor<A, S, M, E>(actor: A, rx: Receiver<M>)
where
    S: Send + 'static,
    M: Send + 'static,
    A: ActorMessageHandlerTrait<S, M, E> + ActorTrait<S> + Send + 'static,
    E: Into<CommandMessage> + Debug + Display,
{
    tokio::spawn(async move {
        run_actor_loop(actor, rx.rx).await;

        logging::info("Actor task finished - channel closed".to_string());
    });
}

async fn run_actor_loop<A, S, M, E>(actor: A, mut rx: mpsc::Receiver<ActorMessage<M>>)
where
    S: Send + 'static,
    M: Send + 'static,
    A: ActorMessageHandlerTrait<S, M, E> + ActorTrait<S> + Send + 'static,
    E: Into<CommandMessage> + Debug + Display,
{
    let mut state = match actor.init().await {
        Ok(value) => value,
        Err(error) => {
            logging::error(format!("Actor startup error: {:?}", error));

            return;
        }
    };

    while let Some(msg) = rx.recv().await {
        let command_result = match msg {
            ActorMessage::CommandMessage(command) => Some(command),
            ActorMessage::ActorMessage(actor_msg) => {
                let result = actor.__handle(&mut state, actor_msg).await;

                match result {
                    Ok(_) => None,
                    Err(err) => Some(match err {
                        HandleError::CallHandleError(call_handle_error) => {
                            logging::error(format!("Call handle error: {}", call_handle_error));

                            CommandMessage::StopActor
                        }
                        HandleError::TellHandleError(error) => {
                            logging::error(format!("Tell handle error: {}", error));

                            error.into()
                        }
                    }),
                }
            }
        };

        if let Some(command) = command_result {
            match command {
                CommandMessage::StopActor => rx.close(),
                CommandMessage::ForceStopActor => {
                    return;
                }
                CommandMessage::RestartActor => {
                    if let Err(error) = actor.on_stop(state).await {
                        logging::error(format!("Actor restart stop hook error: {}", error));
                    }

                    state = match actor.init().await {
                        Ok(value) => value,
                        Err(error) => {
                            logging::error(format!("Actor restart init error: {}", error));

                            return;
                        }
                    };
                }
            };
        }
    }

    if let Err(error) = actor.on_stop(state).await {
        logging::error(format!("Actor stop error: {:?}", error));
    }
}

impl From<DefaultActorError> for CommandMessage {
    fn from(_: DefaultActorError) -> Self {
        CommandMessage::StopActor
    }
}

impl From<CallError> for DefaultActorError {
    fn from(value: CallError) -> Self {
        DefaultActorError::Fatal(value.to_string())
    }
}

impl From<ReceiverClosedError> for DefaultActorError {
    fn from(value: ReceiverClosedError) -> Self {
        DefaultActorError::Fatal(value.to_string())
    }
}
