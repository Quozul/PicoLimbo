#[derive(Default, Clone, Debug, PartialEq, Eq, Copy)]
#[repr(u8)]
pub enum GameMode {
    Survival = 0,
    Creative = 1,
    Adventure = 2,
    #[default]
    Spectator = 3,
}

impl GameMode {
    pub const fn value(self) -> u8 {
        self as u8
    }
}
