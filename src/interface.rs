use std::sync::mpsc;
use std::sync::Mutex;

use crate::frames::{Frames, LEDState};
use crate::worker;
use zbus::dbus_interface;

pub struct RustApa102 {
    pub frames: Frames,
    pub job_tx: Mutex<mpsc::Sender<worker::Job>>,
    pub interrupt_tx: Mutex<mpsc::Sender<bool>>,
}

#[dbus_interface(name = "org.zbus.apa102-dbus")]
impl RustApa102 {
    fn transition(&mut self, leds: Vec<LEDState>, repeat: bool) -> Result<(), zbus::fdo::Error> {
        let job = if repeat {
            worker::Job::Repeat(leds)
        } else {
            worker::Job::OneOff(leds)
        };
        self.interrupt_tx
            .lock()
            .unwrap()
            .send(true)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        self.job_tx
            .lock()
            .unwrap()
            .send(job)
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }
}
