//
// error.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Mar 11 2023
//

use thiserror::Error;

#[derive(Debug, Error)]
pub enum IcarusBleError {
    #[error("The device was not found")]
    DeviceNotFound,
    #[error("Characteristic not found")]
    CharacteristicNotFound,
}
