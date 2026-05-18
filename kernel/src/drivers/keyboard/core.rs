#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum Action {
    Press,
    Release,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum Key {
    /// printable character
    Char(u8),
    /// raw usage id
    Raw(u8),
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, defmt::Format)]
#[repr(transparent)]
pub struct Modifiers(pub u8);

#[allow(unused)]
impl Modifiers {
    pub const CTRL: u8 = 1;
    pub const SHIFT: u8 = 2;
    pub const ALT: u8 = 4;
    pub const META: u8 = 8;

    pub fn shift(self) -> bool {
        self.0 & Self::SHIFT != 0
    }
    pub fn ctrl(self) -> bool {
        self.0 & Self::CTRL != 0
    }
    pub fn alt(self) -> bool {
        self.0 & Self::ALT != 0
    }
    pub fn meta(self) -> bool {
        self.0 & Self::META != 0
    }
}

#[derive(Debug, Copy, Clone, defmt::Format)]
pub struct KeyEvent {
    pub key: Key,
    pub action: Action,
    pub modifiers: Modifiers,
}
