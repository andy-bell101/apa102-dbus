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

    fn transition_hex(&mut self, leds: Vec<(&str, f32)>, repeat: bool) -> Result<(), Error> {
        let mapped = leds
            .iter()
            .map(|(s, t)| LEDState::from_hex(s, *t))
            .collect::<Result<Vec<LEDState>, _>>()
            .map_err(|e| Error::Failed(e.to_string()))?;
        self.transition(mapped, repeat)
    }

    fn flash(&mut self, led: LEDState) -> Result<(), Error> {
        self.transition(vec![led], false)
    }

    fn flash_hex(&mut self, hex: &str, time: f32) -> Result<(), Error> {
        let led = LEDState::from_hex(hex, time).map_err(|e| Error::Failed(e.to_string()))?;
        self.transition(vec![led], false)
    }

    fn pulse(&mut self, led: LEDState) -> Result<(), Error> {
        self.transition(vec![led, LEDState::new(0, 0, 0, 0, led.time)], true)
    }

    fn pulse_hex(&mut self, hex: &str, time: f32) -> Result<(), Error> {
        let led = LEDState::from_hex(hex, time).map_err(|e| Error::Failed(e.to_string()))?;
        self.transition(vec![led], true)
    }

    fn clear(&mut self) -> Result<(), Error> {
        self.transition(vec![LEDState::new(0, 0, 0, 0, 1.0)], false)
    }

    fn rainbow(&mut self, time: f32, repeat: bool) -> Result<(), Error> {
        let v = vec![
            LEDState::from_hex("0xffff0000", time).unwrap(), // red
            LEDState::from_hex("0xffffa500", time).unwrap(), // orange
            LEDState::from_hex("0xffffff00", time).unwrap(), // yellow
            LEDState::from_hex("0xff008000", time).unwrap(), // green
            LEDState::from_hex("0xff0000ff", time).unwrap(), // blue
            LEDState::from_hex("0xff4b0082", time).unwrap(), // indigo
            LEDState::from_hex("0xffee82ee", time).unwrap(), // violet
        ];
        self.transition(v, repeat)
    }
}
