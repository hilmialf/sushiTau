extern crate core;

use std::ops::Deref;
use warp::Filter;
use serde::{Serialize,Deserialize};
use crate::filters::routes;
use crate::repository::{RedisRepository, SmallId};

mod repository;

#[derive(Deserialize, Debug)]
struct OrderRequest {
    menu_ids: Vec<SmallId>
}

#[derive(Deserialize, Debug)]
struct CancelRequest {
    order_id: SmallId
}

#[tokio::main]
async fn main() {
    let mut repository = RedisRepository::new().await.unwrap();
    let routes = routes(repository.clone());
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

mod filters {
    use std::convert::Infallible;
    use std::time::{SystemTime, UNIX_EPOCH};
    use uuid::Uuid;
    use warp::Filter;
    use crate::OrderRequest;
    use crate::repository::{Order, OrderStatus, Repository, SmallId};


    pub fn routes(
        repository: impl Repository + Clone + Send + Sync
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
            hi()
            .or(get_orders(repository.clone()))
            .or(create_orders(repository.clone()))
            .or(delete_order(repository))
    }

    pub fn create_orders(
        mut repository : impl Repository + Clone + Send + Sync
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path!("orders" / SmallId))
            .and(warp::body::json())
            .and(with_repository(repository))
            .and_then(create_orders_handler)
    }

    async fn create_orders_handler(table_id: SmallId, mut order_request : OrderRequest, mut repository: impl Repository + Clone + Send + Sync) -> Result<impl warp::Reply, warp::Rejection> {
        println!("{:?}", order_request);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let orders = order_request.menu_ids.into_iter().map(|menu_id| {
            Order{
                id: Uuid::now_v7().to_string(),
                table_id: table_id.clone(),
                menu_id: menu_id.clone(),
                created_at: now,
                processing_time: 300,
                status: OrderStatus::PROCESSING
            }
        }).collect::<Vec<Order>>();
        let res = repository.store_orders(&table_id, orders.as_slice()).await;
        match res {
            Ok(vec) =>  return Ok(format!("creating order for table {}", table_id)),
            Err(err) => {
                println!("{:?}", err);
                return Ok(format!("Some error happened"))
            }
        }
    }

    pub fn get_orders(
        mut repository : impl Repository + Clone + Send + Sync
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("orders" / SmallId))
            .and(with_repository(repository))
            .and_then(get_orders_handler)
    }

    async fn get_orders_handler(table_id: SmallId, repository: impl Repository + Clone + Send + Sync) -> Result<impl warp::Reply, warp::Rejection> {
        let res = repository.get_orders(&table_id).await;
        Ok(format!("retrieving orders for table {:?}", res.unwrap()))
    }

    pub fn delete_order(
        mut repository : impl Repository + Clone + Send + Sync
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path!("orders" / SmallId / String))
            .and(with_repository(repository))
            .and_then(delete_order_handler)
    }

    async fn delete_order_handler(table_id: SmallId, order_id: String, mut repository: impl Repository + Clone + Send + Sync) -> Result<impl warp::Reply, warp::Rejection> {
        repository.remove_order(&table_id, &order_id).await;
        Ok(format!("deleting order {} for table {}", order_id, table_id))
    }

    fn with_repository(repository : impl Repository + Clone + Send + Sync) -> impl Filter<Extract = (impl Repository + Clone + Send + Sync,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || repository.clone())
    }

    pub fn hi()
        -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get().and(warp::path("hi")).map(|| "Hello, World!")
    }

}









