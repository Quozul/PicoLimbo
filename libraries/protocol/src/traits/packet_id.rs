pub trait PacketId {
    const PACKET_ID: u8;

    fn get_packet_id(&self) -> u8;
}
