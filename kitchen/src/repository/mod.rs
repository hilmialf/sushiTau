use anyhow::{bail, Result};
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt::{Debug, format};
use std::str::Bytes;
use std::sync::{Arc};
use std::time::{SystemTime, UNIX_EPOCH};
use redis::{Client, AsyncCommands, Connection};
use uuid::Uuid;
use crate::api::{Order, SmallId};
use async_trait::async_trait;
use futures::prelude::*;
use tokio::sync::{Mutex, RwLock};


#[async_trait]
pub trait Repository {
    fn name(&self) -> &'static str;
    async fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) -> Result<()>;
    async fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>>;
    async fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<()>;
}

impl Debug for dyn Repository {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

// Cloning gives a new RedisRepository instance with new connection, either
// creating a new one or taking from pool
#[derive(Debug, Clone)]
pub struct RedisRepository {
    client: Client
}

impl RedisRepository {
    pub fn new() -> Result<Self> {
        Ok(Self{
            client : Client::open("redis://127.0.0.1:6379/")?
        })
    }
}

#[async_trait]
impl Repository for RedisRepository {
    fn name(&self) -> &'static str {
        "redis"
    }
    async fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) -> Result<()> {
        println!("storing orders to repository");
        let mut conn = self.client.get_async_connection().await?;
        for o in orders.iter() {
            conn.zadd::<String, &u64, &str, bool>(format!("tables:{}", table_id),  &o.id, &o.created_at).await.unwrap();
            conn.set::<String, String, bool>(format!("tables:{}:{}", table_id, &o.id), serde_json::to_string(&o).unwrap()).await.unwrap();
        }
        Ok(())
    }

    async fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
        let mut conn = self.client.get_async_connection().await?;
        let order_ids = conn.zrange::<String, Vec<String>>(format!("tables:{}", table_id),0, -1).await.unwrap();
        let mut orders = Vec::new();
        for id in  order_ids.iter() {
            let mut order = serde_json::from_str::<Order>(&conn.get::<String, String>(format!("tables:{}:{}", table_id, id)).await.unwrap()).unwrap();
            orders.push(order);
        };
        Ok(orders)

    }

    async fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        conn.zrem::<String, &str, ()>(format!("tables:{}", table_id), order_id).await.unwrap();
        conn.del::<String, ()>(format!("tables:{}:{}", table_id, order_id)).await.unwrap();
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct InMemoryRepository {
    storage: Arc<RwLock<HashMap<SmallId, HashMap<String, Order>>>>
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self{
            storage: Arc::new(RwLock::new(HashMap::new()))
        }
    }
}

#[async_trait]
impl Repository for InMemoryRepository {
    fn name(&self) -> &'static str {
        "in_memory"
    }
    async fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) ->  Result<()> {
        let mut writable_map = self.storage.write().await;
        let table_orders = writable_map.entry(table_id.clone()).or_insert(HashMap::new());
        orders.iter().for_each(|o| {table_orders.insert(o.id.clone(), o.clone());});
        Ok(())
    }
    async fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
        Ok(self.storage.read().await.get(table_id).unwrap_or(&HashMap::new()).into_iter().map(|e|e.1.clone()).collect())
    }
    async fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<()> {
        self.storage.write().await.entry(table_id.clone()).and_modify(|table_orders| { table_orders.remove(order_id); });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::api::{Order, OrderStatus, SmallId};
    use crate::repository::{Repository, InMemoryRepository, RedisRepository};

    #[test]
    fn test_in_memory_repo() {
        let mut repo = InMemoryRepository::new();
        repo_test(Box::new(repo));
    }

    #[test]
    fn test_redis_repo() {
        let mut repo = RedisRepository::new().unwrap();
        repo_test(Box::new(repo));
    }

    async fn repo_test(mut repo: Box<dyn Repository>){
        let table_id = 1 as SmallId;
        let order = Order {
            id: Uuid::now_v7().to_string(),
            table_id: 1,
            menu_id: 1,
            created_at: 0,
            processing_time: 10,
            status: OrderStatus::PROCESSING
        };
        let orders = vec![order.clone()];
        assert_eq!(repo.store_orders(&table_id, &orders[..]).await.unwrap(), ());
        assert_eq!(repo.get_orders(&table_id).await.unwrap(), orders[..]);
        assert_eq!(repo.remove_order(&table_id, &order.id).await.unwrap(), ());
        assert_eq!(repo.get_orders(&table_id).await.unwrap(), vec![]);
    }
}
