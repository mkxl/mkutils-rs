#![cfg_attr(docsrs, feature(doc_cfg))]
#![cfg_attr(feature = "output", feature(try_trait_v2))]
#![cfg_attr(feature = "socket", feature(associated_type_defaults))]

mod is;
mod macros;
mod utils;

#[cfg(feature = "misc")]
mod active_vec;

#[cfg(feature = "fmt")]
mod fmt;

#[cfg(feature = "serde")]
mod as_valuable;

#[cfg(feature = "tui")]
mod content;

#[cfg(feature = "async")]
mod event;

#[cfg(any(feature = "ropey", feature = "tui"))]
mod geometry;

#[cfg(feature = "misc")]
mod indexed;

#[cfg(feature = "async")]
mod into_stream;

#[cfg(feature = "tui")]
mod key_map;

#[cfg(feature = "output")]
mod output;

#[cfg(feature = "process")]
mod process;

#[cfg(feature = "async")]
mod read_value;

#[cfg(feature = "tui")]
mod rgb;

#[cfg(feature = "rope")]
mod rope;

#[cfg(feature = "ropey")]
mod rope_builder;

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

#[cfg(any(feature = "serde", feature = "tui"))]
mod seq_visitor;

#[cfg(feature = "socket")]
mod socket;

#[cfg(feature = "tracing")]
mod status;

#[cfg(feature = "tui")]
mod terminal;

#[cfg(feature = "misc")]
mod timestamped;

#[cfg(feature = "tracing")]
mod tracing;

#[cfg(any(feature = "ropey", feature = "tui"))]
mod transpose;

#[cfg(feature = "fmt")]
pub use crate::fmt::{Debugged, OptionDisplay, ResultDisplay};
#[cfg(feature = "output")]
pub use crate::output::Output;
#[cfg(feature = "process")]
pub use crate::process::{Process, ProcessBuilder};
#[cfg(feature = "rope")]
pub use crate::rope::{
    atoms::{Atom, Atoms},
    chunk::Chunk,
    distance::{Distance, NumExtendedGraphemes, NumNewlines},
    extended_grapheme_iter::ExtendedGraphemeIter,
    line::Line,
    lines::Lines,
    rope::Rope,
};
#[cfg(feature = "ropey")]
pub use crate::rope_builder::RopeBuilder;
#[cfg(feature = "socket")]
pub use crate::socket::{Request, Socket};
#[cfg(feature = "tracing")]
pub use crate::tracing::Tracing;
pub use crate::utils::Utils;
#[cfg(feature = "misc")]
pub use crate::{active_vec::ActiveVec, indexed::Indexed, timestamped::Timestamped};
#[cfg(feature = "tui")]
pub use crate::{
    content::Content,
    key_map::{
        key_binding::KeyBinding,
        key_map::{KeyBindingTrie, KeyMap},
        key_map_session::KeyMapSession,
        key_map_state::{KeyMapIncSearch, KeyMapState},
    },
    rgb::Rgb,
    screen::{Screen, ScreenConfig, ScreenTerminal, Stdout},
    scroll_view::ScrollView,
    scroll_view_state::{ScrollCountType, ScrollViewState, ScrollWhen},
    terminal::Terminal,
};
#[cfg(feature = "async")]
pub use crate::{event::Event, read_value::ReadValue};
#[cfg(any(feature = "ropey", feature = "tui"))]
pub use crate::{
    geometry::{Orientation, Point, PointU16, PointUsize},
    transpose::Transpose,
};
#[cfg(feature = "mkutils-macros")]
pub use mkutils_macros::{Default, FromChain, SaturatingAdd, SaturatingSub, SetVariant, Toggle, TypeAssoc, context};
