Valhalla Limit Order Book Challenge
---
This is an in-memory implementation of a simple limit order book that guarantees thread safety.

This solution includes `Price-time priority` to be fair with the traders (earlier order has priority on the same
price level), allows `partial fill` so if any remainder is there, it is added to the order book.

Prerequisites
---
Latest Rust stable installed

Build
---
```shell
cargo build --release && cp ./target/release/valhalla_limit_order_book .
```

Usage
---
This application exposes an HTTP server on port `9999` locally in order to try out the functionality.

To start the app, simply run
```shell
./valhalla_limit_order_book
```

Possible Improvements
---
During the development I realized that the order book could be implemented a different way too, but to stick to the
time constraint I just finished with my initial idea, but let me explain here, what could be changed.

So instead of `Vec`s I would use `BTreeMap` to store buy and sell orders.

In `BTreeMap` I would use `u64` as the key, so the `price (level)` and `VecDeque<Order>` as value.
`VecDeque<Order>` is useful because we can iterate both from the beginning and the end, so in order to get the best price,
I simply need to call `orders.iter().next()` or `orders.iter().next_back()` on the right deque.
Also using `VecDeque<Order>` makes sure the ordering of orders is fine, so using `push_back` would put the new orders to back
and `pop_front` to get the oldest order for matching.

These changes would bring a bit more complexity to the code, but would accelerate the execution and would avoid the need to 
reorder all the time as in `Vec`.

### Example scenario (from unit tests):

#### Older order wins when they are on the same price level 

Request:
```shell
curl --location 'http://localhost:9999/place_order' \
--header 'Content-Type: application/json' \
--data '{
    "id": 1,
    "side": "buy",
    "price": 100,
    "quantity": 4
}'
```

Response (no trades happened):
```json
[]
```

---

Request:

```shell
curl --location 'http://localhost:9999/place_order' \
--header 'Content-Type: application/json' \
--data '{
    "id": 2,
    "side": "buy",
    "price": 100,
    "quantity": 5
}'
```

Response (no trades happened):
```json
[]
```

---

Request:

```shell
curl --location 'http://localhost:9999/place_order' \
--header 'Content-Type: application/json' \
--data '{
    "id": 3,
    "side": "buy",
    "price": 101,
    "quantity": 1
}'
```

Response (no trades happened):
```json
[]
```

---

Request:

```shell
curl --location 'http://localhost:9999/place_order' \
--header 'Content-Type: application/json' \
--data '{
    "id": 4,
    "side": "sell",
    "price": 100,
    "quantity": 6
}'
```

Response:
```json
[
    {
        "maker_id": 3,
        "taker_id": 4,
        "price": 101,
        "quantity": 1
    },
    {
        "maker_id": 1,
        "taker_id": 4,
        "price": 100,
        "quantity": 4
    },
    {
        "maker_id": 2,
        "taker_id": 4,
        "price": 100,
        "quantity": 1
    }
]
```

---

Checking best buy:

Request:

```shell
curl --location 'http://localhost:9999/best_buy'
```

Response:

```json
{
    "price": 100,
    "total_quantity": 4
}
```

---

Checking best sell:

Request:

```shell
curl --location 'http://localhost:9999/best_sell'
```

Response:

Response is `null` because there are no sell orders as the one sent is already fulfilled!

```json
null
```

---

**Sending another Sell Order to fulfill buys**

Request:

```shell
curl --location 'http://localhost:9999/place_order' \
--header 'Content-Type: application/json' \
--data '{
    "id": 5,
    "side": "sell",
    "price": 100,
    "quantity": 6
}'
```

Response:
```json
[
    {
        "maker_id": 2,
        "taker_id": 5,
        "price": 100,
        "quantity": 4
    }
]
```

---

Checking best buy:

Request:

```shell
curl --location 'http://localhost:9999/best_buy'
```

Response:

Response is `null` because all the buy orders are fulfilled 

```json
null
```

---

Checking best sell:

Request:

```shell
curl --location 'http://localhost:9999/best_sell'
```

Response:

Since we were asking for `6` quantity, then we had a remaining of `2` quantity at it's `limit price`.

```json
{
    "price": 100,
    "total_quantity": 2
}
```

---