# orderbook-rust

An OrderBook for Nasdaq Itch. This is for an infinite L2 book. If we only cared about some number of price levels, swap the Vecs for fixed arrays. If price levels are fixed, consider using price as an index into the array for free sorting.

## Bench

### Itch AAPL orders

I extracted all AAPL orders (see `src/bin/extractor.rs`) from a full day of Itch data.

There were 1,993,352 order messages which were then processed as a benchmark, re-using the same order book.

The order messages were as follows:

ADD: 907,157
DELETE: 869,314
REPLACE: 151,325
EXECUTED: 55,168
EXECUTED_PRICE: 224
CANCEL: 10,161

```
process itch messages   time:   [176.45 ms 176.96 ms 177.53 ms]
                        change: [−0.3495% +0.0917% +0.4911%] (p = 0.68 > 0.05)
                        No change in performance detected.
Found 14 outliers among 100 measurements (14.00%)
  5 (5.00%) high mild
  9 (9.00%) high severe
```

176 ms to process 1,993,352 messages gives an average of 88 ns/message

### Random orders

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

