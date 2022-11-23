//!
//! cargo build --bin rotary-encoder-by-loop
//! avr-objcopy  -j .text -j .data -O ihex target\avr-atmega328p\debug\rotary-encoder-by-loop.elf rotary-encoder-by-loop.hex

#![no_std]
#![no_main]

/// rotary pins should be pull-up input, on other word, With no external circuit pulling
/// the pin low, it will be read high.
use arduino_hal::{
    port::{
        mode::{Input, PullUp},
        Pin,
    },
    prelude::*,
};
use embedded_rotary_encoder::{LatchMode, RotaryEncoder};

use panic_halt as _;

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 115200);
    ufmt::uwriteln!(
        &mut serial,
        "Hello from Arduino! simulate rotary encoder:\r"
    )
    .void_unwrap();

    //   //初始化编码器IO  pin_dt
    let pin_clk: Pin<Input<PullUp>> = pins.d7.into_pull_up_input().downgrade();
    let pin_dt = pins.d8.into_pull_up_input().downgrade();

    let mut rotary = RotaryEncoder::new(pin_clk, pin_dt, LatchMode::FOUR3);

    let mut previous = rotary.get_position();
    let mut cur: i32;

    loop {
        rotary.check_state();
        cur = rotary.get_position();
        let dir = rotary.get_direction();
        if previous != cur {
            ufmt::uwriteln!(&mut serial, "pos:{},dir:{}\r", cur, dir as i32).void_unwrap();
            previous = cur.clone();
        }
    }
}
