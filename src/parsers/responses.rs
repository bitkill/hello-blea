use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Responses {
    TemperatureAndHumidityResponse {
        temperature: f32,
        humidity: f32,
    },
    TemperatureResponse {
        temperature: f32,
    },
    HumidityResponse {
        humidity: f32,
    },
    BatteryStatusResponse {
        charge: u8
    },
    PressureStatusResponse {
        pressure: u16
    },
    SoilConductivityResponse {
        conductivity: i16
    },
    MoistureResponse {
        moisture: i8
    },
    IlluminanceResponse {
        illuminance: i32
    },
    None {}
}