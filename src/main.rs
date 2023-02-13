// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.
mod models;
mod parsers;

use std::{env};
use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use dotenv::dotenv;
use log::{debug, error, info, warn};
use rumqttc::{AsyncClient, Client, MqttOptions, QoS};
use crate::models::{Parser};


async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}

fn banner() {
    const B: &str = r#"
      ___            __      __        ___
|__| |__  |    |    /  \ _ \|__) |    |__   /\
|  | |___ |___ |___ \__/   /|__) |___ |___ /--\

    "#;

    println!("{B}\n");
}

fn env_or_default(key: &str, default: &str) -> String {
    if let Err(_) = env::var(key) {
        return default.to_string()
    }
    env::var(key).unwrap()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    dotenv().ok();
    banner();

    // set default log level to info
    if let Err(_) = env::var("RUST_LOG") {
        env::set_var("RUST_LOG", "info");
    }

    pretty_env_logger::init();

    // get mqtt vars
    let mqtt_hostname = env_or_default("MQTT_HOSTNAME", "localhost");
    let mqtt_client_name = env_or_default("MACHINE_NAME", "hbela-unknown");
    let mqtt_username = env_or_default("MQTT_USERNAME", "mosquitto");
    let mqtt_password = env_or_default("MQTT_PASSWORD", "mosquitto");
    let mqtt_topic = env_or_default("MQTT_TOPIC", "mosquitto");

    let mut mqtt_options = MqttOptions::new(mqtt_client_name, mqtt_hostname, 1883);
    mqtt_options.set_keep_alive(Duration::from_secs(5));
    mqtt_options.set_credentials(mqtt_username, mqtt_password);

    let (mut client, mut eventloop) = AsyncClient::new(mqtt_options, 10);
    client.subscribe("demo/mqtt", QoS::AtMostOnce).await.unwrap();

    info!("Subscribed to mqtt topic, starting up bluetooth reader");

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
                //info!("ServiceDataAdvertisement: {:?}, {:?}", id, service_data);
                for parser in &parsers {
                    for data in &service_data {
                        if parser.is_parseable(&data.0) {
                            let parsed = parser.parse(id.clone(), data.1.clone());
                            warn!("{parsed}");
                            //let _ = client.publish(&MQTT_TOPIC, QoS::ExactlyOnce, false, parsed);
                            //tokio::spawn(async move {
                            //let e = client.publish(&mqtt_topic, QoS::ExactlyOnce, false, parsed);

                            //});
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
