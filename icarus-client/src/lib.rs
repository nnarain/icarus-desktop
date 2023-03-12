//
// lib.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 12 2023
//
mod utils;
mod error;

use error::IcarusBleError;

use btleplug::{
    api::{Peripheral, Characteristic},
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

use uuid::Uuid;
use byteorder::{NetworkEndian, ReadBytesExt};



#[derive(Default)]
pub struct Attitude {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32
}

pub struct Client {
    pub services: Vec<Uuid>,
    pub attitude_recv: Receiver<Attitude>,
}

impl Client {
    pub fn split(self) -> (Receiver<Attitude>, ()) {
        (self.attitude_recv, ())
    }
}

const ATTITUDE_CHARACTERISTIC: Uuid = Uuid::from_u128(0x68af1093_1df9_41ac_98e8_d524a025b4b9);


pub async fn initialize() -> anyhow::Result<Client> {
    // Find the device
    let device = utils::find_device().await?;
    device.discover_services().await?;

    let services: Vec<Uuid> = device.services().iter().map(|s| s.uuid).collect();

    // Setup client streams
    let (attitude_tx, attitude_rx) = mpsc::channel::<Attitude>(10);

    let attitude_char = device
                            .characteristics()
                            .iter()
                            .filter(|c| c.uuid == ATTITUDE_CHARACTERISTIC)
                            .next()
                            .map(|c| c.clone())
                            .ok_or(IcarusBleError::CharacteristicNotFound)?;

    tokio::spawn(attitude_recv_task(device.clone(), attitude_char, attitude_tx));


    let client = Client { services, attitude_recv: attitude_rx };
    Ok(client)
}

async fn attitude_recv_task<P: Peripheral>(p: P, c: Characteristic, tx: Sender<Attitude>) -> anyhow::Result<()> {
    p.subscribe(&c).await?;

    let mut stream = p.notifications().await?;

    while !tx.is_closed() {
        while let Some(data) = stream.next().await {
            let mut cursor = Cursor::new(&data.value[..]);

            let pitch = cursor.read_f32::<NetworkEndian>()?;
            let roll = cursor.read_f32::<NetworkEndian>()?;
            let yaw = cursor.read_f32::<NetworkEndian>()?;

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
