use ascolt::macros::{ask_handler, tell_handler};
use ascolt::{
    error::{
        actor::{ActorInitFailure, ActorStopFailure},
        handler::DefaultHandlerError,
    },
    handler::{AskHandlerTrait, TellHandlerTrait},
    match_messages,
    messaging::{MessageSender, bounded_channel},
    supervision::{ActorTrait, CommandMessage, start_actor},
};
use async_trait::async_trait;
use derive_more::From;
use thiserror::Error;

pub struct CalcActor {}

pub struct CalcActorState {
    number: i32,
}

pub struct AddNumberRequest(i32);
pub struct SubNumberRequest(i32);
pub struct GetNumberRequest;
pub struct GetNumberResponse(i32);

match_messages! {
    actor: CalcActor;
    state: CalcActorState;
    error: DefaultHandlerError;

    CalcActorMessage {
        AddNumberRequest;
        SubNumberRequest;
        GetNumberRequest -> GetNumberResponse;
    }
}

#[derive(Error, Debug, From)]
#[error(transparent)]
struct AnyhowContainer(anyhow::Error);

fn some_function_that_returns_anyhow() -> anyhow::Result<()> {
    Ok(())
}

#[async_trait]
impl ActorTrait<CalcActorState, DefaultHandlerError> for CalcActor {
    async fn init(&self) -> Result<CalcActorState, ActorInitFailure> {
        some_function_that_returns_anyhow().map_err(AnyhowContainer::from)?;

        Ok(CalcActorState { number: 0 })
    }

    async fn on_stop(&self, _: CalcActorState) -> Result<(), ActorStopFailure> {
        println!("Calc actor stopped");

        Ok(())
    }
}

#[ask_handler]
async fn handle(
    self: &CalcActor,
    state: &mut CalcActorState,
    msg: GetNumberRequest,
) -> Result<GetNumberResponse, DefaultHandlerError> {
    Ok(GetNumberResponse(state.number))
}

#[tell_handler]
async fn handle(
    self: &CalcActor,
    state: &mut CalcActorState,
    msg: AddNumberRequest,
) -> Result<(), DefaultHandlerError> {
    state.number += msg.0;
    Ok(())
}

#[async_trait]
impl TellHandlerTrait<CalcActorState, SubNumberRequest, DefaultHandlerError> for CalcActor {
    async fn handle(
        &self,
        state: &mut CalcActorState,
        msg: SubNumberRequest,
    ) -> Result<(), DefaultHandlerError> {
        state.number -= msg.0;
        Ok(())
    }
}

pub struct ProxyActor {
    tx: MessageSender<CalcActorMessage>,
}

#[derive(Default)]
pub struct ProxyActorState {}

pub struct ProxyActorCalcRequest(i32);
pub struct ProxyActorCalcResponse(i32);

match_messages! {
    actor: ProxyActor;
    state: ProxyActorState;
    error: DefaultHandlerError;


    ProxyActorMessage {
        ProxyActorCalcRequest -> ProxyActorCalcResponse;
    }
}

#[async_trait]
impl ActorTrait<ProxyActorState, DefaultHandlerError> for ProxyActor {
    async fn init(&self) -> Result<ProxyActorState, ActorInitFailure> {
        Ok(ProxyActorState::default())
    }

    async fn on_stop(&self, _: ProxyActorState) -> Result<(), ActorStopFailure> {
        println!("Proxy actor stopped");

        Ok(())
    }
}

#[async_trait]
impl
    AskHandlerTrait<
        ProxyActorState,
        ProxyActorCalcRequest,
        ProxyActorCalcResponse,
        DefaultHandlerError,
    > for ProxyActor
{
    async fn handle(
        &self,
        _: &mut ProxyActorState,
        msg: ProxyActorCalcRequest,
    ) -> Result<ProxyActorCalcResponse, DefaultHandlerError> {
        self.tx.tell(AddNumberRequest(msg.0)).await?;

        self.tx.tell(AddNumberRequest(5)).await?;
        self.tx.tell(SubNumberRequest(3)).await?;

        let result = self.tx.ask(GetNumberRequest).await?;

        Ok(ProxyActorCalcResponse(result.0))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let calc_actor = CalcActor {};
    let (calc_actor_tx, calc_actor_rx) = bounded_channel::<CalcActorMessage>(100);
    let (calc_actor_message_tx, calc_actor_command_tx) = calc_actor_tx.split();

    let proxy_actor = ProxyActor {
        tx: calc_actor_message_tx,
    };
    let (proxy_actor_tx, proxy_actor_rx) = bounded_channel::<ProxyActorMessage>(100);

    tokio::spawn(start_actor(calc_actor, calc_actor_rx));
    tokio::spawn(start_actor(proxy_actor, proxy_actor_rx));

    let result = proxy_actor_tx.ask(ProxyActorCalcRequest(10)).await?;

    println!("Result: {}", result.0);

    calc_actor_command_tx
        .command(CommandMessage::StopActor)
        .await?;

    Ok(())
}
