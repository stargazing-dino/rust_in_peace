use embassy_rp::peripherals::{PIN_26, PWM_SLICE5};
use embassy_rp::pwm::{Config as PwmConfig, Pwm, SetDutyCycle};

pub struct ServoController {
    pwm: Pwm<'static>,
}

impl ServoController {
    pub fn new(pwm_slice: PWM_SLICE5, pwm_pin: PIN_26) -> Self {
        let desired_freq_hz = 50;
        let clock_freq_hz = embassy_rp::clocks::clk_sys_freq();
        let divider = 64u8;
        let period = (clock_freq_hz / (desired_freq_hz * divider as u32)) as u16 - 1;

        let mut config = PwmConfig::default();
        config.divider = divider.into();
        config.top = period;
        config.compare_a = period / 20;

        let pwm = Pwm::new_output_a(pwm_slice, pwm_pin, config);

        ServoController { pwm }
    }

    pub fn set_angle(&mut self, angle: u8) {
        let min_duty = self.pwm.max_duty_cycle() / 20;
        let max_duty = self.pwm.max_duty_cycle() / 10;
        
        let duty_range = max_duty - min_duty;
        let duty = min_duty + ((duty_range as u32 * angle as u32) / 180) as u16;
        
        self.pwm.set_duty_cycle(duty).unwrap();
    }

    pub fn control_from_stick(&mut self, stick_x: u8) {
        let angle = (stick_x as u32 * 180) / 255;
        self.set_angle(angle as u8);
    }
}