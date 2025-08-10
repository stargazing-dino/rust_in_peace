//! Dual-core battle bot control system
//! Core 0: Control & Decision Making
//! Core 1: Real-Time Peripheral Control

#![no_std]
#![no_main]
#![allow(dead_code)]
#![allow(unused_assignments)]

mod config;

mod input;
mod hardware;

mod events;
mod control;
mod utils;

use defmt::*;
use embassy_executor::Executor;
use embassy_rp::multicore::{spawn_core1, Stack};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_sync::signal::Signal;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use config::*;
use hardware::split_peripherals;
use input::{ps2_reader_task, receiver_led_task, ControllerData};
use control::{state_controller_task, tank_driver_task, servo_driver_task, led_driver_task};
use events::{TankDriveEvent, ServoEvent, LedEvent};

static CONTROLLER_CHANNEL: Channel<CriticalSectionRawMutex, ControllerData, COMMAND_CHANNEL_SIZE> =
    Channel::new();
static TANK_CHANNEL: Channel<CriticalSectionRawMutex, TankDriveEvent, COMMAND_CHANNEL_SIZE> =
    Channel::new();
static SERVO_CHANNEL: Channel<CriticalSectionRawMutex, ServoEvent, COMMAND_CHANNEL_SIZE> =
    Channel::new();
static LED_CHANNEL: Channel<CriticalSectionRawMutex, LedEvent, COMMAND_CHANNEL_SIZE> =
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
async fn core0_main(spawner: embassy_executor::Spawner, p0: hardware::Peripherals0) {
    info!("Core 0 starting...");

    let controller_sender = CONTROLLER_CHANNEL.sender();

    spawner.must_spawn(ps2_reader_task(p0.controller, controller_sender, &LED_SIGNAL));
    spawner.must_spawn(receiver_led_task(p0.ps2_led, &LED_SIGNAL));
}

#[embassy_executor::task]
async fn core1_main(spawner: embassy_executor::Spawner, p1: hardware::Peripherals1) {
    info!("Core 1 starting...");

    let controller_receiver = CONTROLLER_CHANNEL.receiver();
    let tank_sender = TANK_CHANNEL.sender();
    let tank_receiver = TANK_CHANNEL.receiver();
    let servo_sender = SERVO_CHANNEL.sender();
    let servo_receiver = SERVO_CHANNEL.receiver();
    let led_sender = LED_CHANNEL.sender();
    let led_receiver = LED_CHANNEL.receiver();

    // Spawn the state controller (the "brains")
    spawner.must_spawn(state_controller_task(
        controller_receiver,
        tank_sender,
        servo_sender,
        led_sender,
    ));

    // Spawn hardware driver tasks
    spawner.must_spawn(tank_driver_task(p1.motor, tank_receiver));
    spawner.must_spawn(servo_driver_task(p1.servo, servo_receiver));
    spawner.must_spawn(led_driver_task(p1.state_led, led_receiver));
}
