use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Copy, Debug, PartialEq, Clone, Eq, Hash)]
pub enum Direction {
    Clientbound,
    Serverbound,
}

#[derive(Copy, Debug, PartialEq, Clone, Default, Eq, Hash)]
pub enum State {
    #[default]
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
    Transfer,
}

impl Display for State {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            State::Handshake => f.write_str("handshake"),
            State::Status => f.write_str("status"),
            State::Login => f.write_str("login"),
            State::Configuration => f.write_str("configuration"),
            State::Play => f.write_str("play"),
            State::Transfer => f.write_str("transfer"),
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Direction::Clientbound => f.write_str("clientbound"),
            Direction::Serverbound => f.write_str("serverbound"),
        }
    }
}
