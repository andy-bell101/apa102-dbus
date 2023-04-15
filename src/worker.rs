use crate::frames::{Frames, Interrupted, LEDState};
use std::sync::mpsc::Receiver;

#[derive(Clone)]
pub enum Job {
    OneOff(Vec<LEDState>),
    Repeat(Vec<LEDState>),
}

pub fn update_leds(
    frames: &mut Frames,
    job_rx: Receiver<Job>,
    interrupt_rx: Receiver<bool>,
) -> Result<(), rppal::spi::Error> {
    loop {
        for job in job_rx.try_iter() {
            match job {
                Job::OneOff(v) => {
                    for target in v {
                        match frames.transition(&target, &interrupt_rx) {
                            Interrupted::Yes => break,
                            Interrupted::No(x) => x?,
                        };
                    }
                }
                Job::Repeat(v) => loop {
                    let mut breaker: bool = false;
                    for target in &v {
                        match frames.transition(target, &interrupt_rx) {
                            Interrupted::Yes => {
                                breaker = true;
                                break;
                            }
                            Interrupted::No(x) => x?,
                        };
                    }
                    if breaker {
                        break;
                    }
                },
            }
        }
    }
}
