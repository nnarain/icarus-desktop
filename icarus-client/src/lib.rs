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
    api::{Peripheral, Characteristic, WriteType},
};

use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    time,
};

use futures::stream::StreamExt;

use std::{
    io::Cursor,
    time::Duration,
    mem,
};

use uuid::Uuid;
use byteorder::{NetworkEndian, ReadBytesExt, WriteBytesExt};



#[derive(Default, Debug)]
pub struct Attitude {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32
}

#[derive(Default, Debug)]
pub struct Throttle {
    pub pitch: i16,
    pub roll: i16,
    pub yaw: i16,
}

pub struct Client {
    pub services: Vec<Uuid>,
    pub attitude_recv: Receiver<Attitude>,
    pub throttle_send: Sender<Throttle>,
}

impl Client {
    pub fn split(self) -> (Receiver<Attitude>, Sender<Throttle>) {
        (self.attitude_recv, self.throttle_send)
    }
}

const ATTITUDE_CHARACTERISTIC: Uuid = Uuid::from_u128(0x68af1093_1df9_41ac_98e8_d524a025b4b9);
const THROTTLE_CHARACTERISTIC: Uuid = Uuid::from_u128(0xc346b87e_9a11_4a56_9a53_e421c8ade193);


pub async fn initialize() -> anyhow::Result<Client> {
    // Find the device
    let device = utils::find_device().await?;
    device.discover_services().await?;

    let services: Vec<Uuid> = device.services().iter().map(|s| s.uuid).collect();

    // Setup client streams
    let (attitude_tx, attitude_rx) = mpsc::channel::<Attitude>(10);
    let (throttle_tx, throttle_rx) = mpsc::channel::<Throttle>(10);

    let attitude_char = device
                            .characteristics()
                            .iter()
                            .filter(|c| c.uuid == ATTITUDE_CHARACTERISTIC)
                            .next()
                            .map(|c| c.clone())
                            .ok_or(IcarusBleError::CharacteristicNotFound)?;

    let throttle_char = device
                            .characteristics()
                            .iter()
                            .filter(|c| c.uuid == THROTTLE_CHARACTERISTIC)
                            .next()
                            .map(|c| c.clone())
                            .ok_or(IcarusBleError::CharacteristicNotFound)?;

    tokio::spawn(attitude_recv_task(device.clone(), attitude_char, attitude_tx));
    tokio::spawn(throttle_send_task(device.clone(), throttle_char, throttle_rx));


    let client = Client { services, attitude_recv: attitude_rx, throttle_send: throttle_tx };
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

async fn throttle_send_task<P: Peripheral>(p: P, c: Characteristic, mut rx: Receiver<Throttle>) -> anyhow::Result<()> {
    while let Some(throttle) = rx.recv().await {
        let mut data: [u8; mem::size_of::<Throttle>()] = Default::default();

        let mut cursor = Cursor::new(&mut data[..]);
        cursor.write_i16::<NetworkEndian>(throttle.pitch)?;
        cursor.write_i16::<NetworkEndian>(throttle.roll)?;
        cursor.write_i16::<NetworkEndian>(throttle.yaw)?;

        log::debug!("Sending throttle: ({}, {}, {})", throttle.pitch, throttle.roll, throttle.yaw);

        p.write(&c, &data[..], WriteType::WithoutResponse).await?;
    }

    Ok(())
}
