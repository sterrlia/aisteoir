use aisteoir::{
    error::{ActorInitError, ActorStopError, DefaultActorError},
    handler::{CallHandlerTrait, TellHandlerTrait},
    match_messages,
    messaging::Sender,
    messaging::create_channel,
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
    error: DefaultActorError;

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
impl CallHandlerTrait<CalcState, GetNumberRequest, GetNumberResponse, DefaultActorError>
    for CalcActor
{
    async fn handle(
        &self,
        state: &mut CalcState,
        _: GetNumberRequest,
    ) -> Result<GetNumberResponse, DefaultActorError> {
        Ok(GetNumberResponse(state.number))
    }
}

#[async_trait]
impl TellHandlerTrait<CalcState, AddNumberRequest, DefaultActorError> for CalcActor {
    async fn handle(
        &self,
        state: &mut CalcState,
        msg: AddNumberRequest,
    ) -> Result<(), DefaultActorError> {
        state.number += msg.0;
        Ok(())
    }
}

#[async_trait]
impl TellHandlerTrait<CalcState, SubNumberRequest, DefaultActorError> for CalcActor {
    async fn handle(
        &self,
        state: &mut CalcState,
        msg: SubNumberRequest,
    ) -> Result<(), DefaultActorError> {
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
    error: DefaultActorError;


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
    CallHandlerTrait<
        ProxyActorState,
        ProxyActorCalcRequest,
        ProxyActorCalcResponse,
        DefaultActorError,
    > for ProxyActor
{
    async fn handle(
        &self,
        _: &mut ProxyActorState,
        msg: ProxyActorCalcRequest,
    ) -> Result<ProxyActorCalcResponse, DefaultActorError> {
        self.tx.tell(AddNumberRequest(msg.0)).await?;

        self.tx.tell(AddNumberRequest(5)).await?;
        self.tx.tell(SubNumberRequest(3)).await?;

        let result = self.tx.call(GetNumberRequest).await?;

        Ok(ProxyActorCalcResponse(result.0))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let actor = CalcActor {};
    let (tx, rx) = create_channel(100);

    let another_actor = ProxyActor { tx };
    let (another_tx, another_rx) = create_channel(100);

    start_actor(actor, rx);
    start_actor(another_actor, another_rx);

    let result = another_tx.call(ProxyActorCalcRequest(10)).await?;

    println!("Result: {}", result.0);

    Ok(())
}
