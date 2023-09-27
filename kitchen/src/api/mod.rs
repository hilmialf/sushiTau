use std::borrow::{Borrow, BorrowMut};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt;
use std::fmt::Formatter;
use std::time::{SystemTime, UNIX_EPOCH};
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use crate::repository::Repository;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Order {
    pub id: UUID,
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

pub type UUID = String;

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

#[derive(Debug)]
pub struct Kitchen {
    pub tables: HashSet<SmallId>,
    pub menus: HashMap<SmallId, Menu>,
    pub repository : Box<dyn Repository>
}

impl Kitchen {
    pub fn order_multiple(&mut self, table_id: &SmallId, menu_ids: &[SmallId]) -> Result<Vec<Order>> {
        if !self.is_valid_table(&table_id) || menu_ids.iter().map(|id|self.is_valid_menu(id)).any(|x| !x) {
            bail!(RequestError::new(String::from("Invalid request")))
        }
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let orders = menu_ids.into_iter().map(|menu_id| {
            Order{
                id: String::from("a") as UUID,
                table_id: table_id.clone(),
                menu_id: menu_id.clone(),
                created_at: now,
                processing_time: 300,
                status: OrderStatus::PROCESSING
            }
        }).collect::<Vec<Order>>();
        // store to Repository
        self.repository.store_orders(&table_id, orders.as_slice()).unwrap();
        Ok(orders)
    }

    pub fn list_order(&self, table_id: SmallId) -> Result<Vec<Order>> {
        if !self.is_valid_table(&table_id) {
            bail!(RequestError::new(String::from("Invalid request")))
        }
        // fetch from Repository
        unimplemented!();
    }

    pub fn cancel_order(&self, table_id: SmallId, order_id: UUID) -> Result<bool> {
        if !self.is_valid_table(&table_id) {
            bail!(RequestError::new(String::from("Invalid request")))
        }
        // store to Repository
        unimplemented!()
    }

    fn is_valid_menu(&self, menu_id: &SmallId) -> bool {
        self.menus.contains_key(menu_id)
    }

    fn is_valid_table(&self, table_id: &SmallId) -> bool {
        self.tables.contains(table_id)
    }
}
