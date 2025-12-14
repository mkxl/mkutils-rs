#![cfg_attr(feature = "output", feature(try_trait_v2))]

mod active_vec;
mod fmt;
mod geometry;
mod is;
mod macros;
mod read_value;
mod utils;

#[cfg(feature = "serde")]
mod as_valuable;

#[cfg(feature = "async")]
mod event;

#[cfg(feature = "async")]
mod into_stream;

#[cfg(feature = "output")]
mod output;

#[cfg(feature = "process")]
mod process;

#[cfg(feature = "ropey")]
mod rope_builder;

#[cfg(feature = "async")]
mod run_for;

#[cfg(feature = "socket")]
mod socket;

#[cfg(feature = "tracing")]
mod status;

#[cfg(feature = "tracing")]
mod tracing;

#[cfg(feature = "async")]
pub use crate::event::Event;
#[cfg(feature = "output")]
pub use crate::output::Output;
#[cfg(feature = "process")]
pub use crate::process::Process;
#[cfg(feature = "ropey")]
pub use crate::rope_builder::RopeBuilder;
#[cfg(feature = "socket")]
pub use crate::socket::{Request, Socket};
#[cfg(feature = "tracing")]
pub use crate::tracing::Tracing;
pub use crate::{
    active_vec::ActiveVec,
    fmt::{Debugged, OptionalDisplay},
    geometry::{Orientation, Point, PointU16, PointUsize},
    read_value::ReadValue,
    utils::Utils,
};
#[cfg(feature = "mkutils-macros")]
pub use mkutils_macros::{FromChain, TypeAssoc, context};
