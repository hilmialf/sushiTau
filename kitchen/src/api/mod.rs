use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::future::Future;
use std::process::Output;
use std::sync::{Arc};
use tokio::sync::{Mutex, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{bail, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::repository::{Repository};
use uuid::{Timestamp, Uuid};

pub async fn build_kitchen(repository: impl Repository + Clone + Send + Sync) -> Kitchen<impl Repository + Clone + Send> {
    let mut initial_menus = vec![
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
    let mut menus = HashMap::new();
    for (i , menu) in initial_menus.iter().enumerate() {
        menus.insert(i.clone() as SmallId, Menu{id: i as SmallId, name: String::from(*menu)});
    }

    let num_tables = 5000;
    let tables = HashSet::from_iter(1..num_tables);
    Kitchen::new(Arc::new(tables),Arc::new(menus), repository.clone())
}

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

// 112 |     pub repository : Box<dyn Repository + Send>
// |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `(dyn Repository + std::marker::Send + 'static)` cannot be formatted using `{:?}` because it doesn't implement `std::fmt::Debug`
// |
// = help: the trait `std::fmt::Debug` is not implemented for `(dyn Repository + std::marker::Send + 'static)`
// = help: the following other types implement trait `std::fmt::Debug`:
// (dyn Repository + 'static)
// (dyn tracing_core::field::Value + 'static)
// (dyn std::any::Any + 'static)
// (dyn std::any::Any + std::marker::Send + 'static)
// (dyn std::any::Any + std::marker::Send + Sync + 'static)
// = note: this error originates in the derive macro `Debug` (in Nightly builds, run with -Z macro-backtrace for more info)
// #[derive(Debug)]
#[derive(Clone)]
pub struct Kitchen<Repo: Clone> {
    pub tables: Arc<HashSet<SmallId>>,
    pub menus: Arc<HashMap<SmallId, Menu>>,
    pub repository : Repo
}

#[async_trait]
pub trait KitchenApi {
    async fn order_multiple(&mut self, table_id: &SmallId, menu_ids: &[SmallId]) -> Result<Vec<Order>>;
    async fn list_order(&self, table_id: SmallId) -> Result<Vec<Order>>;
    async fn cancel_order(&mut self, table_id: SmallId, order_id: String) -> Result<bool>;
}

impl<Repo: Repository + Clone + Send + Sync> Kitchen<Repo> {
    fn new(tables: Arc<HashSet<SmallId>>, menus: Arc<HashMap<SmallId, Menu>>, repository: Repo) -> Self{
        Kitchen{ tables, menus, repository }
    }

    fn is_valid_menu(&self, menu_id: &SmallId) -> bool {
        self.menus.contains_key(menu_id)
    }

    fn is_valid_menus(&self, menu_ids: &[SmallId]) -> bool {
        menu_ids.iter().map(|id|{
            self.menus.contains_key(id)
        }).all(|x|x)
    }

    fn is_valid_table(&self, table_id: &SmallId) -> bool {
        self.tables.contains(table_id)
    }
}

#[async_trait]
impl<Repo: Repository + Clone + Send + Sync> KitchenApi for Kitchen<Repo> {
    async fn order_multiple(&mut self, table_id: &SmallId, menu_ids: &[SmallId]) -> Result<Vec<Order>> {
        let is_valid_table = self.is_valid_table(&table_id);
        if !is_valid_table {
            println!("{}", table_id);
            bail!(RequestError::new(String::from("table is invalid")))
        }
        let is_valid_menus = self.is_valid_menus(menu_ids);
        if !is_valid_menus {
            bail!(RequestError::new(String::from("menu is invalid")))
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let orders = menu_ids.into_iter().map(|menu_id| {
            Order{
                id: Uuid::now_v7().to_string(),
                table_id: table_id.clone(),
                menu_id: menu_id.clone(),
                created_at: now,
                processing_time: 300,
                status: OrderStatus::PROCESSING
            }
        }).collect::<Vec<Order>>();
        // store to Repository
        println!("{:?}", orders);
        println!("{:?}", self.repository.name());
        self.repository.store_orders(&table_id, orders.as_slice()).await;
        Ok(orders)
    }

    async fn list_order(&self, table_id: SmallId) -> Result<Vec<Order>> {
        let is_valid_table = self.is_valid_table(&table_id);
        if !is_valid_table {
            bail!(RequestError::new(String::from("Invalid request")))
        }
        self.repository.get_orders(&table_id).await
    }

    async fn cancel_order(&mut self, table_id: SmallId, order_id: String) -> Result<bool> {
        if !self.is_valid_table(&table_id) {
            bail!(RequestError::new(String::from("Invalid request")))
        }
        self.repository.remove_order(&table_id, &order_id).await;

        Ok(true)
    }
}
