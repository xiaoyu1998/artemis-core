use async_trait::async_trait;

use alloy::{
    contract as alloy_contract, 
    //rpc::types::Transaction
};


use futures::StreamExt;
use std::sync::Arc;

use crate::types::{Collector, CollectorStream};
use anyhow::Result;

/// A collector that listens for new transactions in the mempool, and generates a stream of
/// [events](Transaction) which contain the transaction.
pub struct MempoolCollector<T, P, N = alloy_contract::private::Ethereum> {
    provider: Arc<P>,
    _network_transport: ::core::marker::PhantomData<(N, T)>,
}

impl<
    T: alloy_contract::private::Transport + ::core::clone::Clone,
    P: alloy_contract::private::Provider<T, N>,
    N: alloy_contract::private::Network,
> MempoolCollector<T, P, N> {
    pub fn new(provider: Arc<P>) -> Self {
        Self { provider, _network_transport: ::core::marker::PhantomData}
    }
}

/// Implementation of the [Collector](Collector) trait for the [MempoolCollector](MempoolCollector).
/// This implementation uses the [PubsubClient](PubsubClient) to subscribe to new transactions.
#[async_trait]
impl<
    T: alloy_contract::private::Transport + ::core::clone::Clone,
    P: alloy_contract::private::Provider<T, N>,
    N: alloy_contract::private::Network,
> Collector<N::TransactionResponse> for MempoolCollector<T, P, N>
{
    async fn get_event_stream<'a>(&'a self) -> Result<CollectorStream<'a, N::TransactionResponse>> {
        let sub = self.provider.subscribe_pending_transactions().await?;
        let mut stream = sub.into_stream().take(256);
        let stream = stream.filter_map(|tx_hash| {
            let provider = self.provider.clone(); // Clone provider if required
            async move {
                match provider.get_transaction_by_hash(tx_hash).await {
                    Ok(Some(transaction)) => Some(transaction),
                    Ok(None) => None, // Transaction not found
                    Err(err) => {
                        eprintln!("Error fetching transaction: {:?}", err); // Log the error
                        None // Drop the errored transactions
                    }
                }
            }
        });
        Ok(Box::pin(stream))
    }
}
