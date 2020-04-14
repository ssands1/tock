use kernel::common::cells::OptionalCell;

pub struct UnixLed<'a> {
    led: OptionalCell<&'a kernel::hil::led::Led>,
    is_on: bool,
}

impl<'a> UnixLed<'a> {
    pub fn new() -> Self {
        UnixLed {
            led: OptionalCell::empty(),
            is_on: false,
        }
    }

    pub fn set_led(&self, led: &'static kernel::hil::led::Led) {
        self.led.set(led);
    }
}

impl<'a> kernel::hil::gpio::Pin for UnixLed<'a> {
    fn make_output(&self) {}
    fn make_input(&self) {}
    fn disable(&self) {}
    fn set(&self) {}
    fn clear(&self) {}
    fn toggle(&self) {}
    fn read(&self) -> bool {
        return false;
    }
    fn enable_interrupt(&self, identifier: usize, mode: kernel::hil::gpio::InterruptMode) {}
    fn disable_interrupt(&self) {}
}

impl<'a> kernel::hil::led::Led for UnixLed<'a> {
    fn init(&mut self) {
        self.is_on = false;
    }
    fn on(&mut self) {
        self.is_on = true;
        println!("++++++ LED  ON ++++++");
    }
    fn off(&mut self) {
        self.is_on = false;
        println!("------ led off ------");
    }
    fn toggle(&mut self) {
        if self.is_on {
            self.off()
        } else {
            self.on()
        }
    }
    fn read(&self) -> bool {
        return self.is_on;
    }
}
