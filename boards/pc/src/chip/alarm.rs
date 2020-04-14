use kernel::common::cells::OptionalCell;
use std::time::{Duration, SystemTime};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use crate::chip;

pub struct UnixAlarm<'a> {
    client: OptionalCell<&'a kernel::hil::time::AlarmClient>,
    event_sender: Arc<Mutex<Option<Sender<chip::Event>>>>,
}

impl<'a> UnixAlarm<'a> {
    pub fn new(event_sender: Sender<chip::Event>) -> Self {
        UnixAlarm {
            client: OptionalCell::empty(),
            event_sender: Arc::new(Mutex::new(Some(event_sender))),
        }
    }

    pub fn handle_interrupt(&self) {
        self.client.map(|client| {
            client.fired()
        });
    }
}

impl<'a> kernel::hil::time::Time for UnixAlarm<'a> {
    type Frequency = kernel::hil::time::Freq1MHz;

    fn now(&self) -> u32 {
        if let Ok(duration) = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            (duration.as_secs() * 1000 + duration.subsec_micros() as u64) as u32
        } else {
            0 // there was an error, not sure why there would be...
        }
    }

    fn max_tics(&self) -> u32 {
        core::u32::MAX
    }
}

impl<'a> kernel::hil::time::Alarm<'a> for UnixAlarm<'a> {
    fn set_client(&self, client: &'a kernel::hil::time::AlarmClient) {
        self.client.set(client);
    }

    fn set_alarm(&self, tics: u32) {
        use kernel::hil::time::Time;
        let target = Duration::from_millis(tics.wrapping_sub(self.now()) as u64);
        let sender = self.event_sender.clone();
        thread::spawn(move || {
            thread::sleep(target);
            sender.lock().map(|m| m.as_ref().map(|sender| sender.send(chip::Event::Alarm))).unwrap();
        });
    }

    fn get_alarm(&self) -> u32 {
        0
    }

    fn is_enabled(&self) -> bool {
        true
    }

    fn enable(&self) { /* TODO */ }

    fn disable(&self) { /* TODO */ }

}
