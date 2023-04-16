use std::sync::mpsc;
use std::sync::Mutex;

use crate::frames::{Frames, LEDState};
use crate::worker;
use zbus::dbus_interface;
use zbus::fdo::Error;

pub struct RustApa102 {
    pub frames: Frames,
    pub job_tx: Mutex<mpsc::Sender<worker::Job>>,
    pub interrupt_tx: Mutex<mpsc::Sender<bool>>,
}

#[dbus_interface(name = "org.zbus.apa102")]
impl RustApa102 {
    fn transition(&mut self, leds: Vec<LEDState>, repeat: bool) -> Result<(), Error> {
        let job = if repeat {
            worker::Job::Repeat(leds)
        } else {
            worker::Job::OneOff(leds)
        };
        self.interrupt_tx
            .lock()
            .unwrap()
            .send(true)
            .map_err(|e| Error::Failed(e.to_string()))?;
        self.job_tx
            .lock()
            .unwrap()
            .send(job)
            .map_err(|e| Error::Failed(e.to_string()))?;
        Ok(())
    }

    fn transition_hex(&mut self, leds: Vec<(&str, u8, f32)>, repeat: bool) -> Result<(), Error> {
        let mapped = leds
            .iter()
            .map(|(s, b, t)| LEDState::from_hex(s, *b, *t))
            .collect::<Result<Vec<LEDState>, _>>()
            .map_err(|e| Error::Failed(e.to_string()))?;
        self.transition(mapped, repeat)
    }

    fn flash(&mut self, led: LEDState) -> Result<(), Error> {
        self.transition(vec![led], false)
    }

    fn flash_hex(&mut self, hex: &str, brightness: u8, time: f32) -> Result<(), Error> {
        let led = LEDState::from_hex(hex, brightness, time).map_err(|e| Error::Failed(e.to_string()))?;
        self.transition(vec![led], false)
    }

    fn pulse(&mut self, led: LEDState) -> Result<(), Error> {
        self.transition(vec![led, LEDState::new(0, 0, 0, 0, led.time)], true)
    }

    fn pulse_hex(&mut self, hex: &str, brightness: u8,time: f32) -> Result<(), Error> {
        let led = LEDState::from_hex(hex, brightness, time).map_err(|e| Error::Failed(e.to_string()))?;
        self.transition(vec![led], true)
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.transition(vec![LEDState::new(0, 0, 0, 0, 1.0)], false)
    }

    fn rainbow(&mut self, brightness: u8, time: f32, repeat: bool) -> Result<(), Error> {
        let v = vec![
            LEDState::from_hex("ff0000", brightness, time).unwrap(), // red
            LEDState::from_hex("ffa500", brightness, time).unwrap(), // orange
            LEDState::from_hex("ffff00", brightness, time).unwrap(), // yellow
            LEDState::from_hex("008000", brightness, time).unwrap(), // green
            LEDState::from_hex("0000ff", brightness, time).unwrap(), // blue
            LEDState::from_hex("4b0082", brightness, time).unwrap(), // indigo
            LEDState::from_hex("ee82ee", brightness, time).unwrap(), // violet
        ];
        self.transition(v, repeat)
    }
}
