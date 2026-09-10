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

use crate::types::{Address, SpeedSetting, Track};
use std::str::FromStr;
use thiserror::Error;

/// Event sent from the controller to the client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// Response to [`Command::SetPower`].
    SetPower { is_on: bool, track: Option<Track> },

    /// Response to [`Command::MilliAmpValues`].
    GetMilliAmpValues(Box<[u16]>),

    /// Response to [`Command::MaxMilliAmpValues`].
    GetMaxMilliAmpValues(Box<[u16]>),

    /// Response for all loco commands.
    LocoState {
        address: Address,
        speed: SpeedSetting,
        function_map: u8,
    },

    /// Response to [`Request::GetSupportedLocoCount`].
    GetSupportedLocoCount(u8),
}

#[derive(Error, Debug)]
pub enum EventError {
    #[error("unsupported event type: '{0}'")]
    Unsupported(Box<str>),
    #[error("event type is unknown due to missing name")]
    MissingType,
    #[error("required segment was missing")]
    MissingSegment,
    #[error("failed to parse segment: {0}")]
    InvalidSegment(Box<dyn std::error::Error>),
}

impl FromStr for Event {
    type Err = EventError;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        let segments = raw.split(' ');
        event_from(segments)
    }
}

mod types {
    pub const GET_POWER_ON: &str = "p0";
    pub const GET_POWER_OFF: &str = "p1";
    pub const GET_MILLI_AMP_VALUES: &str = "jI";
    pub const GET_MAX_MILLI_AMP_VALUES: &str = "jG";
    pub const LOCO_STATE: &str = "l";
    pub const GET_SUPPORTED_LOCO_COUNT: &str = "#";
}

fn event_from<'a>(mut segments: impl Iterator<Item = &'a str>) -> Result<Event, EventError> {
    let event_type = segments.next().ok_or(EventError::MissingType)?;
    let event = match event_type {
        types::GET_POWER_ON => get_power_event_from(segments, false)?,
        types::GET_POWER_OFF => get_power_event_from(segments, true)?,
        types::GET_MILLI_AMP_VALUES => get_milli_amp_values_event_from(segments, false)?,
        types::GET_MAX_MILLI_AMP_VALUES => get_milli_amp_values_event_from(segments, true)?,
        types::LOCO_STATE => loco_state_event_from(segments)?,
        types::GET_SUPPORTED_LOCO_COUNT => get_supported_loco_count_event_from(segments)?,
        _ => Err(EventError::Unsupported(event_type.into()))?,
    };
    Ok(event)
}

fn get_power_event_from<'a>(
    mut segments: impl Iterator<Item = &'a str>,
    is_on: bool,
) -> Result<Event, EventError> {
    let track = segments
        .next()
        .map(str::parse::<Track>)
        .transpose()
        .map_err(|err| EventError::InvalidSegment(err.into()))?;
    Ok(Event::SetPower { is_on, track })
}

fn get_milli_amp_values_event_from<'a>(
    segments: impl Iterator<Item = &'a str>,
    is_max: bool,
) -> Result<Event, EventError> {
    let values = segments
        .map(|s| s.parse::<u16>())
        .collect::<Result<Box<[u16]>, _>>()
        .map_err(|err| EventError::InvalidSegment(err.into()))?;

    Ok(match is_max {
        true => Event::GetMaxMilliAmpValues(values),
        false => Event::GetMilliAmpValues(values),
    })
}

fn loco_state_event_from<'a>(
    mut segments: impl Iterator<Item = &'a str>,
) -> Result<Event, EventError> {
    let address = segments
        .next()
        .ok_or(EventError::MissingSegment)
        .and_then(|segment| {
            segment
                .parse::<u16>()
                .map_err(|err| EventError::InvalidSegment(err.into()))
        })
        .and_then(|value| {
            Address::try_from(value).map_err(|err| EventError::InvalidSegment(err.into()))
        })?;

    let speed = segments
        .next()
        .ok_or(EventError::MissingSegment)
        .and_then(|segment| {
            segment
                .parse::<u8>()
                .map_err(|err| EventError::InvalidSegment(err.into()))
        })
        .map(SpeedSetting::from_byte)?;

    let function_map = segments
        .next()
        .ok_or(EventError::MissingSegment)
        .and_then(|segment| {
            segment
                .parse::<u8>()
                .map_err(|err| EventError::InvalidSegment(err.into()))
        })?;

    Ok(Event::LocoState {
        address,
        speed,
        function_map,
    })
}

fn get_supported_loco_count_event_from<'a>(
    mut segments: impl Iterator<Item = &'a str>,
) -> Result<Event, EventError> {
    let count = segments
        .next()
        .ok_or(EventError::MissingSegment)
        .and_then(|segment| {
            segment
                .parse::<u8>()
                .map_err(|err| EventError::InvalidSegment(err.into()))
        })?;

    Ok(Event::GetSupportedLocoCount(count))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches;

    #[test]
    fn parse_get_power_event() {
        assert_eq!(
            "p0".parse::<Event>().unwrap(),
            Event::SetPower {
                is_on: false,
                track: None
            }
        );
        assert_eq!(
            "p1".parse::<Event>().unwrap(),
            Event::SetPower {
                is_on: true,
                track: None
            }
        );
        assert_eq!(
            "p1 JOIN".parse::<Event>().unwrap(),
            Event::SetPower {
                is_on: true,
                track: Track::Join.into()
            }
        );
        assert_matches!("p3".parse::<Event>(), Err(EventError::Unsupported(_)));
        assert_matches!(
            "p1 ALT".parse::<Event>(),
            Err(EventError::InvalidSegment(_))
        );
    }

    #[test]
    fn parse_get_milli_amp_values() {
        assert_eq!(
            "jI 124 14234 0".parse::<Event>().unwrap(),
            Event::GetMilliAmpValues([124, 14_234, 0].into())
        );
        assert_eq!(
            "jG 124 14234 0".parse::<Event>().unwrap(),
            Event::GetMaxMilliAmpValues([124, 14_234, 0].into())
        );
    }

    #[test]
    fn parse_loco_state() {
        assert_eq!(
            "l 12 55 16".parse::<Event>().unwrap(),
            Event::LocoState {
                address: Address::try_from(12).unwrap(),
                speed: SpeedSetting::Reverse(54),
                function_map: 16
            }
        );
        assert_matches!("l 12 55".parse::<Event>(), Err(EventError::MissingSegment));
        assert_matches!("l".parse::<Event>(), Err(EventError::MissingSegment));
        assert_matches!(
            "l 12 55 1024".parse::<Event>(),
            Err(EventError::InvalidSegment(_))
        );
    }

    #[test]
    fn parse_get_supported_loco_count() {
        assert_eq!(
            "# 18".parse::<Event>().unwrap(),
            Event::GetSupportedLocoCount(18)
        );
        assert_matches!(
            "# 1024".parse::<Event>(),
            Err(EventError::InvalidSegment(_))
        );
    }
}
