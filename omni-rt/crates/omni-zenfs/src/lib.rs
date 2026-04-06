#[cfg(feature = "native")]
mod native;
#[cfg(not(feature = "native"))]
mod wasm;

#[cfg(feature = "native")]
pub use native::*;
#[cfg(not(feature = "native"))]
pub use wasm::*;
