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

mod connect;
mod contact;
mod worker;

pub use connect::*;
pub use contact::*;
pub use worker::*;

use crate::util::TransportProtocol;

use futures::channel::mpsc;
use thiserror::Error;

use std::{fmt, net::SocketAddr as SocketAddress};

/// Errors that can happen when dealing with `Address`es.
#[derive(Debug, Error)]
pub enum Error {
    #[error("Error resolving domain name to address.")]
    Io(#[from] std::io::Error),

    #[error("Error parsing url.")]
    UrlParseFailure,

    #[error("Unspecified transport protocol.")]
    UnspecifiedTransportProtocol,

    #[error("Unsupported transport protocol.")]
    UnsupportedTransportProtocol,

    // TODO: rename to to 'DomainNameResolutionFailure'
    #[error("Error resolving domain name to address.")]
    DnsFailure,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EndpointId((TransportProtocol, SocketAddress));

impl EndpointId {
    pub fn new(transport_protocol: TransportProtocol, socket_address: SocketAddress) -> Self {
        Self((transport_protocol, socket_address))
    }
}

impl fmt::Display for EndpointId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}://{}", (self.0).0, (self.0).1)
    }
}

pub type DataSender = mpsc::UnboundedSender<Vec<u8>>;
pub type DataReceiver = mpsc::UnboundedReceiver<Vec<u8>>;

pub fn channel() -> (DataSender, DataReceiver) {
    mpsc::unbounded()
}
