mod config;
mod node;

// std
use multiaddr::Multiaddr;
use std::marker::PhantomData;
use std::sync::Mutex;
// crates
// internal
use crate::general::Result;

pub use config::WakuNodeConfig;

/// Shared flag to check if a waku node is already running in the current process
static WAKU_NODE_INITIALIZED: Mutex<bool> = Mutex::new(false);

/// Marker trait to disallow undesired waku node states in the handle
pub trait WakuNodeState {}

/// Waku node initialized state
pub struct Initialized;

/// Waku node running state
pub struct Running;

impl WakuNodeState for Initialized {}
impl WakuNodeState for Running {}

pub struct WakuNodeHandle<State: WakuNodeState>(PhantomData<State>);

impl<State: WakuNodeState> WakuNodeHandle<State> {
    pub fn peer_id(&self) -> Result<String> {
        node::waku_peer_id()
    }

    pub fn listen_addresses(&self) -> Result<Vec<Multiaddr>> {
        node::waku_listen_addressses()
    }
}
fn stop_node() -> Result<()> {
    let mut node_initialized = WAKU_NODE_INITIALIZED
        .lock()
        .expect("Access to the mutex at some point");
    *node_initialized = false;
    node::waku_stop().map(|_| ())
}

impl WakuNodeHandle<Initialized> {
    pub fn start(self) -> Result<WakuNodeHandle<Running>> {
        node::waku_start().map(|_| WakuNodeHandle(Default::default()))
    }

    pub fn stop(self) -> Result<()> {
        stop_node()
    }
}

impl WakuNodeHandle<Running> {
    pub fn stop(self) -> Result<()> {
        stop_node()
    }
}

pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<WakuNodeHandle<Initialized>> {
    let mut node_initialized = WAKU_NODE_INITIALIZED
        .lock()
        .expect("Access to the mutex at some point");
    if *node_initialized {
        return Err("Waku node is already initialized".into());
    }
    *node_initialized = true;
    node::waku_new(config).map(|_| WakuNodeHandle(Default::default()))
}

#[cfg(test)]
mod tests {
    use super::waku_new;

    #[test]
    fn exclusive_running() {
        let handle1 = waku_new(None).unwrap();
        let handle2 = waku_new(None);
        assert!(handle2.is_err());
        let stop_handle = handle1.start().unwrap();
        stop_handle.stop().unwrap();
    }
}
