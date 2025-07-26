//! Motor and servo control task (Core 1)

use defmt::*;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};

use crate::drivers::motor_control::MotorController;
use crate::drivers::servo_control::ServoController;
use crate::messages::*;
use crate::peripherals::Peripherals1;

#[embassy_executor::task]
pub async fn motor_task(
    p1: Peripherals1,
    command_receiver: Receiver<'static, CriticalSectionRawMutex, Core1Command, 8>,
    status_sender: Sender<'static, CriticalSectionRawMutex, StatusReport, 4>,
) {
    info!("Motor control task starting...");

    let mut motor = MotorController::new(
        p1.motor.PWM_SLICE0,
        p1.motor.PIN_16,
        p1.motor.PIN_17,
        p1.motor.PIN_18,
        p1.motor.PIN_19,
    );

    let mut servo = ServoController::new(p1.servo.PWM_SLICE5, p1.servo.PIN_26);

    let mut current_state = BotState::Idle;
    let mut last_motor_active = false;

    loop {
        match command_receiver.receive().await {
            Core1Command::Motor(cmd) => {
                match cmd {
                    MotorCommand::Stop => motor.stop(),
                    MotorCommand::Forward(speed) => {
                        motor.drive_forward(speed);
                        last_motor_active = true;
                    }
                    MotorCommand::Backward(speed) => {
                        motor.drive_backward(speed);
                        last_motor_active = true;
                    }
                    MotorCommand::Brake => motor.brake(),
                }

                if matches!(cmd, MotorCommand::Stop | MotorCommand::Brake) {
                    last_motor_active = false;
                }
            }
            Core1Command::Servo(cmd) => {
                servo.set_angle(cmd.angle);
            }
            Core1Command::System(cmd) => {
                match cmd {
                    SystemCommand::Emergency => {
                        motor.stop();
                        motor.disable();
                        current_state = BotState::Emergency;
                        info!("EMERGENCY STOP");
                    }
                    SystemCommand::Resume => {
                        motor.enable();
                        current_state = BotState::Idle;
                        info!("Resumed");
                    }
                    SystemCommand::SetState(state) => {
                        current_state = state;
                        info!("State changed to {:?}", state);
                    }
                }

                let status = StatusReport {
                    state: current_state,
                    motor_active: last_motor_active,
                    servo_position: 90, // TODO: get actual position
                };
                let _ = status_sender.try_send(status);
            }
        }
    }
}
