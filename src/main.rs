use clap::Parser;
use std::collections::HashMap;

use orderbook_rust::orderbook::{OrderBook, OrderSide};

const ORDER_ADD: u8 = b'A';
const ORDER_ADD_ATTRIBUTED: u8 = b'F';
const ORDER_EXECUTED: u8 = b'E';
const ORDER_EXECUTED_PRICE: u8 = b'C';
const ORDER_CANCEL: u8 = b'X';
const ORDER_DELETE: u8 = b'D';
const ORDER_REPLACE: u8 = b'U';
const STOCK_DIRECTORY: u8 = b'R';

#[derive(Parser)]
struct Args {
    file: String,
    symbol: String,
}

fn main() {
    let args = Args::parse();

    let stream = itchy::MessageStream::from_file(args.file).unwrap();

    let mut stock_directory: HashMap<String, u16> = HashMap::with_capacity(5000);

    let mut book = OrderBook::new();

    let mut processed = 0;

    for msg in stream {
        if processed > 100_000 {
            dbg!(&book.spread());
            dbg!(&book.best_bid());
            dbg!(&book.best_ask());
            dbg!(&book.meta());
            return;
        }
        let m = msg.unwrap();

        match m.tag {
            STOCK_DIRECTORY => {
                let itchy::Body::StockDirectory(dir) = &m.body else {
                    continue;
                };

                let stock = dir.stock.trim_end().to_lowercase();

                stock_directory.insert(stock, m.stock_locate);
            }
            ORDER_ADD | ORDER_ADD_ATTRIBUTED => {
                let aapl = stock_directory[&args.symbol];
                if m.stock_locate != aapl {
                    continue;
                }

                processed += 1;

                let itchy::Body::AddOrder(order) = &m.body else {
                    continue;
                };

                let side = if order.side == itchy::Side::Buy {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                };

                book.add_order(
                    order.reference,
                    order.price.raw(),
                    order.shares as u64,
                    side,
                );
            }
            ORDER_EXECUTED => {
                let aapl = stock_directory[&args.symbol];
                if m.stock_locate != aapl {
                    continue;
                }

                let itchy::Body::OrderExecuted {
                    reference,
                    executed,
                    ..
                } = m.body
                else {
                    continue;
                };

                book.execute_order(reference, executed as u64);
            }
            ORDER_EXECUTED_PRICE => {
                let aapl = stock_directory[&args.symbol];
                if m.stock_locate != aapl {
                    continue;
                }

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

                book.execute_order(reference, executed as u64);
            }
            ORDER_CANCEL => {
                let aapl = stock_directory[&args.symbol];
                if m.stock_locate != aapl {
                    continue;
                }

                let itchy::Body::OrderCancelled {
                    reference,
                    cancelled,
                } = m.body
                else {
                    continue;
                };
                book.cancel_order(reference, cancelled as u64);
            }
            ORDER_DELETE => {
                let aapl = stock_directory[&args.symbol];
                if m.stock_locate != aapl {
                    continue;
                }

                let itchy::Body::DeleteOrder { reference } = m.body else {
                    continue;
                };

                book.delete_order(reference);
            }
            ORDER_REPLACE => {
                let aapl = stock_directory[&args.symbol];
                if m.stock_locate != aapl {
                    continue;
                }

                let itchy::Body::ReplaceOrder(order) = &m.body else {
                    continue;
                };

                book.replace_order(
                    order.old_reference,
                    order.new_reference,
                    order.price.raw(),
                    order.shares as u64,
                );
            }
            _ => continue,
        }
    }
}
