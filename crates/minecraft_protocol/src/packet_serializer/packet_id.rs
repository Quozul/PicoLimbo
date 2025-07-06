pub trait PacketId {
    const PACKET_NAME: &'static str;

    fn get_packet_name(&self) -> &'static str;
}
