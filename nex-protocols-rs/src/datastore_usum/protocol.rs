use super::types::{
    ChangeMetasInput, ChangeMetasOutput, DeletePokemonInput, DownloadMyPokemonInput,
    DownloadMyPokemonOutput, DownloadOtherPokemonInput, DownloadOtherPokemonOutput, GetMetasInput,
    GetMetasOutput, PostMetaBinaryInput, PostMetaBinaryOutput, PrepareTradePokemonInput,
    PrepareTradePokemonOutput, PrepareUploadPokemonOutput, RateObjectInput, RateObjectOutput,
    SearchPokemonV2Input, SearchPokemonV2Output, TradePokemonInput, TradePokemonOutput,
    UploadPokemonInput,
};
use nex_rs::{macros::NexProtocol, route::NexProtocol};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const DATASTORE_PROTOCOL_ID: u8 = 0x73;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, NexProtocol)]
#[repr(u32)]
pub enum DataStoreMethod {
    #[protocol_method(input = "GetMetasInput", output = "GetMetasOutput")]
    GetMetas = 0x9,
    #[protocol_method(input = "RateObjectInput", output = "RateObjectOutput")]
    RateObject = 0xF,
    #[protocol_method(input = "PostMetaBinaryInput", output = "PostMetaBinaryOutput")]
    PostMetaBinary = 0x15,
    #[protocol_method(input = "ChangeMetasInput", output = "ChangeMetasOutput")]
    ChangeMetas = 0x27,
    #[protocol_method(output = "PrepareUploadPokemonOutput")]
    PrepareUploadPokemon = 0x2F,
    #[protocol_method(input = "UploadPokemonInput")]
    UploadPokemon = 0x30,
    #[protocol_method(
        input = "PrepareTradePokemonInput",
        output = "PrepareTradePokemonOutput"
    )]
    PrepareTradePokemon = 0x32,
    #[protocol_method(input = "TradePokemonInput", output = "TradePokemonOutput")]
    TradePokemon = 0x33,
    #[protocol_method(
        input = "DownloadOtherPokemonInput",
        output = "DownloadOtherPokemonOutput"
    )]
    DownloadOtherPokemon = 0x34,
    #[protocol_method(input = "DownloadMyPokemonInput", output = "DownloadMyPokemonOutput")]
    DownloadMyPokemon = 0x35,
    #[protocol_method(input = "DeletePokemonInput")]
    DeletePokemon = 0x36,
    #[protocol_method(input = "SearchPokemonV2Input", output = "SearchPokemonV2Output")]
    SearchPokemonV2 = 0x37,
}

impl NexProtocol for DataStoreMethod {
    const PROTOCOL_ID: u8 = DATASTORE_PROTOCOL_ID;
}
