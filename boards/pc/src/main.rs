#![feature(const_fn)]

extern crate core;
extern crate capsules;
extern crate kernel;
extern crate nix;
extern crate os_pipe;

use kernel::{capabilities, create_capability, static_init};

mod arch;
mod chip;

use chip::alarm::*;

/* piping example starts */
use std::error::Error;
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

static PANGRAM: &'static str =
"This is a message from Rust\n";

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

struct App<'a, A: kernel::hil::time::Alarm> {
    alarm: &'a A,
}

impl<'a, A: kernel::hil::time::Alarm> App<'a, A> {
    fn init(&self) {
        self.alarm.set_alarm(self.alarm.now() + 1000);
    }
}

impl<'a, A: kernel::hil::time::Alarm> kernel::hil::time::Client for App<'a, A> {
    fn fired(&self) {
        println!("Blink");
        self.init();
    }
}

fn main() {
    unsafe {
        let main_loop_capability = create_capability!(capabilities::MainLoopCapability);
        let memory_allocation_capability = create_capability!(capabilities::MemoryAllocationCapability);

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

        let ipc = kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

        let app = static_init!(App<chip::alarm::UnixAlarm>, App { alarm: &chip.alarm});
        chip.alarm.set_client(app);
        app.init();

        println!("Hello World");

        /*
        #[link(name = "doubler", kind="static")]
        extern "C" {
            pub fn doubler(i: ::std::os::raw::c_int) -> ::std::os::raw::c_int;
        }
        println!("Value: {}", doubler(6));
        */
        
        let process = match Command::new("./playground")
                                .stdin(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn() {
            Err(err) => panic!("couldn't spawn process: {}", err.description()),
            Ok(process) => process,
        };
        
        // `stdin` has type `Option<ChildStdin>`, but since we know this instance
        // must have one, we can directly `unwrap` it.
        match process.stdin.unwrap().write_all(PANGRAM.as_bytes()) {
            Err(err) => panic!("couldn't write to process stdin: {}",
                               err.description()),
            Ok(_) => println!("sent pangram to wc"),
        }

        let reader = BufReader::new(process.stdout.unwrap());
        reader.lines()
              .filter_map(|line| line.ok())
              .for_each(|line| println!("{}", line));

        board_kernel.kernel_loop(&Emulator, chip, Some(&ipc), &main_loop_capability);
    }
}
