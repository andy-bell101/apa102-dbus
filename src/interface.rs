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

#[dbus_interface(name = "org.zbus.apa102")]
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

    fn flash(&mut self, led: LEDState) -> Result<(), zbus::fdo::Error> {
        self.transition(vec![led], false)
    }

    fn pulse(&mut self, led: LEDState) -> Result<(), zbus::fdo::Error> {
        self.transition(vec![led, LEDState::new(0, 0, 0, 0, led.time)], true)
    }

    fn clear(&mut self) -> Result<(), zbus::fdo::Error> {
        self.transition(vec![LEDState::new(0, 0, 0, 0, 1.0)], false)
    }

    fn rainbow(&mut self, time: f32, repeat: bool) -> Result<(), zbus::fdo::Error> {
        let v = vec![
            LEDState::new(255, 0xff, 0x00, 0x00, time), // red
            LEDState::new(255, 0xff, 0xa5, 0x00, time), // orange
            LEDState::new(255, 0xff, 0xff, 0x00, time), // yellow
            LEDState::new(255, 0x00, 0x80, 0x00, time), // green
            LEDState::new(255, 0x00, 0x00, 0xff, time), // blue
            LEDState::new(255, 0x4b, 0x00, 0x82, time), // indigo
            LEDState::new(255, 0xee, 0x82, 0xee, time), // violet
        ];
        self.transition(v, repeat)
    }

    // TODO: flash_hex, pulse_hex, rainbow, transition_hex
}
