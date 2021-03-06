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

use crate::{
    message::{
        tlv_into_bytes, Heartbeat, Message, MilestoneRequest, Transaction as TransactionMessage, TransactionRequest,
    },
    milestone::MilestoneIndex,
    protocol::Protocol,
    tangle::tangle,
    worker::{
        BroadcasterWorkerEvent, MilestoneRequesterWorkerEvent, MilestoneSolidifierWorkerEvent,
        TransactionRequesterWorkerEvent,
    },
};

use bee_crypto::ternary::Hash;
use bee_network::{Command::SendMessage, EndpointId};

use log::warn;

use std::marker::PhantomData;

pub(crate) struct Sender<M: Message> {
    marker: PhantomData<M>,
}

macro_rules! implement_sender_worker {
    ($type:ty, $sender:tt, $incrementor:tt) => {
        impl Sender<$type> {
            pub(crate) async fn send(epid: &EndpointId, message: $type) {
                match Protocol::get()
                    .network
                    .clone()
                    .send(SendMessage {
                        receiver_epid: *epid,
                        message: tlv_into_bytes(message),
                    })
                    .await
                {
                    Ok(_) => {
                        // self.peer.metrics.$incrementor();
                        // Protocol::get().metrics.$incrementor();
                    }
                    Err(e) => {
                        warn!("Sending {} to {} failed: {:?}.", stringify!($type), epid, e);
                    }
                }
            }
        }
    };
}

implement_sender_worker!(MilestoneRequest, milestone_request, milestone_requests_sent_inc);
implement_sender_worker!(TransactionMessage, transaction, transactions_sent_inc);
implement_sender_worker!(TransactionRequest, transaction_request, transaction_requests_sent_inc);
implement_sender_worker!(Heartbeat, heartbeat, heartbeats_sent_inc);

impl Protocol {
    // TODO move some functions to workers

    // MilestoneRequest

    pub fn request_milestone(index: MilestoneIndex, to: Option<EndpointId>) {
        if !Protocol::get().requested_milestones.contains_key(&index) && !tangle().contains_milestone(index) {
            if let Err(e) = Protocol::get()
                .milestone_requester_worker
                .unbounded_send(MilestoneRequesterWorkerEvent(index, to))
            {
                warn!("Requesting milestone failed: {}.", e);
            }
        }
    }

    pub fn request_latest_milestone(to: Option<EndpointId>) {
        Protocol::request_milestone(MilestoneIndex(0), to)
    }

    // TransactionMessage

    pub(crate) fn broadcast_transaction_message(source: Option<EndpointId>, transaction: TransactionMessage) {
        if let Err(e) = Protocol::get()
            .broadcaster_worker
            .unbounded_send(BroadcasterWorkerEvent { source, transaction })
        {
            warn!("Broadcasting transaction failed: {}.", e);
        }
    }

    pub fn broadcast_transaction(source: Option<EndpointId>, transaction: &[u8]) {
        Protocol::broadcast_transaction_message(source, TransactionMessage::new(transaction));
    }

    // TransactionRequest

    pub fn request_transaction(hash: Hash, index: MilestoneIndex) {
        if !tangle().contains(&hash)
            && !tangle().is_solid_entry_point(&hash)
            && !Protocol::get().requested_transactions.contains_key(&hash)
        {
            if let Err(e) = Protocol::get()
                .transaction_requester_worker
                .unbounded_send(TransactionRequesterWorkerEvent(hash, index))
            {
                warn!("Requesting transaction failed: {}.", e);
            }
        }
    }

    // Heartbeat

    pub async fn send_heartbeat(
        to: EndpointId,
        latest_solid_milestone_index: MilestoneIndex,
        pruning_milestone_index: MilestoneIndex,
        latest_milestone_index: MilestoneIndex,
    ) {
        Sender::<Heartbeat>::send(
            &to,
            Heartbeat::new(
                *latest_solid_milestone_index,
                *pruning_milestone_index,
                *latest_milestone_index,
                Protocol::get().peer_manager.connected_peers(),
                Protocol::get().peer_manager.synced_peers(),
            ),
        )
        .await;
    }

    pub async fn broadcast_heartbeat(
        latest_solid_milestone_index: MilestoneIndex,
        pruning_milestone_index: MilestoneIndex,
        latest_milestone_index: MilestoneIndex,
    ) {
        for entry in Protocol::get().peer_manager.handshaked_peers.iter() {
            Protocol::send_heartbeat(
                *entry.key(),
                latest_solid_milestone_index,
                pruning_milestone_index,
                latest_milestone_index,
            )
            .await
        }
    }

    // Solidifier

    pub fn trigger_milestone_solidification(target_index: MilestoneIndex) {
        Protocol::get()
            .milestone_solidifier_worker
            .unbounded_send(MilestoneSolidifierWorkerEvent(target_index));
    }
}
