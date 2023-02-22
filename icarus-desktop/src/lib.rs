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

#[derive(Component)]
struct FrameBody;

#[derive(Component)]
struct Orientation;

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
        let attitude_sensor: SensorBuffer<Attitude> = SensorBuffer::new(250);
        let sensors = Sensors {attitude: attitude_sensor};

        // Add the data channels to bevy's resource manager
        app
            .insert_resource(channels)
            .insert_resource(sensors)
            .add_startup_system(setup_3d_shapes)
            .add_system(update_sensors_system)
            .add_system(update_frame_orientation);
    }
}

fn update_sensors_system(mut channels: ResMut<Channels>, mut sensors: ResMut<Sensors>) {
    while let Ok(data) = channels.attitude.try_recv() {
        sensors.attitude.push(data);
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
