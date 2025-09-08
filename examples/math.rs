use ascolt::{
    error::{ActorInitError, ActorStopError, DefaultHandleError},
    handler::{AskHandlerTrait, TellHandlerTrait},
    match_messages,
    messaging::Sender,
    messaging::bounded_channel,
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
    error: DefaultHandleError;

    Message {
        AddNumberRequest;
        SubNumberRequest;
        GetNumberRequest -> GetNumberResponse;
    }
}

#[async_trait]
impl ActorTrait<CalcState> for CalcActor {
    async fn init(&self) -> Result<CalcState, ActorInitError> {
        Ok(CalcState { number: 0 })
    }

    async fn on_stop(&self, _: CalcState) -> Result<(), ActorStopError> {
        Ok(())
    }
}

#[async_trait]
impl AskHandlerTrait<CalcState, GetNumberRequest, GetNumberResponse, DefaultHandleError>
    for CalcActor
{
    async fn handle(
        &self,
        state: &mut CalcState,
        _: GetNumberRequest,
    ) -> Result<GetNumberResponse, DefaultHandleError> {
        Ok(GetNumberResponse(state.number))
    }
}

#[async_trait]
impl TellHandlerTrait<CalcState, AddNumberRequest, DefaultHandleError> for CalcActor {
    async fn handle(
        &self,
        state: &mut CalcState,
        msg: AddNumberRequest,
    ) -> Result<(), DefaultHandleError> {
        state.number += msg.0;
        Ok(())
    }
}

#[async_trait]
impl TellHandlerTrait<CalcState, SubNumberRequest, DefaultHandleError> for CalcActor {
    async fn handle(
        &self,
        state: &mut CalcState,
        msg: SubNumberRequest,
    ) -> Result<(), DefaultHandleError> {
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
    error: DefaultHandleError;


    ProxyActorMessage {
        ProxyActorCalcRequest -> ProxyActorCalcResponse;
    }
}

#[async_trait]
impl ActorTrait<ProxyActorState> for ProxyActor {
    async fn init(&self) -> Result<ProxyActorState, ActorInitError> {
        Ok(ProxyActorState {})
    }

    async fn on_stop(&self, _: ProxyActorState) -> Result<(), ActorStopError> {
        Ok(())
    }
}

#[async_trait]
impl
    AskHandlerTrait<
        ProxyActorState,
        ProxyActorCalcRequest,
        ProxyActorCalcResponse,
        DefaultHandleError,
    > for ProxyActor
{
    async fn handle(
        &self,
        _: &mut ProxyActorState,
        msg: ProxyActorCalcRequest,
    ) -> Result<ProxyActorCalcResponse, DefaultHandleError> {
        self.tx.tell(AddNumberRequest(msg.0)).await?;

        self.tx.tell(AddNumberRequest(5)).await?;
        self.tx.tell(SubNumberRequest(3)).await?;

        let result = self.tx.ask(GetNumberRequest).await?;

        Ok(ProxyActorCalcResponse(result.0))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
