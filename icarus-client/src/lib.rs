//
// lib.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 12 2023
//

use btleplug::{
    api::{Central, Manager as _, Peripheral, ScanFilter, Characteristic},
    platform::Manager
};

use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time,
};

use futures::stream::StreamExt;

use std::{
    io::Cursor,
    time::Duration,
};
use thiserror::Error;
use uuid::Uuid;
use byteorder::{LittleEndian, ReadBytesExt};

#[derive(Debug, Error)]
pub enum Error {
    #[error("The device was not found")]
    DeviceNotFound,
    #[error("Characteristic not found")]
    CharacteristicNotFound,
}

#[derive(Default)]
pub struct Attitude {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32
}

pub struct Client {
    attitude_recv: Receiver<Attitude>,
}

impl Client {
    pub fn split(self) -> (Receiver<Attitude>, ()) {
        (self.attitude_recv, ())
    }
}

const ATTITUDE_CHARACTERISTIC: Uuid = Uuid::from_u128(0x68af1093_1df9_41ac_98e8_d524a025b4b9);


pub async fn initialize() -> anyhow::Result<Client> {
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

        // Find the icarus device
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

                peripheral.discover_services().await?;

                // Setup client streams
                let (attitude_tx, attitude_rx) = mpsc::channel::<Attitude>(10);

                let attitude_char = peripheral
                                        .characteristics()
                                        .iter()
                                        .filter(|c| c.uuid == ATTITUDE_CHARACTERISTIC)
                                        .next()
                                        .map(|c| c.clone())
                                        .ok_or(Error::CharacteristicNotFound)?;

                tokio::spawn(attitude_recv_task(peripheral.clone(), attitude_char, attitude_tx));


                let client = Client { attitude_recv: attitude_rx };
                return Ok(client)
            }
        }
    }

    Err(Error::DeviceNotFound)?
}

async fn attitude_recv_task<P: Peripheral>(p: P, c: Characteristic, tx: Sender<Attitude>) -> anyhow::Result<()> {
    p.subscribe(&c).await?;

    let mut stream = p.notifications().await?;

    while !tx.is_closed() {
        while let Some(data) = stream.next().await {
            let mut cursor = Cursor::new(&data.value[..]);

            let pitch = cursor.read_f32::<LittleEndian>()?;
            let roll = cursor.read_f32::<LittleEndian>()?;
            let yaw = cursor.read_f32::<LittleEndian>()?;

            log::debug!("({}, {}, {})", pitch, roll, yaw);

            let attitude = Attitude {pitch, roll, yaw};

            if let Err(e) = tx.send(attitude).await {
                log::error!("Failed to send attitude data: {}", e);
            }
        }
        time::sleep(Duration::from_millis(10)).await;
    }

    Ok(())
}
