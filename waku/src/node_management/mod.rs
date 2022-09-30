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
const WAKU_NODE_RUNNING: Mutex<bool> = Mutex::new(false);

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

impl WakuNodeHandle<Initialized> {
    pub fn start(self) -> Result<WakuNodeHandle<Running>> {
        let flag = WAKU_NODE_RUNNING;
        let mut node_running = flag.lock().expect("Access to the mutex at some point");
        if *node_running {
            return Err("Waku node is already running".into());
        }
        match node::waku_start() {
            Ok(_) => {
                *node_running = true;
                Ok(WakuNodeHandle(Default::default()))
            }
            Err(e) => Err(e),
        }
    }
}

impl WakuNodeHandle<Running> {
    pub fn stop(self) -> Result<()> {
        node::waku_stop().map(|_| ())
    }
}

pub fn waku_new(config: Option<WakuNodeConfig>) -> Result<WakuNodeHandle<Initialized>> {
    node::waku_new(config).map(|_| WakuNodeHandle(Default::default()))
}

#[cfg(test)]
mod tests {
    use super::waku_new;

    #[test]
    fn exclusive_running() {
        let handle1 = waku_new(None).unwrap();
        let handle2 = waku_new(None).unwrap();
        let stop_handle1 = handle1.start().unwrap();
        assert!(handle2.start().is_err());
        stop_handle1.stop().unwrap();
    }
}
