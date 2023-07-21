//
// throttle.rs
//
// @author Natesh Narain <nnaraindev@gmail.com>
// @date Jul 18 2023
//

/// Handle throttle commands
pub struct ThrottleControl {
    throttle: u8,
}

impl ThrottleControl {
    pub fn set_throttle(&mut self, throttle: u8) {
        if self.throttle != throttle {

        }

        self.throttle = throttle;
    }
}
