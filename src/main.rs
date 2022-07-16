use std::borrow::Borrow;
use std::env;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, select, tick};
use ctrlc::Error;
use uinput::event::keyboard;

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

fn main() -> Result<(), Error> {
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_secs(1));
    let tourbox_port_name = "/dev/ttyACM0";
    let mut args = env::args();
    let tty_path = args.nth(1).unwrap_or_else(|| String::from(tourbox_port_name));
    let mut port = serialport::new(tty_path, 115_200)
        .timeout(Duration::from_millis(10))
        .open().expect("Failed to open port");

    let mut device = uinput::default().unwrap()
        .name("tourbox-rs").unwrap()
        .event(uinput::event::Keyboard::All).unwrap()
        .create().unwrap();

    loop {
        select! {
            recv(ticks) -> _ => {
                println!("working!");
                let mut serial_buf: Vec<u8> = vec![0; 2];
                let have_data = port.read(serial_buf.as_mut_slice());
                if have_data.is_ok() {
                    // kill my own program because I am mad
                    println!("{:?}", serial_buf);
                    device.press(&keyboard::Key::LeftControl).unwrap();
                    device.click(&keyboard::Key::C).unwrap();
                    device.release(&keyboard::Key::LeftControl).unwrap();
                    continue;
                }
                println!("No data!");
            }
            recv(ctrl_c_events) -> _ => {

                println!();
                println!("Goodbye!");
                break;
            }
        }
    }

    Ok(())
}
