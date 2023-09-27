use std::collections::{HashMap, HashSet};
use std::thread;
use crate::api::{Kitchen, Menu};
use crate::repository::get_repository;

mod repository;
mod api;

#[tokio::main]
async fn main() {
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



    // let num_tables = 5000;
    // let tables = HashSet::from_iter(1..num_tables);
    // //
    // let mut kitchen = Kitchen{
    //     tables,
    //     menus,
    //     repository: get_repository()};
    //
    // for customer in 1..num_tables {
    //     thread::spawn(||{
    //         // order from kitchen
    //     });
    // }

}









