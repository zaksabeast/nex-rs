use nex_rs::{
    macros::NexProtocol,
    nex_types::{NexBuffer, NexList, NexMap, NexString, NexStruct, NexVariant, ResultRange},
};
use no_std_io::{EndianRead, EndianWrite};
use num_enum::{IntoPrimitive, TryFromPrimitive};

pub const MATCHMAKE_EXTENSION_PROTOCOL_ID: u8 = 0x6D;

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive, IntoPrimitive, NexProtocol)]
#[repr(u32)]
pub enum MatchmakeExtensionMethod {
    #[protocol_method(input = "CloseParticipationInput")]
    CloseParticipation = 0x1,
    #[protocol_method(input = "OpenParticipationInput")]
    OpenParticipation = 0x2,
    #[protocol_method(
        input = "BrowseMatchmakeSessionInput",
        output = "BrowseMatchmakeSessionOutput"
    )]
    BrowseMatchmakeSession = 0x4,
    #[protocol_method(
        input = "BrowseMatchmakeSessionWithHostUrlsInput",
        output = "BrowseMatchmakeSessionWithHostUrlsOutput"
    )]
    BrowseMatchmakeSessionWithHostUrls = 0x5,
    #[protocol_method(output = "GetAttractionStatusOutput")]
    GetAttractionStatus = 0x31,
    #[protocol_method(input = "SimpleMatchmakeInput", output = "SimpleMatchmakeOutput")]
    SimpleMatchmake = 0x33,
}

#[derive(EndianRead, EndianWrite)]
pub struct CloseParticipationInput {
    pub gid: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct OpenParticipationInput {
    pub gid: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct MatchmakeParam {
    pub parameters: NexMap<NexString, NexVariant>,
}

#[derive(EndianRead, EndianWrite)]
pub struct MatchmakeSessionSearchCriteria {
    pub attributes: NexList<NexString>,
    pub game_mode: NexString,
    pub min_participants: NexString,
    pub max_participants: NexString,
    pub matchmake_system_type: NexString,
    pub vacant_only: bool,
    pub exclude_locked: bool,
    pub exclude_non_host_pid: bool,
    pub selection_method: u32,
    pub vacant_participants: u32,
    pub matchmake_param: MatchmakeParam,
    pub exclude_user_password_set: bool,
    pub exclude_system_password_set: bool,
    pub refer_gid: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct BrowseMatchmakeSessionInput {
    pub search_criteria: MatchmakeSessionSearchCriteria,
    pub result_range: ResultRange,
}

#[derive(EndianRead, EndianWrite)]
pub struct BrowseMatchmakeSessionOutput {
    pub gathering: NexList<u8>,
}

#[derive(EndianRead, EndianWrite)]
pub struct BrowseMatchmakeSessionWithHostUrlsInput {
    pub search_criteria: MatchmakeSessionSearchCriteria,
    pub result_range: ResultRange,
}

#[derive(EndianRead, EndianWrite)]
pub struct BrowseMatchmakeSessionWithHostUrlsOutput {
    pub gathering: NexList<u8>,
    pub gathering_urls: NexList<NexString>,
}

#[derive(EndianRead, EndianWrite)]
pub struct AttractionStatus {
    pub message_interval: u16,
    pub operation_flag: u8,
    pub active_player_invite_param: u16,
    pub active_player_join_param: u16,
    pub extra_params: NexList<u32>,
}

#[derive(EndianRead, EndianWrite)]
pub struct GetAttractionStatusOutput {
    pub attraction_status: NexStruct<AttractionStatus>,
    pub refresh_interval: u16,
}

#[derive(EndianRead, EndianWrite)]
pub struct SimpleMatchmakeInput {
    pub group_id: u32,
}

#[derive(EndianRead, EndianWrite)]
pub struct SimpleMatchmakeOutput {
    pub found: bool,
    pub info: SimpleMatchmakeHostInfo,
}

#[derive(EndianRead, EndianWrite)]
pub struct SimpleMatchmakeHostInfo {
    pub pid: u32,
    pub session_key: NexBuffer,
    pub station_urls: NexList<NexString>,
}
