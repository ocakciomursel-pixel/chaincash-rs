use chain::ChainSource;
use chaincash_predicate::predicates::Predicate;
use chaincash_store::ChainCashStore;
use compiler::Compiler;
use ergo_client::node::{NodeClient, NodeError};
use ergo_lib::{ergo_chain_types::EcPoint, ergotree_ir::chain::address::Address};
use transaction::{TransactionService, TransactionServiceError};
use wallet::LocalWallet;

pub mod chain;
pub mod compiler;
pub mod scanner;
pub mod transaction;
pub mod wallet;

#[derive(Clone)]
pub struct ServerState {
    pub store: ChainCashStore,
    pub node: NodeClient,
    pub chain: ChainSource,
    compiler: Compiler,
    pub predicates: Vec<Predicate>,
    pub wallet: Option<LocalWallet>,
}

impl ServerState {
    pub fn new(
        node: NodeClient,
        api_key: &str,
        store: ChainCashStore,
        predicates: Vec<Predicate>,
    ) -> Result<Self, NodeError> {
        Self::with_wallet(node, api_key, store, predicates, None)
    }

    pub fn with_wallet(
        node: NodeClient,
        api_key: &str,
        store: ChainCashStore,
        predicates: Vec<Predicate>,
        wallet: Option<LocalWallet>,
    ) -> Result<Self, NodeError> {
        let chain = ChainSource::new(node.clone(), api_key)?;
        Ok(ServerState {
            compiler: Compiler::new(chain.clone()),
            chain,
            node,
            store,
            predicates,
            wallet,
        })
    }

    pub async fn wallet_pubkeys(&self) -> Result<Vec<EcPoint>, TransactionServiceError> {
        if let Some(wallet) = &self.wallet {
            return Ok(wallet.pubkeys());
        }
        Ok(self
            .node
            .endpoints()
            .wallet()?
            .get_addresses()
            .await?
            .into_iter()
            .filter_map(|addr| match addr.address() {
                Address::P2Pk(provedlog) => Some(*provedlog.h),
                _ => None,
            })
            .collect())
    }

    pub fn tx_service(&self) -> TransactionService {
        TransactionService::with_wallet(
            &self.node,
            &self.chain,
            &self.store,
            &self.compiler,
            self.wallet.as_ref(),
        )
    }
}
