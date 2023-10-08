use std::ops::Deref;
use crate::api::{build_kitchen, SmallId, Kitchen, KitchenApi};
use warp::Filter;
use serde::{Serialize,Deserialize};
use crate::filters::routes;
use crate::repository::RedisRepository;

mod repository;
mod api;

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
    let repository = RedisRepository::new().unwrap();
    let mut kitchen = build_kitchen(repository).await;

    let routes = routes(kitchen.clone());

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

mod filters {
    use std::convert::Infallible;
    use warp::Filter;
    use crate::api::{Kitchen, KitchenApi};
    use crate::OrderRequest;
    use crate::repository::{Repository};
    use crate::api::{SmallId};


    pub fn routes(
        kitchen: Kitchen<impl Repository + Clone + Send + Sync>
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
            hi()
            .or(get_orders(kitchen.clone()))
            .or(create_orders(kitchen.clone()))
            .or(delete_order(kitchen))
    }

    pub fn create_orders(
        mut kitchen : Kitchen<impl Repository + Clone + Send + Sync>
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path!("orders" / SmallId))
            .and(warp::body::json())
            .and(with_kitchen(kitchen))
            .and_then(create_orders_handler)
    }

    async fn create_orders_handler(table_id: SmallId, mut order_request : OrderRequest, mut kitchen: Kitchen<impl Repository + Clone + Send + Sync>) -> Result<impl warp::Reply, warp::Rejection> {
        println!("{:?}", order_request);
        let res = kitchen.order_multiple(&table_id, order_request.menu_ids.as_slice()).await;
        match res {
            Ok(vec) =>  return Ok(format!("creating order for table {}", table_id)),
            Err(err) => {
                println!("{:?}", err);
                return Ok(format!("Some error happened"))
            }
        }
        // println!("{:?}", res);
        // Ok::<std::string::String, Infallible>(format!("creating order for table {}", table_id))
    }

    pub fn get_orders(
        mut kitchen : Kitchen<impl Repository + Clone + Send + Sync>
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("orders" / SmallId))
            .and(with_kitchen(kitchen))
            .and_then(get_orders_handler)
    }

    async fn get_orders_handler(table_id: SmallId, kitchen: Kitchen<impl Repository + Clone + Send + Sync>) -> Result<impl warp::Reply, warp::Rejection> {
        let res = kitchen.list_order(table_id).await;
        Ok(format!("retrieving orders for table {:?}", res.unwrap()))
    }

    pub fn delete_order(
        mut kitchen : Kitchen<impl Repository + Clone + Send + Sync>
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path!("orders" / SmallId / String))
            .and(with_kitchen(kitchen))
            .and_then(delete_order_handler)
    }

    async fn delete_order_handler(table_id: SmallId, order_id: String, mut kitchen: Kitchen<impl Repository + Clone + Send + Sync>) -> Result<impl warp::Reply, warp::Rejection> {
        kitchen.cancel_order(table_id, order_id.clone()).await;
        Ok(format!("deleting order {} for table {}", order_id, table_id))
    }

    fn with_kitchen(kitchen : Kitchen<impl Repository + Clone + Send + Sync>) -> impl Filter<Extract = (Kitchen<impl Repository + Clone + Send + Sync>,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || kitchen.clone())
    }

    pub fn hi()
        -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get().and(warp::path("hi")).map(|| "Hello, World!")
    }

}









