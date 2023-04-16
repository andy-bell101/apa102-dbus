extern crate xdg;
use std::error::Error;
use std::fs;
use std::future::pending;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

use clap::Parser;
use serde::Deserialize;
use zbus::ConnectionBuilder;

use crate::frames::Frames;

mod frames;
mod interface;
mod worker;

const DEFAULT_NUM_LEDS: Option<u16> = Some(1);
const DEFAULT_CLOCK_RATE: Option<u32> = Some(15_000_000);
const DEFAULT_SLEEP_DURATION: Option<u64> = Some(5);

#[derive(Parser, Debug, Deserialize)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of LEDs in the strip
    #[arg(short, long)]
    num_leds: Option<u16>,
    #[arg(short, long)]
    /// Clock rate to use
    clock_rate: Option<u32>,
    #[arg(short, long)]
    /// Sleep duration between updates in milliseconds
    sleep_duration: Option<u64>,
}

fn get_args_from_config_file() -> Option<Args> {
    let base_dir = xdg::BaseDirectories::new().ok()?;
    let file = base_dir.find_config_file("config.toml")?;
    let contents = fs::read_to_string(file).ok()?;
    toml::from_str(&contents).ok()
}

fn work_out_single_arg<'a, T: std::fmt::Display>(
    cli: &'a Option<T>,
    config: &'a Option<T>,
    default: &'a Option<T>,
    name: &'a str,
) -> Option<&'a T> {
    if let Some(x) = cli {
        println!("{} taken from CLI with value {}", name, x);
        Some(x)
    } else if let Some(x) = config {
        println!("{} taken from config file with value {}", name, x);
        Some(x)
    } else if let Some(x) = default {
        println!("{} taken from defaults with value {}", name, x);
        Some(x)
    } else {
        None
    }
}

fn work_out_args() -> (u16, u32, u64) {
    let cli = Args::parse();
    let config = match get_args_from_config_file() {
        Some(x) => x,
        None => Args { num_leds: None, clock_rate: None, sleep_duration: None },
    };
    let default = Args {
        num_leds: DEFAULT_NUM_LEDS,
        clock_rate: DEFAULT_CLOCK_RATE,
        sleep_duration: DEFAULT_SLEEP_DURATION,
    };
    let num_leds = work_out_single_arg(
        &cli.num_leds,
        &config.num_leds,
        &default.num_leds,
        "Number of LEDs",
    );
    let clock_rate = work_out_single_arg(
        &cli.clock_rate,
        &config.clock_rate,
        &default.clock_rate,
        "Clock rate",
    );
    let sleep_duration = work_out_single_arg(
        &cli.sleep_duration,
        &config.sleep_duration,
        &default.sleep_duration,
        "Sleep duration",
    );
    // ok to unwrap here since the defaults will at least always be Some
    (
        *num_leds.unwrap(),
        *clock_rate.unwrap(),
        *sleep_duration.unwrap(),
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (num_leds, clock_rate, sleep_duration) = work_out_args();
    let (job_tx, job_rx) = mpsc::channel();
    let (interrupt_tx, interrupt_rx) = mpsc::channel();
    #[allow(unused_must_use)]
    thread::spawn(move || {
        worker::update_leds(
            &mut Frames::new(num_leds, clock_rate, sleep_duration),
            job_rx,
            interrupt_rx,
        );
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
