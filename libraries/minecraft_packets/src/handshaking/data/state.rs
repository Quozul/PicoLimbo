use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, PartialEq, Clone)]
pub enum State {
    Handshake,
    Status,
    Login,
    Configuration,
    Play,
    Transfer,
}

#[derive(Error, Debug)]
#[error("Unknown state: {0}")]
pub struct UnknownStateError(pub i32);

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Handshake => write!(f, "Handshake"),
            State::Status => write!(f, "Status"),
            State::Login => write!(f, "Login"),
            State::Configuration => write!(f, "Configuration"),
            State::Play => write!(f, "Play"),
            State::Transfer => write!(f, "Transfer"),
        }
    }
}
