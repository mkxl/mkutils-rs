#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "unstable", feature(associated_type_defaults, try_trait_v2))]

mod active_vec;
mod fmt;
mod indexed;
mod interval_set;
mod is;
mod macros;
mod read_value;
mod saturating_add_signed;
mod seq_visitor;
mod timestamped;
mod utils;

#[cfg(all(feature = "serde", feature = "tracing"))]
mod as_valuable;

#[cfg(feature = "async")]
mod event;

#[cfg(feature = "tui")]
mod geometry;

#[cfg(feature = "async")]
mod into_stream;

#[cfg(feature = "tui")]
mod key_map;

#[cfg(feature = "unstable")]
mod output;

#[cfg(feature = "async")]
mod process;

#[cfg(feature = "tui")]
mod rgb;

// TODO-rope-cc89cb
// #[cfg(feature = "tui")]
// mod rope;

#[cfg(feature = "async")]
mod run_for;

#[cfg(feature = "tui")]
mod screen;

#[cfg(feature = "tui")]
mod scroll_bar;

#[cfg(feature = "tui")]
mod scroll_view;

#[cfg(feature = "tui")]
mod scroll_view_state;

#[cfg(all(feature = "async", feature = "unstable", feature = "serde"))]
mod socket;

#[cfg(feature = "tracing")]
mod status;

#[cfg(feature = "tui")]
mod terminal;

#[cfg(feature = "tracing")]
mod timer;

#[cfg(feature = "tracing")]
mod tracing;

#[cfg(feature = "tui")]
mod transpose;

#[cfg(feature = "unstable")]
pub use crate::output::Output;
#[cfg(all(feature = "async", feature = "unstable", feature = "serde"))]
pub use crate::socket::{Request, Socket};
pub use crate::{
    active_vec::ActiveVec,
    fmt::{Debugged, OptionDisplay, ResultDisplay},
    indexed::Indexed,
    interval_set::{Interval, IntervalSet},
    read_value::ReadValue,
    saturating_add_signed::SaturatingAddSigned,
    timestamped::Timestamped,
    utils::Utils,
};
#[cfg(feature = "async")]
pub use crate::{
    event::Event,
    process::{Process, ProcessBuilder},
};
#[cfg(feature = "tui")]
pub use crate::{
    geometry::{Orientation, Point, PointU16, PointUsize},
    key_map::{
        key_binding::KeyBinding,
        key_map::{KeyBindingTrie, KeyMap},
        key_map_session::KeyMapSession,
        key_map_state::{KeyMapIncSearch, KeyMapState},
    },
    rgb::Rgb,
    // TODO-rope-cc89cb
    // rope::{
    //     atoms::{Atom, Atoms},
    //     builder::RopeBuilder,
    //     chunk::Chunk,
    //     extended_grapheme_iter::ExtendedGraphemeIter,
    //     length_summary::LengthSummary,
    //     line::Line,
    //     lines::Lines,
    //     rope::Rope,
    // },
    screen::{Screen, ScreenConfig, ScreenTerminal, Stdout},
    scroll_view::ScrollView,
    scroll_view_state::{ScrollCountType, ScrollViewState, ScrollWhen},
    terminal::Terminal,
    transpose::Transpose,
};
#[cfg(feature = "tracing")]
pub use crate::{timer::Timer, tracing::Tracing};
pub use mkutils_macros::{
    ConstAssoc, Constructor, Default, FromChain, SaturatingAdd, SaturatingSub, SetVariant, Toggle, TypeAssoc, context,
    tokio_main,
};
