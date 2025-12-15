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
