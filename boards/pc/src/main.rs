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

#[repr(u8)]
enum Syscall { Yield, Memop, Command, Subscribe, Allow }

/* and ends */

struct Emulator {
    alarm: &'static AlarmDriver<'static, UnixAlarm<'static>>,
}

impl Emulator {



    fn do_command(d: &dyn kernel::Driver, app_id: &AppId, args: Vec<usize>) -> ReturnCode {
        return d.command(args[2], args[3], args[4], *app_id);
    }

    // fn do_subscribe(d: &dyn kernel::Driver, app_id: AppId, args:) -> ReturnCode {
    //     d.subscribe(
    //         nums[2], // subscribe_num
    //         nums[3], // callback                                         
    //         *app_id
    //     );
    // /* 
    //         Callback: {
    //             app_id,
    //             callback_id: {
    //                 driver_num,
    //                 subscribe_num
    //             },
    //             appdata,
    //             fn_ptr
    //         } 
    // */
    // }

    // fn do_allow(d: &dyn kernel::Driver, app_id: AppId, args:) -> ReturnCode {
    //     d.allow(
    //         *app_id,
    //         nums[2],
    //         nums[3] // slice ptr of size nums[4]
    //     )
    // }


    // TODO: Return a ReturnCode (e.g., if it can't spawn threads)
    unsafe fn run_app(&self, name: &str, app_id: &AppId) {
        let process = match Command::new(format!("./{}", name))
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Err(err) => panic!("couldn't spawn process: {}", err),
            Ok(proc) => proc,
        };
        
        let writer = &mut process.stdin.unwrap();
        
        // Process calls from apps
        BufReader::new(process.stderr.expect("stdout"))
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| {
                // parse call args into Vec<usize>
                let args: Vec<usize> = line
                    .split(",")
                    .map(|val| val.parse::<usize>())
                    .filter_map(Result::ok)
                    .collect();
                    
                // TODO: Assert length/formatting, add more callse
                self.with_driver(args[1], |driver| {
                    let r_code: usize = match driver {
                        None => ReturnCode::ENODEVICE.into(),
                        Some(d) => match args[0] {
                            0 => match Emulator::do_command(d, app_id, args) {
                                ReturnCode::SuccessWithValue { value } => value,
                                r => r.into()
                            },
                            // 1 => do_subscribe(d, app_id, args),
                            // 2 => do_allow(d, app_id, args),
                            _ => ReturnCode::ENODEVICE.into(),
                        }
                    };
                    
                    println!("hiiii {}", r_code);
                    // `stdin` has type `Option<ChildStdin>`, but since we know 
                    // this instance must have one, we can directly `unwrap` it.
                    match writer.write_all(r_code.to_string().as_bytes()) {
                        Err(e) => panic!("Error: {}", e),
                        Ok(_) => {}//println!("Success!"),
                    } 
                });
            });      
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
                
        let processes = [Some("playground"), None, None, None];

        for i in 0..processes.len() { 
            match processes[i] {
                None => {},
                Some(name) => {
                    let app_id = AppId::new(
                        board_kernel,
                        board_kernel.create_process_identifier(),
                        i
                    );
                    emulator.run_app(name, &app_id);
                }
            }
        }

        board_kernel.kernel_loop(&emulator, chip, Some(&ipc), &main_loop_capability);
    }
}
