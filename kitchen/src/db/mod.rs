use anyhow::{bail, Result};
use std::collections::HashMap;
use std::ffi::CString;
use std::fmt::format;
use std::io::Bytes;
use std::time::{SystemTime, UNIX_EPOCH};
use redis::{Client, Commands, Connection};
use crate::api::{Order, SmallId, UUID};

// all entries to DB should be valid
trait DB {
    fn store_orders(&mut self, table_id: &SmallId, orders: Vec<&Order>) -> Result<()>;
    fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>>;
    fn remove_order(&mut self, table_id: &SmallId, order_id: &UUID) -> Result<()>;
}

struct RedisDB {
    client: Client
}

impl RedisDB {
    pub fn new() -> Result<Self> {
        Ok(Self{
            client : Client::open("redis://127.0.0.1:6379/")?
        })
    }
}

impl DB for RedisDB {
    fn store_orders(&mut self, table_id: &SmallId, orders: Vec<&Order>) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        for &o in orders.iter() {
            conn.zadd::<String, &u64, &UUID, bool>(format!("tables:{}", table_id),  &o.id, &o.created_at).unwrap();
            conn.set::<String, String, bool>(format!("tables:{}:{}", table_id, o.id), serde_json::to_string(&o).unwrap());
        }
        Ok(())
    }

    fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
        let mut conn = self.client.get_connection()?;
        let order_ids = conn.zrange::<String, Vec<String>>(format!("tables:{}", table_id),0, -1).unwrap();
        let orders = order_ids.iter().map(|id| {
            let order = serde_json::from_str::<Order>(&conn.get::<String, String>(format!("tables:{}:{}", table_id, id)).unwrap()).unwrap();
            order
        }).collect();
        Ok(orders)

    }

    fn remove_order(&mut self, table_id: &SmallId, order_id: &UUID) -> Result<()> {
        let mut conn = self.client.get_connection()?;
        conn.zrem::<String, &String, ()>(format!("tables:{}", table_id), order_id).unwrap();
        conn.del::<String, ()>(format!("tables:{}:{}", table_id, order_id)).unwrap();
        Ok(())
    }
}

struct InMemoryDB {
    storage: HashMap<SmallId, HashMap<UUID, Order>>
}

impl InMemoryDB {
    pub fn new() -> Self {
        Self{
            storage: HashMap::new()
        }
    }
}

impl DB for InMemoryDB {
    fn store_orders(&mut self, table_id: &SmallId, orders: Vec<&Order>) ->  Result<()> {
        let table_orders = self.storage.entry(table_id.clone()).or_insert(HashMap::new());
        orders.iter().for_each(|&o| {table_orders.insert(o.id.clone(), o.clone());});
        Ok(())
    }
    fn get_orders(&self, table_id: &SmallId) -> Result<Vec<Order>> {
        Ok(self.storage.get(table_id).unwrap_or(&HashMap::new()).into_iter().map(|e|e.1.clone()).collect())
    }
    fn remove_order(&mut self, table_id: &SmallId, order_id: &UUID) -> Result<()> {
        self.storage.entry(table_id.clone()).and_modify(|table_orders| { table_orders.remove(order_id); });
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::api::{Order, OrderStatus, SmallId, UUID};
    use crate::db::{DB, InMemoryDB, RedisDB};

    #[test]
    fn test_in_memory_db() {
        let mut db = InMemoryDB::new();
        db_test(Box::new(db));
    }

    #[test]
    fn test_redis_db() {
        let mut db = RedisDB::new().unwrap();
        db_test(Box::new(db));
    }

    fn db_test(mut db: Box<dyn DB>){
        let table_id = 1 as SmallId;
        let order = Order {
            id: String::from("test"),
            table_id: 1,
            menu_id: 1,
            created_at: 0,
            processing_time: 10,
            status: OrderStatus::PROCESSING
        };
        assert_eq!(db.store_orders(&table_id, vec![&order]).unwrap(), ());
        assert_eq!(db.get_orders(&table_id).unwrap().iter().collect::<Vec<&Order>>(), vec![&order]);
        assert_eq!(db.remove_order(&table_id, &order.id).unwrap(), ());
        assert_eq!(db.get_orders(&table_id).unwrap(), vec![]);
    }
}
