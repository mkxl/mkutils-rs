#![cfg_attr(feature = "nightly", feature(result_option_map_or_default, try_trait_v2))]

mod active_vec;
mod as_valuable;
mod event;
mod fmt;
mod geometry;
mod into_stream;
mod is;
mod macros;
mod output;
mod process;
mod read_value;
mod rope_builder;
mod run_for;
mod socket;
mod status;
mod tracing;
mod utils;
pub use crate::{
    active_vec::ActiveVec,
    as_valuable::AsValuable,
    event::Event,
    fmt::{Debugged, OptionalDisplay},
    geometry::{Orientation, Point, PointU16, PointUsize},
    output::Output,
    process::Process,
    read_value::ReadValue,
    rope_builder::RopeBuilder,
    socket::{Request, Socket},
    tracing::Tracing,
    utils::Utils,
};
pub use mkutils_macros::{TypeAssoc, context};
