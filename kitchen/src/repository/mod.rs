use anyhow::{bail, Result};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, format, Formatter};
use std::str::Bytes;
use std::sync::{Arc};
use std::time::{SystemTime, UNIX_EPOCH};
use redis::{Client, AsyncCommands, Connection};
use uuid::Uuid;
use async_trait::async_trait;
use futures::prelude::*;
use tokio::sync::{Mutex, RwLock};
use serde::{Deserialize, Serialize};


const initial_menus: [&str; 36] = [
    "Tuna",
    "Lean Tuna",
    "Albacore Tune",
    "Seared Bonito",
    "Salmon",
    "Onion Salmon",
    "Broiled Fatty Salmon",
    "Broiled Fatty Salmon Radish",
    "Broiled Salmon w/ Basil Sauce",
    "Spicy Salmon & Fried Leek",
    "Salmon Basil Mozarella",
    "Young Yellowtail",
    "Pickled Yellowtail",
    "Flounder Fin",
    "Grilled Mackerel",
    "Grilled Herring Sushi",
    "Seabream",
    "Boiled Shrimp",
    "Shrimp w/ Cheese",
    "Shrimp w/ Avocado",
    "Fresh Shrimp",
    "Sweet Shrimp",
    "Abalone",
    "Black Mirugai Clam",
    "Extra Large Scallop",
    "Squid",
    "Cuttlefish",
    "Squid Ume Plum & Shiso",
    "Boiled Octopus",
    "Grilled Eel",
    "Cooked Conger Eel",
    "Premium Grill Conger Eel",
    "Japanese Egg Omelet",
    "Kalbe Beef w/ Salt",
    "Seared Wagyu Beef",
    "Imitaion Crab Meat Tempura"
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub table_id: SmallId,
    pub menu_id: SmallId,
    pub created_at: u64,
    pub processing_time: u64,
    pub status: OrderStatus
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub enum OrderStatus {
    READY,
    PROCESSING,
    CANCELLED
}


#[derive(Debug)]
pub struct Menu {
    pub id: SmallId,
    pub name: String,
}

pub type SmallId = u16;

#[derive(Debug)]
pub struct RequestError  {
    pub message: String
}

impl RequestError {
    pub fn new(message: String) -> Self {
        RequestError {message}
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "RequestError: {}", &self.message)
    }
}
impl Error for RequestError {}

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
    pub async fn new() -> Result<Self> {
        let client = Client::open("redis://127.0.0.1:6379/")?;
        let mut conn = client.get_async_connection().await?;
        for (i, m) in initial_menus.into_iter().enumerate() {
            conn.sadd("menus", i).await?;
            conn.set(format!("menus:{}", i), m).await?;
        }
        conn.sadd("tables", (1..5000).collect::<Vec<i32>>()).await?;
        Ok(Self{ client })
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
        let is_valid_table:bool =  conn.sismember("tables", table_id).await?;
        if !is_valid_table {
            bail!(RequestError::new(String::from("table is invalid")))
        }
        let mut failed_order = Vec::new();

        for o in orders.iter() {
            let is_valid_menu:bool = conn.sismember("menus", o.menu_id.clone()).await?;
            if !is_valid_menu{
                failed_order.push(o);
            }
            conn.zadd::<String, &u64, &str, bool>(format!("tables:{}", table_id),  &o.id, &o.created_at).await.unwrap();
            conn.set::<String, String, bool>(format!("tables:{}:{}", table_id, &o.id), serde_json::to_string(&o).unwrap()).await.unwrap();
        }
        println!("{:?}", failed_order);
        Ok(())
    }

    async fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
        let mut conn = self.client.get_async_connection().await?;
        let is_valid_table:bool =  conn.sismember("tables", table_id).await?;
        if !is_valid_table {
            bail!(RequestError::new(String::from("table is invalid")))
        }
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
        let is_valid_table:bool =  conn.sismember("tables", table_id).await?;
        if !is_valid_table {
            bail!(RequestError::new(String::from("table is invalid")))
        }
        conn.zrem::<String, &str, ()>(format!("tables:{}", table_id), order_id).await.unwrap();
        conn.del::<String, ()>(format!("tables:{}:{}", table_id, order_id)).await.unwrap();
        Ok(())
    }
}

// #[derive(Debug, Clone)]
// struct InMemoryRepository {
//     storage: Arc<RwLock<HashMap<SmallId, HashMap<String, Order>>>>
// }
//
// impl InMemoryRepository {
//     pub fn new() -> Self {
//         Self{
//             storage: Arc::new(RwLock::new(HashMap::new()))
//         }
//     }
// }
//
// #[async_trait]
// impl Repository for InMemoryRepository {
//     fn name(&self) -> &'static str {
//         "in_memory"
//     }
//     async fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) ->  Result<()> {
//         let mut writable_map = self.storage.write().await;
//         let table_orders = writable_map.entry(table_id.clone()).or_insert(HashMap::new());
//         orders.iter().for_each(|o| {table_orders.insert(o.id.clone(), o.clone());});
//         Ok(())
//     }
//     async fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
//         Ok(self.storage.read().await.get(table_id).unwrap_or(&HashMap::new()).into_iter().map(|e|e.1.clone()).collect())
//     }
//     async fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<()> {
//         self.storage.write().await.entry(table_id.clone()).and_modify(|table_orders| { table_orders.remove(order_id); });
//         Ok(())
//     }
// }

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::repository::{Repository, InMemoryRepository, RedisRepository, Order, SmallId, OrderStatus};

    // #[test]
    // fn test_in_memory_repo() {
    //     let mut repo = InMemoryRepository::new();
    //     repo_test(Box::new(repo));
    // }

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
