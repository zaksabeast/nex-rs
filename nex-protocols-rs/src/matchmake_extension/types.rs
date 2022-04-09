use nex_rs::nex_types::{DateTime, NexBuffer, NexList, NexMap, NexString, NexVariant};
use no_std_io::{EndianRead, EndianWrite};

#[derive(Default, EndianRead, EndianWrite)]
pub struct Gathering {
    id: u32,
    owner_pid: u32,
    host_pid: u32,
    min_participants: u16,
    max_participants: u16,
    participation_policy: u32,
    policy_argument: u32,
    flags: u32,
    state: u32,
    description: NexString,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct MatchmakeParam {
    parameters: NexMap<NexString, NexVariant>,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct MatchmakeSession {
    gathering: Gathering,
    game_mode: u32,
    attributes: NexList<u32>,
    open_participation: bool,
    matchmake_system_type: u32,
    application_buffer: NexBuffer,
    participation_count: u32,
    progress_score: u8,
    session_key: NexBuffer,
    option_0: u32,
    matchmake_param: MatchmakeParam,
    started_time: DateTime,
    user_password: NexString,
    refer_gid: u32,
    user_password_enabled: bool,
    system_password_enabled: bool,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct MatchmakeSessionSearchCriteria {
    attributes: NexList<NexString>,
    game_mode: NexString,
    min_participants: NexString,
    max_participants: NexString,
    matchmake_system_type: NexString,
    vacant_only: bool,
    exclude_locked: bool,
    exclude_non_host_pid: bool,
    selection_method: u32,
    vacant_participants: u32,
    matchmake_param: MatchmakeParam,
    exclude_user_password_set: bool,
    exclude_system_password_set: bool,
    refer_gid: u32,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct CreateMatchmakeSessionParam {
    source_matchmake_session: MatchmakeSession,
    additional_participants: NexList<u32>,
    gid_for_participation_check: u32,
    create_matchmake_session_option: u32,
    join_message: NexString,
    participation_count: u16,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct JoinMatchmakeSessionParam {
    gid: u32,
    additional_participants: NexList<u32>,
    gid_for_participation_check: u32,
    join_matchmake_session_option: u32,
    join_matchmake_session_behaviour: u8,
    user_password: NexString,
    system_password: NexString,
    join_message: NexString,
    participation_count: u16,
    extra_participants: u16,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct AutoMatchmakeParam {
    source_matchmake_session: MatchmakeSession,
    additional_participants: NexList<u32>,
    gid_for_participation_check: u32,
    auto_matchmake_option: u32,
    join_message: NexString,
    participation_count: u16,
    search_criteria: NexList<MatchmakeSessionSearchCriteria>,
    target_gids: NexList<u32>,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct UpdateMatchmakeSessionParam {
    gid: u32,
    modification_flag: u32,
    attributes: NexList<u32>,
    open_participation: bool,
    application_buffer: NexBuffer,
    progress_score: u8,
    matchmake_param: MatchmakeParam,
    started_time: DateTime,
    user_password: NexString,
    game_mode: u32,
    description: NexString,
    min_participants: u16,
    max_participants: u16,
    matchmake_system_type: u32,
    participation_policy: u32,
    policy_argument: u32,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct Community {
    gathering: Gathering,
    community_type: u32,
    password: NexString,
    attributes: NexList<u32>,
    application_buffer: NexBuffer,
    participation_start_date: DateTime,
    participation_end_date: DateTime,
    matchmake_session_count: u32,
    participation_count: u32,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct PersistentGathering {
    gathering: Gathering,
    community_type: u32,
    password: NexString,
    attributes: NexList<u32>,
    application_buffer: NexBuffer,
    participation_start_date: DateTime,
    participation_end_date: DateTime,
    matchmake_session_count: u32,
    participation_count: u32,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct SimplePlayingSession {
    principal_id: u32,
    gathering_id: u32,
    game_mode: u32,
    attribute_0: u32,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct SimpleCommunity {
    gathering_id: u32,
    matchmake_session_count: u32,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct AttractionStatus {
    pub message_interval: u16,
    pub operation_flag: u8,
    pub active_player_invite_param: u16,
    pub active_player_join_param: u16,
    pub extra_params: NexList<u32>,
}

#[derive(Default, EndianRead, EndianWrite)]
pub struct SimpleMatchmakeHostInfo {
    pid: u32,
    session_key: NexBuffer,
    station_urls: NexList<NexString>,
}
