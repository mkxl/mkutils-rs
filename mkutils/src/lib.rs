#![cfg_attr(feature = "nightly", feature(result_option_map_or_default, try_trait_v2))]

mod event;
mod fmt;
mod geometry;
mod into_stream;
mod is;
mod join;
mod macros;
mod outcome;
mod process;
mod read_value;
mod rope_builder;
mod status;
mod to_value;
mod tracing;
mod utils;
pub use crate::{
    event::{Event, EventReceiver, EventSender},
    fmt::{Debugged, DisplayOptional},
    geometry::{Orientation, Point, PointU16, PointUsize},
    into_stream::IntoStream,
    is::Is,
    outcome::Outcome,
    process::Process,
    read_value::ReadValue,
    rope_builder::RopeBuilder,
    to_value::ToValue,
    tracing::Tracing,
    utils::Utils,
};
pub use mkutils_macros::context;
