use defmt::*;
use embassy_rp::gpio::{Level, Output};
use embassy_rp::peripherals::{PIN_16, PIN_17, PIN_18, PIN_19, PIN_7, PIN_8, PIN_9, PWM_SLICE0, PWM_SLICE3};
use embassy_rp::pwm::{Config as PwmConfig, Pwm};
use tb6612fng::{DriveCommand, Motor};

pub struct TankDriveController {
    motor_br: Motor<Output<'static>, Output<'static>, Pwm<'static>>, // Back Right
    motor_fl: Motor<Output<'static>, Output<'static>, Pwm<'static>>, // Front Left
    standby: Output<'static>,
}

impl TankDriveController {
    pub fn new(
        pwm_br: PWM_SLICE0,
        pwm_br_pin: PIN_16,
        in1_br_pin: PIN_17,
        in2_br_pin: PIN_18,
        pwm_fl: PWM_SLICE3,
        pwm_fl_pin: PIN_7,
        in1_fl_pin: PIN_9,
        in2_fl_pin: PIN_8,
        standby_pin: PIN_19,
    ) -> Self {
        // Configure PWM for motor speed control
        let mut pwm_config = PwmConfig::default();
        pwm_config.divider = 125.into(); // For 1MHz counting frequency
        pwm_config.top = 100; // For 10kHz PWM frequency (1MHz / 100)
        pwm_config.compare_a = 0; // Start with motors stopped

        // Back Right motor setup
        let pwm_br = Pwm::new_output_a(pwm_br, pwm_br_pin, pwm_config.clone());
        let in1_br = Output::new(in1_br_pin, Level::Low);
        let in2_br = Output::new(in2_br_pin, Level::Low);
        let motor_br = Motor::new(in1_br, in2_br, pwm_br).unwrap();

        // Front Left motor setup
        let pwm_fl = Pwm::new_output_b(pwm_fl, pwm_fl_pin, pwm_config);
        let in1_fl = Output::new(in1_fl_pin, Level::Low);
        let in2_fl = Output::new(in2_fl_pin, Level::Low);
        let motor_fl = Motor::new(in1_fl, in2_fl, pwm_fl).unwrap();

        // Configure standby pin (active high to enable motor driver)
        let mut standby = Output::new(standby_pin, Level::Low);
        standby.set_high(); // Enable the motor driver

        TankDriveController {
            motor_br,
            motor_fl,
            standby,
        }
    }

    /// Control tank drive with omnidirectional movement
    /// x: -100 to 100 (left to right)
    /// y: -100 to 100 (backward to forward)
    /// 
    /// # Differential Drive Mixing
    /// 
    /// This uses differential drive mixing to convert joystick inputs (x,y) into 
    /// individual motor speeds for tank-style movement:
    /// 
    /// ```
    /// left_speed  = forward_speed + turn_speed
    /// right_speed = forward_speed - turn_speed
    /// ```
    /// 
    /// ## Examples:
    /// - **Straight forward** (y=50, x=0): Both motors at 50% → moves straight
    /// - **Straight backward** (y=-50, x=0): Both motors at -50% → reverses straight
    /// - **Turn right** (y=50, x=30): Left at 80%, Right at 20% → curves right  
    /// - **Backward left turn** (y=-40, x=-20): Left at -60%, Right at -20% → reverses while turning left
    /// - **Spin in place** (y=0, x=50): Left at 50%, Right at -50% → rotates on spot
    /// 
    /// This allows smooth omnidirectional control from simple forward/turn inputs.
    pub fn drive(&mut self, x: i8, y: i8) {
        // Calculate individual motor speeds for omnidirectional movement
        // For tank drive with opposite corner motors:
        // BR motor: controls right side thrust
        // FL motor: controls left side thrust
        
        // Apply differential drive mixing algorithm
        let left_speed = y.saturating_add(x);
        let right_speed = y.saturating_sub(x);

        // Control Back Right motor (right side)
        if right_speed > 0 {
            let speed = right_speed.min(100) as u8;
            info!("BR motor: Forward at {}%", speed);
            self.motor_br.drive(DriveCommand::Forward(speed)).unwrap();
        } else if right_speed < 0 {
            let speed = right_speed.saturating_abs().min(100) as u8;
            info!("BR motor: Backward at {}%", speed);
            self.motor_br.drive(DriveCommand::Backward(speed)).unwrap();
        } else {
            self.motor_br.drive(DriveCommand::Stop).unwrap();
        }

        // Control Front Left motor (left side)
        if left_speed > 0 {
            let speed = left_speed.min(100) as u8;
            info!("FL motor: Forward at {}%", speed);
            self.motor_fl.drive(DriveCommand::Forward(speed)).unwrap();
        } else if left_speed < 0 {
            let speed = left_speed.saturating_abs().min(100) as u8;
            info!("FL motor: Backward at {}%", speed);
            self.motor_fl.drive(DriveCommand::Backward(speed)).unwrap();
        } else {
            self.motor_fl.drive(DriveCommand::Stop).unwrap();
        }

        info!("Tank drive: x={}, y={} => L={}, R={}", x, y, left_speed, right_speed);
    }

    pub fn stop(&mut self) {
        self.motor_br.drive(DriveCommand::Stop).unwrap();
        self.motor_fl.drive(DriveCommand::Stop).unwrap();
    }

    pub fn brake(&mut self) {
        self.motor_br.drive(DriveCommand::Brake).unwrap();
        self.motor_fl.drive(DriveCommand::Brake).unwrap();
    }

    pub fn disable(&mut self) {
        self.stop(); // Stop motors before disabling
        self.standby.set_low();
    }

    pub fn enable(&mut self) {
        self.standby.set_high();
    }

    /// Spin in place (rotate)
    /// speed: -100 to 100 (CCW to CW)
    pub fn spin(&mut self, speed: i8) {
        if speed > 0 {
            // Clockwise spin
            self.motor_br.drive(DriveCommand::Backward(speed.min(100) as u8)).unwrap();
            self.motor_fl.drive(DriveCommand::Forward(speed.min(100) as u8)).unwrap();
        } else if speed < 0 {
            // Counter-clockwise spin
            self.motor_br.drive(DriveCommand::Forward((-speed).min(100) as u8)).unwrap();
            self.motor_fl.drive(DriveCommand::Backward((-speed).min(100) as u8)).unwrap();
        } else {
            self.stop();
        }
    }
}