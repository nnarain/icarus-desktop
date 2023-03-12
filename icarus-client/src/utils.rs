//
// utils.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 11 2023
//

use crate::error::IcarusBleError;

use anyhow::Ok;
use btleplug::{
    api::{Central, Manager as _, Peripheral as _, ScanFilter},
    platform::{Manager, Peripheral}
};
use std::time::Duration;
use tokio::time;

pub async fn find_device() -> anyhow::Result<Peripheral> {
    let manager = Manager::new().await?;
    let adaptor_list = manager.adapters().await?;

    for adaptor in adaptor_list.iter() {
        log::debug!("Starting scan of {}...", adaptor.adapter_info().await?);

        adaptor
            .start_scan(ScanFilter::default())
            .await
            .expect("Can't scan BLE adaptor for connected devices");

        time::sleep(Duration::from_secs(10)).await;

        let peripherals = adaptor.peripherals().await?;

        for peripheral in peripherals.iter() {
            let properties = peripheral.properties().await?;
            let is_connected = peripheral.is_connected().await?;
            let local_name = properties
                                .map(|p| p.local_name)
                                .flatten()
                                .unwrap_or(String::from("unknown"));

            if local_name == String::from("icarus") {
                if !is_connected {
                    if let Err(e) = peripheral.connect().await {
                        log::error!("Failed to connect: {}", e);
                        continue;
                    }
                }

                return Ok(peripheral.clone());
            }
        }
    }

    Err(IcarusBleError::DeviceNotFound)?
}
