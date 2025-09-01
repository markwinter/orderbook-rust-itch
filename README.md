# orderbook-rust

An OrderBook for Nasdaq Itch. This is for an infinite L2 book. If we only cared about say 100 price levels, swap the Vecs for fixed arrays.

## Bench

```
add_order               time:   [166.99 ns 167.49 ns 167.93 ns]
Found 2 outliers among 100 measurements (2.00%)
  2 (2.00%) high mild
```

## Credit

Designed with help of Charles Cooper's post [on Quant StackExchange](https://quant.stackexchange.com/a/32482) and their referenced implementation [here](https://github.com/charles-cooper/itch-order-book)

