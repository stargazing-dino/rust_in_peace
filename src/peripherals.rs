//! Peripheral allocation for dual-core battle bot system
//!
//! This module handles splitting the RP2040's hardware peripherals between
//! two CPU cores. We do this to separate concerns:
//!
//! - Core 0: Handles user input
//! - Core 1: Manages real-time motor control and hardware actuation
//!
//! The macro system groups related peripherals into logical units (controller,
//! motors, sensors, etc.), asserting at compile time no shared resources.

use embassy_rp::{peripherals::CORE1, Peripherals};

macro_rules! make_peripherals {
    ($name:ident, ($($pin:ident), *)) => {
        paste::paste! {
            #[allow(non_snake_case)]
            pub struct $name {
                $(pub $pin: embassy_rp::peripherals::$pin,)*
            }

            macro_rules! [<$name:snake>] {
                ($p:ident) => {{
                    use crate::peripherals::*;
                    $name {
                        $($pin: $p.$pin,)*
                    }
                }}
            }
        }
    };
}

make_peripherals! {
    PeripheralsController,
    (SPI1, PIN_12, PIN_13, PIN_14, PIN_15)  // PS2 controller SPI
}

make_peripherals! {
    PeripheralsPs2Led,
    (PIN_22)  // PS2 connection status LED (Core 0)
}

make_peripherals! {
    PeripheralsStateLed,
    (PIN_25)  // Bot state LED (Core 1)
}

make_peripherals! {
    PeripheralsMotor,
    (PWM_SLICE0, PWM_SLICE3, PIN_16, PIN_17, PIN_18, PIN_19, PIN_7, PIN_8, PIN_9)  // Dual motor drivers (BR and FL)
}

make_peripherals! {
    PeripheralsServo,
    (PWM_SLICE5, PIN_26)  // Servo control
}

make_peripherals! {
    PeripheralsWeapon,
    (PWM_SLICE1, PWM_SLICE2, PIN_20, PIN_21)  // Future weapon systems
}

make_peripherals! {
    PeripheralsSensors,
    (I2C0, PIN_0, PIN_1, ADC, PIN_27, PIN_28)  // Future sensors (IMU, current sensing, etc.)
}

pub struct Peripherals0 {
    pub controller: PeripheralsController,
    pub ps2_led: PeripheralsPs2Led,
}

pub struct Peripherals1 {
    pub motor: PeripheralsMotor,
    pub servo: PeripheralsServo,
    pub state_led: PeripheralsStateLed,
    pub weapon: PeripheralsWeapon,
    pub sensors: PeripheralsSensors,
}

pub fn split_peripherals(p: Peripherals) -> (CORE1, Peripherals0, Peripherals1) {
    (
        p.CORE1,
        Peripherals0 {
            controller: peripherals_controller!(p),
            ps2_led: peripherals_ps2_led!(p),
        },
        Peripherals1 {
            motor: peripherals_motor!(p),
            servo: peripherals_servo!(p),
            state_led: peripherals_state_led!(p),
            weapon: peripherals_weapon!(p),
            sensors: peripherals_sensors!(p),
        },
    )
}
