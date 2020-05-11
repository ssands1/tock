//! Interface for LEDs that abstract away polarity and pin.
//!
//!  Author: Philip Levis <pal@cs.stanford.edu>
//!  Date: July 31, 2015
//!

use crate::hil::gpio;

pub trait Led {
    fn init(&mut self);
    fn on(&self);
    fn off(&self);
    fn toggle(&self);
    fn read(&self) -> bool;
}

/// For LEDs in which on is when GPIO is high.
pub struct LedHigh<'a, P: gpio::Pin> {
    pub pin: &'a mut P,
}

/// For LEDs in which on is when GPIO is low.
pub struct LedLow<'a, P: gpio::Pin> {
    pub pin: &'a mut P,
}

impl<'a, P: gpio::Pin> LedHigh<'a, P> {
    pub fn new(p: &'a mut P) -> Self {
        Self { pin: p }
    }
}

impl<'a, P: gpio::Pin> LedLow<'a, P> {
    pub fn new(p: &'a mut P) -> Self {
        Self { pin: p }
    }
}

impl<P: gpio::Pin> Led for LedHigh<'_, P> {
    fn init(&mut self) {
        self.pin.make_output();
    }

    fn on(&self) {
        self.pin.set();
    }

    fn off(&self) {
        self.pin.clear();
    }

    fn toggle(&self) {
        self.pin.toggle();
    }

    fn read(&self) -> bool {
        self.pin.read()
    }
}

impl<P: gpio::Pin> Led for LedLow<'_, P> {
    fn init(&mut self) {
        self.pin.make_output();
    }

    fn on(&self) {
        self.pin.clear();
    }

    fn off(&self) {
        self.pin.set();
    }

    fn toggle(&self) {
        self.pin.toggle();
    }

    fn read(&self) -> bool {
        !self.pin.read()
    }
}
