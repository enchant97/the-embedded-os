use embassy_rp::usb::host::{Allocator, SealedHostInstance};
use embassy_usb_host::{
    BusHandle,
    class::hid::{HidHost, PROTOCOL_BOOT},
};

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

    pub async fn entry(&mut self) -> ! {
        loop {
            match self.hid_host.read_keyboard().await {
                Ok(Some(r)) => {
                    let mapping: key_mapping::KeyboardReport = r.into();
                    defmt::debug!("Keyboard report: {:?}", mapping);
                }
                Ok(None) => {}
                Err(e) => {
                    defmt::error!("Keyboard read failed: {:?}", e);
                }
            }
        }
    }
}
