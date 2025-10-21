#[macro_export]
macro_rules! handler {
    (
        $version:ident,
        $protocol_version:path,
        {
            $(
                $category:ident ($cat_discriminant:expr) => {
                    $(
                        $command:ident ($cmd_discriminant:expr) => $handler:path
                    ),* $(,)?
                }
            ),* $(,)?
        }
    ) => {
        paste::paste! {
            pub struct $version;

            #[repr(u8)]
            #[derive(Debug, Clone, byteable_derive::Byteable)]
            pub enum [<AuroraProtocolCommandCategory $version>] {
                $(
                    $category = $cat_discriminant,
                )*
            }

            impl CommandCategoryEnum for [<AuroraProtocolCommandCategory $version>] {}

            $(
                #[repr(u8)]
                #[derive(Debug, Clone, byteable_derive::Byteable)]
                pub enum [<$category Command $version>] {
                    $(
                        $command = $cmd_discriminant,
                    )*
                }

                impl CommandEnum for [<$category Command $version>] {}

                $(
                    impl AuroraProtocolCommandMetadata for $handler {
                        type CommandCategory = [<AuroraProtocolCommandCategory $version>];
                        type CommandType = [<$category Command $version>];

                        const COMMAND_CATEGORY: [<AuroraProtocolCommandCategory $version>] =
                            [<AuroraProtocolCommandCategory $version>]::$category;
                        const COMMAND: [<$category Command $version>] =
                            [<$category Command $version>]::$command;
                        const VERSION: AuroraProtocolVersion = $protocol_version;
                    }
                )*
            )*

            impl $version {
                pub async fn handle<S: AsyncRead + AsyncWrite + Unpin + Send>(stream: &mut S, state: &ServerState) {
                    let command = [<AuroraProtocolCommandCategory $version>]::decode(stream)
                        .await
                        .unwrap();

                    match command {
                        $(
                            [<AuroraProtocolCommandCategory $version>]::$category => {
                                let command = [<$category Command $version>]::decode(stream)
                                    .await
                                    .unwrap();

                                match command {
                                    $(
                                        [<$category Command $version>]::$command => {
                                            <$handler as AuroraProtocolCommandHandler>::handle(stream, state).await;
                                        }
                                    )*
                                }
                            }
                        ),*
                    }
                }
            }
        }
    };
}
