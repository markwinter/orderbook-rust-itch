# orderbook-rust

An OrderBook for Nasdaq Itch. This is for an infinite L2 book. If we only cared about some number of price levels, swap the Vecs for fixed arrays. If price levels are fixed, consider using price as an index into the array for free sorting.

## Bench

Benched adding Buy and Sell orders across 100 price levels. Performance decreases with the number of price levels for bids or asks separately as more vector linear scanning is required. The prices are random so don't really reflect reality which is more activity at the insides. Could probably setup a better approximate distribution if I cared, or use real order data for test data similar to the main binary in this repo.

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

