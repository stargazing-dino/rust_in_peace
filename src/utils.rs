//! Utility functions for control processing

use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Sender;

use crate::events::TankDriveEvent;
use crate::input::ControllerData;

/// Process movement from controller data and send tank drive events
pub async fn process_movement(
    data: &ControllerData,
    tank_sender: &Sender<'static, CriticalSectionRawMutex, TankDriveEvent, 8>,
) {
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
        tank_sender.send(TankDriveEvent::Spin(spin)).await;
    } else if x == 0 && y == 0 {
        tank_sender.send(TankDriveEvent::Stop).await;
    } else {
        tank_sender.send(TankDriveEvent::Move { x, y }).await;
    }
}