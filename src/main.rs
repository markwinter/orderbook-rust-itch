use orderbook_rust::orderbook::{OrderEntry, OrderSide};
use rust_decimal::prelude::FromPrimitive;
use rust_decimal::Decimal;

const ORDER_ADD: u8 = b'A';
const ORDER_ADD_ATTRIBUTED: u8 = b'F';
const ORDER_EXECUTED: u8 = b'E';
const ORDER_EXECUTED_PRICE: u8 = b'C';
const ORDER_CANCEL: u8 = b'X';
const ORDER_DELETE: u8 = b'D';
const ORDER_REPLACE: u8 = b'U';
const STOCK_DIRECTORY: u8 = b'R';

fn main() {
    let mut processed_orders = 0;

    let mut book = orderbook_rust::orderbook::OrderBook::new(Decimal::from_f64(0.01).unwrap());

    let mut aapl_stock_directory_id = 0;

    let stream = itchy::MessageStream::from_file("01302020.NASDAQ_ITCH50").unwrap();
    for msg in stream {
        if processed_orders > 100000 {
            break;
        }

        let m = msg.unwrap();

        if m.tag == STOCK_DIRECTORY && aapl_stock_directory_id == 0 {
            if let itchy::Body::StockDirectory(dir) = &m.body {
                if *dir.stock == *"AAPL    " {
                    aapl_stock_directory_id = m.stock_locate;
                    println!("found AAPL stock id: {aapl_stock_directory_id}");
                }
            }
            continue;
        }

        if m.stock_locate != aapl_stock_directory_id {
            continue;
        }

        if m.tag == ORDER_EXECUTED {
            dbg!(&m);
            continue;
        }

        if m.tag != ORDER_ADD {
            continue;
        }

        if let itchy::Body::AddOrder(order) = &m.body {
            processed_orders += 1;

            let p: rust_decimal::Decimal = order.price.into();

            //println!(
            //    " = {},{}, {} = ",
            //    order.price.raw(),
            //    rust_decimal::Decimal::from(105),
            //    p,
            //);

            book.add_limit_order(OrderEntry {
                quantity: order.shares as u64,
                price: order.price.into(),
                order_side: if order.side == itchy::Side::Buy {
                    OrderSide::Buy
                } else {
                    OrderSide::Sell
                },
            });
        }

        //println!("{m:?}",);
    }

    //for (i, p) in book.price_levels.iter().enumerate() {
    //    println!("{},{},{}", i, p.quantity, p.depth);
    //}
}
