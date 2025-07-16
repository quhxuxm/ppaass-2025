use deadpool::managed::{Manager, RecycleResult};

use crate::{
    Error,
    proxy::{ProxyConnection, ProxyFramed},
};

pub struct ProxyConnectionManager {}

impl Manager for ProxyConnectionManager {
    type Type = ProxyConnection<ProxyFramed>;

    type Error = Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        todo!()
    }

    async fn recycle(
        &self,
        obj: &mut Self::Type,
        metrics: &deadpool::managed::Metrics,
    ) -> RecycleResult<Self::Error> {
        todo!()
    }
}
