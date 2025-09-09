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
pub trait ActorTrait<S, E>
where
    S: Send + 'static,
    E: Send + Display + Debug + 'static,
{
    async fn init(&self) -> Result<S, ActorInitFailure>;

    #[allow(unused_variables, unused_mut)]
    async fn on_stop(&self, mut state: S) -> Result<(), ActorStopFailure> {
        Ok(())
    }

    #[allow(unused_variables, unused_mut)]
    async fn on_error(
        &self,
        state: &mut S,
        error: BaseHandlerError<E>,
    ) -> Result<Option<CommandMessage>, ActorHandleErrorFailure> {
        Ok(Some(CommandMessage::StopActor))
    }
}

#[async_trait]
impl<A, S, E> ActorTrait<S, E> for A
where
    A: Sync,
    S: Send + Default + 'static,
    E: Send + Display + Debug + 'static,
{
    async fn init(&self) -> Result<S, ActorInitFailure> {
        Ok(S::default())
    }
}

pub async fn start_actor<A, S, M, E>(actor: A, rx: Receiver<M>)
where
    S: Send + 'static,
    M: Send + 'static,
    A: ActorMessageHandlerTrait<S, M, E> + ActorTrait<S, E> + Send + Sync + 'static,
    E: Send + Debug + Display + 'static,
{
    if let Err(error) = run_actor_loop(actor, rx.rx).await {
        log::error(format!("Actor runtime error: {error}"));
    }

    log::info("Actor task finished - channel closed".to_string());
}

async fn run_actor_loop<A, S, M, E>(
    actor: A,
    rx: async_channel::Receiver<ActorMessage<M>>,
) -> Result<(), ActorRuntimeError>
where
    S: Send + 'static,
    M: Send + 'static,
    A: ActorMessageHandlerTrait<S, M, E> + ActorTrait<S, E> + Send + Sync + 'static,
    E: Send + Debug + Display + 'static,
{
    let mut state = actor.init().await?;

    loop {
        let msg = rx.recv().await?;

        let command_result = handle_message(&actor, &mut state, msg).await?;

        if let Some(command) = command_result {
            match command {
                CommandMessage::StopActor => {
                    rx.close();
                }
                CommandMessage::ForceStopActor => {
                    return Ok(());
                }
                CommandMessage::RestartActor => {
                    actor.on_stop(state).await?;

                    state = actor.init().await?;
                }
            };
        }

        if rx.is_empty() || rx.is_closed() {
            break;
        }
    }

    actor.on_stop(state).await?;

    Ok(())
}

async fn handle_message<A, S, M, E>(
    actor: &A,
    state: &mut S,
    msg: ActorMessage<M>,
) -> Result<Option<CommandMessage>, ActorHandleErrorFailure>
where
    S: Send + 'static,
    M: Send + 'static,
    A: Send + Sync + ActorMessageHandlerTrait<S, M, E> + ActorTrait<S, E> + 'static,
    E: Send + Debug + Display + 'static,
{
    Ok(match msg {
        ActorMessage::CommandMessage(command) => Some(command),

        ActorMessage::ActorMessage(actor_msg) => {
            let result = actor
                .__handle(state, actor_msg)
                .await
                .inspect_err(|err| log::error(format!("{err}")));

            match result {
                Ok(_) => None,
                Err(err) => actor.on_error(state, err).await?,
            }
        }
    })
}
