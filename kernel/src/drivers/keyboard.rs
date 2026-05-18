use embassy_rp::usb::host::{Allocator, SealedHostInstance};
use embassy_usb_host::{
    BusHandle,
    class::hid::{HidHost, KeyboardReport, PROTOCOL_BOOT},
};

mod core;
mod layout;

pub use crate::drivers::keyboard::core::{Action, Key, KeyEvent, Modifiers};
use crate::drivers::keyboard::layout::usage_id_to_mapped_key;

fn report_to_keys(report: &KeyboardReport, keys: &mut [Option<Key>; 6]) {
    for (i, key) in report.keycodes.into_iter().enumerate() {
        if key <= 1 {
            keys[i] = None
        } else {
            keys[i] = Some(usage_id_to_mapped_key(
                // TODO get layout from user preferences
                layout::Layout::Uk,
                key,
                Modifiers(report.modifiers),
            ));
        }
    }
}

type KeyboardHidHost<'d, T> = HidHost<'d, BusHandle<'d, Allocator<'d, T>>>;

pub struct Keyboard<'d, T: SealedHostInstance> {
    hid_host: &'d mut KeyboardHidHost<'d, T>,
}

impl<'d, T: SealedHostInstance> Keyboard<'d, T> {
    pub async fn setup(hid_host: &'d mut KeyboardHidHost<'d, T>) -> Self {
        hid_host
            .set_protocol(PROTOCOL_BOOT)
            .await
            .expect("failed to set keyboard to boot mode");
        Self { hid_host }
    }

    async fn push_event(&self, event: KeyEvent) {
        // TODO send events to an actual queue
        defmt::debug!("new key event {:?}", event);
    }

    pub async fn entry(&mut self) -> ! {
        let mut current_keys: [Option<Key>; 6] = [None; 6];
        let mut previous_keys: [Option<Key>; 6] = [None; 6];
        loop {
            match self.hid_host.read_keyboard().await {
                Ok(Some(report)) => {
                    // process new report
                    report_to_keys(&report, &mut current_keys);
                    let modifiers = Modifiers(report.modifiers);

                    // send events for released keys
                    for prev_key in previous_keys {
                        if let Some(prev_key) = prev_key {
                            let mut exists = false;
                            for current_key in current_keys {
                                if let Some(current_key) = current_key
                                    && current_key == prev_key
                                {
                                    exists = true;
                                    break;
                                }
                            }
                            if !exists {
                                self.push_event(KeyEvent {
                                    key: prev_key,
                                    action: Action::Release,
                                    modifiers,
                                })
                                .await;
                            }
                        }
                    }

                    // send events for new keys
                    for current_key in current_keys {
                        if let Some(current_key) = current_key {
                            let mut is_repeat = false;
                            for prev_key in previous_keys {
                                if let Some(prev_key) = prev_key {
                                    if prev_key == current_key {
                                        is_repeat = true;
                                        break;
                                    }
                                }
                            }
                            if !is_repeat {
                                // key is new
                                self.push_event(KeyEvent {
                                    key: current_key,
                                    action: Action::Press,
                                    modifiers,
                                })
                                .await;
                            }
                        }
                    }

                    // replace previous key state for next time
                    previous_keys.copy_from_slice(&current_keys);
                }
                Ok(None) => {}
                Err(e) => {
                    defmt::error!("Keyboard read failed: {:?}", e);
                }
            }
        }
    }
}
