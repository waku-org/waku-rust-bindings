use waku::{
    waku_destroy, waku_new, Encoding, Event, WakuContentTopic, WakuMessage, WakuNodeConfig, LibwakuResponse,
};

pub mut waku: WakuNodeHandle<State: WakuNodeState>;
