#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::port::mode::{Output, PwmOutput};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer1Pwm};
use core::cell::Cell;

const MOVE_FORWARD: bool = false;
const MOVE_SECONDS: u32 = 5;
const BAUD_RATE: u32 = 57600;
const PWM_SPEED: u8 = 255;

static ENCODER_COUNT: avr_device::interrupt::Mutex<Cell<i32>> =
    avr_device::interrupt::Mutex::new(Cell::new(0));

// Stores the previous state of the pins (bits: 000000BA)
static ENCODER_LAST_STATE: avr_device::interrupt::Mutex<Cell<u8>> =
    avr_device::interrupt::Mutex::new(Cell::new(0));

// On the Uno, Pin D9 is PB1.
type MotorPwmPin = Pin<PwmOutput<Timer1Pwm>, arduino_hal::hal::port::PB1>;

struct Motor {
    forward: Pin<Output>,
    backward: Pin<Output>,
    pwm: MotorPwmPin,
}

impl Motor {
    fn stop(&mut self) {
        self.forward.set_low();
        self.backward.set_low();
        self.pwm.set_duty(0);
    }
    fn move_forward(&mut self, speed: u8) {
        self.backward.set_low();
        self.forward.set_high();
        self.pwm.set_duty(speed);
    }
    fn move_backward(&mut self, speed: u8) {
        self.forward.set_low();
        self.backward.set_high();
        self.pwm.set_duty(speed);
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, BAUD_RATE);

    // Setup Motor & PWM (Timer 1 handles Pin D9)
    let timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);

    // Set PIN 9 to output & pwm
    let mut pwm_output = pins.d9.into_output().into_pwm(&timer1);
    pwm_output.enable();

    // Set PIN 5 & 6 as motor forward & backward pins, PIN 9 as speed control
    let mut motor = Motor {
        forward: pins.d5.into_output().downgrade(),
        backward: pins.d6.into_output().downgrade(),
        pwm: pwm_output,
    };
    motor.stop();

    // 2. Setup Pins with Internal Pull-ups
    pins.d2.into_pull_up_input();
    pins.d3.into_pull_up_input();

    let initial_pind = unsafe { &*arduino_hal::pac::PORTD::PTR }.pind().read().bits();
    let a = (initial_pind >> 2) & 1;
    let b = (initial_pind >> 3) & 1;
    let initial_state = (a << 1) | b;

    avr_device::interrupt::free(|cs| {
        ENCODER_LAST_STATE.borrow(cs).set(initial_state);
    });

    // 3. Setup Interrupts on INT0 (D2) AND INT1 (D3)
    unsafe {
        // Trigger INT0 and INT1 on Any Logical Change (0x01)
        dp.EXINT.eicra().modify(|_, w| {
            w.isc0().bits(0x01); // Pin D2
            w.isc1().bits(0x01) // Pin D3
        });
        // Enable both interrupt masks
        dp.EXINT
            .eimsk()
            .modify(|_, w| w.int0().set_bit().int1().set_bit());

        avr_device::interrupt::enable();
    }

    let _ = ufmt::uwriteln!(&mut serial, "--- GETTING READY ---\r");

    // --- Idle (1 second) ---
    for _ in 0..10 {
        log_encoder(&mut serial);
        arduino_hal::delay_ms(100);
    }

    // --- Move actuator ---
    let _ = ufmt::uwriteln!(&mut serial, "--- READY 2 MOVE IT ---\r");
    if MOVE_FORWARD {
        motor.move_forward(PWM_SPEED);
    } else {
        motor.move_backward(PWM_SPEED);
    }

    for _ in 0..(MOVE_SECONDS * 10) {
        log_encoder(&mut serial);
        arduino_hal::delay_ms(100);
    }

    // --- Stop ---
    motor.stop();
    let _ = ufmt::uwriteln!(&mut serial, "--- STOPPED ---\r");

    // --- Keep logging encoder output ---
    loop {
        log_encoder(&mut serial);
        arduino_hal::delay_ms(100);
    }
}

fn log_encoder(serial: &mut arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>) {
    let count = avr_device::interrupt::free(|cs| ENCODER_COUNT.borrow(cs).get());
    let _ = ufmt::uwriteln!(serial, "{}\r", count);
}

/// This is the shared logic for both Pin 2 and Pin 3 interrupts.
/// It uses a standard quadrature lookup table to determine movement.
fn handle_encoder_interrupt() {
    // Read Port D (Pins 0-7)
    let pind = unsafe { &*arduino_hal::pac::PORTD::PTR }
        .pind()
        .read()
        .bits();

    // Extract Pin 2 and Pin 3 states
    let a = (pind >> 2) & 1;
    let b = (pind >> 3) & 1;
    let current_state = (a << 1) | b;

    avr_device::interrupt::free(|cs| {
        let last_state_cell = ENCODER_LAST_STATE.borrow(cs);
        let count_cell = ENCODER_COUNT.borrow(cs);

        let last_state = last_state_cell.get();

        // Quadrature State Machine
        // This handles all valid transitions: 00->01, 01->11, 11->10, 10->00 etc.
        match (last_state, current_state) {
            (0b00, 0b01) | (0b01, 0b11) | (0b11, 0b10) | (0b10, 0b00) => {
                count_cell.set(count_cell.get() - 1);
            }
            (0b00, 0b10) | (0b10, 0b11) | (0b11, 0b01) | (0b01, 0b00) => {
                count_cell.set(count_cell.get() + 1);
            }
            _ => {} // Invalid or no change
        }

        last_state_cell.set(current_state);
    });
}

#[avr_device::interrupt(atmega328p)]
fn INT0() {
    handle_encoder_interrupt();
}

#[avr_device::interrupt(atmega328p)]
fn INT1() {
    handle_encoder_interrupt();
}

// On panic, blink LED
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();
    // Steal control over LED pin
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.d13.into_output();
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);
    let _ = ufmt::uwriteln!(&mut serial, "\nPANIC\n\r");

    loop {
        led.toggle();
        arduino_hal::delay_ms(100);
    }
}
