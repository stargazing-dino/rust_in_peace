//! PS2 Controller input task (Core 0)

use defmt::*;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::spi::{self, Spi};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Sender;
use embassy_time::Timer;
use pscontroller_rs::{dualshock::ControlDS, Device, PlayStationPort};

use crate::config::*;
use crate::messages::*;
use crate::peripherals::Peripherals0;

#[embassy_executor::task]
pub async fn controller_task(
    p0: Peripherals0,
    command_sender: Sender<'static, CriticalSectionRawMutex, Core1Command, 8>,
    status_receiver: embassy_sync::channel::Receiver<
        'static,
        CriticalSectionRawMutex,
        StatusReport,
        4,
    >,
) {
    info!("Controller task starting...");

    let mut led = Output::new(p0.status.PIN_22, Level::Low);

    let mut config = spi::Config::default();
    config.frequency = PS2_SPI_FREQUENCY;
    config.polarity = spi::Polarity::IdleHigh;
    config.phase = spi::Phase::CaptureOnSecondTransition;

    let spi = Spi::new_blocking(
        p0.controller.SPI1,
        p0.controller.PIN_14, // SCK
        p0.controller.PIN_15, // MOSI
        p0.controller.PIN_12, // MISO
        config,
    );

    let cs = Output::new(p0.controller.PIN_13, Level::High);
    let mut psp = PlayStationPort::new(spi, Some(cs));

    let mut small_motor = false;
    let mut big_motor: u8 = 0;
    let mut current_state = BotState::Idle;

    psp.enable_pressure().unwrap();

    loop {
        let motor_cmd = ControlDS::new(small_motor, big_motor);

        let Ok(device) = psp.read_input(Some(&motor_cmd)) else {
            info!("Controller read error");
            command_sender
                .send(Core1Command::Motor(MotorCommand::Stop))
                .await;

            small_motor = false;
            big_motor = 0;
            Timer::after_millis(CONTROLLER_TIMEOUT_MS).await;
            continue;
        };

        led.set_high();
        Timer::after_millis(LED_BLINK_MS).await;
        led.set_low();

        let Device::DualShock2(controller) = device else {
            info!("Not a DualShock 2 controller");
            continue;
        };

        let controller_data = ControllerData {
            left_stick_x: controller.lx,
            left_stick_y: controller.ly,
            right_stick_x: controller.rx,
            right_stick_y: controller.ry,
            l2_pressure: controller.pressures[0],
            r2_pressure: controller.pressures[1],
            buttons: controller.buttons.bits(),
        };

        match current_state {
            BotState::Idle => {
                if controller.buttons.start() {
                    current_state = BotState::Armed;
                    command_sender
                        .send(Core1Command::System(SystemCommand::SetState(
                            BotState::Armed,
                        )))
                        .await;
                    info!("ARMED");
                }
            }
            BotState::Armed => {
                if controller.buttons.select() {
                    current_state = BotState::Idle;
                    command_sender
                        .send(Core1Command::System(SystemCommand::SetState(
                            BotState::Idle,
                        )))
                        .await;
                    command_sender
                        .send(Core1Command::Motor(MotorCommand::Stop))
                        .await;
                    info!("IDLE");
                } else {
                    process_movement(&controller_data, &command_sender).await;

                    if controller.pressures[6] > COMBAT_MODE_PRESSURE {
                        // X button
                        current_state = BotState::Combat;
                        command_sender
                            .send(Core1Command::System(SystemCommand::SetState(
                                BotState::Combat,
                            )))
                            .await;
                        info!("COMBAT MODE");
                    }
                }
            }
            BotState::Combat => {
                process_movement(&controller_data, &command_sender).await;

                if controller.buttons.select() {
                    current_state = BotState::Armed;
                    command_sender
                        .send(Core1Command::System(SystemCommand::SetState(
                            BotState::Armed,
                        )))
                        .await;
                    info!("ARMED");
                }
            }
            BotState::Emergency => {
                if controller.buttons.start() && controller.buttons.select() {
                    current_state = BotState::Idle;
                    command_sender
                        .send(Core1Command::System(SystemCommand::Resume))
                        .await;
                    info!("Emergency cleared, IDLE");
                }
            }
        }

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

        if let Ok(status) = status_receiver.try_receive() {
            if status.state != current_state {
                info!("Core 1 reported state: {:?}", status.state);
                current_state = status.state;
            }
        }

        Timer::after_millis(CONTROL_LOOP_PERIOD_MS).await;
    }
}

async fn process_movement(
    data: &ControllerData,
    sender: &Sender<'static, CriticalSectionRawMutex, Core1Command, 8>,
) {
    if data.left_stick_y < STICK_DEAD_ZONE_LOW {
        let speed = ((STICK_DEAD_ZONE_LOW - data.left_stick_y) as f32 / STICK_DEAD_ZONE_LOW as f32
            * 100.0) as u8;
        sender
            .send(Core1Command::Motor(MotorCommand::Forward(speed)))
            .await;
    } else if data.left_stick_y > STICK_DEAD_ZONE_HIGH {
        let speed = ((data.left_stick_y - STICK_DEAD_ZONE_HIGH) as f32
            / (255 - STICK_DEAD_ZONE_HIGH) as f32
            * 100.0) as u8;
        sender
            .send(Core1Command::Motor(MotorCommand::Backward(speed)))
            .await;
    } else {
        sender.send(Core1Command::Motor(MotorCommand::Stop)).await;
    }

    let angle = (data.right_stick_x as u32 * 180) / 255;
    sender
        .send(Core1Command::Servo(ServoCommand { angle: angle as u8 }))
        .await;
}
