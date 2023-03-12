//
// main.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Feb 12 2023
//

use icarus_client::Attitude;
use tokio::sync::mpsc::Receiver;

use clap::{Parser, Subcommand};

use uuid::Uuid;

#[derive(Debug, Subcommand)]
#[command(author, version, about, long_about = None)]
enum Commands {
    /// Print IMU data to the console
    PrintImu,
    /// List service UUIDs
    ListServices,
}

#[derive(Debug, Parser)]
struct Args {
    #[command(subcommand)]
    cmd: Commands,
}

#[tokio::main(flavor="current_thread")]
async fn main() -> anyhow::Result<()> {
    pretty_env_logger::init();

    let args = Args::parse();

    log::info!("Initializing icarus client");
    let client = icarus_client::initialize().await?;

    let services = client.services.clone();

    // let services = client.services.clone();
    let (attitude, _) = client.split();

    let task = match args.cmd {
        Commands::PrintImu => tokio::spawn(print_sensors_task(attitude)),
        Commands::ListServices => tokio::spawn(list_services(services)),
    };

    tokio::select! {
        _ = task => {},
        _ = tokio::signal::ctrl_c() => {}
    };

    Ok(())
}

async fn print_sensors_task(mut rx: Receiver<Attitude>) -> anyhow::Result<()> {
    while let Some(attitude) = rx.recv().await {
        log::info!("Pitch: {:.5}, Roll: {:.5}, Yaw: {:.5}", attitude.pitch, attitude.roll, attitude.yaw);
    }

    Ok(())
}

async fn list_services(services: Vec<Uuid>) -> anyhow::Result<()> {
    // NOTE: Not really an async task, just need it to fit in the select! macro.
    //       Need to refactor the client initialization code a bit.

    for uuid in services {
        println!("{}", uuid);
    }

    Ok(())
}
