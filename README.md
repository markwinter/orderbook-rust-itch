# orderbook-rust

An OrderBook for Nasdaq Itch. This is for an infinite L2 book. If we only cared about say 100 price levels, swap the Vecs for fixed arrays.

## Bench

Benched adding Buy and Sell orders across 100 price levels. Performance decreases with the number of price levels for bids or asks separately as more vector linear scanning is required.

```
add_order               time:   [30.861 ns 30.981 ns 31.098 ns]
                        change: [−1.6560% +0.2015% +1.8561%] (p = 0.84 > 0.05)
                        No change in performance detected.
Found 3 outliers among 100 measurements (3.00%)
  1 (1.00%) high mild
  2 (2.00%) high severe
```

## Credit

Designed with help of Charles Cooper's post [on Quant StackExchange](https://quant.stackexchange.com/a/32482) and their referenced implementation [here](https://github.com/charles-cooper/itch-order-book)

