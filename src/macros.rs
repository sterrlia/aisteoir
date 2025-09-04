#[macro_export]
macro_rules! match_messages {
    (
        actor: $actor:ident;
        state: $state:ident;
        error: $error:ident;

        $(#[$meta:meta])*
        $msg_enum:ident {
            $(
                $variant:ident($req:ty) $(-> $resp:ty)?;
            )*
        }
    ) => {
        $(#[$meta])*
        pub enum $msg_enum {
            $(
                $variant(
                    match_messages!(@wrap $req $(, $resp)?)
                )
            ),*
        }

        #[async_trait::async_trait]
        impl $crate::handler::ActorMessageHandlerTrait<$state, $msg_enum, $error> for $actor {
            async fn __handle(&self, state: &mut $state, msg: $msg_enum) -> Result<(), $crate::error::HandleError<$error>> {
                use $crate::handler::BaseHandlerTrait;

                match msg {
                    $(
                        $msg_enum::$variant(inner) => {
                            $crate::handler::BaseHandler::_handle(self, state, inner).await.map_err(|err| err.into())
                        }
                    ),*
                }
            }
        }
    };

    // Helper to wrap messages: if -> Resp is provided, use CallMessage<Req, Resp>
    (@wrap $req:ty, $resp:ty) => {
        $crate::channel::CallMessage<$req, $resp>
    };
    (@wrap $req:ty) => {
        $crate::channel::TellMessage<$req>
    };
}

