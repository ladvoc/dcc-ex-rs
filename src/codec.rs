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

use crate::{Command, Event};
use bytes::{Buf, BytesMut};
use std::{
    fmt::{self, Write},
    io,
};
use tokio_util::codec::{Decoder, Encoder};

pub struct Codec;

const MAX_MESSAGE_LEN: usize = 1024;
const MESSAGE_START: u8 = b'<';
const MESSAGE_END: u8 = b'>';

impl Encoder<Command> for Codec {
    type Error = io::Error;

    fn encode(&mut self, command: Command, dst: &mut BytesMut) -> Result<(), Self::Error> {
        dst.extend_from_slice(&[MESSAGE_START]);
        write!(BytesWriter(dst), "{command}")
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "could not format command"))?;
        dst.extend_from_slice(&[MESSAGE_END]);
        Ok(())
    }
}

impl Decoder for Codec {
    type Item = Event;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Event>, Self::Error> {
        loop {
            // Discard anything before the next frame.
            let Some(start) = src.iter().position(|byte| *byte == MESSAGE_START) else {
                src.clear();
                return Ok(None);
            };

            src.advance(start);

            // Wait until the frame's first closing bracket arrives.
            let Some(end) = src[1..]
                .iter()
                .position(|byte| *byte == MESSAGE_END)
                .map(|position| position + 1)
            else {
                if src.len() > MAX_MESSAGE_LEN {
                    src.clear();

                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "message is too long",
                    ));
                }

                return Ok(None);
            };

            let frame = src.split_to(end + 1);
            let body = &frame[1..end];

            let Ok(body) = str::from_utf8(body) else {
                continue;
            };

            match body.parse::<Event>() {
                Ok(event) => return Ok(Some(event)),

                // Unknown and diagnostic messages are intentionally ignored.
                Err(_) => continue,
            }
        }
    }
}

struct BytesWriter<'a>(&'a mut BytesMut);

impl fmt::Write for BytesWriter<'_> {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        self.0.extend_from_slice(value.as_bytes());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Address, MomentumType, SpeedSetting, Track};

    fn encode_all(commands: impl IntoIterator<Item = Command>) -> String {
        let mut codec = Codec;
        let mut buffer = BytesMut::new();

        for command in commands {
            codec.encode(command, &mut buffer).unwrap();
        }
        String::from_utf8(buffer.to_vec()).expect("invalid string")
    }

    fn decode_all(input: &str) -> Vec<Event> {
        let mut codec = Codec;
        let mut buffer = BytesMut::from(input.as_bytes());
        let mut events = Vec::new();

        while let Some(event) = codec.decode(&mut buffer).unwrap() {
            events.push(event);
        }
        events
    }

    #[test]
    fn test_encode_command_sequence() {
        let sequence = [
            Command::SetPower {
                is_on: true,
                track: Track::Main.into(),
            },
            Command::SetMomentumType(MomentumType::Linear),
            Command::GetMaxMilliAmpValues,
            Command::EmergencyStop,
        ];
        assert_eq!(encode_all(sequence), "<1 MAIN><m LINEAR><JG><!>");
    }

    #[test]
    fn test_decode_event_sequence() {
        let encoded = "<p1 JOIN><l 12 55 16>";
        let sequence = [
            Event::SetPower {
                is_on: true,
                track: Track::Join.into(),
            },
            Event::LocoState {
                address: Address::try_from(12).unwrap(),
                speed: SpeedSetting::Reverse(54),
                function_map: 16,
            },
        ];

        assert_eq!(decode_all(encoded), sequence);
    }

    #[test]
    fn test_decode_ignores_noise() {
        let sequence = [
            "before",
            "<p1 JOIN>",
            "after",
            "  ",
            "<# 18>",
            "final",
            "\n",
        ];
        let events = [
            Event::SetPower {
                is_on: true,
                track: Track::Join.into(),
            },
            Event::GetSupportedLocoCount(18),
        ];

        assert_eq!(decode_all(&sequence.join(" ")), events);
        assert_eq!(decode_all(&sequence.join("\n")), events);
        assert_eq!(decode_all(&sequence.join("\r\n")), events);
    }
}
