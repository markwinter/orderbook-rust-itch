use std::path::PathBuf;

use criterion::{criterion_group, criterion_main, Criterion};
use itchy::MessageStream;
use orderbook_rust::orderbook::*;

const ORDER_ADD: u8 = b'A';
const ORDER_ADD_ATTRIBUTED: u8 = b'F';
const ORDER_EXECUTED: u8 = b'E';
const ORDER_EXECUTED_PRICE: u8 = b'C';
const ORDER_CANCEL: u8 = b'X';
const ORDER_DELETE: u8 = b'D';
const ORDER_REPLACE: u8 = b'U';

fn load_messages(path: &PathBuf) -> Vec<itchy::Message> {
    let stream = MessageStream::from_file(path).expect("failed to open ITCH file");
    let mut messages = Vec::with_capacity(2_000_000);
    for msg in stream.into_iter().flatten() {
        messages.push(msg);
    }
    messages
}

fn process_messages(book: &mut OrderBook, messages: &[itchy::Message]) {
    for m in messages {
        match m.tag {
            ORDER_ADD | ORDER_ADD_ATTRIBUTED => {
                let itchy::Body::AddOrder(order) = &m.body else {
                    continue;
                };
                let side = if order.side == itchy::Side::Buy {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                };
                book.add_order(order.reference, order.price.raw(), order.shares, side);
            }
            ORDER_EXECUTED => {
                let itchy::Body::OrderExecuted {
                    reference,
                    executed,
                    ..
                } = m.body
                else {
                    continue;
                };
                book.execute_order(reference, executed);
            }
            ORDER_EXECUTED_PRICE => {
                let itchy::Body::OrderExecutedWithPrice {
                    reference,
                    executed,
                    printable,
                    ..
                } = m.body
                else {
                    continue;
                };
                if !printable {
                    continue;
                }
                book.execute_order(reference, executed);
            }
            ORDER_CANCEL => {
                let itchy::Body::OrderCancelled {
                    reference,
                    cancelled,
                } = m.body
                else {
                    continue;
                };
                book.cancel_order(reference, cancelled);
            }
            ORDER_DELETE => {
                let itchy::Body::DeleteOrder { reference } = m.body else {
                    continue;
                };
                book.delete_order(reference);
            }
            ORDER_REPLACE => {
                let itchy::Body::ReplaceOrder(order) = &m.body else {
                    continue;
                };
                book.replace_order(
                    order.old_reference,
                    order.new_reference,
                    order.price.raw(),
                    order.shares,
                );
            }
            _ => {}
        }
    }
}

fn bench_orderbook(c: &mut Criterion) {
    let path = PathBuf::from("aapl_orders.itch"); // adjust path
    let messages = load_messages(&path);
    let mut book = OrderBook::new();

    println!("Loaded {} messages", messages.len());

    c.bench_function("process itch messages", |b| {
        b.iter(|| {
            process_messages(&mut book, &messages);
        });
    });
}

criterion_group!(benches, bench_orderbook);
criterion_main!(benches);
