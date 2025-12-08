use crate::domain::accelerator::Node;

use super::errors::UsecaseError;
use super::ports::NodeRepository;

pub struct NodeUseCase<N> {
    node_repo: N,
}

impl<N> NodeUseCase<N> {
    pub fn new(node_repo: N) -> Self {
        Self { node_repo }
    }
}

impl<N> NodeUseCase<N>
where
    N: NodeRepository,
{
    pub async fn register_node(&self, node: Node) -> Result<(), UsecaseError> {
        self.node_repo.register_node(node).await?;
        Ok(())
    }

    pub async fn unregister_node(&self, node_id: &str) -> Result<(), UsecaseError> {
        self.node_repo.unregister_node(node_id).await?;
        Ok(())
    }

    pub async fn heartbeat(&self, node_id: &str) -> Result<(), UsecaseError> {
        self.node_repo.update_node_heartbeat(node_id).await?;
        Ok(())
    }

    pub async fn get_node(&self, node_id: &str) -> Result<Option<Node>, UsecaseError> {
        let node = self.node_repo.get_node(node_id).await?;
        Ok(node)
    }

    pub async fn list_nodes(&self) -> Result<Vec<Node>, UsecaseError> {
        let nodes = self.node_repo.list_nodes().await?;
        Ok(nodes)
    }

    pub async fn list_active_nodes(&self) -> Result<Vec<Node>, UsecaseError> {
        let nodes = self.node_repo.list_active_nodes().await?;
        Ok(nodes)
    }
}

