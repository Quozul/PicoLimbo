use minecraft_protocol::prelude::*;

#[derive(PacketOut)]
pub struct DeathLocation {
    /// Name of the dimension the player died in.
    death_dimension_name: Omitted<Identifier>,
    /// The location that the player died at.
    death_location: Omitted<Position>,
}
