# orderbook-rust

An OrderBook for Nasdaq Itch. This is for an infinite L2 book. If we only cared about say 100 price levels, swap the Vecs for fixed arrays.

## Bench

```
add_order               time:   [159.26 ns 159.60 ns 159.91 ns]
Found 3 outliers among 100 measurements (3.00%)
  2 (2.00%) low mild
  1 (1.00%) high severe
```

## Credit

Designed with help of Charles Cooper's post [on Quant StackExchange](https://quant.stackexchange.com/a/32482) and their referenced implementation [here](https://github.com/charles-cooper/itch-order-book)

