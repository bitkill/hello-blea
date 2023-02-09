// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.
mod models;
mod parsers;

use std::{env, process};
use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use dotenv::dotenv;
use log::{debug, info};
use rumqttc::{Client, MqttOptions, QoS};
use crate::models::{Parser};


async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    dotenv().ok();

    // set default log level to info
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }

    // get mqtt vars
    let mqtt_hostname = env::var("MQTT_HOSTNAME").unwrap();
    let mqtt_client_name = env::var("MACHINE_NAME").unwrap();
    let mqtt_username = env::var("MQTT_USERNAME").unwrap();
    let mqtt_password = env::var("MQTT_PASSWORD").unwrap();
    let mqtt_topic = env::var("MQTT_TOPIC").unwrap();

    let mut mqtt_options = MqttOptions::new(mqtt_client_name, mqtt_hostname, 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_credentials(mqtt_username, mqtt_password);

    let (mut client, mut connection) = Client::new(mqtt_options, 10);
    client.subscribe("demo/mqtt", QoS::AtMostOnce).unwrap();

    pretty_env_logger::init();

    info!("Starting up hello-blea.");

    let parsers = Vec::from([
        parsers::xiaomi::get_parser(),
        parsers::qingping::get_parser()
    ]);

    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    // connect to the adapter
    let central = get_central(&manager).await;

    // Each adapter has an event stream, we fetch via events(),
    // simplifying the type, this will return what is essentially a
    // Future<Result<Stream<Item=CentralEvent>>>.
    let mut events = central.events().await?;

    // start scanning for devices
    central.start_scan(ScanFilter::default()).await?;

    // Print based on whatever the event receiver outputs. Note that the event
    // receiver blocks, so in a real program, this should be run in its own
    // thread (not task, as this library does not yet use async channels).
    while let Some(event) = events.next().await {
        match event {
            CentralEvent::DeviceDiscovered(id) => {
                debug!("DeviceDiscovered: {:?}", id);
            }
            CentralEvent::DeviceConnected(id) => {
                debug!("DeviceConnected: {:?}", id);
            }
            CentralEvent::DeviceDisconnected(id) => {
                debug!("DeviceDisconnected: {:?}", id);
            }
            CentralEvent::ManufacturerDataAdvertisement {
                id,
                manufacturer_data,
            } => {
                debug!("ManufacturerDataAdvertisement: {:?}, {:?}", id, manufacturer_data);
            }
            CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                info!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                for parser in &parsers {
                    for data in &service_data {
                        if parser.is_parseable(&data.0) {
                            let parsed = parser.parse(data.1.clone());
                            let _ = client.publish(&mqtt_topic, QoS::ExactlyOnce, false, parsed);
                        }
                    }
                }
            }
            CentralEvent::ServicesAdvertisement { id, services } => {
                let services: Vec<String> =
                    services.into_iter().map(|s| s.to_short_string()).collect();
                debug!("ServicesAdvertisement: {:?}, {:?}", id, services);
            }
            _ => {}
        }
    }
    Ok(())
}
