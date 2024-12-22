use std::{
    ops::{Div, Mul},
    sync::Arc,
};

use crate::types::Executor;
use anyhow::{Context, Result};
use async_trait::async_trait;
use alloy::{
    contract as alloy_contract,
    primitives::{U256, U64, U128},
    network::{Ethereum, Network, TransactionBuilder},
};

/// An executor that sends transactions to the mempool.
pub struct MempoolExecutor<T, P, N = alloy_contract::private::Ethereum> {
    client: Arc<P>,
    _network_transport: ::core::marker::PhantomData<(N, T)>,
}

/// Information about the gas bid for a transaction.
#[derive(Debug, Clone)]
pub struct GasBidInfo {
    /// Total profit expected from opportunity
    pub total_profit: U256,

    /// Percentage of bid profit to use for gas
    pub bid_percentage: u64,
}

#[derive(Debug, Clone)]
pub struct SubmitTxToMempool<N: alloy::providers::Network> {
    //pub tx: alloy_contract::private::Network::TransactionRequest,
    pub tx: <N as Network>::TransactionRequest,
    pub gas_bid_info: Option<GasBidInfo>,
}

impl<
    T: alloy_contract::private::Transport + ::core::clone::Clone,
    P: alloy_contract::private::Provider<T, N>,
    N: alloy_contract::private::Network,
> MempoolExecutor<T, P, N> {
    pub fn new(client: Arc<P>) -> Self {
        Self { client, _network_transport: ::core::marker::PhantomData }
    }
}

#[async_trait]
impl<
    T: alloy_contract::private::Transport + ::core::clone::Clone,
    P: alloy_contract::private::Provider<T, N>,
    N: alloy_contract::private::Network,
> Executor<SubmitTxToMempool<N>> for MempoolExecutor<T, P, N>
{
    /// Send a transaction to the mempool.
    async fn execute(&self, mut action: SubmitTxToMempool<N>) -> Result<()> {
        let gas_usage = self
            .client
            .estimate_gas(&action.tx)
            .await
            .context("Error estimating gas usage: {}")?;

        let bid_gas_price:u128;
        //TODO:Should be calc from profit
        // if let Some(gas_bid_info) = action.gas_bid_info {
        //     // gas price at which we'd break even, meaning 100% of profit goes to validator

        //     let breakeven_gas_price:U128 = gas_bid_info.total_profit / gas_usage.into();
        //     // gas price corresponding to bid percentage
        //     bid_gas_price = breakeven_gas_price
        //         .mul(gas_bid_info.bid_percentage.into())
        //         .div(100)
        //         .into();
        // } else {
            bid_gas_price = self
                .client
                .get_gas_price()
                .await
                .context("Error getting gas price: {}")?;
        // }
        action.tx.set_gas_price(bid_gas_price);
        //action.tx.gas_price = Some(bid_gas_price.into());
        self.client
        .send_transaction(action.tx)
        .await?
        .get_receipt()
        .await?;
        
        Ok(())
    }
}
