use std::collections::HashMap;
use btleplug::platform::PeripheralId;
use byteorder::{LittleEndian, ReadBytesExt};
use log::{info, log, warn};
use phf::phf_map;
use uuid::Uuid;
use crate::models::{BleaParserBase, Parser};
use enum_iterator::{all, Sequence};
use hex::encode;
use rumqttc::Event;
use crate::parsers::responses::Responses;
use num_derive::FromPrimitive;
use num_traits::{FromPrimitive, ToPrimitive};

static DEVICES: phf::Map<u16, &'static str> = phf_map! {
    0x01AAu16 => "LYWSDCGQ",
    0x045Bu16 => "LYWSD02",
    0x055Bu16 => "LYWSD03MMC",
    0x0098u16 => "HHCCJCY01",
    0x03BCu16 => "GCLS002",
    0x015Du16 => "HHCCPOT002",
    0x040Au16 => "WX08ZM",
    0x098Bu16 => "MCCGQ02HL",
    0x0083u16 => "YM-K1501",
    0x0113u16 => "YM-K1501EU",
    0x045Cu16 => "V-SK152",
    0x0863u16 => "SJWS01LM",
    0x07F6u16 => "MJYD02YL",
    0x03DDu16 => "MUE4094RT",
    0x0A8Du16 => "RTCGQ02LM",
    0x00DBu16 => "MMC-T201-1",
    0x0489u16 => "M1S-T500",
    0x0C3Cu16 => "CGC1",
    0x0576u16 => "CGD1",
    0x066Fu16 => "CGDK2",
    0x0347u16 => "CGG1",
    0x0B48u16 => "CGG1-ENCRYPTED",
    0x03D6u16 => "CGH1",
    0x0A83u16 => "CGPR1",
    0x06d3u16 => "MHO-C303",
    0x0387u16 => "MHO-C401",
    0x02DFu16 => "JQJCY01YM",
    0x0997u16 => "JTYJGD03MI",
    0x1568u16 => "K9B-1BTN",
    0x1569u16 => "K9B-2BTN",
    0x0DFDu16 => "K9B-3BTN",
    0x07BFu16 => "YLAI003",
    0x0153u16 => "YLYK01YL",
    0x068Eu16 => "YLYK01YL-FANCL",
    0x04E6u16 => "YLYK01YL-VENFAN",
    0x03BFu16 => "YLYB01YL-BHFRC",
    0x03B6u16 => "YLKG07YL/YLKG08YL",
    0x069Eu16 => "ZNMS16LM",
    0x069Fu16 => "ZNMS17LM",
};

enum FrameControlFlags {
    FactoryNew,
    Connected,
    Central,
    Encrypted,
    HasMacAddress,
    HasCapabilities,
    HasEvent,
    HasCustomData,
    HasSubtitle,
    HasBinding
}

static FRAME_CONTROL_FLAGS : phf::Map<&'static str, u16> = phf_map! {
    "FactoryNew" => 1 << 0,
    "Connected" => 1 << 1,
    "Central" => 1 << 2,
    "Encrypted" => 1 << 3,
    "HasMacAddress" => 1 << 4,
    "HasCapabilities" => 1 << 5,
    "HasEvent" => 1 << 6,
    "HasCustomData" => 1 << 7,
    "HasSubtitle" => 1 << 8,
    "HasBinding" => 1 << 9,
};

#[derive(FromPrimitive, Debug)]
enum EventTypes {
    // basic events
    Connection = 0x0001,
    Pairing = 0x0002,

    // base supported events
    Temperature = 0x1004,
    Humidity = 0x1006,
    Illuminance = 0x1007,
    Moisture = 0x1008,
    SoilConductivity = 0x1009,
    Formaldehyde = 0x1010,
    // not supported
    Switch = 0x1012,
    // consumable, in percentage, not supported
    Consumable = 0x1013,

    Moisture2 = 0x1014,
    // not supported
    Smoke = 0x1015,
    // not supported
    Motion2 = 0x1017,
    // not supported
    LightIntensity = 0x1018,
    // not supported
    Door = 0x1019,
    // not supported
    Battery = 0x100a,
    TemperatureAndHumidity = 0x100d,

    // not supported by this lib
    Motion = 0x0003,
    FingerPrint = 0x0006,
    ToothBrush = 0x0010,
    Lock = 0x000b,
    MoveWithLight = 0x000f,
    Remote = 0x1001,
    BodyTemperature = 0x2000,
}


pub struct XiaomiParser {
    base: BleaParserBase,
}

pub fn get_parser() -> Box<dyn Parser> {
    Box::new(XiaomiParser {
        base: BleaParserBase {
            name: "xiaomi".to_string(),
            service_advertisement_ids: Vec::from(["fe95".to_string()]),
        }
    })
}


impl Parser for XiaomiParser {
    fn is_parseable(&self, id: &Uuid) -> bool {
        for service in &self.base.service_advertisement_ids {
            let search_for = format!("{service}-");
            let id_in_string = id.to_string();
            if id_in_string.contains(&search_for) {
                return true
            }
        }
        false
    }
    fn parse(&self, id : PeripheralId, data: Vec<u8>) -> String {
        let mut base_byte_length =5;

        let mut current = &data[0..4];
        let frame_control = current.read_u16::<LittleEndian>().unwrap();
        let device = current.read_u16::<LittleEndian>().unwrap();

        let mut flags : HashMap<String, bool> = HashMap::new();
        for (flag, mask) in FRAME_CONTROL_FLAGS.into_iter() {
            let has_flag = (frame_control & mask) != 0;
            flags.insert(flag.to_string(), has_flag);
        }
        let version = data[1] >> 4;

        if flags.get("Encrypted").unwrap() == &true || flags.get("HasEvent").unwrap() == &false  {
            return String::from("");
        }

        let mut mac_address: String = String::from("");
        if flags.contains_key("HasMacAddress") {
            let mac_addr_buffer = &data[base_byte_length..base_byte_length + 6];
            mac_address = encode(mac_addr_buffer);
        }

        warn!("{mac_address}");

        let mut offset = base_byte_length;
        // get event offset
        if DEVICES.contains_key(&device) {
            info!("Device: {}, v{version} \n flags: {:?}", DEVICES.get(&device).unwrap(), flags);

            if flags.contains_key("HasEvent") {
                // calculate event offset
                if flags.get("HasMacAddress").unwrap() == &true {
                    offset = 11;
                }
                if flags.get("HasCapabilities").unwrap() == &true {
                    offset += 1;
                }
            }
        }

        // no data
        if data.len() < offset + 2 {
            return String::from("{}");
        }

        // get event type
        let event_id = (&data[offset..offset+2]).read_u16::<LittleEndian>().unwrap();
        let event_type = FromPrimitive::from_i32(event_id as i32);

        warn!("{:?}", &event_type);

        let event = match event_type {
            Some(EventTypes::Battery) => Responses::BatteryStatusResponse {
                charge: data[offset + 3]
            },
            Some(EventTypes::TemperatureAndHumidity) => Responses::TemperatureAndHumidityResponse {
                temperature: (&data[offset+3..offset+5]).read_i16::<LittleEndian>().unwrap() as f32 / 10f32,
                humidity: (&data[offset+5..offset+7]).read_u16::<LittleEndian>().unwrap() as f32 / 10f32,
            },
            Some(EventTypes::Temperature) => Responses::TemperatureResponse {
                temperature: (&data[offset+3..offset+5]).read_i16::<LittleEndian>().unwrap() as f32 / 10f32,
            },
            Some(EventTypes::Humidity) => Responses::HumidityResponse {
                humidity: (&data[offset+3..offset+5]).read_u16::<LittleEndian>().unwrap() as f32 / 10f32,
            },
            _ => Responses::None {}
        };

        warn!("mac: {mac_address} event: {:?}", event);
        serde_json::to_string(&event).unwrap()
    }
}