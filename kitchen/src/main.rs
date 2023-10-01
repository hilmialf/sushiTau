use std::ops::Deref;
use crate::api::{build_kitchen, SmallId};
use warp::Filter;
use serde::{Serialize,Deserialize};
use crate::filters::routes;
use crate::repository::get_repository;

mod repository;
mod api;

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct OrderRequest {
    menu_ids: Vec<SmallId>
}

#[derive(Deserialize, Debug)]
#[serde(crate = "rocket::serde")]
struct CancelRequest {
    order_id: SmallId
}

#[tokio::main]
async fn main() {
    let repository = get_repository();
    let mut kitchen = build_kitchen(repository);

    let routes = routes(kitchen.clone());

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}

mod filters {
    use warp::Filter;
    use crate::api::Kitchen;
    use crate::OrderRequest;
    use crate::repository::Db;
    use crate::api::{SmallId};


    pub fn routes(
        kitchen: Kitchen
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
            hi()
            .or(get_orders(kitchen.clone()))
            .or(create_orders(kitchen.clone()))
            .or(delete_order(kitchen))
    }

    pub fn create_orders(
        mut kitchen : Kitchen
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path!("orders" / SmallId))
            .and(warp::body::json())
            .and(with_kitchen(kitchen))
            .map(|table_id, mut order_request : OrderRequest, mut kitchen:Kitchen|{
                println!("{:?}", order_request);
                let res = kitchen.order_multiple(&table_id, order_request.menu_ids.as_slice()).unwrap();
                println!("{:?}", res);
                format!("creating order for table {}", table_id)
            })
    }

    pub fn get_orders(
        mut kitchen : Kitchen
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get()
            .and(warp::path!("orders" / SmallId))
            .and(with_kitchen(kitchen))
            .map(|table_id, kitchen: Kitchen|{
                let res = kitchen.list_order(table_id);
                format!("retrieving orders for table {:?}", res.unwrap())
            })
    }

    pub fn delete_order(
        mut kitchen : Kitchen
    ) -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::post()
            .and(warp::path!("orders" / SmallId / String))
            .and(with_kitchen(kitchen))
            .map(|table_id, order_id: String, kitchen: Kitchen|{
                kitchen.cancel_order(table_id, order_id.clone());
                format!("deleting order {} for table {}", order_id, table_id)
            })
    }

    fn with_kitchen(kitchen : Kitchen) -> impl Filter<Extract = (Kitchen,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || kitchen.clone())
    }

    pub fn hi()
        -> impl Filter<Extract = (impl warp::Reply,), Error = warp::Rejection> + Clone {
        warp::get().and(warp::path("hi")).map(|| "Hello, World!")
    }

}









