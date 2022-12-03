
use btleplug::platform::PeripheralId;
use uuid::Uuid;

#[derive(Hash, Eq, PartialEq, Debug)]
pub struct DeviceRegistry {
    pub id: PeripheralId,
    pub services: Vec<String>
}

pub struct BleaParserBase {
    pub name: String,
    pub service_advertisement_ids: Vec<String>
}

pub trait Parser {
    fn is_parseable(&self, id: &Uuid) -> bool;
    fn parse(&self, data: Vec<u8>) -> &str;
}