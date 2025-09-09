#[macro_export]
macro_rules! match_messages {
    (
        actor: $actor:ident;
        state: $state:ident;
        error: $error:ident;

        $(#[$meta:meta])*
        $msg_enum:ident {
            $(
                $req:ident $(-> $resp:ty)?;
            )*
        }
    ) => {
        $(#[$meta])*
        pub enum $msg_enum {
            $(
                $req(
                    match_messages!(@wrap $req $(, $resp)?)
                )
            ),*
        }

        #[async_trait::async_trait]
        impl $crate::handler::ActorMessageHandlerTrait<$state, $msg_enum, $error> for $actor {
            async fn __handle(&self, state: &mut $state, msg: $msg_enum) -> Result<(), $crate::error::handler::BaseHandlerError<$error>> {
                use $crate::handler::BaseHandlerTrait;

                match msg {
                    $(
                        $msg_enum::$req(inner) => {
                            $crate::handler::BaseHandler::_handle(self, state, inner).await.map_err(|err| err.into())
                        }
                    ),*
                }
            }
        }

        $(
            impl $crate::messaging::MessageRequest<$msg_enum> for match_messages!(@wrap $req $(, $resp)?) {
                fn get_case() -> fn(Self) -> $msg_enum {
                    $msg_enum::$req
                }
            }
        )*
    };

    // Helper to wrap messages: if -> Resp is provided, use AskMessage<Req, Resp>
    (@wrap $req:ty, $resp:ty) => {
        $crate::messaging::AskMessage<$req, $resp>
    };
    (@wrap $req:ty) => {
        $crate::messaging::TellMessage<$req>
    };
}
