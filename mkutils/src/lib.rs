#![allow(clippy::missing_errors_doc)]

mod as_rope_slice;
mod debugged;
mod geometry;
mod into_stream;
mod is;
mod process;
mod rope_builder;
mod status;
mod tracing;
mod utils;

pub use crate::{
    debugged::Debugged,
    geometry::{Orientation, Point, PointU16, PointUsize},
    into_stream::IntoStream,
    is::Is,
    process::Process,
    rope_builder::RopeBuilder,
    tracing::Tracing,
    utils::Utils,
};
pub use mkutils_macros::context;
