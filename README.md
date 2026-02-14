arduino-antuator-rs
=================

Rust project for the _Arduino Uno_.

## Build Instructions

> Windows specific instructions [here](https://dev.to/ryankopf/how-to-use-rust-on-arduino-windows-rust-143g)

1. Install prerequisites as described in the [`avr-hal` README] (`avr-gcc`, `avr-libc`, `avrdude`, [`ravedude`]).

2. Run `cargo build` to build the firmware.

3. Run `cargo run` to flash the firmware to a connected board. If `ravedude`
   fails to detect your board, check its documentation at
   <https://crates.io/crates/ravedude>.

4. `ravedude` will open a console session after flashing where you can interact
   with the UART console of your board.

[`avr-hal` README]: https://github.com/Rahix/avr-hal#readme

[`ravedude`]: https://crates.io/crates/ravedude

### Pin Mapping

| Arduino Pin | Role        | Connection                  |
|-------------|-------------|-----------------------------|
| **D2**      | Encoder A   | Hall effect sensor A        |
| **D3**      | Encoder B   | Hall effect sensor B        |
| **D5**      | Motor Dir 1 | Motor Driver IN1 / Forward  |
| **D6**      | Motor Dir 2 | Motor Driver IN2 / Backward |
| **D9**      | Motor PWM   | Motor Driver PWM / Speed    |
| **USB**     | Serial      | 57600 baud console          |

### Software Configuration

Key constants in `src/main.rs`:

* `BAUD_RATE`: `57600` (Matches `Ravedude.toml`).
* `PWM_SPEED`: `0-255` (Default `255` for full speed).
* `MOVE_SECONDS`: Duration of execution.
* `MOVE_FORWARD`: Boolean to toggle direction.

### Theory of Operation

The system uses a **Quadrature Encoder State Machine**. Interrupts are triggered on any logical change of pins D2 (
INT0) and D3 (INT1).
The transition from the previous state to the current state determines if the actuator moved forward or backward,
allowing for precise relative position tracking even if power is toggled during movement.

## Hardware

See [HARDWARE.md](docs/HARDWARE.md) for detailed part numbers and wiring schematics.

### Wiring Pictures

See pictures of how it's wired in [/docs/wiring-pics](docs/wiring-pics).

![Wiring Overview](docs/wiring-pics/wiring.jpg)
