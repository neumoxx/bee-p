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

use crate::{PrivateKeyGenerator, Seed, TernarySeed, WotsError, WotsPrivateKey, WotsSecurityLevel};

use bee_crypto::Sponge;
use bee_ternary::{
    bigint::{
        common::{BigEndian, U8Repr},
        I384, T242, T243,
    },
    Btrit, TritBuf,
};

use sha3::{
    digest::{ExtendableOutput, Input, XofReader},
    Shake256,
};

use std::marker::PhantomData;

#[derive(Default)]
pub struct WotsShakePrivateKeyGeneratorBuilder<S> {
    security_level: Option<WotsSecurityLevel>,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> WotsShakePrivateKeyGeneratorBuilder<S> {
    pub fn security_level(mut self, security_level: WotsSecurityLevel) -> Self {
        self.security_level.replace(security_level);
        self
    }

    pub fn build(self) -> Result<WotsShakePrivateKeyGenerator<S>, WotsError> {
        Ok(WotsShakePrivateKeyGenerator {
            security_level: self.security_level.ok_or(WotsError::MissingSecurityLevel)?,
            _sponge: PhantomData,
        })
    }
}

pub struct WotsShakePrivateKeyGenerator<S> {
    security_level: WotsSecurityLevel,
    _sponge: PhantomData<S>,
}

impl<S: Sponge + Default> PrivateKeyGenerator for WotsShakePrivateKeyGenerator<S> {
    type Seed = TernarySeed<S>;
    type PrivateKey = WotsPrivateKey<S>;
    type Error = WotsError;

    fn generate(&self, seed: &Self::Seed, index: u64) -> Result<Self::PrivateKey, Self::Error> {
        let mut state = TritBuf::zeros(self.security_level as usize * 6561);
        let mut shake = Shake256::default();
        let mut ternary_buffer = T243::<Btrit>::default();
        ternary_buffer.inner_mut().copy_from(seed.trits());
        let mut binary_buffer: I384<BigEndian, U8Repr> = ternary_buffer.into_t242().into();

        shake.input(&binary_buffer.inner_ref()[..]);
        let mut reader = shake.xof_result();

        for trit_chunk in state.chunks_mut(243) {
            reader.read(&mut binary_buffer.inner_mut()[..]);
            let ternary_value = T242::from_i384_ignoring_mst(binary_buffer).into_t243();

            trit_chunk.copy_from(&ternary_value.inner_ref());
        }

        Ok(Self::PrivateKey {
            state,
            _sponge: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use bee_crypto::Kerl;
    use bee_ternary::{T1B1Buf, TryteBuf};

    const SEED: &str = "NNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNNN";
    const MESSAGE: &str = "CHXHLHQLOPYP9NSUXTMWWABIBSBLUFXFRNWOZXJPVJPBCIDI99YBSCFYILCHPXHTSEYSYWIGQFERCRVDD";

    #[test]
    fn wots_shake() {
        let seed_trits =
            TryteBuf::try_from_str("CEFLDDLMF9TO9ZLLTYXIPVFIJKAOFRIQLGNYIDZCTDYSWMNXPYNGFAKHQDY9ABGGQZHEFTXKWKWZXEIUD")
                .unwrap()
                .as_trits()
                .encode::<T1B1Buf>();
        let seed = TernarySeed::<Kerl>::from_buf(seed_trits).unwrap();
        let private_key_generator = WotsShakePrivateKeyGeneratorBuilder::<Kerl>::default()
            .security_level(WotsSecurityLevel::Medium)
            .build()
            .unwrap();
        let private_key = private_key_generator.generate(&seed, 0).unwrap();

        println!("{:?}", private_key.state.as_i8_slice());
    }
}