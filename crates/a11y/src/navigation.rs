//! Keyboard navigation and event handling

use serde::{Deserialize, Serialize};

use crate::A11yAnnouncement;

/// Key code representation
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyCode {
    /// Arrow keys
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    /// Navigation keys
    Home,
    End,
    PageUp,
    PageDown,
    /// Action keys
    Enter,
    Space,
    Escape,
    Tab,
    /// Modifier keys
    Shift,
    Control,
    Alt,
    /// Letter keys
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    /// Number keys
    Digit0,
    Digit1,
    Digit2,
    Digit3,
    Digit4,
    Digit5,
    Digit6,
    Digit7,
    Digit8,
    Digit9,
    /// Function keys
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    /// Other
    Backspace,
    Delete,
    Insert,
    Unknown(u32),
}

impl From<u32> for KeyCode {
    fn from(code: u32) -> Self {
        match code {
            37 => Self::ArrowLeft,
            38 => Self::ArrowUp,
            39 => Self::ArrowRight,
            40 => Self::ArrowDown,
            9 => Self::Tab,
            13 => Self::Enter,
            27 => Self::Escape,
            32 => Self::Space,
            36 => Self::Home,
            35 => Self::End,
            33 => Self::PageUp,
            34 => Self::PageDown,
            8 => Self::Backspace,
            46 => Self::Delete,
            _ if (65..=90).contains(&code) => match code {
                65 => Self::A,
                66 => Self::B,
                67 => Self::C,
                68 => Self::D,
                69 => Self::E,
                70 => Self::F,
                71 => Self::G,
                72 => Self::H,
                73 => Self::I,
                74 => Self::J,
                75 => Self::K,
                76 => Self::L,
                77 => Self::M,
                78 => Self::N,
                79 => Self::O,
                80 => Self::P,
                81 => Self::Q,
                82 => Self::R,
                83 => Self::S,
                84 => Self::T,
                85 => Self::U,
                86 => Self::V,
                87 => Self::W,
                88 => Self::X,
                89 => Self::Y,
                90 => Self::Z,
                _ => Self::Unknown(code),
            },
            _ if (48..=57).contains(&code) => match code {
                48 => Self::Digit0,
                49 => Self::Digit1,
                50 => Self::Digit2,
                51 => Self::Digit3,
                52 => Self::Digit4,
                53 => Self::Digit5,
                54 => Self::Digit6,
                55 => Self::Digit7,
                56 => Self::Digit8,
                57 => Self::Digit9,
                _ => Self::Unknown(code),
            },
            _ => Self::Unknown(code),
        }
    }
}

/// Keyboard modifier state
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Modifiers {
    /// Shift key pressed
    pub shift: bool,
    /// Control key pressed
    pub ctrl: bool,
    /// Alt key pressed
    pub alt: bool,
    /// Meta/Command key pressed
    pub meta: bool,
}

/// Keyboard event
#[derive(Clone, Debug)]
pub struct KeyEvent {
    /// Key code
    pub key_code: KeyCode,
    /// Modifier keys
    pub modifiers: Modifiers,
    /// Whether this is a key down event (false for key up)
    pub key_down: bool,
    /// Whether the key was repeated
    pub repeated: bool,
}

/// Result of processing a keyboard event
#[derive(Clone, Debug)]
pub struct KeyEventResult {
    /// Whether the event was handled
    pub handled: bool,
    /// Announcement to make (if any)
    pub announcement: Option<A11yAnnouncement>,
    /// Whether focus changed
    pub focus_changed: bool,
    /// New focus index (if changed)
    pub new_focus_index: Option<usize>,
}

/// Keyboard navigation mode
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationMode {
    /// Normal mode (select shapes, pan, zoom)
    Normal,
    /// Focus mode (tab through elements)
    Focus,
    /// Read mode (read canvas content)
    Read,
}

/// Navigation direction for keyboard navigation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavigationDirection {
    /// Next element in tab order
    Next,
    /// Previous element in tab order
    Previous,
    /// Element above current
    Up,
    /// Element below current
    Down,
    /// Element to the left
    Left,
    /// Element to the right
    Right,
    /// First element
    First,
    /// Last element
    Last,
}
