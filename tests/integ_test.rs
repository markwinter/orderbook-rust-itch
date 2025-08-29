use orderbook_rust::orderbook;
use rust_decimal::Decimal;

#[test]
fn test_only_buys() {
    let mut book = orderbook::OrderBook::default();

    for i in 10..1000 {
        book.add_limit_order(orderbook::OrderEntry {
            price: Decimal::from(i),
            quantity: 10,
            order_side: orderbook::OrderSide::Buy,
        });
    }

    assert_eq!(book.bid_max(), Decimal::from(999));
    assert_eq!(book.ask_min().normalize(), Decimal::from(1000000));
}

#[test]
fn test_only_sells() {
    let mut book = orderbook::OrderBook::default();

    for i in 10..1000 {
        book.add_limit_order(orderbook::OrderEntry {
            price: Decimal::from(i),
            quantity: 10,
            order_side: orderbook::OrderSide::Sell,
        });
    }

    assert_eq!(book.bid_max(), Decimal::from(0));
    assert_eq!(book.ask_min(), Decimal::from(10));
}

#[test]
fn test_simple_match() {
    let mut book = orderbook::OrderBook::default();

    book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(10),
        quantity: 10,
        order_side: orderbook::OrderSide::Sell,
    });

    book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(11),
        quantity: 10,
        order_side: orderbook::OrderSide::Sell,
    });

    book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(9),
        quantity: 10,
        order_side: orderbook::OrderSide::Buy,
    });

    book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(10),
        quantity: 3,
        order_side: orderbook::OrderSide::Buy,
    });

    assert_eq!(book.bid_max().normalize(), Decimal::from(9));
    assert_eq!(book.ask_min().normalize(), Decimal::from(10));

    let p = &book.price_levels[1000].orders[0].quantity;
    assert_eq!(*p, 7);
}

#[test]
fn test_buy_full_fill_multi_level() {
    let mut book = orderbook::OrderBook::default();

    // 5 levels of asks, 50 volume total
    for i in 10..15 {
        book.add_limit_order(orderbook::OrderEntry {
            price: Decimal::from(i),
            quantity: 10,
            order_side: orderbook::OrderSide::Sell,
        });
    }

    // Buy should fill from 10-14, leaving 5 quantity at level 14
    book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(20),
        quantity: 45,
        order_side: orderbook::OrderSide::Buy,
    });

    assert_eq!(book.ask_min(), Decimal::from(14));

    let p = &book.price_levels[1400].orders[0].quantity;
    assert_eq!(*p, 5);
}

#[test]
fn test_buy_partial_fill_multi_level() {
    let mut book = orderbook::OrderBook::default();

    // 5 levels of asks, 50 volume total
    for i in 10..15 {
        book.add_limit_order(orderbook::OrderEntry {
            price: Decimal::from(i),
            quantity: 10,
            order_side: orderbook::OrderSide::Sell,
        });
    }

    // Buy should fill from 10-14, leaving 5 quantity at level 20
    book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(20),
        quantity: 55,
        order_side: orderbook::OrderSide::Buy,
    });

    assert_eq!(book.bid_max(), Decimal::from(20));

    let p = &book.price_levels[2000].orders[0].quantity;
    assert_eq!(*p, 5);
}

#[test]
fn test_buy_partial_fill_multi_level_and_cancel() {
    let mut book = orderbook::OrderBook::default();

    // 5 levels of asks, 50 volume total
    for i in 10..15 {
        book.add_limit_order(orderbook::OrderEntry {
            price: Decimal::from(i),
            quantity: 10,
            order_side: orderbook::OrderSide::Sell,
        });
    }

    // Buy should fill from 10-14, leaving 5 quantity at level 20
    let result = book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(20),
        quantity: 55,
        order_side: orderbook::OrderSide::Buy,
    });

    assert_eq!(book.bid_max(), Decimal::from(20));

    let p = &book.price_levels[2000].orders[0].quantity;
    assert_eq!(*p, 5);

    match result {
        orderbook::OrderResult::Filled => panic!("order should not be filled"),
        orderbook::OrderResult::PartialFill(order_id) => book.cancel_order(order_id),
    }

    let p = &book.price_levels[2000].orders[0].quantity;
    assert_eq!(*p, 0);
}

#[test]
fn test_depth_and_quantity() {
    let mut book = orderbook::OrderBook::default();

    // Add 5 levels with 25 quantity
    for _ in 10..15 {
        book.add_limit_order(orderbook::OrderEntry {
            price: Decimal::from(10),
            quantity: 5,
            order_side: orderbook::OrderSide::Sell,
        });
    }

    let p = &book.price_levels[1000];
    assert_eq!(p.quantity, 25);
    assert_eq!(p.depth, 5);

    book.cancel_order(2);

    let p = &book.price_levels[1000];
    assert_eq!(p.quantity, 20);
    assert_eq!(p.depth, 4);

    book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(10),
        quantity: 7,
        order_side: orderbook::OrderSide::Buy,
    });

    let p = &book.price_levels[1000];
    assert_eq!(p.quantity, 13);
    assert_eq!(p.depth, 3);

    book.add_limit_order(orderbook::OrderEntry {
        price: Decimal::from(10),
        quantity: 16,
        order_side: orderbook::OrderSide::Buy,
    });

    let p = &book.price_levels[1000];
    assert_eq!(p.quantity, 3);
    assert_eq!(p.depth, 1);
}
