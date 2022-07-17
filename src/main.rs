use std::env;
use std::time::Duration;

use crossbeam_channel::{bounded, Receiver, select, tick};
use ctrlc::Error;
use serialport::SerialPort;
use uinput::event::keyboard;

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(100);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let ctrl_c_events = ctrl_channel()?;
    let ticks = tick(Duration::from_millis(500));
    let tourbox_default_port_name = "/dev/ttyACM0";
    let mut args = env::args();
    let first_argument = 1;
    let tty_path = args
        .nth(first_argument)
        .unwrap_or_else(|| String::from(tourbox_default_port_name));

    let tokio_port = tokio_serial::new(tty_path, 9600);

    let mut device = uinput::default()
        .expect("Cannot open default uinput device")
        .name("tourbox-rs")
        .expect("Cannot name uinput device")
        .event(uinput::event::Keyboard::All)
        .expect("")
        .create()
        .expect("Error creating uinput device for tour box neo");

    let mut stream =
        tokio_serial::SerialStream::open(&tokio_port).expect("Cannot open serial port");

    stream
        .set_timeout(Duration::from_millis(100))
        .expect("Error setting timeout");

    stream
        .set_exclusive(true)
        .expect("Cannot set device as exclusive");

    loop {
        select! {
            recv(ticks) -> _ => {
                let mut vec = vec![0; 32];
                match stream.try_read(&mut vec) {
                    Ok(size) => {
                        // Kill my own program because I am crazy
                        println!("Read {} bytes: {:?}", size, vec);
                        device.press(&keyboard::Key::LeftControl).expect("Cannot send key");
                        device.click(&keyboard::Key::C).expect("Cannot send key");
                        device.synchronize().unwrap();
                    },
                    Err(e) => {
                        println!("Error reading serial port: {}", e);
                    },
                }
            }
            recv(ctrl_c_events) -> _ => {
                println!();
                println!("Goodbye!");
                break;
            }
        }
    }

    return Ok(());
}
