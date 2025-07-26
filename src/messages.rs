//! Message types for inter-core communication

use defmt::Format;

#[derive(Clone, Copy, Debug, Format)]
pub enum MotorCommand {
    Stop,
    Forward(u8),  // 0-100%
    Backward(u8), // 0-100%
    Brake,
}

#[derive(Clone, Copy, Debug, Format)]
pub struct ServoCommand {
    pub angle: u8, // 0-180°
}

#[derive(Clone, Copy, Debug, Format)]
pub enum SystemCommand {
    Emergency,
    Resume,
    SetState(BotState),
}

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum BotState {
    Idle,
    Armed,
    Combat,
    Emergency,
}

#[derive(Clone, Copy, Debug, Format)]
pub struct ControllerData {
    pub left_stick_x: u8,
    pub left_stick_y: u8,
    pub right_stick_x: u8,
    pub right_stick_y: u8,
    pub l2_pressure: u8,
    pub r2_pressure: u8,
    pub buttons: u16,
}

#[derive(Clone, Copy, Debug, Format)]
pub enum Core1Command {
    Motor(MotorCommand),
    Servo(ServoCommand),
    System(SystemCommand),
}

#[derive(Clone, Copy, Debug, Format)]
pub struct StatusReport {
    pub state: BotState,
    pub motor_active: bool,
    pub servo_position: u8,
}
