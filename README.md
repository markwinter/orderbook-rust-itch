# orderbook-rust

Experimenting with OrderBook implementations

1. Array of PriceLevels containg a Vec of Orders 
2. Array of PriceLevels using slab for Vec to pre-allocate
3. Array of PriceLevels and intrusive list of Orders
4. Some tree or hashmap of PriceLevels
5. Skiplist?

Some questions to think about:
- Do we need fast deletes
- How easy to keep metadata about Orders
    - Volume at a level
    - Volume between PriceLevels
    - Depth of a PriceLevel and depth of a particular order
- We can do better pre-alloc if we know general metadata (avg daily num of trades, orders etc) about a particular symbol

Measure:
- Insert order
- Cancel order
- Memory usage
- Cpu usage

Use Itch data and process a days worth of orders for a high volume equity
