#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::port::mode::{Output, PwmOutput};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer1Pwm};
use core::cell::Cell;

// Configuration Constants
const MOVE_FORWARD: bool = true;
const BAUD_RATE: u32 = 57600;
const MAX_SPEED: u8 = 255; // Adjusted for a more noticeable range (0-255)
const RAMP_TIME_MS: u32 = 3000;
const FULL_SPEED_TIME_MS: u32 = 5000;
const UPDATE_INTERVAL_MS: u32 = 20; // How often to update PWM (50Hz update rate)

static ENCODER_COUNT: avr_device::interrupt::Mutex<Cell<i32>> =
    avr_device::interrupt::Mutex::new(Cell::new(0));

static ENCODER_LAST_STATE: avr_device::interrupt::Mutex<Cell<u8>> =
    avr_device::interrupt::Mutex::new(Cell::new(0));

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

    fn set_direction(&mut self, forward: bool) {
        if forward {
            self.backward.set_low();
            self.forward.set_high();
        } else {
            self.forward.set_low();
            self.backward.set_high();
        }
    }

    fn set_speed(&mut self, speed: u8) {
        self.pwm.set_duty(speed);
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, BAUD_RATE);

    let timer1 = Timer1Pwm::new(dp.TC1, Prescaler::Prescale64);
    let mut pwm_output = pins.d9.into_output().into_pwm(&timer1);
    pwm_output.enable();

    let mut motor = Motor {
        forward: pins.d5.into_output().downgrade(),
        backward: pins.d6.into_output().downgrade(),
        pwm: pwm_output,
    };
    motor.stop();

    pins.d2.into_pull_up_input();
    pins.d3.into_pull_up_input();

    // Initialize encoder state
    let initial_pind = unsafe { &*arduino_hal::pac::PORTD::PTR }.pind().read().bits();
    let initial_state = ((initial_pind >> 2) & 1) << 1 | ((initial_pind >> 3) & 1);

    avr_device::interrupt::free(|cs| {
        ENCODER_LAST_STATE.borrow(cs).set(initial_state);
    });

    unsafe {
        dp.EXINT.eicra().modify(|_, w| {
            w.isc0().bits(0x01); // INT0 Any change
            w.isc1().bits(0x01)  // INT1 Any change
        });
        dp.EXINT.eimsk().modify(|_, w| w.int0().set_bit().int1().set_bit());
        avr_device::interrupt::enable();
    }

    let _ = ufmt::uwriteln!(&mut serial, "--- INITIALIZING ---\r");
    arduino_hal::delay_ms(1000);

    // Set direction once
    motor.set_direction(MOVE_FORWARD);

    // --- 1. RAMP UP (3 Seconds) ---
    let _ = ufmt::uwriteln!(&mut serial, "--- RAMPING UP ---\r");
    let ramp_steps = RAMP_TIME_MS / UPDATE_INTERVAL_MS;
    for i in 0..=ramp_steps {
        let speed = ((i as u32 * MAX_SPEED as u32) / ramp_steps) as u8;
        motor.set_speed(speed);

        log_encoder(&mut serial);
        arduino_hal::delay_ms(UPDATE_INTERVAL_MS);
    }

    // --- 2. FULL SPEED (1 Second) ---
    let _ = ufmt::uwriteln!(&mut serial, "--- FULL SPEED ---\r");
    let plateau_steps = FULL_SPEED_TIME_MS / UPDATE_INTERVAL_MS;
    motor.set_speed(MAX_SPEED);
    for _ in 0..plateau_steps {
        log_encoder(&mut serial);
        arduino_hal::delay_ms(UPDATE_INTERVAL_MS);
    }

    // --- 3. RAMP DOWN (3 Seconds) ---
    let _ = ufmt::uwriteln!(&mut serial, "--- RAMPING DOWN ---\r");
    for i in (0..=ramp_steps).rev() {
        let speed = ((i as u32 * MAX_SPEED as u32) / ramp_steps) as u8;
        motor.set_speed(speed);

        log_encoder(&mut serial);
        arduino_hal::delay_ms(UPDATE_INTERVAL_MS);
    }

    motor.stop();
    let _ = ufmt::uwriteln!(&mut serial, "--- FINISHED ---\r");

    loop {
        log_encoder(&mut serial);
        arduino_hal::delay_ms(200);
    }
}

fn log_encoder(serial: &mut arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>) {
    let count = avr_device::interrupt::free(|cs| ENCODER_COUNT.borrow(cs).get());
    let _ = ufmt::uwriteln!(serial, "POS: {}\r", count);
}

fn handle_encoder_interrupt() {
    let pind = unsafe { &*arduino_hal::pac::PORTD::PTR }.pind().read().bits();
    let a = (pind >> 2) & 1;
    let b = (pind >> 3) & 1;
    let current_state = (a << 1) | b;

    avr_device::interrupt::free(|cs| {
        let last_state_cell = ENCODER_LAST_STATE.borrow(cs);
        let count_cell = ENCODER_COUNT.borrow(cs);
        let last_state = last_state_cell.get();

        match (last_state, current_state) {
            (0b00, 0b01) | (0b01, 0b11) | (0b11, 0b10) | (0b10, 0b00) => {
                count_cell.set(count_cell.get() + 1);
            }
            (0b00, 0b10) | (0b10, 0b11) | (0b11, 0b01) | (0b01, 0b00) => {
                count_cell.set(count_cell.get() - 1);
            }
            _ => {}
        }
        last_state_cell.set(current_state);
    });
}

#[avr_device::interrupt(atmega328p)]
fn INT0() { handle_encoder_interrupt(); }

#[avr_device::interrupt(atmega328p)]
fn INT1() { handle_encoder_interrupt(); }

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();
    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);
    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(100);
    }
}