use nex_rs::nex_types::{
    DateTime, NexList, NexQBuffer, NexString, NexStruct, ResultCode, ResultRange,
};
use no_std_io::{EndianRead, EndianWrite};

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GetMetasRequest {
    pub data_ids: NexList<u64>,
    pub param: NexStruct<DataStoreGetMetaParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GetMetasResponse {
    pub p_meta_info: NexList<NexStruct<DataStoreMetaInfo>>,
    pub p_results: NexList<NexStruct<ResultCode>>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct RateObjectRequest {
    pub target: NexStruct<DataStoreRatingTarget>,
    pub param: NexStruct<DataStoreRateObjectParam>,
    pub fetch_ratings: bool,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct RateObjectResponse {
    pub p_rating: NexStruct<DataStoreRatingInfo>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreRatingInfoWithSlot {
    pub slot: i8,
    pub rating: NexStruct<DataStoreRatingInfo>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreMetaInfo {
    pub data_id: u64,
    pub owner_id: u32,
    pub size: u32,
    pub name: NexString,
    pub data_type: u16,
    pub meta_binary: NexQBuffer,
    pub permission: NexStruct<DataStorePermission>,
    pub del_permission: NexStruct<DataStorePermission>,
    pub created_time: DateTime,
    pub updated_time: DateTime,
    pub period: u16,
    pub status: u8,
    pub referred_cnt: u32,
    pub refer_data_id: u32,
    pub flag: u32,
    pub referred_time: DateTime,
    pub expire_time: DateTime,
    pub tags: NexList<NexString>,
    pub ratings: NexList<NexStruct<DataStoreRatingInfoWithSlot>>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreRatingInfo {
    pub total_value: i64,
    pub count: u32,
    pub initial_value: i64,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStorePersistenceTarget {
    pub owner_id: u32,
    pub persistence_slot_id: u16,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreGetMetaParam {
    pub data_id: u64,
    pub persistence_target: DataStorePersistenceTarget,
    pub result_option: u8,
    pub access_password: u64,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreRatingTarget {
    pub data_id: u64,
    pub slot: i8,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreRateObjectParam {
    pub rating_value: i32,
    pub access_password: u64,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreRatingInitParam {
    pub flag: u8,
    pub internal_flag: u8,
    pub lock_type: u8,
    pub initial_value: i64,
    pub range_min: i32,
    pub range_max: i32,
    pub period_hour: i8,
    pub period_duration: i16,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStorePermission {
    pub permission: u8,
    pub recipient_ids: NexList<u32>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreRatingInitParamWithSlot {
    pub slot: i8,
    pub param: NexStruct<DataStoreRatingInitParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStorePersistenceInitParam {
    pub persistence_slot_id: u16,
    pub delete_last_object: bool,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStorePreparePostParam {
    pub size: u32,
    pub name: NexString,
    pub data_type: u16,
    pub meta_binary: NexQBuffer,
    pub permission: NexStruct<DataStorePermission>,
    pub del_permission: NexStruct<DataStorePermission>,
    pub flag: u32,
    pub period: u16,
    pub refer_data_id: u32,
    pub tags: NexList<NexString>,
    pub rating_init_params: NexList<NexStruct<DataStoreRatingInitParamWithSlot>>,
    pub persistence_init_param: NexStruct<DataStorePersistenceInitParam>,
    pub extra_data: NexList<NexString>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct PostMetaBinaryRequest {
    pub param: NexStruct<DataStorePreparePostParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct PostMetaBinaryResponse {
    pub data_id: u64,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreChangeMetaCompareParam {
    pub comparison_flag: u32,
    pub name: NexString,
    pub permission: NexStruct<DataStorePermission>,
    pub del_permission: NexStruct<DataStorePermission>,
    pub period: u16,
    pub meta_binary: NexQBuffer,
    pub tags: NexList<NexString>,
    pub referred_cnt: u32,
    pub data_type: u16,
    pub status: u8,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DataStoreChangeMetaParam {
    pub data_id: u64,
    pub modifies_flag: u32,
    pub name: NexString,
    pub permission: NexStruct<DataStorePermission>,
    pub del_permission: NexStruct<DataStorePermission>,
    pub period: u16,
    pub meta_binary: NexQBuffer,
    pub tags: NexList<NexString>,
    pub update_password: u64,
    pub referred_cnt: u32,
    pub data_type: u16,
    pub status: u8,
    pub compare_param: NexStruct<DataStoreChangeMetaCompareParam>,
    pub persistence_target: NexStruct<DataStorePersistenceTarget>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct ChangeMetasRequest {
    pub data_ids: NexList<u64>,
    pub params: NexList<NexStruct<DataStoreChangeMetaParam>>,
    pub transactional: bool,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct ChangeMetasResponse {
    pub p_results: NexList<ResultCode>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationRecordKey {
    pub data_id: u64,
    pub password: u64,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct PrepareUploadPokemonResponse {
    pub p_record_key: NexStruct<GlobalTradeStationRecordKey>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationUploadPokemonParam {
    pub prepare_upload_key: NexStruct<GlobalTradeStationRecordKey>,
    pub period: u16,
    pub index_data: NexQBuffer,
    pub pokemon_data: NexQBuffer,
    pub signature: NexQBuffer,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct UploadPokemonRequest {
    pub param: NexStruct<GlobalTradeStationUploadPokemonParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationTradeKey {
    pub data_id: u64,
    pub version: u32,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationPrepareTradePokemonParam {
    pub trade_key: NexStruct<GlobalTradeStationTradeKey>,
    pub prepare_upload_key: NexStruct<GlobalTradeStationRecordKey>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct PrepareTradePokemonRequest {
    pub param: NexStruct<GlobalTradeStationPrepareTradePokemonParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct PrepareTradePokemonResponse {
    pub p_result: NexStruct<GlobalTradeStationPrepareTradePokemonResult>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationDownloadPokemonResult {
    pub data_id: u64,
    pub index_data: NexQBuffer,
    pub pokemon_data: NexQBuffer,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationPrepareTradePokemonResult {
    pub result: NexStruct<GlobalTradeStationDownloadPokemonResult>,
    pub prepare_trade_key: NexStruct<GlobalTradeStationRecordKey>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationTradePokemonParam {
    pub trade_key: NexStruct<GlobalTradeStationTradeKey>,
    pub prepare_trade_key: NexStruct<GlobalTradeStationRecordKey>,
    pub prepare_upload_key: NexStruct<GlobalTradeStationRecordKey>,
    pub period: u16,
    pub index_data: NexQBuffer,
    pub pokemon_data: NexQBuffer,
    pub signature: NexQBuffer,
    pub need_data: bool,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct TradePokemonRequest {
    pub param: NexStruct<GlobalTradeStationTradePokemonParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct TradePokemonResponse {
    pub p_result: NexStruct<GlobalTradeStationTradePokemonResult>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationTradePokemonResult {
    pub result: NexStruct<GlobalTradeStationDownloadPokemonResult>,
    pub my_data_id: u64,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationDownloadOtherPokemonParam {
    pub prepare_upload_key: NexStruct<GlobalTradeStationRecordKey>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DownloadOtherPokemonRequest {
    pub param: NexStruct<GlobalTradeStationDownloadOtherPokemonParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DownloadOtherPokemonResponse {
    pub p_result: NexStruct<GlobalTradeStationTradePokemonResult>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationDownloadMyPokemonParam {
    pub prepare_upload_key: NexStruct<GlobalTradeStationRecordKey>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationDownloadMyPokemonResult {
    pub result: NexStruct<GlobalTradeStationDownloadPokemonResult>,
    pub is_traded: bool,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DownloadMyPokemonRequest {
    pub param: NexStruct<GlobalTradeStationDownloadMyPokemonParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DownloadMyPokemonResponse {
    pub p_result: NexStruct<GlobalTradeStationDownloadMyPokemonResult>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationDeletePokemonParam {
    pub prepare_upload_key: NexStruct<GlobalTradeStationRecordKey>,
    pub delete_flag: u8,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct DeletePokemonRequest {
    pub param: NexStruct<GlobalTradeStationDeletePokemonParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationSearchPokemonParam {
    pub prepare_upload_key: NexStruct<GlobalTradeStationRecordKey>,
    pub conditions: NexList<u32>,
    pub result_order_column: u8,
    pub result_order: u8,
    pub uploaded_after: DateTime,
    pub uploaded_before: DateTime,
    pub result_range: NexStruct<ResultRange>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct SearchPokemonV2Request {
    pub param: NexStruct<GlobalTradeStationSearchPokemonParam>,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationData {
    pub data_id: u64,
    pub owner_id: u32,
    pub updated_time: DateTime,
    pub index_data: NexQBuffer,
    pub version: u32,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct GlobalTradeStationSearchPokemonResult {
    pub total_count: u32,
    pub result: NexList<NexStruct<GlobalTradeStationData>>,
    pub total_count_type: u8,
}

#[derive(Debug, EndianRead, EndianWrite)]
pub struct SearchPokemonV2Response {
    pub p_result: NexStruct<GlobalTradeStationSearchPokemonResult>,
}
