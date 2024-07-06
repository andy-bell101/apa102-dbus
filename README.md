# apa102-dbus

A Rust utility to allow you to control an APA102 LED strip via DBUS on a
Raspberry PI

## Installation

1. Add your user on the Raspberry Pi to the `spi` group.
1. Download the rust toolchain using `rustup`.
1. Add the cargo binary directory to your `$PATH`. It's usually located at
   `~/.cargo/bin`.
1. Clone the repo to your Raspberry Pi.
1. In the repository folder, run `cargo install --path .` to install the
   program to your cargo bin directory.

## Starting and running

`apa102-dbus` requires the following data to work correctly:

- The number of LEDs in your strip.
- The clock rate to use when outputting data to the strip.
- The sleep duration between updates of the strip in milliseconds.

You can pass this information when you invoke `apa102-dbus` on the command
line. Use `apa102-dbus -h` for the exact syntax required. Alternatively, you
can set the options in the `~/.config/apa102-dbus/config.toml` file like this:

```toml
num_leds = 1
clock_rate = 15000000
sleep_duration = 5
```

Any arguments not set from the command line or in the `config.toml` use the
default values shown above.

It's recommended that you create a `systemd` service to run the program. You
can specify the command line arguments there but it's better to rely on the
`config.toml` instead. To add a new `systemd` service file, use the command
`systemctl --user edit --force --full apa102-dbus.service`. An example file is
given below

```systemd
[Unit]
Description=APA102 DBUS

[Service]
Type=simple
ExecStart=/home/pi/.cargo/bin/apa102-dbus
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
```

## Methods

`apa102-dbus` implements the following methods that you can call via DBUS.

- Clear: switches the LED strip off.
- Set: Set the LED strip to the given state. Argument order is brightness, red,
  green, blue, transition time in seconds.
- SetHex: Set the LED strip to the given state using a hex colour code.
  Argument order is colour hex (as a string), brightness, transition time in
  seconds.
- Flash: Flash the LED strip to the given state then back to off. Does not
  repeat. Argument order is brightness, red, green, blue, transition time in
  seconds.
- FlashHex: As above, but provide the colours as a hexcode instead.
- Pulse: Set the LED strip to the given state then back to off repeatedly.
  Argument order is brightness, red, green, blue, transition time in seconds.
- PulseHex: As above, but provide colours as a hexcode instead.
- Transition: Provide an array of states to transition through. Argument order
  is the array of brightness, red, green, blue and transition time in seconds,
  then whether or not to repeat the sequence.
- TransitionHex: As above, but provide colours as a hexcode instead.
