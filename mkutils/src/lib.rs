//! # mkutils
//!
//! A versatile Rust utility library providing extension traits, async utilities, and
//! feature-gated functionality for common development tasks.
//!
//! ## Features
//!
//! mkutils is designed with modularity in mind. All major functionality is gated behind
//! optional features that can be enabled as needed:
//!
//! ### Core Features
//!
//! - **`utils`** (always available) - The [`Utils`] trait providing 150+ extension methods
//!   for working with standard library types, futures, I/O, serialization, and more.
//! - **`active-vec`** (always available) - The [`ActiveVec`] type for managing a vector
//!   with a tracked "active" element.
//!
//! ### Optional Features
//!
//! - **`async`** - Async utilities including [`Event`], async I/O helpers, and stream operations.
//!   Requires tokio runtime.
//! - **`fmt`** - Display formatting helpers like [`Debugged`] and [`OptionalDisplay`].
//! - **`fs`** - File system utilities with UTF-8 path support via camino.
//! - **`serde`** - Serialization helpers for JSON, YAML, and MessagePack.
//! - **`rmp`** - MessagePack serialization support.
//! - **`tui`** - Terminal UI utilities using ratatui, including [`Terminal`], [`Screen`],
//!   and geometry types like [`Point`].
//! - **`process`** - The [`Process`] type for managing child processes with tokio.
//! - **`socket`** - Unix socket communication with the [`Socket`] type and [`Request`] trait.
//! - **`tracing`** - Integration with the tracing ecosystem via [`Status`] and [`Tracing`].
//! - **`ropey`** - Text rope data structure support with [`RopeBuilder`].
//! - **`reqwest`** - HTTP client utilities.
//! - **`poem`** - Web framework utilities for the Poem framework.
//! - **`output`** - The [`Output`] type for tri-state results (Ok/EndOk/EndErr).
//!
//! ## Usage
//!
//! Add mkutils to your `Cargo.toml` with the features you need:
//!
//! ```toml
//! [dependencies]
//! mkutils = { version = "0.1", features = ["async", "serde", "tui"] }
//! ```
//!
//! ### Example: Using the Utils trait
//!
//! ```rust
//! use mkutils::Utils;
//!
//! // Chain futures together
//! async {
//!     // Extension methods available on all types
//!     let result = "hello".cat(" world"); // "hello world"
//!
//!     // Convenient conversions
//!     let opt = 42.some(); // Some(42)
//!     let res = "value".ok::<String, ()>(); // Ok("value")
//! }
//! ```
//!
//! ### Example: Async Event
//!
//! ```rust,ignore
//! use mkutils::Event;
//!
//! let mut event = Event::new();
//! event.wait().await; // Waits until event.set() is called
//! ```
//!
//! ## Design Philosophy
//!
//! mkutils embraces the extension trait pattern to provide ergonomic utilities that feel
//! natural in Rust. The [`Utils`] trait is implemented for all types (`impl<T: ?Sized> Utils for T`),
//! making its methods available everywhere while relying on type bounds to ensure they're
//! only callable when appropriate.

#![cfg_attr(feature = "output", feature(try_trait_v2))]
#![cfg_attr(feature = "socket", feature(associated_type_defaults))]

mod active_vec;
mod is;
mod macros;
mod utils;

#[cfg(feature = "fmt")]
mod fmt;

#[cfg(feature = "serde")]
mod as_valuable;

#[cfg(feature = "async")]
mod event;

#[cfg(feature = "tui")]
mod geometry;

#[cfg(feature = "async")]
mod into_stream;

#[cfg(feature = "output")]
mod output;

#[cfg(feature = "process")]
mod process;

#[cfg(feature = "async")]
mod read_value;

#[cfg(feature = "ropey")]
mod rope_builder;

#[cfg(feature = "async")]
mod run_for;

#[cfg(feature = "socket")]
mod socket;

#[cfg(feature = "tui")]
mod terminal;

#[cfg(feature = "tui")]
mod screen;

#[cfg(feature = "tracing")]
mod status;

#[cfg(feature = "tracing")]
mod tracing;

#[cfg(feature = "async")]
pub use crate::event::Event;
#[cfg(feature = "fmt")]
pub use crate::fmt::{Debugged, OptionalDisplay};
#[cfg(feature = "output")]
pub use crate::output::Output;
#[cfg(feature = "process")]
pub use crate::process::Process;
#[cfg(feature = "ropey")]
pub use crate::rope_builder::RopeBuilder;
#[cfg(feature = "tui")]
pub use crate::screen::Screen;
#[cfg(feature = "socket")]
pub use crate::socket::{Request, Socket};
#[cfg(feature = "tui")]
pub use crate::terminal::Terminal;
#[cfg(feature = "tracing")]
pub use crate::tracing::Tracing;
pub use crate::{active_vec::ActiveVec, utils::Utils};
#[cfg(feature = "tui")]
pub use geometry::{Orientation, Point, PointU16, PointUsize};
#[cfg(feature = "mkutils-macros")]
pub use mkutils_macros::{FromChain, TypeAssoc, context};
#[cfg(feature = "async")]
pub use read_value::ReadValue;
