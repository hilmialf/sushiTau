use std::collections::{HashMap, HashSet};
use common::api::{Order, SmallId};
use std::sync::Arc;
use rand::Rng;
use reqwest;
use futures::future::join_all;
use std::time::Instant;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let menus = reqwest::get("http://localhost:3030/menus")
        .await?
        .json::<Vec<(usize, String)>>()
        .await?;

    let menus_arc = Arc::new(menus);
    let table_nums = 100;
    let mut join_handle = Vec::with_capacity(table_nums);
    let now = Instant::now();
    for table_id in 0..table_nums {
        let menus = menus_arc.clone();
        join_handle.push(tokio::spawn( async move {
            run_client(table_id as SmallId + 1, menus).await;
        }));
    }
    join_all(join_handle).await;

    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    Ok(())
}


async fn run_client(table_id: SmallId, menus: Arc<Vec<(usize, String)>>) -> Result<(), Box<dyn std::error::Error>> {
    let num_orders = rand::thread_rng().gen_range(10..20);
    println!("initial orders for table {} is {}", table_id, num_orders);

    let mut order_menu_ids = Vec::with_capacity(num_orders);
    for i in 0..num_orders {
        order_menu_ids.push(rand::thread_rng().gen_range(1..menus.len()))
    }
    let client = reqwest::Client::new();
    let mut request_body = HashMap::new();
    request_body.insert("menu_ids", &order_menu_ids);
    let resp = client.post(format!("http://localhost:3030/orders/{}", table_id))
        .json(&request_body)
        .send()
        .await?;
    let mut submitted_orders: Vec<Order> = client.get(format!("http://localhost:3030/orders/{}", table_id))
        .send()
        .await?
        .json::<Vec<Order>>()
        .await?;
    assert_eq!(&submitted_orders.len(), &order_menu_ids.len());
    // pick random order to cancel
    let cancel_order_num = rand::thread_rng().gen_range(0..submitted_orders.len() as u64);
    for _ in 0..cancel_order_num {
        let cancel_order: Order = submitted_orders.pop().unwrap();
        let resp = client.delete(format!("http://localhost:3030/orders/{}/{}", table_id, cancel_order.id))
            .send()
            .await?;
    }
    let remaining_orders = submitted_orders.into_iter().collect::<HashSet<Order>>();

    // poll again remaining orders
    let mut last_submitted_orders: Vec<Order> = client.get(format!("http://localhost:3030/orders/{}", table_id))
        .send()
        .await?
        .json::<Vec<Order>>()
        .await?;
    let last_remaining_orders = last_submitted_orders.into_iter().collect::<HashSet<Order>>();
    assert_eq!(remaining_orders, last_remaining_orders);

    Ok(())
}