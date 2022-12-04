// See the "macOS permissions note" in README.md before running this on macOS
// Big Sur or later.
mod models;
mod parsers;

use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter};
use btleplug::platform::{Adapter, Manager};
use futures::stream::StreamExt;
use std::error::Error;
use log::{debug, info, LevelFilter};
use crate::models::{Parser};


async fn get_central(manager: &Manager) -> Adapter {
    let adapters = manager.adapters().await.unwrap();
    adapters.into_iter().nth(0).unwrap()
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    // set default log level to info
    if let Err(_) = std::env::var("RUST_LOG") {
        std::env::set_var("RUST_LOG", "info");
    }

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
                            let _ = parser.parse(data.1.clone());
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
