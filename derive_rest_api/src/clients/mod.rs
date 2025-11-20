//! HTTP client implementations for various backends.

#[derive(Clone, Debug, Default)]
pub struct UnimplementedClient;

#[cfg(feature = "reqwest-blocking")]
mod reqwest_blocking;

#[cfg(feature = "reqwest-async")]
mod reqwest_async;

#[cfg(feature = "ureq-blocking")]
mod ureq_blocking;

#[cfg(feature = "reqwest-blocking")]
pub use reqwest_blocking::ReqwestBlockingClient;

#[cfg(feature = "reqwest-async")]
pub use reqwest_async::ReqwestAsyncClient;

#[cfg(feature = "ureq-blocking")]
pub use ureq_blocking::UreqBlockingClient;
