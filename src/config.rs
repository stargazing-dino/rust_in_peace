//! Configuration constants for the battle bot
//! 
//! This module contains all the magic numbers and configuration values
//! used throughout the system, making them easy to find and modify.

// Controller Configuration
/// PS2 controller SPI frequency in Hz
pub const PS2_SPI_FREQUENCY: u32 = 10_000;

/// Dead zone thresholds for analog sticks (0-255)
pub const STICK_DEAD_ZONE_LOW: u8 = 118;
pub const STICK_DEAD_ZONE_HIGH: u8 = 138;

/// Control loop frequency in Hz
pub const CONTROL_LOOP_HZ: u32 = 60;
pub const CONTROL_LOOP_PERIOD_MS: u64 = 1000 / CONTROL_LOOP_HZ as u64;

/// Controller feedback thresholds
pub const RUMBLE_THRESHOLD: u8 = 30;
pub const RUMBLE_MAX_SUBTRACT: u8 = 30;
pub const RUMBLE_MAX_DIVISOR: u16 = 225;

/// Button pressure thresholds
pub const COMBAT_MODE_PRESSURE: u8 = 100;

// Communication Configuration
/// Channel buffer sizes
pub const COMMAND_CHANNEL_SIZE: usize = 8;
pub const STATUS_CHANNEL_SIZE: usize = 4;

// Timing Configuration
/// LED blink duration in milliseconds
pub const LED_BLINK_MS: u64 = 10;

/// Controller timeout in milliseconds
pub const CONTROLLER_TIMEOUT_MS: u64 = 100;

// Core Configuration
/// Stack size for Core 1 in bytes
pub const CORE1_STACK_SIZE: usize = 8192;

// Pin Mapping Documentation
// Pin assignments for the RP2040
// 
// PS2 Controller (SPI1):
// - PIN_12: MISO (Data from controller)
// - PIN_13: CS/SS (Chip Select)
// - PIN_14: SCK (Clock)
// - PIN_15: MOSI (Commands to controller)
// 
// Motor Driver (TB6612FNG):
// - PIN_16: PWM (Speed control)
// - PIN_17: IN1 (Direction control)
// - PIN_18: IN2 (Direction control)
// - PIN_19: STBY (Standby/Enable)
// 
// Servo:
// - PIN_26: PWM signal
// 
// Status:
// - PIN_22: Status LED
// 
// Future Expansion:
// - PIN_20/21: Weapon system PWM
// - PIN_0/1: I2C for sensors
// - PIN_27/28: ADC for current sensing