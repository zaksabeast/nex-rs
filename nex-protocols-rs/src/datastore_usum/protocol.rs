use crate::datastore_usum::{
    ChangeMetasRequest, DataStoreGetMetaParam, DataStorePreparePostParam, DataStoreRateObjectParam,
    DataStoreRatingTarget, GlobalTradeStationDeletePokemonParam,
    GlobalTradeStationDownloadMyPokemonParam, GlobalTradeStationDownloadOtherPokemonParam,
    GlobalTradeStationPrepareTradePokemonParam, GlobalTradeStationSearchPokemonParam,
    GlobalTradeStationTradePokemonParam, GlobalTradeStationUploadPokemonParam,
};
use async_trait::async_trait;
use nex_rs::{
    client::ClientConnection,
    nex_types::{NexList, NexStruct, ResultCode},
    result::NexResult,
    rmc::RMCRequest,
    server::Server,
};
use no_std_io::{StreamContainer, StreamReader};
use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::sync::Arc;
use tokio::sync::RwLock;

pub const DATASTORE_PROTOCOL_ID: u8 = 0x73;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[repr(u32)]
pub enum DataStoreMethod {
    GetMetas = 0x9,
    RateObject = 0xF,
    PostMetaBinary = 0x15,
    ChangeMetas = 0x27,
    PrepareUploadPokemon = 0x2F,
    UploadPokemon = 0x30,
    PrepareTradePokemon = 0x32,
    TradePokemon = 0x33,
    DownloadOtherPokemon = 0x34,
    DownloadMyPokemon = 0x35,
    DeletePokemon = 0x36,
    SearchPokemonV2 = 0x37,
}

#[async_trait]
pub trait DataStoreProtocol: Server {
    async fn get_metas(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        data_ids: NexList<u64>,
        param: DataStoreGetMetaParam,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn rate_object(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        target: DataStoreRatingTarget,
        param: DataStoreRateObjectParam,
        fetch_ratings: bool,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn post_meta_binary(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: DataStorePreparePostParam,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn change_metas(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: ChangeMetasRequest,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn prepare_upload_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn upload_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: GlobalTradeStationUploadPokemonParam,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn prepare_trade_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: GlobalTradeStationPrepareTradePokemonParam,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn trade_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: GlobalTradeStationTradePokemonParam,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn download_other_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: GlobalTradeStationDownloadOtherPokemonParam,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn download_my_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: GlobalTradeStationDownloadMyPokemonParam,
    ) -> Result<Vec<u8>, ResultCode>;
    async fn delete_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: GlobalTradeStationDeletePokemonParam,
    ) -> Result<(), ResultCode>;
    async fn search_pokemon_v2(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        param: GlobalTradeStationSearchPokemonParam,
    ) -> Result<Vec<u8>, ResultCode>;

    async fn handle_get_metas(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let data_ids = parameters_stream
            .read_stream_le::<NexList<u64>>()
            .map_err(|_| "Can not read data ids list")?;
        let param = parameters_stream
            .read_stream_le::<DataStoreGetMetaParam>()
            .map_err(|_| "Can not read DataStoreGetMetaParam")?;

        match self.get_metas(Arc::clone(&client), data_ids, param).await {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_rate_object(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let target = parameters_stream
            .read_stream_le::<DataStoreRatingTarget>()
            .map_err(|_| "Can not read DataStoreRatingTarget")?;
        let param = parameters_stream
            .read_stream_le::<DataStoreRateObjectParam>()
            .map_err(|_| "Can not read DataStoreRateObjectParam")?;
        let fetch_ratings = parameters_stream
            .read_stream_le::<bool>()
            .map_err(|_| "Can not read fetch ratings bool")?;

        match self
            .rate_object(Arc::clone(&client), target, param, fetch_ratings)
            .await
        {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_post_meta_binary(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<NexStruct<DataStorePreparePostParam>>()
            .map_err(|_| "Can not read DataStorePreparePostParam")?;

        match self
            .post_meta_binary(Arc::clone(&client), param.into_raw())
            .await
        {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_change_metas(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<ChangeMetasRequest>()
            .map_err(|_| "Can not read ChangeMetasRequest")?;

        match self.change_metas(Arc::clone(&client), param).await {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_prepare_upload_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        match self.prepare_upload_pokemon(Arc::clone(&client)).await {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_upload_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<NexStruct<GlobalTradeStationUploadPokemonParam>>()
            .map_err(|_| "Can not read GlobalTradeStationUploadPokemonParam")?;

        match self
            .upload_pokemon(Arc::clone(&client), param.into_raw())
            .await
        {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_prepare_trade_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<NexStruct<GlobalTradeStationPrepareTradePokemonParam>>()
            .map_err(|_| "Can not read GlobalTradeStationPrepareTradePokemonParam")?;

        match self
            .prepare_trade_pokemon(Arc::clone(&client), param.into_raw())
            .await
        {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_trade_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<NexStruct<GlobalTradeStationTradePokemonParam>>()
            .map_err(|_| "Can not read GlobalTradeStationTradePokemonParam")?;

        match self
            .trade_pokemon(Arc::clone(&client), param.into_raw())
            .await
        {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_download_other_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<NexStruct<GlobalTradeStationDownloadOtherPokemonParam>>()
            .map_err(|_| "Can not read GlobalTradeStationDownloadOtherPokemonParam")?;

        match self
            .download_other_pokemon(Arc::clone(&client), param.into_raw())
            .await
        {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_download_my_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<NexStruct<GlobalTradeStationDownloadMyPokemonParam>>()
            .map_err(|_| "Can not read GlobalTradeStationDownloadMyPokemonParam")?;

        match self
            .download_my_pokemon(Arc::clone(&client), param.into_raw())
            .await
        {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_delete_pokemon(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<NexStruct<GlobalTradeStationDeletePokemonParam>>()
            .map_err(|_| "Can not read GlobalTradeStationDeletePokemonParam")?;

        match self
            .delete_pokemon(Arc::clone(&client), param.into_raw())
            .await
        {
            Ok(_) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    vec![],
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }

    async fn handle_search_pokemon_v2(
        &self,
        client: Arc<RwLock<ClientConnection>>,
        request: &RMCRequest,
    ) -> NexResult<()> {
        let parameters = request.parameters.as_slice();

        let mut parameters_stream = StreamContainer::new(parameters);

        let param = parameters_stream
            .read_stream_le::<NexStruct<GlobalTradeStationSearchPokemonParam>>()
            .map_err(|_| "Can not read GlobalTradeStationSearchPokemonParam")?;

        match self
            .search_pokemon_v2(Arc::clone(&client), param.into_raw())
            .await
        {
            Ok(data) => {
                self.send_success(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    data,
                )
                .await?
            }
            Err(error_code) => {
                self.send_error(
                    client,
                    request.protocol_id,
                    request.method_id,
                    request.call_id,
                    error_code.into(),
                )
                .await?
            }
        }
        Ok(())
    }
}
