// Copyright 2020 IOTA Stiftung
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use this file except in compliance with
// the License. You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on
// an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and limitations under the License.

use bee_common::{shutdown::Shutdown, worker::Error as WorkerError};
use bee_event::Bus;
use bee_protocol::events::{LastMilestone, LastSolidMilestone};

use async_std::task::{spawn};
use futures::{
    future::FutureExt,
    select,
    stream::StreamExt,
    channel::{mpsc, oneshot}
};
use rumqtt::{MqttClient, MqttOptions, QoS};

use std::{ptr, sync::Arc, thread, time::Duration};

struct Mqtt {
    lmi: mpsc::UnboundedSender<LastMilestone>,
}

static mut MQTT: *const Mqtt = ptr::null();

pub(crate) fn mqtt() -> &'static Mqtt {
    if unsafe { MQTT.is_null() } {
        panic!("Uninitialized protocol.");
    } else {
        unsafe { &*MQTT }
    }
}

 async fn lmi_worker(
    receiver: mpsc::UnboundedReceiver<LastMilestone>,
    shutdown: oneshot::Receiver<()>,
) -> Result<(), WorkerError> {
    let mut receiver_fused = receiver.fuse();
    let mut shutdown_fused = shutdown.fuse();

    loop {
        select! {
            event = receiver_fused.next() => {
                if let Some(LastMilestone(milestone)) = event {
                    let mqtt_options = MqttOptions::new("test-pubsub1", "localhost", 1883);
                    let (mut mqtt_client, _) = MqttClient::start(mqtt_options).unwrap();

                    let payload = format!("{}", *milestone.index());
                    mqtt_client.publish("lmi", QoS::AtLeastOnce, false, payload).unwrap();
                }
            },
            _ = shutdown_fused => {
                break;
            }
        }
    }

    Ok(())
}

fn lmi_listener(lmi: &LastMilestone) {
    mqtt().lmi.unbounded_send(lmi.clone());
}

fn lsmi_listener(lmi: &LastSolidMilestone) {}

pub fn init(bus: Arc<Bus<'_>>, shutdown: &mut Shutdown) {
    if unsafe { !MQTT.is_null() } {
        // warn!("Already initialized.");
        return;
    }

    let (lmi_tx, lmi_rx) = mpsc::unbounded();
    let (lmi_shutdown_tx, lmi_shutdown_rx) = oneshot::channel();

    shutdown.add_worker_shutdown(lmi_shutdown_tx, spawn(lmi_worker(lmi_rx, lmi_shutdown_rx)));

    let mqtt= Mqtt{
        lmi : lmi_tx,
    };

    unsafe {
        MQTT = Box::leak(mqtt.into()) as *const _;
    }



    bus.add_listener(lmi_listener);
    bus.add_listener(lsmi_listener);
}
