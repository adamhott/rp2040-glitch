#![no_std]
#![no_main]

use cortex_m_rt::entry;
use rp2040_hal as hal;
use hal::{
    clocks::init_clocks_and_plls,
    gpio::{Pins, FunctionUart},
    pac,
    sio::Sio,
    uart::{DataBits, StopBits, UartConfig, UartPeripheral},
    watchdog::Watchdog,
    Clock,
    fugit::RateExtU32,
};
use panic_probe as _;

const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

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

    // Initialize the UART
    let sio = Sio::new(pac.SIO);
    let pins = Pins::new(pac.IO_BANK0, pac.PADS_BANK0, sio.gpio_bank0, &mut resets);
    let tx_pin = pins.gpio0.into_function::<FunctionUart>();
    let rx_pin = pins.gpio1.into_function::<FunctionUart>();

    let uart_pins = (tx_pin, rx_pin);
    let uart = UartPeripheral::new(pac.UART0, uart_pins, &mut resets)
        .enable(
            UartConfig::new(9600_u32.Hz(), DataBits::Eight, None, StopBits::One),
            clocks.peripheral_clock.freq(),
        )
        .unwrap();
    
    // Secret password (in a real scenario, do not hardcode sensitive data)
    let secret_password = b"pico123";
    let mut user_input = [0u8; 7]; // Buffer to hold the user input

    // Prompt user for password
    uart.write_full_blocking(b"\nEnter password: \n");

    // Loop to read the input and check password
    let mut idx = 0;

    loop {
        // Read input byte by byte from UART (blocking)
        let mut byte = [0u8; 1];
        let _ = uart.read_full_blocking(&mut byte);

        // Echo the received character back to the terminal
        uart.write_full_blocking(&byte);

        // Store the received byte in the user input buffer
        if idx < user_input.len() {
            user_input[idx] = byte[0];
            idx += 1;
        }

        // Once we have enough input to compare
        if idx == user_input.len() {
            // Compare the entered password with the secret
            if &user_input == secret_password {
                uart.write_full_blocking(b"\nUnlock successful!\n");
                unlock_success();
            } else {
                uart.write_full_blocking(b"\nIncorrect password. Locked!\n");
                lock_failure();
            }
            idx = 0; // Reset index to prompt for password again
            uart.write_full_blocking(b"\nEnter password: \n"); // Prompt for password again
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
