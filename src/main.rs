#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::prelude::*;
use arduino_hal::simple_pwm::*;
use core::cell::Cell;
use panic_halt as _;

// ================= CONFIGURATION =================
const MOVE_FORWARD: bool = true; // true = D5, false = D6
const MOVE_SECONDS: u32 = 5;
const BAUD_RATE: u32 = 57600;
// =================================================

static ENCODER_COUNT: avr_device::interrupt::Mutex<Cell<i32>> =
    avr_device::interrupt::Mutex::new(Cell::new(0));

#[avr_device::interrupt(atmega328p)]
fn INT0() {
    let portd = unsafe { &*arduino_hal::pac::PORTD::PTR };
    let pins = portd.pind().read();
    let pin2 = pins.bits() & (1 << 2) != 0;
    let pin3 = pins.bits() & (1 << 3) != 0;

    avr_device::interrupt::free(|cs| {
        let counter = ENCODER_COUNT.borrow(cs);
        if pin2 == pin3 {
            counter.set(counter.get() + 1);
        } else {
            counter.set(counter.get() - 1);
        }
    });
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);

    // 1. Serial Setup (Match this in RustRover/Serial Monitor!)
    let mut serial = arduino_hal::default_serial!(dp, pins, BAUD_RATE);

    // 2. Pins Setup
    let mut forward_pin = pins.d5.into_output();
    let mut backward_pin = pins.d6.into_output();
    let timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);
    let mut pwm_pin = pins.d9.into_output().into_pwm(&timer1);
    pwm_pin.enable();

    // 3. Encoder Interrupt Setup
    unsafe {
        dp.EXINT.eicra().modify(|_, w| w.isc0().bits(0x01));
        dp.EXINT.eimsk().modify(|_, w| w.int0().set_bit());
        avr_device::interrupt::enable();
    }

    // Ensure stopped
    forward_pin.set_low();
    backward_pin.set_low();
    pwm_pin.set_duty(0);

    let mut last_sent_count = 0;

    // --- Part 1: Initial Idle (1 second) ---
    for _ in 0..100 {
        last_sent_count = print_if_changed(&mut serial, last_sent_count);
        arduino_hal::delay_ms(10);
    }

    // --- Part 2: Move ---
    let _ = ufmt::uwriteln!(&mut serial, "MOVING START...\r");
    if MOVE_FORWARD {
        forward_pin.set_high();
    } else {
        backward_pin.set_high();
    }
    pwm_pin.set_duty(255);

    // Using a simpler loop. 5 seconds = 50 iterations of 100ms.
    // This reduces serial traffic significantly.
    for _ in 0..(MOVE_SECONDS * 10) {
        last_sent_count = print_if_changed(&mut serial, last_sent_count);
        arduino_hal::delay_ms(100);
    }

    // --- Part 3: Force Stop ---
    pwm_pin.set_duty(0);
    forward_pin.set_low();
    backward_pin.set_low();
    let _ = ufmt::uwriteln!(&mut serial, "STOPPED.\r");

    // --- Part 4: Final Loop ---
    loop {
        last_sent_count = print_if_changed(&mut serial, last_sent_count);
        arduino_hal::delay_ms(100);
    }
}

/// Only prints if the encoder value has actually changed.
/// Returns the new count to be stored in last_sent_count.
fn print_if_changed(
    serial: &mut arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>,
    last_count: i32,
) -> i32 {
    let current_count = avr_device::interrupt::free(|cs| ENCODER_COUNT.borrow(cs).get());
    if current_count != last_count {
        let _ = ufmt::uwriteln!(serial, "{}\r", current_count);
    }
    current_count
}
