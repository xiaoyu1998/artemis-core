use crate::types::{Collector, CollectorStream};
use anyhow::Result;
use async_trait::async_trait;

use alloy::{
    contract as alloy_contract,
    primitives::{B256},
    consensus::BlockHeader,
};
use std::sync::Arc;
use tokio_stream::StreamExt;

/// A collector that listens for new blocks, and generates a stream of
/// [events](NewBlock) which contain the block number and hash.
pub struct BlockCollector<T, P, N = alloy_contract::private::Ethereum> {
    provider: Arc<P>,
    _network_transport: ::core::marker::PhantomData<(N, T)>,
}

/// A new block event, containing the block number and hash.
#[derive(Debug, Clone)]
pub struct NewBlock {
    pub hash: B256,
    pub number: u64,
}

impl<
    T: alloy_contract::private::Transport + ::core::clone::Clone,
    P: alloy_contract::private::Provider<T, N>,
    N: alloy_contract::private::Network,
> BlockCollector<T, P, N> {
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider, _network_transport: ::core::marker::PhantomData }
    }
}

/// Implementation of the [Collector](Collector) trait for the [BlockCollector](BlockCollector).
/// This implementation uses the [PubsubClient](PubsubClient) to subscribe to new blocks.
#[async_trait]
impl<
    T: alloy_contract::private::Transport + ::core::clone::Clone,
    P: alloy_contract::private::Provider<T, N>,
    N: alloy_contract::private::Network,
> Collector<NewBlock> for BlockCollector<T, P, N>
{
    async fn get_event_stream<'a>(&'a self) -> Result<CollectorStream<'a, NewBlock>> {
        let sub = self.provider.subscribe_blocks().await?;
        let mut stream = sub.into_stream().take(2);
        let stream = stream.filter_map(|block| match block.requests_hash() {
            Some(hash) => Some(NewBlock { hash, number: block.number() }),
            None => None,
        });     
        Ok(Box::pin(stream))
    }
}
