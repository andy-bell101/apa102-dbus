use std::{thread, time};

use bitvec::prelude::*;
use rppal::gpio::{Gpio, Level, OutputPin};

#[derive(Debug, PartialEq)]
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
            brightness,
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
    buffer: Vec<u32>,
    num_leds: u16,
    data_pin: u8,
    clock_pin: u8,
    clock_rate: f32,
}

impl Frames {
    pub fn new(num_leds: u16, data_pin: u8, clock_pin: u8, clock_rate: f32) -> Self {
        Self {
            state: LEDState::new(0, 0, 0, 0, 0.0),
            buffer: Self::initialise_frames(&num_leds),
            num_leds,
            data_pin,
            clock_pin,
            clock_rate,
        }
    }

    pub fn update_current_led_state(&mut self, state: LEDState) {
        self.state = state;
    }

    fn get_start_frame() -> u32 {
        0
    }

    fn get_led_frame(led_state: &LEDState) -> u32 {
        let LEDState {
            brightness,
            blue,
            green,
            red,
            ..
        } = *led_state;
        // ignore any brightness values that are too high
        let first_bits: u8 = 0b1110_0000;
        let mod_brightness: u32 = ((first_bits | brightness) as u32) << 24;
        mod_brightness | ((blue as u32) << 16) | ((green as u32) << 8) | (red as u32)
    }

    fn get_end_frame() -> u32 {
        // Note: according to https://cpldcpu.wordpress.com/2014/11/30/understanding-the-apa102-superled/
        // the end frame needs to consist of at least n/2 bits of 1, where n
        // in the number of LEDs in the strip.
        //
        // Using u32::MAX means we can only address a 64 LED strip
        u32::MAX
    }

    pub fn set_led_frames(&mut self, led_state: &LEDState) {
        for i in 0..self.num_leds {
            self.buffer[(i + 1) as usize] = Self::get_led_frame(led_state)
        }
    }

    fn get_end_frame_count(num_leds: &u16) -> u16 {
        (num_leds / 64) + 1
    }

    fn get_required_vector_length(num_leds: &u16) -> u16 {
        1 + num_leds + Self::get_end_frame_count(num_leds)
    }

    pub fn transition(&mut self, target: &LEDState) -> Result<(), rppal::gpio::Error> {
        let start_time = time::Instant::now();
        while start_time.elapsed().as_secs_f32() < target.time {
            let delta_time: f32 = start_time.elapsed().as_secs_f32();
            self.set_led_frames(&LEDState::lerp(&self.state, target, delta_time));
            self.output_frames()?;
        }
        // make sure we actually achieved the final state, in case of rounding
        // errors in the lerp
        self.set_led_frames(target);
        self.output_frames()
    }

    fn initialise_frames(num_leds: &u16) -> Vec<u32> {
        let length = Self::get_required_vector_length(num_leds) as usize;
        let mut frames = vec![0; length];
        frames[0] = Self::get_start_frame();
        for frame in frames.iter_mut().skip(1 + (*num_leds as usize)) {
            *frame = Self::get_end_frame();
        }
        frames
    }

    fn write_to_pin(
        &self,
        data_pin: &mut OutputPin,
        clock_pin: &mut OutputPin,
        num: &BitSlice<u32, Msb0>,
    ) {
        for b in num {
            data_pin.write(Level::from(*b));
            thread::sleep(time::Duration::from_secs_f32(self.clock_rate));
            clock_pin.toggle();
        }
    }

    pub fn output_frames(&self) -> Result<(), rppal::gpio::Error> {
        let gpio = Gpio::new()?;
        let mut data_pin = gpio.get(self.data_pin)?.into_output();
        let mut clock_pin = gpio.get(self.clock_pin)?.into_output();
        self.write_to_pin(&mut data_pin, &mut clock_pin, self.buffer.view_bits());
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
            0xffffffff
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
            0xe0000000
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
            0xffff0000
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
            0xff00ff00
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
            0xff0000ff
        );
    }

    #[test]
    fn test_check_expected_vector_length() {
        assert_eq!(Frames::get_required_vector_length(&1), 3);
        assert_eq!(Frames::get_required_vector_length(&63), 1 + 63 + 1);
        // NOTE: not sure if 64 should have 2 end frames or 1, but having 2 is safer for now. will have
        // to test after wiring up
        assert_eq!(Frames::get_required_vector_length(&64), 1 + 64 + 2);
        assert_eq!(Frames::get_required_vector_length(&65), 1 + 65 + 2);
    }

    #[test]
    fn test_vector_initialised_correctly() {
        assert_eq!(Frames::initialise_frames(&1), vec![0, 0, 0xffffffff]);
        assert_eq!(Frames::initialise_frames(&2), vec![0, 0, 0, 0xffffffff]);
        assert_eq!(
            Frames::initialise_frames(&63),
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0xffffffff
            ]
        );
        assert_eq!(
            Frames::initialise_frames(&64),
            // NOTE: not sure if 64 should have 2 end frames or 1, but having 2 is safer for now. will have
            // to test after wiring up
            vec![
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0xffffffff, 0xffffffff
            ]
        );
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
}
