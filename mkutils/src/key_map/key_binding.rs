use crate::utils::Utils;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_more::Into;
use serde::{Deserialize, Deserializer};

#[derive(Clone, Deserialize, Into)]
pub struct KeyBinding<T> {
    #[serde(deserialize_with = "KeyBinding::<T>::deserialize_keys")]
    pub keys: Vec<KeyEvent>,

    #[serde(flatten)]
    pub command: T,
}

impl<T> KeyBinding<T> {
    const PARSE_ERROR_MESSAGE: &str = "unable to parse keystroke";

    fn key_event_from_text(text: &str) -> Result<KeyEvent, &'static str> {
        let (code, modifiers) = Self::key_event_from_text_helper(text).ok_or(Self::PARSE_ERROR_MESSAGE)?;
        let key_event = KeyEvent::new(code, modifiers);

        key_event.ok()
    }

    fn key_event_from_text_helper(text: &str) -> Option<(KeyCode, KeyModifiers)> {
        let mut modifiers = KeyModifiers::NONE;

        for text in text.split('+') {
            match text {
                "ctrl" => modifiers |= KeyModifiers::CONTROL,
                "shift" => modifiers |= KeyModifiers::SHIFT,
                "alt" => modifiers |= KeyModifiers::ALT,
                "super" => modifiers |= KeyModifiers::SUPER,
                text => return Self::key_code_from_text(text)?.pair(modifiers).some(),
            }
        }

        None
    }

    fn key_code_from_text(text: &str) -> Option<KeyCode> {
        match text {
            // NOTE:
            // - use underscores rather than dashes for more visual distinction with plus sign
            // - skip Media(MediaKeyCode) and Modifier(ModifierKeyCode) variants: [https://docs.rs/crossterm/latest/crossterm/event/enum.KeyCode.html]
            "backspace" => KeyCode::Backspace,
            "enter" => KeyCode::Enter,
            "left" => KeyCode::Left,
            "right" => KeyCode::Right,
            "up" => KeyCode::Up,
            "down" => KeyCode::Down,
            "home" => KeyCode::Home,
            "end" => KeyCode::End,
            "page_up" => KeyCode::PageUp,
            "page_down" => KeyCode::PageDown,
            "tab" => KeyCode::Tab,
            "back_tab" => KeyCode::BackTab,
            "delete" => KeyCode::Delete,
            "insert" => KeyCode::Insert,
            "null" => KeyCode::Null,
            "esc" => KeyCode::Esc,
            "caps_lock" => KeyCode::CapsLock,
            "scroll_lock" => KeyCode::ScrollLock,
            "num_lock" => KeyCode::NumLock,
            "print_screen" => KeyCode::PrintScreen,
            "pause" => KeyCode::Pause,
            "menu" => KeyCode::Menu,
            "keypad_begin" => KeyCode::KeypadBegin,
            text => Self::key_code_from_text_helper(text)?,
        }
        .some()
    }

    fn key_code_from_text_helper(text: &str) -> Option<KeyCode> {
        let mut extended_graphemes = text.extended_graphemes();
        let head = extended_graphemes.next()?;
        let tail = extended_graphemes.as_str();

        if tail.is_empty() {
            KeyCode::Char(head.parse().ok()?).some()
        } else if let Ok(number) = tail.parse::<u8>()
            && head == "f"
        {
            KeyCode::F(number).some()
        } else {
            None
        }
    }

    fn deserialize_keys<'a, D: Deserializer<'a>>(deserializer: D) -> Result<Vec<KeyEvent>, D::Error> {
        Vec::deserialize_from_seq(deserializer, Self::key_event_from_text)
    }
}
