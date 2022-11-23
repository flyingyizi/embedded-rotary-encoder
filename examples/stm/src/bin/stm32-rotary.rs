//! cargo objcopy --bin stm32-rotary -- -j .text -j .data -O ihex  stm32-rotary.hex

#![no_std]
#![no_main]

use cortex_m::interrupt::{free, Mutex};
use stm32f4xx_hal::gpio::{EPin, Input};
type RotaryPin = EPin<Input>;
use core::fmt::Write; // for pretty formatting of the serial output
use core::{cell::RefCell, ops::DerefMut};
use cortex_m_rt::entry;
use embedded_rotary_encoder::{LatchMode, RotaryDirecton, RotaryEncoder};
use panic_halt as _;
use stm32f4xx_hal::{
    gpio::Edge,
    pac::{interrupt, Interrupt, Peripherals},
    prelude::*,
};

use help_for_isr::{interrupt_rotary_current, ROTARY_ENCODER};

mod help_for_isr {
    use super::{free, Mutex, RefCell, RotaryDirecton, RotaryEncoder, RotaryPin};
    use core::ops::DerefMut;
    /// helper rotary encoder instance prepared for isr model, it will be initialized in your app.
    /// notes: the pin_clk/pin_dt pins should associated with the interrupts in your app at the same time.
    pub static ROTARY_ENCODER: Mutex<RefCell<Option<RotaryEncoder<RotaryPin>>>> =
        Mutex::new(RefCell::new(None));

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

#[entry]
fn main() -> ! {
    let mut dp = Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    // let clocks = rcc.cfgr.sysclk(16.MHz()).pclk1(8.MHz()).freeze();
    let clocks = rcc.cfgr.use_hse(8.MHz()).freeze();
    let mut syscfg = dp.SYSCFG.constrain();

    let (gpioa,) = (dp.GPIOA.split(),);

    let mut pin_clk = gpioa.pa0.into_pull_up_input();
    let mut pin_dt = gpioa.pa1.into_pull_up_input();
    // Configure CLK typically as pullup input & interrupt on Rising and Falling edges
    // pa0(EXTI0),pa1(EXTI1),
    {
        pin_dt.make_interrupt_source(&mut syscfg);
        pin_dt.enable_interrupt(&mut dp.EXTI);
        pin_dt.trigger_on_edge(&mut dp.EXTI, Edge::RisingFalling);
        pin_clk.make_interrupt_source(&mut syscfg);
        pin_clk.enable_interrupt(&mut dp.EXTI);
        pin_clk.trigger_on_edge(&mut dp.EXTI, Edge::RisingFalling);
    }
    // Initialize Rotary Encoder and safely store in static global
    free(|cs| {
        ROTARY_ENCODER.borrow(cs).replace(Some(RotaryEncoder::new(
            pin_clk.erase(),
            pin_dt.erase(),
            LatchMode::FOUR3,
        )));
    });
    //enable interrupt
    {
        cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI0);
        cortex_m::peripheral::NVIC::unpend(Interrupt::EXTI1);
        unsafe {
            cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI0);
            cortex_m::peripheral::NVIC::unmask(Interrupt::EXTI1);
        }
    }

    // define RX/TX pins
    let tx_pin = gpioa.pa2.into_alternate();
    // configure serial
    // let mut tx = Serial::tx(dp.USART2, tx_pin, 115200.bps(), &clocks).unwrap();
    // or 57600
    let mut tx = dp.USART2.tx(tx_pin, 115200.bps(), &clocks).unwrap();

    writeln!(tx, "Hello from stm32! simulate rotary encoder:\r").unwrap();

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
            writeln!(tx, "pos:{}, dir:{}\r", cur, direction).unwrap();
            previous = cur.clone();
        }

        // (cur, direction) = free(|cs| {
        //     if let Some(ref mut encoder) = ROTARY_ENCODER.borrow(cs).borrow_mut().deref_mut() {
        //         (encoder.get_position(), encoder.get_direction() as i32)
        //     } else {
        //         unreachable!()
        //     }
        // });
        // if previous != cur {
        //     writeln!(tx, "pos:{}, dir:{}\r", cur, direction).unwrap();
        //     previous = cur.clone();
        // }
    }
}

// Interrupt Handler
#[interrupt]
fn EXTI0() {
    free(|cs| {
        if let Some(ref mut rotary_encoder) = ROTARY_ENCODER.borrow(cs).borrow_mut().deref_mut() {
            rotary_encoder.pin_clk.clear_interrupt_pending_bit();
            // Update the encoder, which will compute its direction
            rotary_encoder.check_state();
        }
    });
}
#[interrupt]
fn EXTI1() {
    free(|cs| {
        if let Some(ref mut rotary_encoder) = ROTARY_ENCODER.borrow(cs).borrow_mut().deref_mut() {
            rotary_encoder.pin_dt.clear_interrupt_pending_bit();
            // Update the encoder, which will compute its direction
            rotary_encoder.check_state();
        }
    });
}
