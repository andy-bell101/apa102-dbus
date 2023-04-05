use std::error::Error;
use std::future::pending;

use frames::Frames;
use zbus::{dbus_interface, ConnectionBuilder};

mod frames;

struct RustApa102 {
    frames: Frames,
}

#[dbus_interface(name = "org.zbus.rust_apa102")]
impl RustApa102 {
    fn transition(
        &mut self,
        brightness: u8,
        blue: u8,
        green: u8,
        red: u8,
        time: f32,
    ) -> Result<(), zbus::fdo::Error> {
        let target = frames::LEDState::new(brightness, blue, green, red, time);
        self.frames
            .transition(&target)
            .map_err(|e| zbus::Error::Failure(e.to_string()))?;
        self.frames.update_current_led_state(target);
        Ok(())
    }

    fn set(
        &mut self,
        brightness: u8,
        blue: u8,
        green: u8,
        red: u8,
    ) -> Result<(), zbus::fdo::Error> {
        let target = frames::LEDState::new(brightness, blue, green, red, 0.01);
        self.frames
            .transition(&target)
            .map_err(|e| zbus::Error::Failure(e.to_string()))?;
        self.frames.update_current_led_state(target);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let num_leds = 16;
    let clock_rate = 1_500_000;
    let inst = RustApa102 {
        frames: Frames::new(num_leds, clock_rate, 5),
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
