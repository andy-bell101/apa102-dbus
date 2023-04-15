use std::error::Error;
use std::future::pending;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

use clap::Parser;
use zbus::ConnectionBuilder;

use crate::frames::Frames;

mod frames;
mod interface;
mod worker;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of LEDs in the strip
    #[arg(short, long)]
    num_leds: u16,
    #[arg(short, long)]
    /// Clock rate to use
    clock_rate: u32,
    #[arg(short, long)]
    /// Sleep duration between updates in milliseconds
    sleep_duration: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let Args {num_leds, clock_rate, sleep_duration} = Args::parse();
    let (job_tx, job_rx) = mpsc::channel();
    let (interrupt_tx, interrupt_rx) = mpsc::channel();
    #[allow(unused_must_use)]
    thread::spawn(move || {
        worker::update_leds(&mut Frames::new(num_leds, clock_rate, sleep_duration), job_rx, interrupt_rx);
    });
    let inst = interface::RustApa102 {
        frames: Frames::new(num_leds, clock_rate, sleep_duration),
        job_tx: Mutex::new(job_tx),
        interrupt_tx: Mutex::new(interrupt_tx),
    };
    let _conn = ConnectionBuilder::session()?
        .name("org.zbus.rust_apa102")?
        .serve_at("/org/zbus/rust_apa102", inst)?
        .build()
        .await?;

    // Wait forever
    pending::<()>().await;

    Ok(())
}
