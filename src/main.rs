use std::error::Error;
use std::future::pending;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;

use zbus::ConnectionBuilder;

use crate::frames::Frames;

mod frames;
mod interface;
mod worker;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let num_leds = 16;
    let clock_rate = 1_500_000;
    let (job_tx, job_rx) = mpsc::channel();
    let (interrupt_tx, interrupt_rx) = mpsc::channel();
    #[allow(unused_must_use)]
    thread::spawn(move || {
        worker::update_leds(&mut Frames::new(num_leds, clock_rate, 5), job_rx, interrupt_rx);
    });
    let inst = interface::RustApa102 {
        frames: Frames::new(num_leds, clock_rate, 5),
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
