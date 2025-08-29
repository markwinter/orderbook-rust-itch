use rust_decimal::prelude::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use std::collections::HashMap;

const MAX_PRICE_LEVELS: usize = 10000000;

#[derive(Debug)]
pub enum OrderSide {
    Buy,
    Sell,
}

pub enum OrderResult {
    Filled,
    PartialFill(u64),
}

#[derive(Debug)]
pub struct Order {
    order_id: u64,
    price: Decimal,
    pub quantity: u64,
    order_side: OrderSide,
}

pub struct OrderEntry {
    pub price: Decimal,
    pub quantity: u64,
    pub order_side: OrderSide,
}

#[derive(Debug)]
pub struct OrderBook {
    current_order_id: u64,
    pub price_levels: Box<[PriceLevel]>,
    order_id_lookup: HashMap<u64, (Decimal, usize)>,

    // ask_min and bid_max are converted to ticks
    ask_min: Decimal,
    bid_max: Decimal,

    tick_size: Decimal,
}

#[derive(Debug)]
pub struct PriceLevel {
    pub orders: Vec<Order>,
    pub quantity: u64,
    pub depth: u64,
}

impl PriceLevel {
    fn new() -> PriceLevel {
        PriceLevel {
            orders: Vec::new(),
            depth: 0,
            quantity: 0,
        }
    }
}

impl Default for OrderBook {
    fn default() -> Self {
        OrderBook::new(Decimal::from_f32(0.01).unwrap())
    }
}

impl OrderBook {
    pub fn new(tick_size: Decimal) -> OrderBook {
        OrderBook {
            current_order_id: 0,
            price_levels: (0..MAX_PRICE_LEVELS)
                .map(|_| PriceLevel::new())
                .collect::<Vec<_>>()
                .into_boxed_slice(),
            order_id_lookup: HashMap::new(),

            ask_min: Decimal::from(100000000),
            bid_max: Decimal::ZERO,

            tick_size,
        }
    }

    pub fn bid_max(&self) -> Decimal {
        self.bid_max * self.tick_size
    }

    pub fn ask_min(&self) -> Decimal {
        self.ask_min * self.tick_size
    }

    pub fn add_limit_order(&mut self, order: OrderEntry) -> OrderResult {
        let new_order_id = self.current_order_id;
        self.current_order_id += 1;

        let order = Order {
            order_id: new_order_id,
            price: order.price,
            quantity: order.quantity,
            order_side: order.order_side,
        };

        match order.order_side {
            OrderSide::Buy => self.handle_buy_order(order),
            OrderSide::Sell => self.handle_sell_order(order),
        }
    }

    fn handle_buy_order(&mut self, mut order: Order) -> OrderResult {
        let mut remaining = order.quantity;
        let mut current_level = self.ask_min;
        let order_id = order.order_id;
        let price = order.price / self.tick_size;

        while current_level.le(&price) {
            let index = current_level.to_usize().unwrap();
            let plevel = &mut self.price_levels[index];

            for o in &mut plevel.orders {
                if o.quantity == 0 {
                    continue;
                }

                if remaining <= o.quantity {
                    // ugly
                    if remaining == o.quantity {
                        plevel.depth -= 1;
                    }
                    plevel.quantity -= remaining;
                    o.quantity -= remaining;
                    return OrderResult::Filled;
                }

                plevel.quantity -= o.quantity;
                plevel.depth -= 1;
                remaining -= o.quantity;
                o.quantity = 0;
            }

            current_level += self.tick_size;
            self.ask_min += self.tick_size;
        }

        if price > self.bid_max {
            self.bid_max = price;
        }

        order.quantity = remaining;

        let index = price.to_usize().unwrap();
        let plevel = &mut self.price_levels[index];
        plevel.depth += 1;
        plevel.quantity += order.quantity;
        plevel.orders.push(order);

        self.order_id_lookup
            .insert(order_id, (price, plevel.orders.len() - 1));

        OrderResult::PartialFill(order_id)
    }

    fn handle_sell_order(&mut self, mut order: Order) -> OrderResult {
        let mut remaining = order.quantity;
        let mut current_level = self.bid_max;
        let order_id = order.order_id;
        let price = order.price / self.tick_size;

        while current_level.ge(&price) {
            let index = current_level.to_usize().unwrap();
            let plevel = &mut self.price_levels[index];

            for o in &mut plevel.orders {
                if o.quantity == 0 {
                    continue;
                }

                if remaining <= o.quantity {
                    if remaining == o.quantity {
                        plevel.depth -= 1;
                    }
                    plevel.quantity -= remaining;
                    o.quantity -= remaining;
                    return OrderResult::Filled;
                }

                plevel.quantity -= o.quantity;
                plevel.depth -= 1;
                remaining -= o.quantity;
                o.quantity = 0;
            }

            current_level -= self.tick_size;
            self.ask_min -= self.tick_size;
        }

        if price < self.ask_min {
            self.ask_min = price;
        }

        order.quantity = remaining;

        let index = price.to_usize().unwrap();
        let plevel = &mut self.price_levels[index];
        plevel.depth += 1;
        plevel.quantity += order.quantity;
        plevel.orders.push(order);

        self.order_id_lookup
            .insert(order_id, (price, plevel.orders.len() - 1));

        OrderResult::PartialFill(order_id)
    }

    pub fn cancel_order(&mut self, order_id: u64) {
        match self.order_id_lookup.get(&order_id) {
            None => (),
            Some((price, pos)) => {
                let index = price.to_usize().unwrap();
                let p = &mut self.price_levels[index];
                let order = &mut p.orders[*pos];

                p.quantity -= order.quantity;
                p.depth -= 1;

                order.quantity = 0;

                self.order_id_lookup.remove(&order_id);
            }
        }
    }
}
