//! Dual-core battle bot control system
//! Core 0: Control & Decision Making
//! Core 1: Real-Time Peripheral Control

#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused_assignments)]

mod config;
mod peripherals;

mod ps2_input;
mod motor;
mod motor_controller;
mod servo_controller;
mod tank_drive_controller;

use defmt::*;
use embassy_executor::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use config::*;
use peripherals::split_peripherals;
use ps2_input::{controller_task, receiver_led_task, ControllerData};
use motor::motor_task;

static CONTROLLER_CHANNEL: Channel<CriticalSectionRawMutex, ControllerData, COMMAND_CHANNEL_SIZE> =
    Channel::new();
static LED_SIGNAL: Signal<CriticalSectionRawMutex, ()> = Signal::new();

static mut CORE1_STACK: Stack<CORE1_STACK_SIZE> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());

    let (core1, p0, p1) = split_peripherals(p);

    spawn_core1(
        core1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                spawner.must_spawn(core1_main(spawner, p1));
            });
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        spawner.must_spawn(core0_main(spawner, p0));
    });
}

#[embassy_executor::task]
async fn core0_main(spawner: embassy_executor::Spawner, p0: peripherals::Peripherals0) {
    info!("Core 0 starting...");

    let controller_sender = CONTROLLER_CHANNEL.sender();

    spawner.must_spawn(controller_task(p0.controller, controller_sender, &LED_SIGNAL));
    spawner.must_spawn(receiver_led_task(p0.ps2_led, &LED_SIGNAL));
}

#[embassy_executor::task]
async fn core1_main(spawner: embassy_executor::Spawner, p1: peripherals::Peripherals1) {
    info!("Core 1 starting...");

    let controller_receiver = CONTROLLER_CHANNEL.receiver();

    spawner.must_spawn(motor_task(p1, controller_receiver));
}
