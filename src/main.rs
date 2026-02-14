#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

use arduino_hal::port::mode::{Output, PwmOutput};
use arduino_hal::port::Pin;
use arduino_hal::simple_pwm::{IntoPwmPin, Prescaler, Timer1Pwm};
use core::cell::Cell;

const MOVE_FORWARD: bool = true;
const MOVE_SECONDS: u32 = 5;
const BAUD_RATE: u32 = 57600;
const PWM_SPEED: u8 = 255;

static ENCODER_COUNT: avr_device::interrupt::Mutex<Cell<i32>> =
    avr_device::interrupt::Mutex::new(Cell::new(0));

// --- HARDWARE TYPE DEFINITIONS ---
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

    // Setup External Interrupt (INT0 on Pin D2)
    unsafe {
        // Trigger INT0 on any logical change
        dp.EXINT.eicra().modify(|_, w| w.isc0().bits(0x01));
        dp.EXINT.eimsk().modify(|_, w| w.int0().set_bit());
        avr_device::interrupt::enable();
    }

    let _ = ufmt::uwriteln!(&mut serial, "--- GETTING READY ---");

    // --- Idle (1 second) ---
    for _ in 0..10 {
        log_encoder(&mut serial);
        arduino_hal::delay_ms(100);
    }

    // --- Move actuator ---
    let _ = ufmt::uwriteln!(&mut serial, "--- READY 2 MOVE IT ---");
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
    let _ = ufmt::uwriteln!(&mut serial, "--- STOPPED ---");

    // --- Keep logging encoder output ---
    loop {
        log_encoder(&mut serial);
        arduino_hal::delay_ms(100);
    }
}

fn get_encoder_count() -> i32 {
    avr_device::interrupt::free(|cs| ENCODER_COUNT.borrow(cs).get())
}

fn log_encoder(serial: &mut arduino_hal::hal::usart::Usart0<arduino_hal::DefaultClock>) {
    let count = get_encoder_count();
    let _ = ufmt::uwriteln!(serial, "{}\r", count);
}

// Event listener for hall effect sensor pulse
#[avr_device::interrupt(atmega328p)]
fn INT0() {
    let pind = unsafe { &*arduino_hal::pac::PORTD::PTR }
        .pind()
        .read()
        .bits();

    let pin2 = (pind & (1 << 2)) != 0;
    let pin3 = (pind & (1 << 3)) != 0;

    avr_device::interrupt::free(|cs| {
        let counter = ENCODER_COUNT.borrow(cs);
        if pin2 == pin3 {
            counter.set(counter.get() + 1);
        } else {
            counter.set(counter.get() - 1);
        }
    });
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
    let _ = ufmt::uwriteln!(&mut serial, "\nPANIC\n");

    loop {
        led.toggle();
        arduino_hal::delay_ms(150);
    }
}
