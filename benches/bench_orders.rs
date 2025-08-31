use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
use orderbook_rust::orderbook::*;
use rand::Rng;
use rust_decimal::Decimal;
use std::hint::black_box;

pub fn bench_add_order(c: &mut Criterion) {
    let mut rng = rand::rng();
    let mut next_id: u64 = 0;
    let mut ob = OrderBook::new();

    c.bench_function("add_order", |b| {
        b.iter_batched(
            || {
                let id = {
                    let id = next_id;
                    next_id += 1;
                    id
                };
                let price = Decimal::new(rng.random_range(330..380), 2);
                let volume = rng.random_range(1..10000);
                let side = if rng.random_bool(0.7) {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                };
                (id, price, volume, side)
            },
            |(id, price, volume, side)| {
                ob.add_order(
                    black_box(id),
                    black_box(price),
                    black_box(volume),
                    black_box(side),
                );
            },
            BatchSize::SmallInput,
        )
    });

    dbg!(ob.meta());
}

criterion_group!(benches, bench_add_order);
criterion_main!(benches);
