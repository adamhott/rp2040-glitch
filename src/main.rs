#![no_std]
#![no_main]

use cortex_m_rt::entry;
use hal::{clocks::init_clocks_and_plls, pac, usb::UsbBus, watchdog::Watchdog};
use panic_probe as _;
use rp2040_hal as hal;
use usb_device::class_prelude::UsbBusAllocator;
use usb_device::prelude::*;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

// Bootloader definition for RP2040
#[link_section = ".boot_loader"]
#[used]
pub static BOOT_LOADER: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

#[entry]
fn main() -> ! {
    // Initialize the peripherals
    let pac = pac::Peripherals::take().unwrap();
    let mut watchdog = Watchdog::new(pac.WATCHDOG);

    // Initialize the clocks
    let mut resets = pac.RESETS;
    let clocks = init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut resets,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Setup USB
    let usb_bus = UsbBusAllocator::new(UsbBus::new(
        pac.USBCTRL_REGS,
        pac.USBCTRL_DPRAM,
        clocks.usb_clock,
        true,
        &mut resets,
    ));
    let mut serial = SerialPort::new(&usb_bus);

    // Build USB device with string descriptors
    let mut usb_dev = UsbDeviceBuilder::new(&usb_bus, UsbVidPid(0x16c0, 0x27dd))
        .device_class(USB_CLASS_CDC)
        .max_packet_size_0(64)
        .expect("Something went wrong.")
        .build();

    // Secret password (in a real scenario, do not hardcode sensitive data)
    let secret_password = b"pico123";
    let mut user_input = [0u8; 7]; // Buffer to hold the user input

    let mut idx = 0;

    loop {
        if usb_dev.poll(&mut [&mut serial]) {
            let mut buf = [0u8; 64];

            // Read data from USB serial
            match serial.read(&mut buf) {
                Ok(count) if count > 0 => {
                    for &b in buf.iter().take(count) {
                        // Echo the received character back to the terminal
                        let _ = serial.write(&[b]);

                        // Store the received byte in the user input buffer
                        if idx < user_input.len() {
                            user_input[idx] = b;
                            idx += 1;
                        }

                        // Once we have enough input to compare
                        if idx == user_input.len() {
                            // Compare the entered password with the secret
                            if &user_input == secret_password {
                                let _ = serial.write(b"\nUnlock successful!\n");
                                unlock_success();
                            } else {
                                let _ = serial.write(b"\nIncorrect password. Locked!\n");
                                lock_failure();
                            }
                            idx = 0; // Reset index to prompt for password again
                            let _ = serial.write(b"\nEnter password: \n"); // Prompt for password again
                        }
                    }
                }
                _ => {}
            }
        }

        watchdog.feed(); // Keep watchdog alive
    }
}

fn unlock_success() {
    // Placeholder: Here you could turn off an LED or perform some action to indicate success
}

fn lock_failure() {
    // Placeholder: Here you could keep an LED on or perform some action to indicate failure
}
