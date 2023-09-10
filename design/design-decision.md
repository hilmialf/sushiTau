# General

# Client
Communicate via REST

# Server
## Throughput calculation
How busy can a kaiten sushi restaurant be?
1. Assuming this is a very big self-order sushi restaurant, 5000 tablets in total for both single seat and multiple seat is reasonable assumption.
    a. 3000 for single
    b. 2000 for multiple seats (4 person per table)
2. Maximum number of visitor at one time is thus 3000 + (2000 * 4) = 11000
3. On average, people spend 1 hour in a sushi restaurant:
   - On average people order 10 menus.
   - People rarely cancel order, so 10% cancellation probability is 1 cancellation in 1 hour.
   - People sometimes want to check status of their order, so 4 times checking order during 1 hour.
4. RPS estimation:
   - Full occupancy request per second: (11000 * (10 + 1 + 4))/(1 * 60) ~= 2750 rps
   - As a safety factor, lets assume 5000 RPS max at once
5. Data size estimation:
   - Assuming each order contains the following schema, total size of single record is 2 + 2 + 8 + 8 + 1 = 21B
     - table_id (uint16) (65535 max)
     - menu_id (uint16) (65535 max)
     - created_at (int64)
     - processing_time (int64)
     - status: (uint8) (id to store status between READY, CANCELLED, IN_PROCESS)
  - Assuming the data is deleted from the system (or moved somewhere to other system) after the client finish checking out, maximum data stored is
    (11000 * 10 * 21B) = 2310000 B ~= 2MB

## Selected design
- Redis is chosen as DB since:
  - Redis is fast due to being in-memory DB. The [benchmark](https://redis.io/docs/management/optimization/benchmarks/) result is far above our requirements.
  - Data size is small, so our system will remain cheap in terms of server cost.
- Rust is selected as a programming language since it is a compiled language (and it does not have GC), so final binary is small.


### Redis schema
- table:{table_id} -> sorted_set of orders in the mentioned table
- table:{table_id}:{order_id} -> hash, details of order