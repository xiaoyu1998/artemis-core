use crate::types::{Collector, CollectorStream};
use anyhow::Result;
use async_trait::async_trait;

use alloy::{
    contract as alloy_contract,
    rpc::types::{Filter, Log},
};

use std::sync::Arc;
use tokio_stream::StreamExt;

/// A collector that listens for new blockchain event logs based on a [Filter](Filter),
/// and generates a stream of [events](Log).
pub struct LogCollector<T, P, N = alloy_contract::private::Ethereum> {
    provider: Arc<P>,
    filter: Filter,
    _network_transport: ::core::marker::PhantomData<(N, T)>,
}

impl<
    T: alloy_contract::private::Transport + ::core::clone::Clone,
    P: alloy_contract::private::Provider<T, N>,
    N: alloy_contract::private::Network,
> LogCollector<T, P, N> {
    pub fn new(provider: Arc<P>, filter: Filter) -> Self {
        Self { provider, filter, _network_transport: ::core::marker::PhantomData}
    }
}

/// Implementation of the [Collector](Collector) trait for the [LogCollector](LogCollector).
/// This implementation uses the [PubsubClient](PubsubClient) to subscribe to new logs.
#[async_trait]
impl<
    T: alloy_contract::private::Transport + ::core::clone::Clone,
    P: alloy_contract::private::Provider<T, N>,
    N: alloy_contract::private::Network,
> Collector<Log> for LogCollector<T, P, N>
{
    async fn get_event_stream<'a>(&'a self) -> Result<CollectorStream<'a, Log>> {
        let sub = self.provider.subscribe_logs(&self.filter).await?;
        let mut stream = sub.into_stream();
        let stream = stream.filter_map(Some);
        Ok(Box::pin(stream))
    }
}
