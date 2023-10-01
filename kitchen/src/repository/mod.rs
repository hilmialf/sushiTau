use anyhow::{bail, Result};
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt::{Debug, format};
use std::str::Bytes;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};
use redis::{Client, Commands, Connection};
use uuid::Uuid;
use crate::api::{Order, SmallId};


pub type Db = Arc<Mutex<Box<dyn Repository + Send>>>;
// all entries to DB should be valid
pub fn get_repository() -> Db {
    // Arc::new(Mutex::new(Box::new(InMemoryRepository::new())))
    Arc::new(Mutex::new(Box::new(RedisRepository::new().unwrap())))

}

pub trait Repository {
    fn name(&self) -> &'static str;
    fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) -> Result<()>;
    fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>>;
    fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<()>;
}

impl Debug for dyn Repository {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug)]
struct RedisRepository {
    client: Client
}

impl RedisRepository {
    pub fn new() -> Result<Self> {
        Ok(Self{
            client : Client::open("redis://127.0.0.1:6379/")?
        })
    }
}

impl Repository for RedisRepository {
    fn name(&self) -> &'static str {
        "redis"
    }
    fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) -> Result<()> {
        println!("storing orders to repository");
        let mut conn = self.client.get_connection()?;
        for o in orders.iter() {
            conn.zadd::<String, &u64, &str, bool>(format!("tables:{}", table_id),  &o.id, &o.created_at).unwrap();
            conn.set::<String, String, bool>(format!("tables:{}:{}", table_id, &o.id), serde_json::to_string(&o).unwrap());
        }
        Ok(())
    }

    fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
        let mut conn = self.client.get_connection()?;
        let order_ids = conn.zrange::<String, Vec<String>>(format!("tables:{}", table_id),0, -1).unwrap();
        let orders = order_ids.iter().map(|id| {
            let mut order = serde_json::from_str::<Order>(&conn.get::<String, String>(format!("tables:{}:{}", table_id, id)).unwrap()).unwrap();
            order
        }).collect();
        Ok(orders)

    }

    fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        conn.zrem::<String, &str, ()>(format!("tables:{}", table_id), order_id).unwrap();
        conn.del::<String, ()>(format!("tables:{}:{}", table_id, order_id)).unwrap();
        Ok(())
    }
}

#[derive(Debug)]
struct InMemoryRepository {
    storage: HashMap<SmallId, HashMap<String, Order>>
}

impl InMemoryRepository {
    pub fn new() -> Self {
        Self{
            storage: HashMap::new()
        }
    }
}

impl Repository for InMemoryRepository {
    fn name(&self) -> &'static str {
        "in_memory"
    }
    fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) ->  Result<()> {
        let table_orders = self.storage.entry(table_id.clone()).or_insert(HashMap::new());
        orders.iter().for_each(|o| {table_orders.insert(o.id.clone(), o.clone());});
        Ok(())
    }
    fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
        Ok(self.storage.get(table_id).unwrap_or(&HashMap::new()).into_iter().map(|e|e.1.clone()).collect())
    }
    fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<()> {
        self.storage.entry(table_id.clone()).and_modify(|table_orders| { table_orders.remove(order_id); });
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

    fn repo_test(mut repo: Box<dyn Repository>){
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
        assert_eq!(repo.store_orders(&table_id, &orders[..]).unwrap(), ());
        assert_eq!(repo.get_orders(&table_id).unwrap(), orders[..]);
        assert_eq!(repo.remove_order(&table_id, &order.id).unwrap(), ());
        assert_eq!(repo.get_orders(&table_id).unwrap(), vec![]);
    }
}
