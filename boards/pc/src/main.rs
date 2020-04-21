#![feature(const_fn)]

extern crate capsules;
extern crate core;
extern crate kernel;
extern crate nix;

use capsules::alarm::AlarmDriver;

use kernel::{capabilities, create_capability, static_init};
use kernel::hil::time::{Alarm};
use kernel::{Platform, ReturnCode};

mod arch;
mod chip;

use chip::alarm::*;
use chip::led;

/* piping example starts */
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;

use kernel::AppId;

static PANGRAM: &'static str = "This is a message from Rust\n";

const NUM_PROCS: usize = 4; // set to reflect most chips

enum Syscall { command, subscribe, allow, yield_call, memop }

/* and ends */

struct Emulator {
    alarm: &'static AlarmDriver<'static, UnixAlarm<'static>>,
}

impl Emulator {
    // TODO: Return a ReturnCode (e.g., if it can't spawn threads)
    unsafe fn run_apps(&self, processes: [Option<&str>; NUM_PROCS]) {
        for p in processes.iter() {
            match p {
                None => {}
                Some(name) => {
                    let process = match Command::new(format!("./{}", name))
                        .stdin(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                    {
                        Err(err) => panic!("couldn't spawn process: {}", err),
                        Ok(process) => process,
                    };
                    
                    let reader = BufReader::new(process.stderr.expect("stdout"));
                    let child_reader = thread::spawn(move || {
                        reader
                            .lines()
                            .filter_map(|line| line.ok())
                            .for_each(|line| {
                                // println!("{}", line);
                                let nums: Vec<usize> = line
                                    .split(",")
                                    .map(|val| val.parse::<usize>())
                                    .filter_map(Result::ok)
                                    .collect();
                                
                                // TODO: find a better way to check length
                                // TODO: Not every syscall has the same # of args
                                // Q: Will this return from the closure?
                                // Or will it return from run_apps?
                                if nums.len() != 5 { return }
                                match nums[0] {
                                    // TODO: Use enum
                                    0 => {
                                        // TODO: fix AppId (e.g., index?) or get a real one
                                        // let app_id = AppId::new(
                                        //     board_kernel, 
                                        //     board_kernel
                                        //         .create_process_identifier(), 
                                        //     0
                                        // );
                                        // TODO: send res back to app
                                        // let _res = 
                                        //     self.with_driver(0, 
                                        //         |driver| match driver {
                                        //             Some(d) => d.command(
                                        //                 nums[1],
                                        //                 nums[2],
                                        //                 nums[3],
                                        //                 nums[4],
                                        //                 app_id
                                        //             ),
                                        //             None => ReturnCode::ENODEVICE,
                                        //         },
                                        //     );
                                    }
                                    _ => {}
                                }
                            });      
                    });
                    
                    // `stdin` has type `Option<ChildStdin>`, but since we know 
                    // this instance must have one, we can directly `unwrap` it.
                    match &process.stdin.unwrap().write_all(PANGRAM.as_bytes()) {
                        Err(err) => panic!("couldn't write to stdin: {}", err),
                        Ok(_) => println!("sent message to playground"),
                    }
                    let ok = child_reader.join();
                }
            }
        }
    }
}

impl kernel::Platform for Emulator {
    fn with_driver<F, R>(&self, driver_num: usize, f: F) -> R
    where
        F: FnOnce(Option<&dyn kernel::Driver>) -> R,
    {
        match driver_num {
            capsules::console::DRIVER_NUM => f(Some(self.alarm)),
            _ => f(None),
        }
    }
}

struct App<'a, A, B>
where
    A: kernel::hil::time::Alarm<'a>,
    B: kernel::hil::led::Led,
{
    alarm: &'a A,
    led: &'a B,
}

impl<'a, A, B> App<'a, A, B> where
    A: kernel::hil::time::Alarm<'a>,
    B: kernel::hil::led::Led,
{
    fn init(&self) {
        self.alarm.set_alarm(self.alarm.now() + 1000);
    }
}

impl<'a, A, B> kernel::hil::time::AlarmClient for App<'a, A, B>
where
    A: kernel::hil::time::Alarm<'a>,
    B: kernel::hil::led::Led,
{
    fn fired(&self) {
        println!("Blink");
        self.init();
    }
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

        // let led_pins = static_init!(
        //     [(
        //         &'static dyn kernel::hil::gpio::Pin,
        //         kernel::hil::gpio::ActivationMode
        //     ); 1],
        //     [(
        //         &led::GPIOPin::new(),
        //         kernel::hil::gpio::ActivationMode::ActiveHigh
        //     )]
        // );
        // let led = static_init!(
        //     capsules::led::LED<'static>,
        //     capsules::led::LED::new(led_pins)
        // );

        let ipc = kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);

        let app = static_init!(App<chip::alarm::UnixAlarm, chip::led::UnixLed>, 
                               App { alarm: &chip.alarm, led: &chip.led });
        chip.alarm.set_client(app);
        app.init();

        println!("Hello World");

        
        let emulator = Emulator {
            alarm,
        };
        
        emulator.run_apps([Some("playground"), None, None, None]);

        board_kernel.kernel_loop(&emulator, chip, Some(&ipc), &main_loop_capability);
    }
}
