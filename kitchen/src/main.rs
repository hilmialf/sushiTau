use std::collections::{HashMap, HashSet};
use crate::api::{Kitchen, Menu};
mod db;
fn main() {
    let mut menus = HashMap::new();
    menus.insert(1, Menu{id: 1, name: String::from("nigiri1")});
    menus.insert(2, Menu{id: 2, name: String::from("nigiri2")});
    menus.insert(3, Menu{id: 3, name: String::from("nigiri3")});
    menus.insert(4, Menu{id: 4, name: String::from("nigiri4")});
    menus.insert(5, Menu{id: 5, name: String::from("nigiri5")});
    menus.insert(6, Menu{id: 6, name: String::from("nigiri6")});
    menus.insert(7, Menu{id: 7, name: String::from("nigiri7")});
    menus.insert(8, Menu{id: 8, name: String::from("nigiri8")});
    menus.insert(9, Menu{id: 9, name: String::from("nigiri9")});
    menus.insert(10, Menu{id: 10, name: String::from("nigiri10")});

    // let tables = HashSet::from_iter((1..5000));
    //
    // let kitchen = Kitchen{tables,menus};





}

pub mod api {
    use std::borrow::{Borrow, BorrowMut};
    use std::collections::{HashMap, HashSet};
    use std::error::Error;
    use std::fmt;
    use std::fmt::Formatter;
    use anyhow::{bail, Result};
    use serde::{Deserialize, Serialize};


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
        // pub db : Box<dyn DB>
    }

    impl Kitchen {
        pub fn order_multiple(&self, table_id: SmallId, menu_ids: Vec<SmallId>) -> Result<Vec<Order>> {
            if !self.is_valid_table(&table_id) || menu_ids.iter().map(|id|self.is_valid_menu(id)).any(|x| !x) {
                bail!(RequestError::new(String::from("Invalid request")))
            }
            // store to DB
            unimplemented!()
        }

        pub fn list_order(&self, table_id: SmallId) -> Result<Vec<Order>> {
            if !self.is_valid_table(&table_id) {
                bail!(RequestError::new(String::from("Invalid request")))
            }
            // fetch from DB
            unimplemented!();
        }

        pub fn cancel_order(&self, table_id: SmallId, order_id: UUID) -> Result<bool> {
            if !self.is_valid_table(&table_id) {
                bail!(RequestError::new(String::from("Invalid request")))
            }
            // store to DB
            unimplemented!()
        }

        fn is_valid_menu(&self, menu_id: &SmallId) -> bool {
            self.menus.contains_key(menu_id)
        }

        fn is_valid_table(&self, table_id: &SmallId) -> bool {
            self.tables.contains(table_id)
        }
    }
}






