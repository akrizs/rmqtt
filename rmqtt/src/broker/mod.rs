use anyhow::Result;
use std::iter::Iterator;

use crate::broker::session::{Connection, Session};
use crate::broker::types::*;
use crate::settings::listener::Listener;
use crate::{ClientId, Id, NodeId, QoS, Topic, TopicFilter};

pub mod default;
pub mod error;
pub mod fitter;
pub mod hook;
pub mod inflight;
pub mod queue;
pub mod retain;
pub mod session;
pub mod topic;
pub mod types;
pub mod v3;
pub mod v5;

#[async_trait]
pub trait Entry: Sync + Send {
    fn try_lock(&self) -> Result<Box<dyn Entry>>;
    async fn set(&mut self, session: Session, tx: Tx, conn: Connection) -> Result<()>;
    async fn remove(&mut self) -> Result<Option<(Session, Tx, Connection)>>;
    async fn kick(&mut self, clear_subscriptions: bool) -> Result<Option<(Session, Connection)>>;
    async fn is_connected(&self) -> bool;
    async fn session(&self) -> Option<Session>;
    async fn connection(&self) -> Option<Connection>;
    fn tx(&self) -> Option<Tx>;
    async fn subscribe(&self, subscribe: Subscribe) -> Result<SubscribeAck>;
    async fn unsubscribe(&self, unsubscribe: &Unsubscribe) -> Result<UnsubscribeAck>;
    async fn forward(&self, from: From, publish: Publish) -> Result<(), (From, Publish, Reason)>;
}

#[async_trait]
pub trait Shared: Sync + Send {
    ///
    fn entry(&self, id: Id) -> Box<dyn Entry>;

    ///
    async fn forwards(
        &self,
        from: From,
        publish: Publish,
    ) -> Result<(), Vec<(To, From, Publish, Reason)>>;

    ///Returns the number of current node connections
    async fn connections(&self) -> usize;

    ///Returns the number of current node sessions
    async fn sessions(&self) -> usize;

    ///
    fn iter(&self) -> Box<dyn Iterator<Item = Box<dyn Entry>> + Sync + Send>;

    ///
    fn random_session(&self) -> Option<(Session, Connection)>;
}

#[async_trait]
pub trait Router: Sync + Send {
    async fn add(
        &self,
        topic_filter: &TopicFilter,
        node_id: NodeId,
        client_id: &str,
        qos: QoS,
    ) -> Result<()>;

    async fn remove(
        &self,
        topic_filter: &TopicFilter,
        node_id: NodeId,
        client_id: &str,
    ) -> Result<()>;

    async fn matches(
        &self,
        topic: &Topic,
    ) -> (
        Vec<(TopicFilter, ClientId, QoS)>,
        std::collections::HashMap<NodeId, Vec<TopicFilter>>,
    );

    ///get current node id
    async fn get_node_id(&self) -> NodeId;

    ///get router infos, by top n
    fn list(&self, top: usize) -> Vec<String>;
}

#[async_trait]
pub trait RetainStorage: Sync + Send {
    ///topic - concrete topic
    async fn set(&self, topic: &Topic, retain: Retain) -> Result<()>;

    ///topic_filter - Topic filter
    async fn get(&self, topic_filter: &Topic) -> Result<Vec<(Topic, Retain)>>;
}

#[async_trait]
pub trait LimiterManager: Sync + Send {
    fn get(&self, name: String, listen_cfg: Listener) -> Result<Box<dyn Limiter>>;
}

#[async_trait]
pub trait Limiter: Sync + Send {
    async fn acquire_one(&self) -> Result<()>;
    async fn acquire(&self, amount: usize) -> Result<()>;
}