//! Simplified PS2 DualShock 2 controller test with motor control
//! Using pscontroller-rs with Embassy on RP2350

#![no_std]
#![no_main]

use defmt::*;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{self, Spi};
use embassy_time::{Instant, Timer};
use pscontroller_rs::{Device, PlayStationPort, dualshock::ControlDS};
use {defmt_rtt as _, panic_probe as _};

// Program metadata for `picotool info`
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"PS2 Controller Test"),
    embassy_rp::binary_info::rp_program_description!(
        c"Test PS2 DualShock 2 controller with rumble motor control"
    ),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    // LED for status indication
    let mut led = Output::new(p.PIN_16, Level::Low);

    // SPI configuration for PS2 controllers
    // PlayStation controllers use SPI mode 3 (CPOL=1, CPHA=1)
    let mut config = spi::Config::default();
    config.frequency = 10_000;
    config.polarity = spi::Polarity::IdleHigh;
    config.phase = spi::Phase::CaptureOnSecondTransition;

    // Setup SPI pins - same configuration as original
    // GP12 - MISO (controller DATA) - needs pull-up resistor (1k-10k to 3.3V)
    // GP15 - MOSI (controller CMD)
    // GP14 - SCK (controller CLK)
    // GP13 - CS (controller ATT)
    let spi = Spi::new_blocking(
        p.SPI1,   // Use SPI1
        p.PIN_14, // SCK
        p.PIN_15, // MOSI
        p.PIN_12, // MISO
        config,
    );

    // Chip select pin
    let cs = Output::new(p.PIN_13, Level::High);

    // Create PlayStation port
    let mut psp = PlayStationPort::new(spi, Some(cs));

    info!("PS2 DualShock 2 controller starting...");

    // Motor control state
    let mut small_motor = false;
    let mut big_motor: u8 = 0;

    psp.enable_pressure().unwrap();

    loop {
        // Create motor command
        let motor_cmd = ControlDS::new(small_motor, big_motor);

        // Read controller input with motor command
        let Ok(device) = psp.read_input(Some(&motor_cmd)) else {
            info!("Error reading controller input, resetting motors");
            // Controller read error - no controller connected or communication error
            // Reset motors
            small_motor = false;
            big_motor = 0;

            Timer::after_millis(100).await; // Wait before retrying

            continue;
        };

        // Blink LED to show communication
        led.set_high();
        Timer::after_millis(10).await;
        led.set_low();

        let Device::DualShock2(controller) = device else {
            // Not a DualShock 2, skip to next iteration
            info!("Not a DualShock 2 controller, skipping...");
            continue;
        };

        // DualShock 2 with pressure sensitivity
        let buttons = controller.buttons;
        info!("DualShock 2 (Pressure mode):");
        info!(
            "  Left Stick: X={:02x}, Y={:02x}",
            controller.lx, controller.ly
        );
        info!(
            "  Right Stick: X={:02x}, Y={:02x}",
            controller.rx, controller.ry
        );

        // Show pressure values for face buttons
        let x_pressure = controller.pressures[6];
        let o_pressure = controller.pressures[5];
        let square_pressure = controller.pressures[7];
        let triangle_pressure = controller.pressures[4];

        if x_pressure > 0 || o_pressure > 0 || square_pressure > 0 || triangle_pressure > 0 {
            info!(
                "  Button Pressures: X={:02x}, O={:02x}, Square={:02x}, Triangle={:02x}",
                x_pressure, o_pressure, square_pressure, triangle_pressure
            );
        }

        // Show pressure values for shoulder buttons
        let l1_pressure = controller.pressures[2];
        let r1_pressure = controller.pressures[3];
        let l2_pressure = controller.pressures[0];
        let r2_pressure = controller.pressures[1];

        if l1_pressure > 0 || r1_pressure > 0 || l2_pressure > 0 || r2_pressure > 0 {
            info!(
                "  Shoulder Pressures: L1={:02x}, R1={:02x}, L2={:02x}, R2={:02x}",
                l1_pressure, r1_pressure, l2_pressure, r2_pressure
            );
        }

        // Advanced motor control using pressure sensitivity
        // L2 pressure controls small motor
        small_motor = l2_pressure > 30; // Threshold for activation

        // R2 pressure controls big motor strength
        big_motor = if r2_pressure > 30 {
            // Scale pressure to motor strength (30-255 pressure -> 0-255 motor)
            ((r2_pressure.saturating_sub(30) as u16 * 255) / 225) as u8
        } else {
            0
        };

        // Alternative: Use face button pressure for effects
        // X button = pulse effect
        if x_pressure > 100 {
            // Create pulsing effect
            big_motor = if (Instant::now().as_millis() / 100) % 2 == 0 {
                x_pressure
            } else {
                0
            };
        }

        if small_motor || big_motor > 0 {
            info!(
                "Motors active: small={}, big={:02x}",
                small_motor, big_motor
            );
        }

        // Show stick button states (L3/R3)
        if buttons.l3() || buttons.r3() {
            info!("  Stick buttons: L3={}, R3={}", buttons.l3(), buttons.r3());
        }

        // Poll at ~60Hz for smooth response
        Timer::after_millis(16).await;
    }
}
