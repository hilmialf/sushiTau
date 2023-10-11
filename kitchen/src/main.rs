extern crate core;

use std::fmt::{Display, Formatter};
use common::api::SmallId;
use crate::filters::routes;
use crate::repository::{RedisRepository};
use serde::{Deserialize, Serialize};
use anyhow::Result;
use log::info;


mod repository;

#[derive(Deserialize, Debug)]
pub struct OrderRequest {
    menu_ids: Vec<SmallId>
}

impl Display for OrderRequest {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let repr = self.menu_ids.iter().map(|id|format!("{}", id))
            .collect::<Vec<String>>().join(",");
        write!(f,"{}", repr)
    }
}

#[derive(Deserialize, Debug)]
pub struct CancelRequest {
    order_id: SmallId
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    let redis_url = std::env::var("REDIS_URL").unwrap_or("localhost:6379".to_string());
    info!("REDIS URL is set to {}", &redis_url);
    let mut repository = RedisRepository::new(&redis_url).await?;
    let routes = routes(repository.clone());
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    Ok(())
}

mod filters {
    use std::convert::Infallible;
    use warp::Filter;
    use crate::handler::{create_orders_handler, delete_order_handler, get_menus_handler, get_orders_handler};
    use crate::repository::{Repository};
    use common::api::SmallId;


    pub fn routes(
        repository: impl Repository + Clone + Send + Sync
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
            hi()
            .or(get_menu(repository.clone()))
            .or(get_orders(repository.clone()))
            .or(create_orders(repository.clone()))
            .or(delete_order(repository))
    }

    pub fn get_menu(
        mut repository : impl Repository + Clone + Send + Sync
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("menus"))
            .and(with_repository(repository))
            .and_then(get_menus_handler)
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


    pub fn get_orders(
        mut repository : impl Repository + Clone + Send + Sync
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("orders" / SmallId))
            .and(with_repository(repository))
            .and_then(get_orders_handler)
    }



    pub fn delete_order(
        mut repository : impl Repository + Clone + Send + Sync
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::delete()
            .and(warp::path!("orders" / SmallId / String))
            .and(with_repository(repository))
            .and_then(delete_order_handler)
    }



    fn with_repository(repository : impl Repository + Clone + Send + Sync) -> impl Filter<Extract = (impl Repository + Clone + Send + Sync,), Error = Infallible> + Clone {
        warp::any().map(move || repository.clone())
    }

    pub fn hi()
        -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get().and(warp::path("hi")).map(|| "Hello, World!")
    }

}

mod handler {
    use std::time::{SystemTime, UNIX_EPOCH};
    use common::api::{Order, OrderStatus, SmallId};
    use uuid::Uuid;
    use crate::OrderRequest;
    use crate::repository::{Repository};
    use serde::{Deserialize, Serialize};
    use rand::random;

    pub async fn create_orders_handler(table_id: SmallId, mut order_request : OrderRequest, mut repository: impl Repository + Clone + Send + Sync) -> Result<impl warp::Reply, warp::Rejection> {
        log::info!("TABLE {}: CREATE ORDER REQUEST WITH MENUS {}", &table_id, &order_request);
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let orders = order_request.menu_ids.into_iter().map(|menu_id| {
            Order{
                id: Uuid::new_v4().to_string(),
                table_id: table_id.clone(),
                menu_id: menu_id.clone(),
                created_at: now,
                processing_time: (5 + (10f32 * random::<f32>()) as u64) * 60,
                status: OrderStatus::PROCESSING
            }
        }).collect::<Vec<Order>>();
        let res = repository.store_orders(&table_id, orders.as_slice()).await;
        match res {
            Ok(failed_orders) =>  return Ok(warp::reply::json(&failed_orders)),
            Err(err) => {
                println!("{:?}", err);
                return Ok(warp::reply::json(&()))
            }
        }
    }

    pub async fn get_orders_handler(table_id: SmallId, repository: impl Repository + Clone + Send + Sync) -> Result<impl warp::Reply, warp::Rejection> {
        log::info!("TABLE {}: LIST ORDER REQUEST", table_id);
        let orders = match repository.get_orders(&table_id).await{
            Ok(orders) => orders,
            Err(e) => {
                println!("{}", e);
                return Ok(warp::reply::json(&()));
            }
        };
        Ok(warp::reply::json(&orders))
    }

    pub async fn get_menus_handler(repository: impl Repository + Clone + Send + Sync) -> Result<impl warp::Reply, warp::Rejection> {
        log::info!("GET MENU REQUEST");
        let menus = match repository.get_menus().await{
            Ok(menus) => menus,
            Err(e) => {
                println!("{}", e);
                return Ok(warp::reply::json(&()));
            }
        };
        Ok(warp::reply::json(&menus))
    }

    pub async fn delete_order_handler(table_id: SmallId, order_id: String, mut repository: impl Repository + Clone + Send + Sync) -> Result<impl warp::Reply, warp::Rejection> {
        log::info!("TABLE {}: DELETE REQUEST FOR ORDER {}", table_id, &order_id);
        match repository.remove_order(&table_id, &order_id).await {
            Ok(order) => return Ok(warp::reply::json(&order)),
            Err(e) => {
                println!("{}", e);
                return Ok(warp::reply::json(&()));
            }
        };
    }
}









