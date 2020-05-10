#![feature(const_fn)]

extern crate capsules;
extern crate core;
extern crate kernel;
extern crate libc;
extern crate nix;
extern crate num_derive;
extern crate num_traits;

use capsules::alarm::{AlarmData, AlarmDriver};
use capsules::led::LED;

use kernel::{capabilities, create_capability, static_init};
use kernel::{AppId, AppSlice, Callback, Platform, ReturnCode};
use kernel::procs::SimProcess;
use kernel::hil::time::{Alarm};

mod arch;
mod chip;

use chip::alarm::*;
use chip::led;

use num_derive::FromPrimitive;    
use num_traits::FromPrimitive;

use std::alloc::{alloc, dealloc, Layout, System};
use std::io::prelude::*;
use std::io::{BufRead, BufReader};
use std::mem::size_of;
use std::process::{Command, Stdio};
use std::ptr::NonNull;
use std::thread;

const NUM_PROCS: usize = 4; // set to reflect most chips
const NUM_LEDS: usize = 1;

#[global_allocator]
static A: System = System;

static mut PROCESSES: [Option<&'static dyn kernel::procs::ProcessType>; NUM_PROCS] =
    [None; NUM_PROCS];

#[derive(FromPrimitive)]
enum Syscall { Command, Subscribe, Yield }

struct Emulator {
    alarm: &'static AlarmDriver<'static, UnixAlarm<'static>>,
    led: &'static LED<'static>,
}

impl Emulator {
    fn do_command(
        driver: &dyn kernel::Driver, 
        app_id: &AppId, 
        args: Vec<usize>
    ) -> ReturnCode {
        driver.command(args[2], args[3], args[4], *app_id)
    }

    // TODO: Throw error if args[3] is null or simply pass None?
    // TODO: appdata is usize but args[4] is void*?
    fn do_subscribe(
        driver: &dyn kernel::Driver,
        app_id: &AppId,
        args: Vec<usize>
    ) -> ReturnCode {
        let f_ptr = NonNull::new(args[3] as *mut *mut ()).unwrap();
        let cb = Callback::new(*app_id, args[1], args[2], args[4] as usize, f_ptr);
        driver.subscribe(args[2], Some(cb), *app_id)
    }

    // TODO: Throw error if args[3] is null or simply pass None?
    unsafe fn do_yield(
        driver: &dyn kernel::Driver, 
        app_id: &AppId,
        args: Vec<usize>
    ) -> ReturnCode {
        let p = NonNull::new(args[3] as *mut u8).unwrap();
        let slice = AppSlice::new(p, args[4], *app_id);
        driver.allow(*app_id, args[2], Some(slice))
    }
    
    // TODO: use dedicated pipe instead of stderr
    unsafe fn run_app(&self, name: &str, app_id: &AppId) {
        let process = match Command::new(format!("./{}", name))
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn() 
        {
            Err(err) => panic!("couldn't spawn process: {}", err),
            Ok(process) => process,
        };
        
        // `stdin` has type `Option<ChildStdin>`, but since we know 
        // this instance must have one, we can directly `unwrap` it.
        let writer = &mut process.stdin.unwrap();
        
        BufReader::new(process.stderr.expect("stdout"))
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| {
                // TODO: Assert length/formatting
                let args: Vec<usize> = line
                    .split(",")
                    .map(|val| val.parse::<usize>())
                    .filter_map(Result::ok)
                    .collect();

                // TODO: add more calls eg memop
                self.with_driver(args[1], |driver| {
                    let r_code: isize = match driver {
                        None => ReturnCode::ENODEVICE.into(),
                        Some(d) => match FromPrimitive::from_usize(args[0]) {
                            Some(Syscall::Command) => 
                                Emulator::do_command(d, app_id, args),
                            Some(Syscall::Subscribe) => 
                                Emulator::do_subscribe(d, app_id, args),
                            Some(Syscall::Yield) => 
                                Emulator::do_yield(d, app_id, args),
                            None => {
                                println!("Error: unknown syscall");
                                ReturnCode::EINVAL
                            }
                        }.into()
                    };
                    
                    println!("I'm writing {}", r_code.to_string());
                    match writer.write_all(r_code.to_string().as_bytes()) {
                        Err(err) => panic!("Error writing: {}", err),
                        Ok(_) => {}
                    };
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
            capsules::alarm::DRIVER_NUM => f(Some(self.alarm)),
            capsules::led::DRIVER_NUM => f(Some(self.led)),
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

        let board_kernel = static_init!(kernel::Kernel, kernel::Kernel::new(&PROCESSES));

        let chip = static_init!(chip::Chip<'static>, chip::Chip::new());

        // Set up Alarm
        let alarm = static_init!(
            AlarmDriver<'static, UnixAlarm>,
            AlarmDriver::new(
                &chip.alarm,
                board_kernel.create_grant(&memory_allocation_capability)
            )
        );

        chip.alarm.set_client(alarm);

        // Set up LED
        let unix_led = static_init!(led::UnixLed<'static>, led::UnixLed::new());
        let pins = static_init!([&dyn kernel::hil::led::Led; NUM_LEDS], [unix_led]);
        let led = static_init!(LED<'static>, LED::new(pins));

        // Set up app
        let app = static_init!(App<chip::alarm::UnixAlarm, chip::led::UnixLed>, 
            App { alarm: &chip.alarm, led: &chip.led });
            chip.alarm.set_client(app);
            app.init();
        
        let ipc = kernel::ipc::IPC::new(board_kernel, &memory_allocation_capability);
            
        let emulator = Emulator { alarm, led };

        let processes: [Option<&str>; NUM_PROCS] = 
            [Some("playground"), None, None, None];

        // TODO: Fix index arg
        for i in 0..processes.len() { 
            match processes[i] {
                None => {},
                Some(name) => {
                    let app_id = AppId::new(
                        board_kernel,
                        board_kernel.create_process_identifier(),
                        i
                    );

                    // TODO: Allocate memory in process.rs, not here
                    // Note: A grant region is valid once allocated
                    // for the lifetime of the process.
                    let alarm_layout = Layout::new::<AlarmData>();
                    let alarm_ptr = alloc(alarm_layout);
                    
                    let process = static_init!(
                        SimProcess,
                        SimProcess::create(board_kernel, app_id, alarm_ptr as *mut u8)
                    );
                    PROCESSES[i] = Some(process);
                    emulator.run_app(name, &app_id);
                    dealloc(alarm_ptr, alarm_layout);
                }
            }
        }
        board_kernel.kernel_loop(&emulator, chip, Some(&ipc), &main_loop_capability);
    }
}
