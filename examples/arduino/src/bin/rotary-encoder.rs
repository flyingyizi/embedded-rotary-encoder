//! cargo build --bin rotary-encoder
//! avr-objcopy  -j .text -j .data -O ihex target\avr-atmega328p\debug\rotary-encoder.elf rotary-encoder.hex

#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::{
    port::{
        mode::{Input, PullUp},
        Pin,
    },
    prelude::*,
};

use avr_device::interrupt::{free, Mutex};
type RotaryPin = Pin<Input<PullUp>>;
use core::cell::RefCell;

use help_for_isr::{interrupt_rotary_current, interrupt_rotary_default_isr, ROTARY_ENCODER};

mod help_for_isr {
    use super::{free, Mutex, RefCell, RotaryDirecton, RotaryEncoder, RotaryPin};
    use core::ops::DerefMut;
    /// helper rotary encoder instance prepared for isr model, it will be initialized in your app.
    /// notes: the pin_clk/pin_dt pins should associated with the interrupts in your app at the same time.
    pub static ROTARY_ENCODER: Mutex<RefCell<Option<RotaryEncoder<RotaryPin>>>> =
        Mutex::new(RefCell::new(None));
    #[inline]
    /// helper default isr, it just only calls [check_state], not do any other things.
    /// if your board need special deal likes clear associated pins interrupt-pending-bit, you should not use it.
    pub fn interrupt_rotary_default_isr() {
        // Retrieve Rotary Encoder from safely stored static global
        free(|cs| {
            if let Some(ref mut rotary_encoder) = ROTARY_ENCODER.borrow(cs).borrow_mut().deref_mut()
            {
                // Borrow the pins to clear the pending interrupt bit (which varies depending on HAL)
                // let mut pins = rotary_encoder.borrow_pins();
                // pins.0.clear_interrupt_pending_bit();
                // pins.1.clear_interrupt_pending_bit();
                // // Update the encoder, which will compute its direction
                rotary_encoder.check_state();
            }
        });
    }

    /// helper for isr model. get the rotary-encoder current positon and dirction.
    pub fn interrupt_rotary_current() -> Option<(i32, RotaryDirecton)> {
        free(|cs| {
            if let Some(ref mut encoder) = ROTARY_ENCODER.borrow(cs).borrow_mut().deref_mut() {
                Some((encoder.get_position(), encoder.get_direction()))
            } else {
                None
            }
        })
    }
}

use embedded_rotary_encoder::{LatchMode, RotaryDirecton, RotaryEncoder};

///Configure pin change interrupt on Pin8
pub fn d7_d8_exint_init(
    pin_clk: Pin<Input<PullUp>>,
    pin_dt: Pin<Input<PullUp>>,
    exint: arduino_hal::hal::pac::EXINT,
) {
    // PCINT0(D8) AND PCINT23(D7)

    // Enable the PCINT0/PCINT23 pin change interrupt
    exint.pcicr.write(|w| unsafe { w.bits(0b101) });
    // Enable pin change interrupts on PCINTX
    exint.pcmsk0.write(|w| unsafe { w.bits(0b001) });
    exint.pcmsk2.write(|w| unsafe { w.bits(0x80) });
    // Clear interrupt flag
    exint.pcifr.write(|w| w.pcif().bits(0b101));

    // Initialize Rotary Encoder and safely store in static global
    let rotary = RotaryEncoder::new(pin_clk, pin_dt, LatchMode::FOUR3);
    free(|cs| ROTARY_ENCODER.borrow(cs).replace(Some(rotary)));
    // Enable interrupts globally, not a replacement for the specific interrupt enable
    unsafe { avr_device::interrupt::enable() };
}

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

    let pin_clk: Pin<Input<PullUp>> = pins.d7.into_pull_up_input().downgrade();
    let pin_dt = pins.d8.into_pull_up_input().downgrade();
    d7_d8_exint_init(pin_clk, pin_dt, dp.EXINT);

    let mut previous = 0;
    let mut cur: i32;
    let mut direction: i32;

    loop {
        if let Some((pos, dir)) = interrupt_rotary_current() {
            (cur, direction) = (pos, dir as i32);
        } else {
            unreachable!()
        }

        if previous != cur {
            ufmt::uwriteln!(&mut serial, "pos:{}, dir:{}\r", cur, direction).void_unwrap();
            previous = cur.clone();
        }

        // arduino_hal::delay_ms(100);
    }
}

//旋转编码器触发中断
#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT0() {
    interrupt_rotary_default_isr();
}

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT2() {
    interrupt_rotary_default_isr();
}
