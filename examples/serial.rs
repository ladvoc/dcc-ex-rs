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

use dcc_ex::{Codec, Command, Event};
use futures_util::{SinkExt, StreamExt};
use std::env;
use tokio_serial::SerialPortBuilderExt;
use tokio_util::codec::Framed;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = env::args().nth(1) else {
        return Err("No device path specified".into());
    };
    let port = tokio_serial::new(path, 115_200).open_native_async()?;
    let mut device = Framed::new(port, Codec);

    // Send some command.
    device.send(Command::GetSupportedLocoCount).await?;

    // Create one listener and keep it alive for the entire loop.
    let shutdown = tokio::signal::ctrl_c();
    tokio::pin!(shutdown);

    loop {
        tokio::select! {
            result = &mut shutdown => {
                result?;
                println!("Shutting down...");
                break;
            }
            event = device.next() => match event.transpose()? {
                Some(event) => handle_event(event),
                None => {
                    println!("Serial connection closed");
                    break;
                }
            }
        }
    }

    // Flush pending writes and shut down the framed transport.
    device.close().await?;

    Ok(())
}

fn handle_event(event: Event) {
    match event {
        Event::GetSupportedLocoCount(count) => {
            println!("Controller supports {count} loco(s)");
        }
        other => {
            println!("Unhandled event: {other:?}");
        }
    }
}
