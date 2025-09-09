use ascolt::{
    error::{actor::ActorInitFailure, handler::DefaultHandlerError},
    handler::{AskHandlerTrait, TellHandlerTrait},
    match_messages,
    messaging::{Sender, bounded_channel},
    supervision::{ActorTrait, start_actor},
};
use async_trait::async_trait;

pub struct CalcActor {}

pub struct CalcState {
    number: i32,
}

pub struct AddNumberRequest(i32);
pub struct SubNumberRequest(i32);
pub struct GetNumberRequest;
pub struct GetNumberResponse(i32);

match_messages! {
    actor: CalcActor;
    state: CalcState;
    error: DefaultHandlerError;

    Message {
        AddNumberRequest;
        SubNumberRequest;
        GetNumberRequest -> GetNumberResponse;
    }
}

#[async_trait]
impl ActorTrait<CalcState, DefaultHandlerError> for CalcActor {
    async fn init(&self) -> Result<CalcState, ActorInitFailure> {
        Ok(CalcState { number: 0 })
    }
}

#[async_trait]
impl AskHandlerTrait<CalcState, GetNumberRequest, GetNumberResponse, DefaultHandlerError>
    for CalcActor
{
    async fn handle(
        &self,
        state: &mut CalcState,
        _: GetNumberRequest,
    ) -> Result<GetNumberResponse, DefaultHandlerError> {
        Ok(GetNumberResponse(state.number))
    }
}

#[async_trait]
impl TellHandlerTrait<CalcState, AddNumberRequest, DefaultHandlerError> for CalcActor {
    async fn handle(
        &self,
        state: &mut CalcState,
        msg: AddNumberRequest,
    ) -> Result<(), DefaultHandlerError> {
        state.number += msg.0;
        Ok(())
    }
}

#[async_trait]
impl TellHandlerTrait<CalcState, SubNumberRequest, DefaultHandlerError> for CalcActor {
    async fn handle(
        &self,
        state: &mut CalcState,
        msg: SubNumberRequest,
    ) -> Result<(), DefaultHandlerError> {
        state.number -= msg.0;
        Ok(())
    }
}

pub struct ProxyActor {
    tx: Sender<Message>,
}

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
        Ok(ProxyActorState {})
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

    let actor = CalcActor {};
    let (tx, rx) = bounded_channel(100);

    let another_actor = ProxyActor { tx };
    let (another_tx, another_rx) = bounded_channel(100);

    tokio::spawn(start_actor(actor, rx));
    tokio::spawn(start_actor(another_actor, another_rx));

    let result = another_tx.ask(ProxyActorCalcRequest(10)).await?;

    println!("Result: {}", result.0);

    Ok(())
}
