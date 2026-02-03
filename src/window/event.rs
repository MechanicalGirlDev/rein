//! Event types for input handling
//!
//! Provides platform-independent event types for mouse and keyboard input.

/// Mouse button type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Keyboard key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    // Letters
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

    // Numbers
    Key0,
    Key1,
    Key2,
    Key3,
    Key4,
    Key5,
    Key6,
    Key7,
    Key8,
    Key9,

    // Function keys
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

    // Special keys
    Escape,
    Tab,
    Space,
    Enter,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,

    // Arrow keys
    Left,
    Right,
    Up,
    Down,

    // Modifier keys
    Shift,
    Control,
    Alt,
}

impl Key {
    /// Convert from winit key.
    pub fn from_winit(key: &winit::keyboard::Key) -> Option<Self> {
        use winit::keyboard::{Key as WKey, NamedKey};

        match key {
            WKey::Character(c) => {
                let c = c.chars().next()?;
                match c.to_ascii_lowercase() {
                    'a' => Some(Key::A),
                    'b' => Some(Key::B),
                    'c' => Some(Key::C),
                    'd' => Some(Key::D),
                    'e' => Some(Key::E),
                    'f' => Some(Key::F),
                    'g' => Some(Key::G),
                    'h' => Some(Key::H),
                    'i' => Some(Key::I),
                    'j' => Some(Key::J),
                    'k' => Some(Key::K),
                    'l' => Some(Key::L),
                    'm' => Some(Key::M),
                    'n' => Some(Key::N),
                    'o' => Some(Key::O),
                    'p' => Some(Key::P),
                    'q' => Some(Key::Q),
                    'r' => Some(Key::R),
                    's' => Some(Key::S),
                    't' => Some(Key::T),
                    'u' => Some(Key::U),
                    'v' => Some(Key::V),
                    'w' => Some(Key::W),
                    'x' => Some(Key::X),
                    'y' => Some(Key::Y),
                    'z' => Some(Key::Z),
                    '0' => Some(Key::Key0),
                    '1' => Some(Key::Key1),
                    '2' => Some(Key::Key2),
                    '3' => Some(Key::Key3),
                    '4' => Some(Key::Key4),
                    '5' => Some(Key::Key5),
                    '6' => Some(Key::Key6),
                    '7' => Some(Key::Key7),
                    '8' => Some(Key::Key8),
                    '9' => Some(Key::Key9),
                    _ => None,
                }
            }
            WKey::Named(named) => match named {
                NamedKey::Escape => Some(Key::Escape),
                NamedKey::Tab => Some(Key::Tab),
                NamedKey::Space => Some(Key::Space),
                NamedKey::Enter => Some(Key::Enter),
                NamedKey::Backspace => Some(Key::Backspace),
                NamedKey::Delete => Some(Key::Delete),
                NamedKey::Insert => Some(Key::Insert),
                NamedKey::Home => Some(Key::Home),
                NamedKey::End => Some(Key::End),
                NamedKey::PageUp => Some(Key::PageUp),
                NamedKey::PageDown => Some(Key::PageDown),
                NamedKey::ArrowLeft => Some(Key::Left),
                NamedKey::ArrowRight => Some(Key::Right),
                NamedKey::ArrowUp => Some(Key::Up),
                NamedKey::ArrowDown => Some(Key::Down),
                NamedKey::Shift => Some(Key::Shift),
                NamedKey::Control => Some(Key::Control),
                NamedKey::Alt => Some(Key::Alt),
                NamedKey::F1 => Some(Key::F1),
                NamedKey::F2 => Some(Key::F2),
                NamedKey::F3 => Some(Key::F3),
                NamedKey::F4 => Some(Key::F4),
                NamedKey::F5 => Some(Key::F5),
                NamedKey::F6 => Some(Key::F6),
                NamedKey::F7 => Some(Key::F7),
                NamedKey::F8 => Some(Key::F8),
                NamedKey::F9 => Some(Key::F9),
                NamedKey::F10 => Some(Key::F10),
                NamedKey::F11 => Some(Key::F11),
                NamedKey::F12 => Some(Key::F12),
                _ => None,
            },
            _ => None,
        }
    }
}

/// Modifier key state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
}

impl Modifiers {
    /// Check if any modifier is pressed.
    pub fn any(&self) -> bool {
        self.shift || self.ctrl || self.alt
    }

    /// Check if no modifier is pressed.
    pub fn none(&self) -> bool {
        !self.any()
    }
}

/// Input event.
#[derive(Debug, Clone)]
pub enum Event {
    /// Mouse button pressed.
    MousePress {
        button: MouseButton,
        position: (f32, f32),
        modifiers: Modifiers,
        handled: bool,
    },

    /// Mouse button released.
    MouseRelease {
        button: MouseButton,
        position: (f32, f32),
        modifiers: Modifiers,
        handled: bool,
    },

    /// Mouse moved.
    MouseMotion {
        delta: (f32, f32),
        position: (f32, f32),
        modifiers: Modifiers,
        handled: bool,
    },

    /// Mouse wheel scrolled.
    MouseWheel {
        delta: (f32, f32),
        position: (f32, f32),
        modifiers: Modifiers,
        handled: bool,
    },

    /// Key pressed.
    KeyPress {
        key: Key,
        modifiers: Modifiers,
        handled: bool,
    },

    /// Key released.
    KeyRelease {
        key: Key,
        modifiers: Modifiers,
        handled: bool,
    },

    /// Window resized.
    Resize { width: u32, height: u32 },
}

impl Event {
    /// Check if the event has been handled.
    pub fn is_handled(&self) -> bool {
        match self {
            Event::MousePress { handled, .. } => *handled,
            Event::MouseRelease { handled, .. } => *handled,
            Event::MouseMotion { handled, .. } => *handled,
            Event::MouseWheel { handled, .. } => *handled,
            Event::KeyPress { handled, .. } => *handled,
            Event::KeyRelease { handled, .. } => *handled,
            Event::Resize { .. } => false,
        }
    }

    /// Mark the event as handled.
    pub fn set_handled(&mut self) {
        match self {
            Event::MousePress { handled, .. } => *handled = true,
            Event::MouseRelease { handled, .. } => *handled = true,
            Event::MouseMotion { handled, .. } => *handled = true,
            Event::MouseWheel { handled, .. } => *handled = true,
            Event::KeyPress { handled, .. } => *handled = true,
            Event::KeyRelease { handled, .. } => *handled = true,
            Event::Resize { .. } => {}
        }
    }
}
