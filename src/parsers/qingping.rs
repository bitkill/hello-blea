use btleplug::platform::PeripheralId;
use byteorder::{LittleEndian, ReadBytesExt};
use hex::encode;
use log::warn;
use phf::phf_map;
use uuid::Uuid;
use crate::models::BleaParserBase;
use crate::Parser;
use crate::parsers::responses::Responses;


static DEVICES: phf::Map<u16, &'static str> = phf_map! {
  0x01u16 => "CGG1",
  0x07u16 => "CGG1",
  0x09u16 => "CGP1W",
  0x0cu16 => "CGD1",
  0x10u16 => "CGDK2"
};

enum EventTypes {
    TemperatureAndHumidity = 0x01,
    Battery = 0x02,
    Pressure = 0x07,
}


pub struct QingPingParser {
    base: BleaParserBase,
}

pub fn get_parser() -> Box<dyn Parser> {
    Box::new(QingPingParser {
        base: BleaParserBase {
            name: "qingping".to_string(),
            service_advertisement_ids: Vec::from([
                "fff9".to_string(),
                "fdcd".to_string()
            ]),
        }
    })
}

impl Parser for QingPingParser {
    #[allow(unused_variables)]
    fn is_parseable(&self, id: &Uuid) -> bool {
        for service in self.base.service_advertisement_ids.clone() {
            let search_for = format!("{service}-");
            if id.to_string().contains(&search_for) {
                return true;
            }
        }

        false
    }

    #[allow(unused_variables)]
    fn parse(&self, id : PeripheralId, data: Vec<u8>) -> String {
        let mut base_byte_length = 5;

        let mut mac_addr_buffer = Vec::from(&data[base_byte_length..base_byte_length + 6]);
        mac_addr_buffer.reverse();

        let mac_address: String = encode(&mac_addr_buffer);
        base_byte_length = 11;

        let event_id = &data[8];

        let data_position = 10;

        let event = match event_id {
            0x01 => Responses::TemperatureAndHumidityResponse {
                temperature: ((&data[data_position..data_position + 2]).read_i16::<LittleEndian>().unwrap() as f32) / 10f32,
                humidity: ((&data[data_position+2..data_position + 4]).read_u16::<LittleEndian>().unwrap() as f32) / 10f32,
            },
            0x02 => Responses::BatteryStatusResponse { charge: data[data_position] },
            0x07 => Responses::PressureStatusResponse {
                pressure: (&data[data_position..data_position + 2]).read_u16::<LittleEndian>().unwrap()
            },
            _ => { Responses::None {} }
        };


        warn!("mac: {mac_address} event: {:?}", event);
        serde_json::to_string(&event).unwrap()
    }
}
