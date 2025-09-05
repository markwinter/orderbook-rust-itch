use clap::Parser;

use orderbook_rust::orderbook::{OrderBook, OrderSide};

const ORDER_ADD: u8 = b'A';
const ORDER_ADD_ATTRIBUTED: u8 = b'F';
const ORDER_EXECUTED: u8 = b'E';
const ORDER_EXECUTED_PRICE: u8 = b'C';
const ORDER_CANCEL: u8 = b'X';
const ORDER_DELETE: u8 = b'D';
const ORDER_REPLACE: u8 = b'U';

#[derive(Parser)]
struct Args {
    file: String,

    #[arg(long)]
    max_messages: Option<usize>,
}

fn main() {
    let args = Args::parse();

    let stream = itchy::MessageStream::from_file(args.file).unwrap();

    let mut book = OrderBook::new();

    for (processed, msg) in stream.enumerate() {
        if let Some(max) = args.max_messages {
            if processed > max {
                dbg!(&book.spread());
                dbg!(&book.best_bid());
                dbg!(&book.best_ask());
                dbg!(&book.meta());
                return;
            }
        }
        let m = msg.unwrap();

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
            _ => continue,
        }
    }
}
