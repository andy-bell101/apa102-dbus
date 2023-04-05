use std::{thread, time};

use rppal::spi::{Bus, Mode, SlaveSelect, Spi};

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct LEDState {
    brightness: u8,
    blue: u8,
    green: u8,
    red: u8,
    time: f32,
}

fn lerp_single(start: u8, end: u8, factor: f32) -> u8 {
    ((start as f32) * (1f32 - factor)) as u8 + ((end as f32) * factor) as u8
}

impl LEDState {
    pub fn new(brightness: u8, blue: u8, green: u8, red: u8, time: f32) -> Self {
        Self {
            brightness: if brightness > 31 { 31 } else { brightness },
            blue,
            green,
            red,
            time,
        }
    }

    fn lerp(init: &Self, target: &Self, cur_time: f32) -> Self {
        let t: f32 = cur_time / target.time;
        Self {
            brightness: lerp_single(init.brightness, target.brightness, t),
            blue: lerp_single(init.blue, target.blue, t),
            green: lerp_single(init.green, target.green, t),
            red: lerp_single(init.red, target.red, t),
            time: cur_time,
        }
    }

    #[cfg(test)]
    fn almost_equal(s1: &Self, s2: &Self) -> bool {
        let Self {
            brightness: br1,
            blue: b1,
            green: g1,
            red: r1,
            time: t1,
        } = s1;
        let Self {
            brightness: br2,
            blue: b2,
            green: g2,
            red: r2,
            time: t2,
        } = s2;
        fn close(x1: &u8, x2: &u8) -> bool {
            if x1 > x2 {
                (x1 - x2) <= 1
            } else {
                (x2 - x1) <= 1
            }
        }
        fn float_close(f1: &f32, f2: &f32, tolerance: f32) -> bool {
            (f1 - f2).abs() <= tolerance
        }
        close(br1, br2)
            && close(b1, b2)
            && close(g1, g2)
            && close(r1, r2)
            && float_close(t1, t2, 0.0001)
    }
}

pub struct Frames {
    state: LEDState,
    buffer: Vec<u8>,
    num_leds: u16,
    clock_rate: u32,
    sleep_duration: time::Duration,
}

impl Frames {
    pub fn new(num_leds: u16, clock_rate: u32, sleep_duration_millis: u64) -> Self {
        Self {
            state: LEDState::new(0, 0, 0, 0, 0.0),
            buffer: Self::initialise_frames(&num_leds),
            num_leds,
            clock_rate,
            sleep_duration: time::Duration::from_millis(sleep_duration_millis),
        }
    }

    pub fn update_current_led_state(&mut self, state: LEDState) {
        self.state = state;
    }

    fn get_start_frame() -> [u8; 4] {
        [0; 4]
    }

    fn get_led_frame(led_state: &LEDState) -> [u8; 4] {
        let LEDState {
            brightness,
            blue,
            green,
            red,
            ..
        } = *led_state;
        // ignore any brightness values that are too high
        let first_bits: u8 = 0b1110_0000;
        [first_bits | brightness, blue, green, red]
    }

    fn get_end_frames(num_leds: &u16) -> Vec<u8> {
        // Note: according to https://cpldcpu.wordpress.com/2014/11/30/understanding-the-apa102-superled/
        // the end frame needs to consist of at least n/2 bits of 1, where n
        // in the number of LEDs in the strip.
        //
        // Using u32::MAX means we can only address a 64 LED strip
        vec![0; Self::get_end_frame_count(num_leds).into()]
    }

    pub fn set_led_frames(&mut self, led_state: &LEDState) {
        for i in 0..(self.num_leds as usize) {
            let leds = Self::get_led_frame(led_state);
            let index = (i + 1) * 4;
            for (j, led) in leds.iter().enumerate() {
                self.buffer[index + j] = *led;
            }
        }
    }

    fn get_end_frame_count(num_leds: &u16) -> u16 {
        ((num_leds / 64) + 1) * 4
    }

    pub fn transition(&mut self, target: &LEDState) -> Result<(), rppal::spi::Error> {
        let start_time = time::Instant::now();
        while start_time.elapsed().as_secs_f32() < target.time {
            let delta_time: f32 = start_time.elapsed().as_secs_f32();
            self.set_led_frames(&LEDState::lerp(&self.state, target, delta_time));
            self.output_frames()?;
            thread::sleep(self.sleep_duration);
        }
        // make sure we actually achieved the final state, in case of rounding
        // errors in the lerp
        self.set_led_frames(target);
        self.state = *target;
        self.output_frames()
    }

    fn initialise_frames(num_leds: &u16) -> Vec<u8> {
        let mut frames: Vec<u8> = vec![];
        let start_frames = Self::get_start_frame();
        frames.extend(start_frames);
        frames.extend(vec![0; (num_leds * 4).into()]);
        frames.extend(Self::get_end_frames(num_leds));
        frames
    }

    pub fn output_frames(&self) -> Result<(), rppal::spi::Error> {
        let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, self.clock_rate, Mode::Mode0).unwrap();
        spi.write(&self.buffer)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_maximum_brightness_white() {
        assert_eq!(
            Frames::get_led_frame(&LEDState {
                brightness: 255,
                blue: 255,
                green: 255,
                red: 255,
                time: 0.0,
            }),
            [0xff, 0xff, 0xff, 0xff]
        );
    }

    #[test]
    fn test_zero_brightness_black() {
        assert_eq!(
            Frames::get_led_frame(&LEDState {
                brightness: 0,
                blue: 0,
                green: 0,
                red: 0,
                time: 0.0,
            }),
            [0xe0, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn test_max_brightness_blue() {
        assert_eq!(
            Frames::get_led_frame(&LEDState {
                brightness: 255,
                blue: 255,
                green: 0,
                red: 0,
                time: 0.0
            }),
            [0xff, 0xff, 0x00, 0x00]
        );
    }

    #[test]
    fn test_max_brightness_green() {
        assert_eq!(
            Frames::get_led_frame(&LEDState {
                brightness: 255,
                blue: 0,
                green: 255,
                red: 0,
                time: 0.0
            }),
            [0xff, 0x00, 0xff, 0x00]
        );
    }

    #[test]
    fn test_max_brightness_red() {
        assert_eq!(
            Frames::get_led_frame(&LEDState {
                brightness: 255,
                blue: 0,
                green: 0,
                red: 255,
                time: 0.0
            }),
            [0xff, 0x00, 0x00, 0xff]
        );
    }

    #[test]
    fn test_vector_initialised_correctly() {
        fn checker(mut frames: Vec<u8>, expected_len: usize, expected_0xff_count: usize) {
            assert_eq!(frames.len(), expected_len);
            frames.reverse();
            for (i, v) in frames.iter().enumerate() {
                if i < expected_0xff_count {
                    assert_eq!(*v, 0xff);
                } else {
                    assert_eq!(*v, 0);
                }
            }
        }
        checker(Frames::initialise_frames(&1), 12, 4);
        checker(Frames::initialise_frames(&2), 16, 4);
        checker(Frames::initialise_frames(&64), (1 + 64 + 2) * 4, 8);
    }

    #[test]
    fn test_lerp_blue_to_green() {
        let init = LEDState {
            brightness: 255,
            blue: 255,
            green: 0,
            red: 0,
            time: 0.0,
        };
        let target = LEDState {
            brightness: 255,
            blue: 0,
            green: 255,
            red: 0,
            time: 10.0,
        };

        let state_2_point_5 = LEDState {
            brightness: 255,
            blue: 191,
            green: 63,
            red: 0,
            time: 2.5,
        };
        let state_5_point_0 = LEDState {
            brightness: 255,
            blue: 127,
            green: 127,
            red: 0,
            time: 5.0,
        };
        let state_7_point_5 = LEDState {
            brightness: 255,
            blue: 63,
            green: 191,
            red: 0,
            time: 7.5,
        };
        assert!(LEDState::almost_equal(
            &LEDState::lerp(&init, &target, 2.5),
            &state_2_point_5
        ));
        assert!(LEDState::almost_equal(
            &LEDState::lerp(&init, &target, 5.0),
            &state_5_point_0
        ));
        assert!(LEDState::almost_equal(
            &LEDState::lerp(&init, &target, 7.5),
            &state_7_point_5
        ));
    }

    const TESTING_NUM_LEDS: u16 = 60;

    #[test]
    fn test_red_output_for_2_seconds() {
        let mut frames = Frames::new(TESTING_NUM_LEDS, 15_000_000, 5);
        let target: LEDState = LEDState::new(255, 0, 0, 255, 0.1);
        let result = frames.transition(&target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_green_output_for_2_seconds() {
        let mut frames = Frames::new(TESTING_NUM_LEDS, 15_000_000, 5);
        let target: LEDState = LEDState::new(255, 0, 255, 0, 0.1);
        let result = frames.transition(&target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_blue_output_for_2_seconds() {
        let mut frames = Frames::new(TESTING_NUM_LEDS, 15_000_000, 5);
        let target: LEDState = LEDState::new(255, 255, 0, 0, 0.1);
        let result = frames.transition(&target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clear_leds_post_testing() {
        let mut frames = Frames::new(TESTING_NUM_LEDS, 15_000_000, 5);
        let target: LEDState = LEDState::new(0, 0, 0, 0, 0.1);
        let result = frames.transition(&target);
        assert!(result.is_ok());
    }

    #[test]
    fn test_rgb_roundtrip() {
        let mut frames = Frames::new(TESTING_NUM_LEDS, 15_000_000, 5);
        let red = LEDState::new(255, 0, 0, 255, 1.0);
        let green = LEDState::new(255, 0, 255, 0, 1.0);
        let blue = LEDState::new(255, 255, 0, 0, 1.0);
        let clear = LEDState::new(0, 0, 0, 0, 1.0);
        assert!(frames.transition(&red).is_ok());
        assert!(frames.transition(&green).is_ok());
        assert!(frames.transition(&blue).is_ok());
        assert!(frames.transition(&clear).is_ok());
    }
}
