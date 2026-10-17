use chain::ChainSource;
use chaincash_predicate::predicates::Predicate;
use chaincash_store::ChainCashStore;
use compiler::Compiler;
use ergo_client::node::{endpoints::wallet::StatusResponse, NodeClient, NodeError};
use ergo_lib::{ergo_chain_types::EcPoint, ergotree_ir::chain::address::Address};
use transaction::{TransactionService, TransactionServiceError};

pub mod chain;
pub mod compiler;
pub mod scanner;
pub mod transaction;

#[derive(Clone)]
pub struct ServerState {
    pub store: ChainCashStore,
    pub node: NodeClient,
    pub chain: ChainSource,
    compiler: Compiler,
    pub predicates: Vec<Predicate>,
    wallet_password: Option<String>,
}

impl ServerState {
    pub fn new(
        node: NodeClient,
        api_key: &str,
        store: ChainCashStore,
        predicates: Vec<Predicate>,
    ) -> Result<Self, NodeError> {
        Self::with_wallet_password(node, api_key, store, predicates, None)
    }

    pub fn with_wallet_password(
        node: NodeClient,
        api_key: &str,
        store: ChainCashStore,
        predicates: Vec<Predicate>,
        wallet_password: Option<String>,
    ) -> Result<Self, NodeError> {
        let chain = ChainSource::new(node.clone(), api_key)?;
        Ok(ServerState {
            compiler: Compiler::new(chain.clone()),
            chain,
            node,
            store,
            predicates,
            wallet_password,
        })
    }

    pub async fn ensure_wallet_unlocked(&self) -> Result<StatusResponse, TransactionServiceError> {
        self.tx_service().ensure_wallet_unlocked().await
    }

    pub async fn wallet_pubkeys(&self) -> Result<Vec<EcPoint>, TransactionServiceError> {
        self.ensure_wallet_unlocked().await?;
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
        TransactionService::with_wallet_password(
            &self.node,
            &self.chain,
            &self.store,
            &self.compiler,
            self.wallet_password.as_deref(),
        )
    }
}
