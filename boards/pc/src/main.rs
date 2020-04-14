#![feature(const_fn)]

extern crate capsules;
extern crate core;
extern crate kernel;
extern crate nix;

use kernel::{capabilities, create_capability, static_init};

mod arch;
mod chip;

use chip::alarm::*;

/* piping example starts */
use std::error::Error;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

static PANGRAM: &'static str = "This is a message from Rust\n";

/* and ends */

struct Emulator;

impl kernel::Platform for Emulator {
    fn with_driver<F, R>(&self, _driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&kernel::Driver>) -> R,
    {
        f(None)
    }
}

struct App<'a, A, B>
where
    A: kernel::hil::time::Alarm,
    B: kernel::hil::led::Led,
{
    alarm: &'a A,
    led: &'a B,
}

impl<'a, A, B> App<'a, A, B> where
    A: kernel::hil::time::Alarm,
    B: kernel::hil::led::Led,
{
    fn init(&self) {
        self.alarm.set_alarm(self.alarm.now() + 1000);
    }
}

impl<'a, A, B> kernel::hil::time::Client for App<'a, A, B>
where
    A: kernel::hil::time::Alarm,
    B: kernel::hil::led::Led,
{
    fn fired(&self) {
        println!("Blink");
        self.init();
    }
}

unsafe fn run_app(name: &str) {
    let process = match Command::new(format!("./{}", name))
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Err(err) => panic!("couldn't spawn process: {}", err.description()),
        Ok(process) => process,
    };
    // `stdin` has type `Option<ChildStdin>`, but since we know this instance
    // must have one, we can directly `unwrap` it.
    match process.stdin.unwrap().write_all(PANGRAM.as_bytes()) {
        Err(err) => panic!("couldn't write to process stdin: {}", err.description()),
        Ok(_) => println!("sent message to playground"),
    }

    let reader = BufReader::new(process.stdout.unwrap());
    reader
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| println!("{}", line)); // TODO: parse protocol
    
    // kernel::Driver::command(&self, 0, 4, 1000, 0); // What is self here?
}

fn main() {
    unsafe {
        let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
        let memory_allocation_capability =
            create_capability!(capabilities::MemoryAllocationCapability);

        let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&[]));

        let chip = static_init!(chip::Chip<'static>, chip::Chip::new());

        let alarm = static_init!(
            capsules::alarm::AlarmDriver<'static, UnixAlarm>,
            capsules::alarm::AlarmDriver::new(
                &chip.alarm,
                board_kernel.create_grant(&memory_allocation_capability)
            )
        );

        chip.alarm.set_client(alarm);

        // let led = static_init!(
        //     capsules::led::LED<'static, UnixLed>,
        //     capsules::led::LED::new()
        // )

        let ipc = kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

        let app = static_init!(App<chip::alarm::UnixAlarm, chip::led::UnixLed>, 
                               App { alarm: &chip.alarm, led: &chip.led });
        chip.alarm.set_client(app);
        app.init();

        println!("Hello World");

        run_app("playground");

        board_kernel.kernel_loop(&Emulator, chip, Some(&ipc), &main_loop_capability);
    }
}
