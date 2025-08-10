//! State controller - the "brains" that converts PS2 input to hardware events
//! Also contains hardware driver tasks

use defmt::*;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::{Receiver, Sender};
use embassy_time::{Duration, Ticker};

use crate::events::{LedEvent, ServoEvent, TankDriveEvent};
use crate::hardware::{PeripheralsMotor, PeripheralsServo, PeripheralsStateLed};
use crate::input::{bits_to_buttons, ControllerData};
use crate::hardware::{ServoController, TankDriveController};
use crate::utils::process_movement;

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum BotState {
    Idle,
    Combat,
    Emergency,
}

#[embassy_executor::task]
pub async fn state_controller_task(
    controller_receiver: Receiver<'static, CriticalSectionRawMutex, ControllerData, 8>,
    tank_sender: Sender<'static, CriticalSectionRawMutex, TankDriveEvent, 8>,
    servo_sender: Sender<'static, CriticalSectionRawMutex, ServoEvent, 8>,
    led_sender: Sender<'static, CriticalSectionRawMutex, LedEvent, 8>,
) {
    info!("State controller starting...");

    let mut current_state = BotState::Idle;

    loop {
        let controller_data = controller_receiver.receive().await;
        let buttons = bits_to_buttons(controller_data.buttons);

        // State transitions and LED control
        match current_state {
            BotState::Idle => {
                led_sender.send(LedEvent::Off).await;
                tank_sender.send(TankDriveEvent::Stop).await;

                if buttons.start() {
                    current_state = BotState::Combat;
                    info!("COMBAT MODE");
                    led_sender.send(LedEvent::FastBlink).await;
                }
            }
            BotState::Combat => {
                if buttons.select() {
                    current_state = BotState::Idle;
                    info!("IDLE");
                    led_sender.send(LedEvent::Off).await;
                    tank_sender.send(TankDriveEvent::Stop).await;
                } else {
                    // Process movement and servo in combat mode
                    process_movement(&controller_data, &tank_sender).await;
                    let angle = (controller_data.right_stick_y as u32 * 180) / 255;
                    servo_sender.send(ServoEvent::SetAngle(angle as u8)).await;
                }
            }
            BotState::Emergency => {
                led_sender.send(LedEvent::Solid).await;
                tank_sender.send(TankDriveEvent::Disable).await;

                if buttons.start() && buttons.select() {
                    tank_sender.send(TankDriveEvent::Enable).await;
                    current_state = BotState::Idle;
                    info!("Emergency cleared, IDLE");
                }
            }
        }
    }
}

#[embassy_executor::task]
pub async fn tank_driver_task(
    motor_peripherals: PeripheralsMotor,
    tank_receiver: Receiver<'static, CriticalSectionRawMutex, TankDriveEvent, 8>,
) {
    info!("Tank driver task starting...");

    let mut tank_drive = TankDriveController::new(
        motor_peripherals.PWM_SLICE0,
        motor_peripherals.PIN_16,
        motor_peripherals.PIN_17,
        motor_peripherals.PIN_18,
        motor_peripherals.PWM_SLICE3,
        motor_peripherals.PIN_7,
        motor_peripherals.PIN_9,
        motor_peripherals.PIN_8,
        motor_peripherals.PIN_19,
    );

    loop {
        let event = tank_receiver.receive().await;

        match event {
            TankDriveEvent::Move { x, y } => {
                tank_drive.drive(x, y);
            }
            TankDriveEvent::Spin(speed) => {
                tank_drive.spin(speed);
            }
            TankDriveEvent::Stop => {
                tank_drive.stop();
            }
            TankDriveEvent::Enable => {
                tank_drive.enable();
            }
            TankDriveEvent::Disable => {
                tank_drive.stop();
                tank_drive.disable();
            }
        }
    }
}

#[embassy_executor::task]
pub async fn servo_driver_task(
    servo_peripherals: PeripheralsServo,
    servo_receiver: Receiver<'static, CriticalSectionRawMutex, ServoEvent, 8>,
) {
    info!("Servo driver task starting...");

    let mut servo = ServoController::new(
        servo_peripherals.PWM_SLICE5,
        servo_peripherals.PIN_26,
    );

    loop {
        let event = servo_receiver.receive().await;

        match event {
            ServoEvent::SetAngle(angle) => {
                servo.set_angle(angle);
            }
        }
    }
}

#[embassy_executor::task]
pub async fn led_driver_task(
    led_peripherals: PeripheralsStateLed,
    led_receiver: Receiver<'static, CriticalSectionRawMutex, LedEvent, 8>,
) {
    info!("LED driver task starting...");

    let mut led = Output::new(led_peripherals.PIN_25, Level::Low);
    let mut current_pattern = LedEvent::Off;
    let mut ticker = Ticker::every(Duration::from_millis(100));

    loop {
        // Check for new LED pattern
        if let Ok(event) = led_receiver.try_receive() {
            current_pattern = event;
            ticker = match event {
                LedEvent::SlowBlink => Ticker::every(Duration::from_millis(500)),
                LedEvent::FastBlink => Ticker::every(Duration::from_millis(100)),
                _ => Ticker::every(Duration::from_millis(100)),
            };
        }

        // Execute current pattern
        match current_pattern {
            LedEvent::Off => {
                led.set_low();
                ticker.next().await;
            }
            LedEvent::Solid => {
                led.set_high();
                ticker.next().await;
            }
            LedEvent::SlowBlink | LedEvent::FastBlink => {
                led.toggle();
                ticker.next().await;
            }
        }
    }
}

