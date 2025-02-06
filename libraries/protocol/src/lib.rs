extern crate core;

pub mod prelude {
    pub use data_types::prelude::*;
    pub use macro_traits::prelude::*;
    pub use macros::PacketIn;
    pub use macros::PacketOut;
    pub use macros::packet_id;
    pub use minecraft_packets::prelude::*;
    pub use nbt::prelude::*;
    pub use protocol_version::ProtocolVersion;
    pub use uuid::Uuid;
}
