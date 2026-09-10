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

use crate::types::{Address, MomentumType, SpeedSetting, Track};
use std::fmt::{self, Write};

/// Command sent from the client to the controller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Set power state of all tracks or a specific track.
    ///
    /// Response is [`Response::SetPower`].
    ///
    SetPower { is_on: bool, track: Option<Track> },

    /// Restart the command station.
    ///
    /// No response.
    ///
    Reboot,

    // TODO: unify regular and max variants.
    /// Request per-track current values in milliamps.
    ///
    /// Response is [`Response::MilliAmpValues`].
    ///
    GetMilliAmpValues,

    /// Request per-track max current values milliamps.
    ///
    /// Response is [`Response::MaxMilliAmpValues`].
    ///
    GetMaxMilliAmpValues,

    /// Force loco update.
    ///
    /// Response is [`Response::LocoState`].
    ///
    ForceUpdate { address: Address },

    /// Set loco speed.
    ///
    /// Response is [`Response::LocoState`].
    ///
    SetSpeed {
        address: Address,
        speed: SpeedSetting,
    },

    /// Set loco decoder functions on or off.
    ///
    /// Response is [`Response::LocoState`].
    ///
    SetDecoderFunction {
        address: Address,
        function: u8,
        is_on: bool,
    },

    /// Set the momentum for a loco.
    ///
    /// No response.
    ///
    SetMomentum {
        address: Address,
        acceleration: u8,
        deceleration: Option<u8>,
    },

    /// Change the momentum type between "linear" or "power" globally.
    ///
    /// No response.
    ///
    SetMomentumType(MomentumType),

    /// Emergency stop.
    ///
    /// Response is [`Response::LocoState`] for all locos in the reminders list.
    ///
    EmergencyStop,

    /// Request the number of supported locos.
    ///
    /// Response is [`Response::GetSupportedLocoCount`].
    ///
    GetSupportedLocoCount,
}

mod types {
    pub const DIAGNOSTIC: char = 'D';
    pub const LOCO: char = 't';
    pub const SET_DECODER_FUNCTION: char = 'F';
    pub const SET_MOMENTUM: char = 'm';
    pub const GET_MILLI_AMP_VALUES: &str = "JI";
    pub const GET_MAX_MILLI_AMP_VALUES: &str = "JG";
    pub const EMERGENCY_STOP: char = '!';
    pub const GET_SUPPORTED_LOCO_COUNT: char = '#';
}

impl fmt::Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::SetPower { is_on, track } => {
                f.write_char(if *is_on { '1' } else { '0' })?;
                if let Some(track) = track {
                    f.write_char(' ')?;
                    f.write_str(track.into())?
                }
            }
            Command::Reboot => {
                f.write_char(types::DIAGNOSTIC)?;
                f.write_str(" RESET")?
            }
            Command::GetMilliAmpValues => f.write_str(types::GET_MILLI_AMP_VALUES)?,
            Command::GetMaxMilliAmpValues => f.write_str(types::GET_MAX_MILLI_AMP_VALUES)?,
            Command::ForceUpdate { address } => {
                f.write_char(types::LOCO)?;
                f.write_fmt(format_args!(" {}", address))?
            }
            Command::SetSpeed { address, speed } => {
                f.write_char(types::LOCO)?;
                f.write_fmt(format_args!(" {} ", address))?;
                match speed {
                    SpeedSetting::Forward(speed) => f.write_fmt(format_args!("{} 1", speed))?,
                    SpeedSetting::Reverse(speed) => f.write_fmt(format_args!("{} 0", speed))?,
                    SpeedSetting::EmergencyStop => f.write_str("-1 1")?, // TODO: any direction is ok?
                }
            }
            Command::SetDecoderFunction {
                address,
                function,
                is_on,
            } => {
                f.write_char(types::SET_DECODER_FUNCTION)?;
                f.write_fmt(format_args!(" {} {} ", address, function))?;
                f.write_char(if *is_on { '1' } else { '0' })?;
            }
            Command::SetMomentum {
                address,
                acceleration,
                deceleration,
            } => {
                f.write_char(types::SET_MOMENTUM)?;
                f.write_fmt(format_args!(" {} {}", address, acceleration))?;
                if let Some(deceleration) = deceleration {
                    f.write_fmt(format_args!(" {}", deceleration))?;
                }
            }
            Command::SetMomentumType(momentum_type) => {
                f.write_char(types::SET_MOMENTUM)?;
                f.write_char(' ')?;
                f.write_str(momentum_type.into())?
            }
            Command::EmergencyStop => f.write_char(types::EMERGENCY_STOP)?,
            Command::GetSupportedLocoCount => f.write_char(types::GET_SUPPORTED_LOCO_COUNT)?,
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fmt_set_power() {
        assert_eq!(
            Command::SetPower {
                is_on: false,
                track: None
            }
            .to_string(),
            "0"
        );
        assert_eq!(
            Command::SetPower {
                is_on: true,
                track: None
            }
            .to_string(),
            "1"
        );
        assert_eq!(
            Command::SetPower {
                is_on: false,
                track: Track::F.into()
            }
            .to_string(),
            "0 F"
        );
        assert_eq!(
            Command::SetPower {
                is_on: true,
                track: Track::Main.into()
            }
            .to_string(),
            "1 MAIN"
        );
    }

    #[test]
    fn fmt_get_milli_amp_values() {
        assert_eq!(Command::GetMilliAmpValues.to_string(), "JI");
    }

    #[test]
    fn fmt_get_max_milli_amp_values() {
        assert_eq!(Command::GetMaxMilliAmpValues.to_string(), "JG");
    }

    #[test]
    fn fmt_force_update() {
        assert_eq!(
            Command::ForceUpdate {
                address: Address::try_from(16).unwrap()
            }
            .to_string(),
            "t 16"
        );
    }

    #[test]
    fn fmt_set_speed() {
        assert_eq!(
            Command::SetSpeed {
                address: Address::try_from(16).unwrap(),
                speed: SpeedSetting::Forward(43)
            }
            .to_string(),
            "t 16 43 1"
        );
        assert_eq!(
            Command::SetSpeed {
                address: Address::try_from(16).unwrap(),
                speed: SpeedSetting::Reverse(23)
            }
            .to_string(),
            "t 16 23 0"
        );
        assert_eq!(
            Command::SetSpeed {
                address: Address::try_from(16).unwrap(),
                speed: SpeedSetting::EmergencyStop
            }
            .to_string(),
            "t 16 -1 1"
        );
    }

    #[test]
    fn fmt_set_decoder_function() {
        assert_eq!(
            Command::SetDecoderFunction {
                address: Address::try_from(16).unwrap(),
                function: 23,
                is_on: false
            }
            .to_string(),
            "F 16 23 0"
        );
        assert_eq!(
            Command::SetDecoderFunction {
                address: Address::try_from(16).unwrap(),
                function: 23,
                is_on: true
            }
            .to_string(),
            "F 16 23 1"
        );
    }

    #[test]
    fn fmt_set_momentum() {
        assert_eq!(
            Command::SetMomentum {
                address: Address::try_from(3).unwrap(),
                acceleration: 0,
                deceleration: None
            }
            .to_string(),
            "m 3 0"
        );
        assert_eq!(
            Command::SetMomentum {
                address: Address::try_from(3).unwrap(),
                acceleration: 21,
                deceleration: None
            }
            .to_string(),
            "m 3 21"
        );
        assert_eq!(
            Command::SetMomentum {
                address: Address::try_from(3).unwrap(),
                acceleration: 21,
                deceleration: 42.into()
            }
            .to_string(),
            "m 3 21 42"
        );
    }

    #[test]
    fn fmt_momentum_type() {
        assert_eq!(
            Command::SetMomentumType(MomentumType::Linear).to_string(),
            "m LINEAR"
        );
        assert_eq!(
            Command::SetMomentumType(MomentumType::Power).to_string(),
            "m POWER"
        );
    }

    #[test]
    fn fmt_reboot() {
        assert_eq!(Command::Reboot.to_string(), "D RESET");
    }

    #[test]
    fn fmt_emergency_stop() {
        assert_eq!(Command::EmergencyStop.to_string(), "!");
    }

    #[test]
    fn fmt_get_supported_loco_count() {
        assert_eq!(Command::GetSupportedLocoCount.to_string(), "#");
    }
}
