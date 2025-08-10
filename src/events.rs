//! Event types for Core 1 hardware control

use defmt::Format;

/// Events for controlling the tank drive motors
#[derive(Clone, Copy, Debug, Format)]
pub enum TankDriveEvent {
    /// Drive with x/y coordinates (-100 to 100)
    Move { x: i8, y: i8 },
    /// Spin in place (-100 to 100)
    Spin(i8),
    /// Stop all motors
    Stop,
    /// Enable motor drivers (after emergency)
    Enable,
    /// Disable motor drivers (for emergency)
    Disable,
}

/// Events for controlling the servo
#[derive(Clone, Copy, Debug, Format)]
pub enum ServoEvent {
    /// Set servo angle (0-180 degrees)
    SetAngle(u8),
}

/// Events for LED state indication
#[derive(Clone, Copy, Debug, Format)]
pub enum LedEvent {
    /// Turn LED off
    Off,
    /// Slow blink pattern
    SlowBlink,
    /// Fast blink pattern
    FastBlink,
    /// Solid on
    Solid,
}