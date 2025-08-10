//! Hardware abstraction layer for robot components

pub mod motor_controller;
pub mod peripherals;
pub mod servo_controller;
pub mod tank_drive_controller;

pub use peripherals::{split_peripherals, Peripherals0, Peripherals1};
pub use peripherals::{PeripheralsController, PeripheralsPs2Led, PeripheralsStateLed};
pub use peripherals::{PeripheralsMotor, PeripheralsServo};
pub use servo_controller::ServoController;
pub use tank_drive_controller::TankDriveController;