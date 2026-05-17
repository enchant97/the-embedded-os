use embassy_rp::usb::host::{Allocator, SealedHostInstance};
use embassy_usb_host::{
    BusHandle,
    class::hid::{HidHost, PROTOCOL_BOOT},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum Action {
    Press,
    Release,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, defmt::Format)]
pub enum Key {
    /// printable character
    Char(char),
    /// raw usage id
    Raw(u8),
}

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq, defmt::Format)]
#[repr(transparent)]
pub struct Modifiers(pub u8);

impl Modifiers {
    pub const SHIFT: u8 = 1 << 0;
    pub const CTRL: u8 = 1 << 1;
    pub const ALT: u8 = 1 << 2;
    pub const META: u8 = 1 << 3;

    pub fn shift(self) -> bool {
        self.0 & Self::SHIFT != 0
    }
    pub fn ctrl(self) -> bool {
        self.0 & Self::CTRL != 0
    }
    pub fn alt(self) -> bool {
        self.0 & Self::ALT != 0
    }
    pub fn meta(self) -> bool {
        self.0 & Self::META != 0
    }
}

#[derive(Debug, Copy, Clone, defmt::Format)]
pub struct KeyEvent {
    pub key: Key,
    pub action: Action,
    pub modifiers: Modifiers,
}

fn report_to_keys(report: key_mapping::KeyboardReport, keys: &mut [Option<Key>; 6]) {
    for (i, key) in report.keys.into_iter().enumerate() {
        if key == key_mapping::Keys::None {
            keys[i] = None;
        } else if key == key_mapping::Keys::Spacebar {
            keys[i] = Some(Key::Char(' '));
        } else {
            // TODO make own mapping table
            let mapping = key_mapping::MAPPED_KEYS.get(&(key as u8)).unwrap();
            if mapping.key_type == key_mapping::MappedKeyType::Printable {
                // handle printable keys
                let mut c = mapping
                    .visual
                    .chars()
                    .next()
                    .expect("mapping visual is empty");
                if report.shift {
                    // XXX this does not handle all keys
                    c = c.to_ascii_uppercase();
                }
                keys[i] = Some(Key::Char(c));
            } else {
                // handle non-printable keys
                keys[i] = Some(Key::Raw(key as u8));
            }
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
                Ok(Some(r)) => {
                    // process new report
                    let report: key_mapping::KeyboardReport = r.into();
                    report_to_keys(report, &mut current_keys);
                    let modifiers = Modifiers(report.get_modifer_code());

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
