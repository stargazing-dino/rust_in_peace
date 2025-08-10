//! Motor and servo control task (Core 1)

use defmt::*;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Receiver;

use crate::config::*;
use crate::ps2_input::{bits_to_buttons, ControllerData};
use crate::servo_controller::ServoController;
use crate::tank_drive_controller::TankDriveController;
use crate::peripherals::Peripherals1;

#[derive(Clone, Copy, Debug, Format, PartialEq)]
pub enum BotState {
    Idle,
    Armed,
    Combat,
    Emergency,
}

#[embassy_executor::task]
pub async fn motor_task(
    p1: Peripherals1,
    controller_receiver: Receiver<'static, CriticalSectionRawMutex, ControllerData, 8>,
) {
    info!("Motor control task starting...");
    
    // Bot state LED
    let mut state_led = Output::new(p1.state_led.PIN_25, Level::Low);

    let mut tank_drive = TankDriveController::new(
        p1.motor.PWM_SLICE0,
        p1.motor.PIN_16,
        p1.motor.PIN_17,
        p1.motor.PIN_18,
        p1.motor.PWM_SLICE3,
        p1.motor.PIN_7,
        p1.motor.PIN_9,
        p1.motor.PIN_8,
        p1.motor.PIN_19,
    );

    let mut servo = ServoController::new(p1.servo.PWM_SLICE5, p1.servo.PIN_26);

    let mut current_state = BotState::Idle;
    let mut led_timer = 0u32;

    loop {
        let controller_data = controller_receiver.receive().await;
        let buttons = bits_to_buttons(controller_data.buttons);

        match current_state {
            BotState::Idle => {
                tank_drive.stop();
                state_led.set_low(); // LED off when idle
                
                if buttons.start() {
                    current_state = BotState::Armed;
                    info!("ARMED");
                    led_timer = 0;
                }
            }
            BotState::Armed => {
                // Slow blink for armed state
                led_timer += 1;
                if led_timer % 30 == 0 {
                    state_led.toggle();
                }
                
                if buttons.select() {
                    current_state = BotState::Idle;
                    tank_drive.stop();
                    info!("IDLE");
                } else if controller_data.l2_pressure > COMBAT_MODE_PRESSURE {
                    current_state = BotState::Combat;
                    info!("COMBAT MODE");
                    led_timer = 0;
                } else {
                    process_movement(&controller_data, &mut tank_drive);
                    process_servo(&controller_data, &mut servo);
                }
            }
            BotState::Combat => {
                // Fast blink for combat mode
                led_timer += 1;
                if led_timer % 6 == 0 {
                    state_led.toggle();
                }
                
                if buttons.select() {
                    current_state = BotState::Armed;
                    info!("ARMED");
                    led_timer = 0;
                } else {
                    process_movement(&controller_data, &mut tank_drive);
                    process_servo(&controller_data, &mut servo);
                }
            }
            BotState::Emergency => {
                tank_drive.stop();
                tank_drive.disable();
                state_led.set_high(); // Solid LED for emergency
                
                if buttons.start() && buttons.select() {
                    tank_drive.enable();
                    current_state = BotState::Idle;
                    info!("Emergency cleared, IDLE");
                }
            }
        }
    }
}

fn process_movement(data: &ControllerData, tank_drive: &mut TankDriveController) {
    let x_raw = data.left_stick_x as i16 - 128;
    let y_raw = 128 - data.left_stick_y as i16;

    let x = if x_raw.abs() < 10 {
        0
    } else {
        (x_raw * 100 / 128).clamp(-100, 100) as i8
    };
    let y = if y_raw.abs() < 10 {
        0
    } else {
        (y_raw * 100 / 128).clamp(-100, 100) as i8
    };

    let rx_raw = data.right_stick_x as i16 - 128;
    let spin = if rx_raw.abs() > 20 {
        (rx_raw * 100 / 128).clamp(-100, 100) as i8
    } else {
        0
    };

    if spin != 0 {
        tank_drive.spin(spin);
    } else if x == 0 && y == 0 {
        tank_drive.stop();
    } else {
        tank_drive.drive(x, y);
    }
}

fn process_servo(data: &ControllerData, servo: &mut ServoController) {
    let angle = (data.right_stick_y as u32 * 180) / 255;
    servo.set_angle(angle as u8);
}