The project is divided into:
- Kitchen (API)
- Table (Client)


API can be run with the provided docker-compose:
`docker compose up`

or just running using cargo run:
`cd kitchen`
`RUST_LOG=info cargo run`


Then, for the client, we can just run directly via cargo run for stress test
`cd table`
`cargo run`

or just using curl
`curl -X GET localhost:3030/menus`
`curl -X POST localhost:3030/orders/1 -H "Content-Type: application/json" -d '{"menu_ids": [1,2,3,45, 51]}`
`curl -X GET localhost:3030/orders/1`
`curl -X DELETE localhost:3030/orders/1/018b146a-34ee-7d74-ac47-02596b6816db`  
