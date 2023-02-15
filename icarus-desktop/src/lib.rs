//
// lib.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 14 2023
//
mod sensors;

use sensors::SensorBuffer;

use bevy::prelude::*;
use icarus_client::Attitude;
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    runtime::Builder
};
use std::thread;
use log;

#[derive(Resource)]
pub struct Channels {
    pub attitude: Receiver<Attitude>,
}

#[derive(Resource)]
pub struct Sensors {
    pub attitude: SensorBuffer<Attitude>,
}

pub struct IcarusPlugin;

impl Plugin for IcarusPlugin {
    fn build(&self, app: &mut App) {
        // Setup data channels to communicate with the async runtime
        let (attitude_tx, attitude_rx) = mpsc::channel::<Attitude>(50);
        let channels = Channels { attitude: attitude_rx };

        // Spawn the async runtime
        thread::spawn(|| icarus_async_runtime(attitude_tx));

        // Buffers for sensor data
        let attitude_sensor: SensorBuffer<Attitude> = SensorBuffer::new(500);

        let sensors = Sensors {attitude: attitude_sensor};

        // Add the data channels to bevy's resource manager
        app
            .insert_resource(channels)
            .insert_resource(sensors)
            .add_system(update_sensors_system);
    }
}

fn update_sensors_system(mut channels: ResMut<Channels>, mut sensors: ResMut<Sensors>) {
    while let Ok(data) = channels.attitude.try_recv() {
        sensors.attitude.push(data);
    }
}

fn icarus_async_runtime(tx: Sender<Attitude>) -> anyhow::Result<()> {
    // Spawn an async runtime to collect sensor and state data
    let runtime = Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(collect_icarus_data(tx))?;

    Ok(())
}

async fn collect_icarus_data(tx: Sender<Attitude>) -> anyhow::Result<()> {
    let (mut attitude, _) = icarus_client::initialize().await?.split();

    while let Some(attitude) = attitude.recv().await {
        if let Err(e) = tx.send(attitude).await {
            log::error!("Error sending data: {}", e);
        }
    }

    Ok(())
}
