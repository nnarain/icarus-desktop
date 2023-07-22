//
// lib.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 14 2023
//
mod sensors;

use sensors::SensorBuffer;
// use throttle::ThrottleControl;

use bevy::prelude::*;
use icarus_client::{Attitude, Throttle};
use tokio::{
    sync::mpsc::{self, Receiver, Sender},
    runtime::Builder
};
use std::{thread, collections::VecDeque};
use log;

#[derive(Component)]
struct FrameBody;

#[derive(Component)]
struct Orientation;

#[derive(Resource)]
pub struct Channels {
    pub attitude: Receiver<Attitude>,
    pub throttle: Sender<Throttle>,
}

#[derive(Resource)]
pub struct Sensors {
    pub attitude: SensorBuffer<Attitude>,
}

#[derive(Resource)]
pub struct ThrottleControl {
    queue: VecDeque<Throttle>,
    last_command: Throttle,
}

impl Default for ThrottleControl {
    fn default() -> Self {
        ThrottleControl { queue: VecDeque::with_capacity(3), last_command: Throttle::default() }
    }
}

impl ThrottleControl {
    pub fn enqueue(&mut self, throttle: Throttle) {
        if let Some(item) = self.queue.back() {
            if *item != throttle {
                self.queue.push_back(throttle.clone());
            }
        }
        else {
            self.queue.push_back(throttle.clone());
        }

        self.last_command = throttle;
    }

    pub fn dequeue(&mut self) -> Option<Throttle> {
        self.queue.pop_front()
    }

    pub fn last(&self) -> &Throttle {
        &self.last_command
    }
}

pub struct IcarusPlugin;

impl Plugin for IcarusPlugin {
    fn build(&self, app: &mut App) {
        // Setup data channels to communicate with the async runtime
        let (attitude_tx, attitude_rx) = mpsc::channel::<Attitude>(50);
        let (throttle_tx, throttle_rx) = mpsc::channel::<Throttle>(50);

        let channels = Channels { attitude: attitude_rx, throttle: throttle_tx };

        // Spawn the async runtime
        log::error!("spawning async runtime");
        thread::spawn(|| icarus_async_runtime(attitude_tx, throttle_rx));

        // Buffers for sensor data
        let attitude_sensor: SensorBuffer<Attitude> = SensorBuffer::new(250);
        let sensors = Sensors {attitude: attitude_sensor};

        // Throttle control
        // let throttle_control = ThrottleControl
        let throttle_control = ThrottleControl::default();

        // Add the data channels to bevy's resource manager
        app
            .insert_resource(channels)
            .insert_resource(sensors)
            .insert_resource(throttle_control)
            .add_startup_system(setup_3d_shapes)
            .add_system(update_sensors_system)
            .add_system(update_throttle_system)
            .add_system(update_frame_orientation);
    }
}

fn update_sensors_system(mut channels: ResMut<Channels>, mut sensors: ResMut<Sensors>) {
    while let Ok(data) = channels.attitude.try_recv() {
        sensors.attitude.push(data);
    }
}

fn update_throttle_system(channels: ResMut<Channels>, mut throttle: ResMut<ThrottleControl>) {
    // let throttle = Throttle::default();

    while let Some(throttle) = throttle.dequeue() {
        if let Err(e) = channels.throttle.try_send(throttle) {
            log::error!("{}", e);
        }
    }

}

fn update_frame_orientation(sensors: Res<Sensors>, mut query: Query<&mut Transform, With<FrameBody>>) {
    if let Some(Attitude { pitch, roll, yaw }) = sensors.attitude.iter().last() {
        for mut transform in &mut query {
            // Create a new rotation quat
            // Note: In the 3D environment Y is "Up".
            let mx = Mat4::from_rotation_x(*pitch);
            let my = Mat4::from_rotation_y(*yaw);
            let mz = Mat4::from_rotation_z(*roll);

            let mr = mx * my * mz;
            transform.rotation = Quat::from_mat4(&mr);
        }
    }
}

/// Setup 3D shapes for visualization
fn setup_3d_shapes(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut mats: ResMut<Assets<StandardMaterial>>) {
    let material = mats.add(StandardMaterial {
        base_color: Color::rgb(0.7, 0.0, 0.0),
        ..Default::default()
    });

    let body_mesh = meshes.add(shape::Cube::default().into());

    commands.spawn((
        PbrBundle {
            mesh: body_mesh.clone(),
            material: material,
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        },
        FrameBody
    ));
}

/// Build async runtime to run bluetooth client
fn icarus_async_runtime(attitude_tx: Sender<Attitude>, throttle_rx: Receiver<Throttle>) -> anyhow::Result<()> {
    // Spawn an async runtime to collect sensor and state data
    let runtime = Builder::new_current_thread().enable_all().build()?;
    runtime.block_on(run_icarus_client(attitude_tx, throttle_rx))?;

    Ok(())
}

/// Send and receive data from the icarus controller
async fn run_icarus_client(attitude_tx: Sender<Attitude>, throttle_rx: Receiver<Throttle>) -> anyhow::Result<()> {
    let (attitude_rx, throttle_tx) = icarus_client::initialize().await?.split();

    // let sensor_rx_task = tokio::spawn(async {
    //     while let Some(attitude) = attitude.recv().await {
    //         if let Err(e) = attitude_tx.send(attitude).await {
    //             log::error!("Error sending data: {}", e);
    //         }
    //     }
    // });
    // tokio::spawn(sensor_rx_task);

    // let throttle_task = tokio::spawn(async {
    //     while let Some(cmd) = throttle_rx.recv().await {
    //         if let Err(e) = throttle.send(cmd).await {
    //             log::error!("Error sending throttle command: {}", e);
    //         }
    //     }
    // });

    // join!(sensor_rx_task, throttle_task)

    // tokio::join!(collect_sensor_data_task(attitude, attitude_tx));
    log::error!("starting tasks");
    tokio::select! {
        _ = command_throttle_task(throttle_rx, throttle_tx) => {}
        _ = collect_sensor_data_task(attitude_rx, attitude_tx) => {}
    };

    log::error!("exiting icarus async runtime");

    Ok(())
}

async fn command_throttle_task(mut throttle_rx: Receiver<Throttle>, throttle_tx: Sender<Throttle>) -> anyhow::Result<()> {
    log::error!("throttle task");
    while let Some(throttle) = throttle_rx.recv().await {
        if let Err(e) = throttle_tx.send(throttle).await {
            log::error!("Error sending data: {}", e);
        }
    }

    Ok(())
}

async fn collect_sensor_data_task(mut attitude_rx: Receiver<Attitude>, attitude_tx: Sender<Attitude>) -> anyhow::Result<()> {
    while let Some(attitude) = attitude_rx.recv().await {
        // println!("{:?}", attitude);
        if let Err(e) = attitude_tx.send(attitude).await {
            log::error!("Error sending data: {}", e);
        }
    }

    Ok(())
}
