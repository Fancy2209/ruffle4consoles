use async_channel::{Receiver, Sender};
use ruffle_core::backend::navigator::{
    ErrorResponse, NavigationMethod, NavigatorBackend, NullExecutor, NullSpawner, OwnedFuture,
    Request, SuccessResponse, fetch_path, resolve_url_with_relative_base_path,
};
use ruffle_core::indexmap::IndexMap;
use ruffle_core::loader::Error;
use ruffle_core::socket::{ConnectionState, SocketAction, SocketHandle};
use sdl2::url::open_url;
/// A null implementation for platforms that do not live in a web browser.
///
/// The NullNavigatorBackend includes a trivial executor that holds owned
/// futures and runs them to completion, blockingly.
use std::path::{Path, PathBuf};
use std::time::Duration;
use url::{ParseError, Url};

pub struct SdlNavigatorBackend {
    spawner: NullSpawner,

    /// The base path for all relative fetches.
    relative_base_path: PathBuf,
}

impl SdlNavigatorBackend {
    pub fn new() -> Self {
        let executor = NullExecutor::new();
        Self {
            spawner: executor.spawner(),
            relative_base_path: PathBuf::new(),
        }
    }

    pub fn with_base_path(path: &Path, executor: &NullExecutor) -> Result<Self, std::io::Error> {
        Ok(Self {
            spawner: executor.spawner(),
            relative_base_path: path.canonicalize()?,
        })
    }
}

impl Default for SdlNavigatorBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl NavigatorBackend for SdlNavigatorBackend {
    fn navigate_to_url(
        &self,
        url: &str,
        _target: &str,
        _vars_method: Option<(NavigationMethod, IndexMap<String, String>)>,
    ) {
        let _ = open_url(url);
    }

    fn fetch(&self, request: Request) -> OwnedFuture<Box<dyn SuccessResponse>, ErrorResponse> {
        fetch_path(self, "SdlNavigatorBackend", request.url(), None)
    }

    fn resolve_url(&self, url: &str) -> Result<Url, ParseError> {
        resolve_url_with_relative_base_path(self, self.relative_base_path.clone(), url)
    }

    fn spawn_future(&mut self, future: OwnedFuture<(), Error>) {
        self.spawner.spawn_local(future);
    }

    fn pre_process_url(&self, url: Url) -> Url {
        url
    }

    fn connect_socket(
        &mut self,
        _host: String,
        _port: u16,
        _timeout: Duration,
        handle: SocketHandle,
        _receiver: Receiver<Vec<u8>>,
        sender: Sender<SocketAction>,
    ) {
        sender
            .try_send(SocketAction::Connect(handle, ConnectionState::Failed))
            .expect("working channel send");
    }
}
