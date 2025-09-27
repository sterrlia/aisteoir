use async_trait::async_trait;
use std::fmt::{Debug, Display};

use crate::{
    error::{
        actor::{ActorHandleErrorFailure, ActorInitFailure, ActorRuntimeError, ActorStopFailure},
        handler::BaseHandlerError,
    },
    handler::ActorMessageHandlerTrait,
    log,
    messaging::Receiver,
};

pub enum CommandMessage {
    StopActor,
    ForceStopActor,
    RestartActor,
}

#[doc(hidden)]
pub enum ActorMessage<M> {
    CommandMessage(CommandMessage),
    ActorMessage(M),
}

#[async_trait]
pub trait ActorTrait<E>
where
    E: Send + Display + Debug + 'static,
{
    async fn init(&self) -> Result<(), ActorInitFailure> {
        Ok(())
    }

    #[allow(unused_variables, unused_mut)]
    async fn on_stop(&self) -> Result<(), ActorStopFailure> {
        Ok(())
    }

    #[allow(unused_variables, unused_mut)]
    async fn on_error(
        &self,
        error: BaseHandlerError<E>,
    ) -> Result<Option<CommandMessage>, ActorHandleErrorFailure> {
        Ok(Some(CommandMessage::StopActor))
    }
}

pub async fn run<A, M, E>(actor: A, rx: Receiver<M>)
where
    M: Send + 'static,
    A: ActorMessageHandlerTrait<M, E> + ActorTrait<E> + Send + Sync + 'static,
    E: Send + Debug + Display + 'static,
{
    if let Err(error) = run_actor_loop(actor, rx.rx).await {
        log::error(format!("Actor runtime error: {error}"));
    }

    log::info("Actor task finished - channel closed".to_string());
}

async fn run_actor_loop<A, M, E>(
    actor: A,
    rx: async_channel::Receiver<ActorMessage<M>>,
) -> Result<(), ActorRuntimeError>
where
    M: Send + 'static,
    A: ActorMessageHandlerTrait<M, E> + ActorTrait<E> + Send + Sync + 'static,
    E: Send + Debug + Display + 'static,
{
    actor.init().await?;

    loop {
        let msg = rx.recv().await?;

        let command_result = handle_message(&actor, msg).await?;

        if let Some(command) = command_result {
            match command {
                CommandMessage::StopActor => {
                    rx.close();
                }
                CommandMessage::ForceStopActor => {
                    return Ok(());
                }
                CommandMessage::RestartActor => {
                    actor.on_stop().await?;

                    actor.init().await?;
                }
            };
        }

        if rx.is_closed() {
            break;
        }
    }

    actor.on_stop().await?;

    Ok(())
}

async fn handle_message<A, M, E>(
    actor: &A,
    msg: ActorMessage<M>,
) -> Result<Option<CommandMessage>, ActorHandleErrorFailure>
where
    M: Send + 'static,
    A: Send + Sync + ActorMessageHandlerTrait<M, E> + ActorTrait<E> + 'static,
    E: Send + Debug + Display + 'static,
{
    Ok(match msg {
        ActorMessage::CommandMessage(command) => Some(command),

        ActorMessage::ActorMessage(actor_msg) => {
            let result = actor
                .__handle(actor_msg)
                .await
                .inspect_err(|err| log::error(format!("{err}")));

            match result {
                Ok(_) => None,
                Err(err) => actor.on_error(err).await?,
            }
        }
    })
}
