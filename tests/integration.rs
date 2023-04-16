use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use apa102_dbus::{frames, worker};

#[test]
fn test_round_trip_via_threads() {
    let num_leds = 5;
    let clock_rate = 1_500_000;
    let (job_tx, job_rx) = mpsc::channel();
    let (interrupt_tx, interrupt_rx) = mpsc::channel();
    #[allow(unused_must_use)]
    thread::spawn(move || {
        worker::update_leds(
            &mut frames::Frames::new(num_leds, clock_rate, 5),
            job_rx,
            interrupt_rx,
        );
    });

    let red = frames::LEDState::new(255, 0, 0, 255, 1.0);
    let green = frames::LEDState::new(255, 0, 255, 0, 1.0);
    let blue = frames::LEDState::new(255, 255, 0, 0, 1.0);
    let clear = frames::LEDState::new(0, 0, 0, 0, 1.0);

    assert!(job_tx
        .send(worker::Job::OneOff(vec![red, green, blue, clear]))
        .is_ok());
    thread::sleep(Duration::from_secs(5));

    assert!(job_tx
        .send(worker::Job::Repeat(vec![red, green, blue, clear]))
        .is_ok());
    thread::sleep(Duration::from_secs(10));

    let purple = frames::LEDState::new(255, 255, 0, 255, 1.0);
    assert!(job_tx.send(worker::Job::OneOff(vec![purple])).is_ok());
    assert!(interrupt_tx.send(true).is_ok());
    thread::sleep(Duration::from_secs(3));

    assert!(job_tx.send(worker::Job::OneOff(vec![clear])).is_ok());
    thread::sleep(Duration::from_secs(2));
}
