//
// main.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 12 2023
//

use icarus_client::Attitude;
use tokio::sync::mpsc::Receiver;

#[tokio::main(flavor="current_thread")]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    log::info!("Initializing icarus client");
    let client = icarus_client::initialize().await?;
    let (attitude, _) = client.split();

    tokio::spawn(print_sensors_task(attitude));

    tokio::signal::ctrl_c().await?;
    log::info!("Exiting");

    Ok(())
}

async fn print_sensors_task(mut rx: Receiver<Attitude>) -> anyhow::Result<()> {
    while let Some(attitude) = rx.recv().await {
        log::info!("Pitch: {:.5}, Roll: {:.5}, Yaw: {:.5}", attitude.pitch, attitude.roll, attitude.yaw);
    }

    Ok(())
}
