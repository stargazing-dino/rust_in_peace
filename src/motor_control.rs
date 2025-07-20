use defmt::*;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIN_16, PIN_17, PIN_18, PIN_19, PWM_SLICE0};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use tb6612fng::{DriveCommand, Motor};

pub struct MotorController {
    motor: Motor<Output<'static>, Output<'static>, Pwm<'static>>,
    standby: Output<'static>,
}

impl MotorController {
    pub fn new(
        pwm: PWM_SLICE0,
        pwm_pin: PIN_16,
        in1_pin: PIN_17,
        in2_pin: PIN_18,
        standby_pin: PIN_19,
    ) -> Self {
        // Configure PWM for motor speed control
        let mut pwm_config = PwmConfig::default();
        pwm_config.divider = 125.into(); // For 1MHz counting frequency
        pwm_config.top = 100; // For 10kHz PWM frequency (1MHz / 100)
        pwm_config.compare_a = 0; // Start with motor stopped

        let pwm = Pwm::new_output_a(pwm, pwm_pin, pwm_config);

        // Configure direction pins
        let in1 = Output::new(in1_pin, Level::Low);
        let in2 = Output::new(in2_pin, Level::Low);

        // Configure standby pin (active high to enable motor)
        let mut standby = Output::new(standby_pin, Level::Low);
        standby.set_high(); // Enable the motor driver

        let motor = Motor::new(in1, in2, pwm).unwrap();

        MotorController { motor, standby }
    }

    pub fn control_from_stick(&mut self, stick_y: u8) {
        // PS2 stick values: 0-255, with 128 being center
        // Convert to motor control with dead zone

        if stick_y < 118 {
            // Forward (stick pushed up)
            // Map 0-117 to 100-0% speed
            let speed = ((118 - stick_y) as f32 / 118.0 * 100.0) as u8;
            info!("Stick Y: {} => Motor forward at {}%", stick_y, speed);
            self.motor.drive(DriveCommand::Forward(speed)).unwrap();
        } else if stick_y > 138 {
            // Backward (stick pushed down)
            // Map 139-255 to 0-100% speed
            let speed = ((stick_y - 138) as f32 / 117.0 * 100.0) as u8;
            info!("Stick Y: {} => Motor backward at {}%", stick_y, speed);
            self.motor.drive(DriveCommand::Backward(speed)).unwrap();
        } else {
            // Dead zone (118-138)
            self.motor.drive(DriveCommand::Stop).unwrap();
        }
    }

    pub fn stop(&mut self) {
        self.motor.drive(DriveCommand::Stop).unwrap();
    }

    pub fn brake(&mut self) {
        self.motor.drive(DriveCommand::Brake).unwrap();
    }

    pub fn disable(&mut self) {
        self.stop(); // Stop motor before disabling
        self.standby.set_low();
    }

    pub fn enable(&mut self) {
        self.standby.set_high();
    }

    pub fn current_drive_command(&self) -> DriveCommand {
        *self.motor.current_drive_command()
    }
}
