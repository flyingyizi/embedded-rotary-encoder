// orig c++ implement: http://www.mathertel.de/Arduino/RotaryEncoderLibrary.aspx
//
//! rotary encoder produces are based on a 2-bit gray code available on 2 digital data signal lines.
//! There are 4 possible combinations of on and off on these 2 signal lines,
//! related gray code shows below:
//! |Position	|Bit A	|Bit B|
//! |-----------|-------|-----|
//! |0	       |0	   |0    |
//! |1/4	   |1      |0    |
//! |1/2	   |1      |1    |
//! |3/4	   |0      |1    |
//! |1	       |0      |0    |
//! cw:
//!   a   b   c   d
//!   │   |   |   |   │               │
//! ──┼───┐       ┌───┼───┐       ┌───┼───┐       ┌───────┐
//!   │   │       │   │   │       │   │   │       │       │          pin2
//!   │   └───────┘   │   └───────┘   │   └───────┘       └────
//!   ├───────┐       ├───────┐       ├───────┐
//!   │       │       │       │       │       │                      pin1
//! ──┤       └───────┤       └───────┤       └───────
//!   │               │               │
//!
//!   if ((old_state == 2) && (thisState == 3))  {
//!     knobPosition++;
//!   } else if ((old_state == 3) && (thisState == 1)) {
//!     knobPosition++;
//!   } else if ((old_state == 1) && (thisState == 0)){
//!     knobPosition++;
//!   } else if ((old_state == 0) && (thisState == 2)) {
//!     knobPosition++;
//! ccw:
//!   │               │               │
//! ──┼───┐       ┌───┼───┐       ┌───┼───┐       ┌───────┐
//!   │   │       │   │   │       │   │   │       │       │        pin2
//!   │   └───────┘   │   └───────┘   │   └───────┘       └────
//! ──┤       ┌───────┤       ┌───────┤       ┌───────┐
//!   │       │       │       │       │       │       │            pin1
//!   ├───────┘       ├───────┘       ├───────┘       └────
//!   │               │               │
//!
//!   } else if ((old_state == 3) && (thisState == 2)) {
//!     knobPosition--;
//!   } else if ((old_state == 2) && (thisState == 0)) {
//!     knobPosition--;
//!   } else if ((old_state == 0) && (thisState == 1)) {
//!     knobPosition--;
//!   } else if ((old_state == 1) && (thisState == 3)) {
//!     knobPosition--;
//!   }
//!
use embedded_hal::digital::v2::InputPin;

/// The array holds the values -1 for the entries where a position was decremented,
/// a 1 for the entries where the position was incremented
/// and 0 in all the other (no change or not valid) cases.
/// visit is using index, "index = thisState | (oldState<<2);"
#[rustfmt::skip]
const KNOBDIR: [i32; 16] = [
    0, -1, 1, 0, 
    1, 0, 0, -1, 
    -1, 0, 0, 1, 
    0, 1, -1, 0];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RotaryDirecton {
    NOROTATION = 0,
    CLOCKWISE = 1,
    COUNTERCLOCKWISE = -1,
}

pub enum LatchMode {
    /// 4 steps, Latch at position 3 only (compatible to older versions)
    FOUR3 = 1,
    /// 4 steps, Latch at position 0 (reverse wirings)
    FOUR0 = 2,
    /// 2 steps, Latch at position 0 and 3
    TWO03 = 3,
}

impl Default for LatchMode {
    fn default() -> Self {
        LatchMode::FOUR0
    }
}

#[derive(Default)]
pub struct RotaryEncoder<PIN>
where
    PIN: InputPin,
{
    /// pins used for the encoder.
    pub pin_clk: PIN,
    pub pin_dt: PIN,

    // Latch mode from initialization
    mode: LatchMode,

    old_state: u8,

    /// The internal (physical) position of the rotary encoder library remains by stepping with the increment 1.
    /// Internal(rotary encoder physical) position (ROTARYSTEPS times logic-position)
    _position: i32,
    /// logic position
    ext_position: i32,
    ext_position_prev: i32,
}

impl<PIN> RotaryEncoder<PIN>
where
    PIN: InputPin,
{
    /// pins should be inited as pullup input in app, e.g.
    /// "let mut ROTARY_1_PIN = pins.d7.into_pull_up_input().downgrade();"
    ///
    /// notes: when in isr model, you should associate ISR for pin_dt and pin_clk.
    pub fn new(pin_clk: PIN, pin_dt: PIN, mode: LatchMode) -> Self {
        let state = {
            // when not started in motion, the current state of the encoder should be 3

            let (mut sig1, mut dt) = (0_u8, 0_u8);
            if let Ok(s) = pin_clk.is_high() {
                if s {
                    sig1 = 1;
                }
            }
            if let Ok(s) = pin_dt.is_high() {
                if s {
                    dt = 1;
                }
            }
            sig1 | (dt << 1)
        };

        Self {
            pin_clk,
            pin_dt,
            mode,

            // start with position 0;
            _position: 0,

            ext_position: 0,
            ext_position_prev: 0,
            old_state: state,
        }
    }

    /// retrieve the current logical position
    pub fn get_position(&self) -> i32 {
        self.ext_position
    }

    // adjust the current logic absloute position.
    pub fn set_position(&mut self, new_logic_position: i32) {
        match self.mode {
            LatchMode::FOUR3 | LatchMode::FOUR0 => {
                // only adjust the external part of the position.
                self._position = (new_logic_position << 2) | (self._position & 0x03);
                self.ext_position = new_logic_position;
                self.ext_position_prev = new_logic_position;
            }
            LatchMode::TWO03 => {
                // only adjust the external part of the position.
                self._position = (new_logic_position << 1) | (self._position & 0x01);
                self.ext_position = new_logic_position;
                self.ext_position_prev = new_logic_position;
            }
        }
    }

    // simple retrieve of the direction the knob was rotated last time. 0 = No rotation, 1 = Clockwise, -1 = Counter Clockwise
    pub fn get_direction(&mut self) -> RotaryDirecton {
        if self.ext_position_prev > self.ext_position {
            self.ext_position_prev = self.ext_position;
            RotaryDirecton::COUNTERCLOCKWISE
        } else if self.ext_position_prev < self.ext_position {
            self.ext_position_prev = self.ext_position;
            RotaryDirecton::CLOCKWISE
        } else {
            RotaryDirecton::NOROTATION
            // self.ext_position_prev = self.ext_position;
        }
    }
    /// checking the state of the rotary encoder is to poll the signals as often as you can. e.g. call it
    /// in loop or in ISR handle
    pub fn check_state(&mut self) {
        const LATCH0: u8 = 0; // input state at position 0
        const LATCH3: u8 = 3; // input state at position 3

        let this_state = {
            let (mut sig1, mut sig2) = (0_u8, 0_u8);
            if let Ok(s) = self.pin_clk.is_high() {
                if s {
                    sig1 = 1;
                }
            }
            if let Ok(s) = self.pin_dt.is_high() {
                if s {
                    sig2 = 1;
                }
            }
            sig1 | (sig2 << 1)
        };

        if self.old_state == this_state {
            return;
        }

        let index = (this_state | (self.old_state << 2)) as usize;
        self._position += KNOBDIR[index];
        self.old_state = this_state;

        match self.mode {
            LatchMode::FOUR3 => {
                if this_state == LATCH3 {
                    // The hardware has 4 steps with a latch on the input state 3
                    self.ext_position = self._position >> 2;
                }
            }

            LatchMode::FOUR0 => {
                if this_state == LATCH0 {
                    // The hardware has 4 steps with a latch on the input state 0
                    self.ext_position = self._position >> 2;
                }
            }

            LatchMode::TWO03 => {
                if (this_state == LATCH0) || (this_state == LATCH3) {
                    // The hardware has 2 steps with a latch on the input state 0 and 3
                    self.ext_position = self._position >> 1;
                }
            }
        } // switch
    }

    /// Borrow a mutable reference to the underlying InputPins. This is useful for clearing hardware interrupts.
    /// reutn (pin_dt, pin_clk)
    pub fn borrow_pins(&mut self) -> (&mut PIN, &mut PIN) {
        (&mut self.pin_dt, &mut self.pin_clk)
    }
}
