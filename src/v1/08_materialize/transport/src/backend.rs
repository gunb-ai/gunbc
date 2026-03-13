//! Transport backend dispatch.
//!
//! Allows tests to install a custom backend (e.g., virtual filesystem) while
//! keeping the default executor for normal runs.

use crate::executor::{execute_transport, TransportError};
use gunbc_ir::transport::{TransportRequest, TransportResponse};
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Transport backend interface.
pub trait TransportBackend: Send + Sync {
    fn execute(&self, request: &TransportRequest) -> Result<TransportResponse, TransportError>;
}

#[derive(Debug)]
struct DefaultBackend;

impl TransportBackend for DefaultBackend {
    fn execute(&self, request: &TransportRequest) -> Result<TransportResponse, TransportError> {
        execute_transport(request)
    }
}

static BACKEND: OnceLock<RwLock<Arc<dyn TransportBackend>>> = OnceLock::new();
static BACKEND_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn backend_cell() -> &'static RwLock<Arc<dyn TransportBackend>> {
    BACKEND.get_or_init(|| RwLock::new(Arc::new(DefaultBackend)))
}

fn backend_lock() -> &'static Mutex<()> {
    BACKEND_LOCK.get_or_init(|| Mutex::new(()))
}

/// Execute a transport request using the currently installed backend.
pub fn execute_transport_with_backend(
    request: &TransportRequest,
) -> Result<TransportResponse, TransportError> {
    let backend = {
        let guard = backend_cell()
            .read()
            .expect("transport backend lock poisoned");
        guard.clone()
    };
    backend.execute(request)
}

/// Guard that installs a backend for the duration of a scope.
pub struct TransportBackendGuard {
    _lock: std::sync::MutexGuard<'static, ()>,
    previous: Arc<dyn TransportBackend>,
}

impl TransportBackendGuard {
    /// Install a backend until the guard is dropped.
    ///
    /// This acquires a global mutex to avoid concurrent backend swaps.
    pub fn install(backend: Arc<dyn TransportBackend>) -> Self {
        let lock = backend_lock()
            .lock()
            .expect("transport backend global lock poisoned");
        let mut guard = backend_cell()
            .write()
            .expect("transport backend lock poisoned");
        let previous = guard.clone();
        *guard = backend;
        drop(guard);
        Self {
            _lock: lock,
            previous,
        }
    }
}

impl Drop for TransportBackendGuard {
    fn drop(&mut self) {
        let mut guard = backend_cell()
            .write()
            .expect("transport backend lock poisoned");
        *guard = self.previous.clone();
    }
}
