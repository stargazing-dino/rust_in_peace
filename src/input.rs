//! PS2 Controller input and receiver LED tasks (Core 0)

use defmt::*;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{self, Spi};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Sender;
use embassy_sync::signal::Signal;
use embassy_time::Timer;
use pscontroller_rs::{dualshock::ControlDS, Device, PlayStationPort};
use pscontroller_rs::classic::GamepadButtons;

use crate::config::*;
use crate::hardware::{PeripheralsController, PeripheralsPs2Led};

/// Data from the PS2 controller sent to motor task
#[derive(Clone, Copy, Debug, Format)]
pub struct ControllerData {
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub l2_pressure: u8,
    pub r2_pressure: u8,
    pub buttons: u16,  // Raw button bits from PS2 controller
}

/// Helper to safely convert button bits to GamepadButtons
/// This keeps PS2-specific conversions in the controller module
pub fn bits_to_buttons(bits: u16) -> GamepadButtons {
    // Safe because GamepadButtons is repr(C) with single u16 field
    unsafe { core::mem::transmute(bits) }
}

#[embassy_executor::task]
pub async fn ps2_reader_task(
    controller_peripherals: PeripheralsController,
    controller_sender: Sender<'static, CriticalSectionRawMutex, ControllerData, 8>,
    led_signal: &'static Signal<CriticalSectionRawMutex, ()>,
) {
    info!("PS2 reader task starting...");

    let mut config = spi::Config::default();
    config.frequency = PS2_SPI_FREQUENCY;
    config.polarity = spi::Polarity::IdleHigh;
    config.phase = spi::Phase::CaptureOnSecondTransition;

    let spi = Spi::new_blocking(
        controller_peripherals.SPI1,
        controller_peripherals.PIN_14, // SCK
        controller_peripherals.PIN_15, // MOSI
        controller_peripherals.PIN_12, // MISO
        config,
    );

    let cs = Output::new(controller_peripherals.PIN_13, Level::High);
    let mut psp = PlayStationPort::new(spi, Some(cs));

    let mut small_motor = false;
    let mut big_motor: u8 = 0;

    psp.enable_pressure().unwrap();

    loop {
        let motor_cmd = ControlDS::new(small_motor, big_motor);

        let Ok(device) = psp.read_input(Some(&motor_cmd)) else {
            info!("Controller read error");
            Timer::after_millis(CONTROLLER_TIMEOUT_MS).await;
            continue;
        };

        let Device::DualShock2(controller) = device else {
            info!("Not a DualShock 2 controller");
            continue;
        };

        // Good connection - signal LED pulse
        led_signal.signal(());
        
        let controller_data = ControllerData {
            left_stick_x: controller.lx,
            left_stick_y: controller.ly,
            right_stick_x: controller.rx,
            right_stick_y: controller.ry,
            l2_pressure: controller.pressures[0],
            r2_pressure: controller.pressures[1],
            buttons: controller.buttons.bits(),  // Send raw bits
        };

        controller_sender.send(controller_data).await;
        
        // Simple rumble based on triggers
        small_motor = controller_data.l2_pressure > RUMBLE_THRESHOLD;
        big_motor = if controller_data.r2_pressure > RUMBLE_THRESHOLD {
            ((controller_data
                .r2_pressure
                .saturating_sub(RUMBLE_MAX_SUBTRACT) as u16
                * 255)
                / RUMBLE_MAX_DIVISOR) as u8
        } else {
            0
        };
    }
}

#[embassy_executor::task]
pub async fn receiver_led_task(
    ps2_led: PeripheralsPs2Led,
    led_signal: &'static Signal<CriticalSectionRawMutex, ()>,
) {
    info!("Receiver LED task starting...");

    let mut led = Output::new(ps2_led.PIN_22, Level::Low);
    
    loop {
        led_signal.wait().await;
        
        // Pulse LED for 500ms
        led.set_high();
        Timer::after_millis(500).await;
        led.set_low();
    }
}