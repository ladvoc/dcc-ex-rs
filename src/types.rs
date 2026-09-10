// Copyright 2026 Jacob Gelman
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use derive_more::Display;
use strum::{EnumString, IntoStaticStr};
use thiserror::Error;

#[derive(Display, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Address(u16);

#[derive(Error, Debug)]
#[error("invalid DCC address: {0}")]
pub struct AddressError(u16);

impl TryFrom<u16> for Address {
    type Error = AddressError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if !(1..10_239).contains(&value) {
            Err(AddressError(value))?;
        }
        Ok(Self(value))
    }
}

impl From<Address> for u16 {
    fn from(value: Address) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, EnumString, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum Track {
    Main,
    Prog,
    Join,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[derive(Error, Debug)]
#[error("unknown track name: '{0}'")]
pub struct TrackError(Box<str>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpeedSetting {
    Forward(u8), // TODO: limit 0-127
    Reverse(u8),
    EmergencyStop,
}

impl SpeedSetting {
    pub(crate) const fn from_byte(raw: u8) -> Self {
        match raw {
            0 => Self::Reverse(0),
            1 => Self::EmergencyStop,
            2..=127 => Self::Reverse(raw - 1),
            128 => Self::Forward(0),
            129 => Self::EmergencyStop,
            130..=255 => Self::Forward(raw - 129),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoStaticStr)]
#[strum(serialize_all = "UPPERCASE")]
pub enum MomentumType {
    Linear,
    Power,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_speed_setting() {
        assert_eq!(SpeedSetting::from_byte(0), SpeedSetting::Reverse(0));
        assert_eq!(SpeedSetting::from_byte(1), SpeedSetting::EmergencyStop);
        assert_eq!(SpeedSetting::from_byte(2), SpeedSetting::Reverse(1));
        assert_eq!(SpeedSetting::from_byte(127), SpeedSetting::Reverse(126));

        assert_eq!(SpeedSetting::from_byte(128), SpeedSetting::Forward(0));
        assert_eq!(SpeedSetting::from_byte(129), SpeedSetting::EmergencyStop);
        assert_eq!(SpeedSetting::from_byte(130), SpeedSetting::Forward(1));
        assert_eq!(SpeedSetting::from_byte(255), SpeedSetting::Forward(126));
    }
}
