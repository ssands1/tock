use core::cell::Cell;
use kernel::common::cells::OptionalCell;

pub struct UnixLed<'a> {
    led: OptionalCell<&'a dyn kernel::hil::led::Led>,
    is_on: Cell<bool>,
}

impl<'a> UnixLed<'a> {
    pub fn new() -> Self {
        UnixLed {
            led: OptionalCell::empty(),
            is_on: Cell::new(false),
        }
    }

    pub fn set_led(&self, led: &'static dyn kernel::hil::led::Led) {
        self.led.set(led);
    }
}

impl<'a> kernel::hil::led::Led for UnixLed<'a> {
    fn init(&mut self) {
        // N/A
    }
    fn on(&self) {
        self.is_on.set(true);
        println!("++++++ LED  ON ++++++");
    }
    fn off(&self) {
        self.is_on.set(false);
        println!("------ led off ------");
    }
    fn toggle(&self) {
        if self.read() {
            self.off()
        } else {
            self.on()
        }
    }
    fn read(&self) -> bool {
        return self.is_on.get();
    }
}
