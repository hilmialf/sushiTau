use anyhow::{bail, Result};
use std::error::Error;
use std::fmt;
use std::fmt::{Debug, Formatter};
use redis::{Client, AsyncCommands, Connection};
use async_trait::async_trait;
use common::api::{Order, SmallId};
use futures::prelude::*;
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
    async fn get_menus(&self) -> Result<Vec<(SmallId, String)>>;
    async fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) -> Result<Vec<Order>>;
    async fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>>;
    async fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<Order>;
}

impl Debug for dyn Repository {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(format!("redis://{}", redis_url))?;
        let mut conn = client.get_async_connection().await?;
        let enumerated_menus = initial_menus.into_iter().enumerate().collect::<Vec<(usize, &str)>>();
        conn.hset_multiple::<&str, usize, &str, _>("menus", &enumerated_menus).await?;
        conn.sadd("tables", (1..5000).collect::<Vec<i32>>()).await?;
        Ok(Self{ client })
    }
}

#[async_trait]
impl Repository for RedisRepository {
    fn name(&self) -> &'static str {
        "redis"
    }

    async fn get_menus(&self) -> Result<Vec<(SmallId, String)>> {
        let mut conn = self.client.get_async_connection().await?;
        let menus = conn.hgetall::<&str, Vec<(SmallId,String)>>("menus").await?;
        Ok(menus)
    }
    async fn store_orders(&mut self, table_id: &SmallId, orders: &[Order]) -> Result<Vec<Order>> {
        let mut conn = self.client.get_async_connection().await?;
        let is_valid_table:bool =  conn.sismember("tables", table_id).await?;
        if !is_valid_table {
            bail!(RequestError::new(String::from("Table is invalid")))
        }
        let mut failed_order = Vec::new();
        for o in orders.iter() {
            let (is_menu_valid, zadd_res, set_res) : (bool, bool, bool) = redis::pipe().atomic()
                .hexists("menus", o.menu_id.clone())
                .zadd::<String, &u64, &str>(format!("tables:{}", table_id),  &o.id, &o.created_at)
                .set::<String, String>(format!("tables:{}:{}", table_id, &o.id), serde_json::to_string(&o).unwrap()).query_async(&mut conn).await?;

            if !is_menu_valid {
                let (rollback_res1, rollback_res2) : (bool, bool) = redis::pipe().atomic()
                    .zrem::<String,&str>(format!("tables:{}", table_id),  &o.id)
                    .del::<String>(format!("tables:{}:{}", table_id, &o.id)).query_async(&mut conn).await?;
                failed_order.push(o.to_owned());
            }
        }
        Ok(failed_order)
    }

    async fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
        let mut conn = self.client.get_async_connection().await?;
        let is_valid_table:bool =  conn.sismember("tables", table_id).await?;
        if !is_valid_table {
            bail!(RequestError::new(String::from("Table is invalid")))
        }
        let order_ids = conn.zrange::<String, Vec<String>>(format!("tables:{}", table_id),0, -1).await.unwrap();
        let mut orders = Vec::new();
        for id in  order_ids.iter() {
            let mut order = serde_json::from_str::<Order>(&conn.get::<String, String>(format!("tables:{}:{}", table_id, id)).await.unwrap()).unwrap();
            orders.push(order);
        };
        Ok(orders)

    }

    async fn remove_order(&mut self, table_id: &SmallId, order_id: &str) -> Result<Order> {
        let mut conn = self.client.get_async_connection().await?;
        let is_valid_table:bool =  conn.sismember("tables", table_id).await?;
        if !is_valid_table {
            bail!(RequestError::new(String::from("Table is invalid")))
        }
        let (order_str, _zrem_res, _del_res) : (String, (), ()) = redis::pipe().atomic()
            .get::<String>(format!("tables:{}:{}", table_id, order_id))
            .zrem::<String, &str>(format!("tables:{}", table_id), order_id)
            .del::<String>(format!("tables:{}:{}", table_id, order_id)).query_async(&mut conn).await?;
        let mut order = serde_json::from_str::<Order>(&order_str).unwrap();
        Ok(order)
    }
}

#[cfg(test)]
mod tests {
    use common::api::OrderStatus;
    use uuid::Uuid;
    use crate::repository::{Repository, InMemoryRepository, RedisRepository};

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
