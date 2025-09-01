mod ordermap;

use ordermap::OrderMap;
use rust_decimal::Decimal;
use slab::Slab;

#[derive(Debug)]
pub struct Order {
    pub id: u64,
    pub side: OrderSide,
    pub price: Decimal,
    pub volume: u64,

    // The price_level (Slab index) this order is stored at
    price_level: usize,
}

#[derive(Debug, PartialEq, Clone)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
struct PriceLevel {
    price: Decimal,
    depth: usize,
    volume: u64,
}

#[derive(Debug, Default)]
pub struct OrderBook {
    // Smallest -> Largest
    // Tuple of price and pricelevel slab index
    bids: Vec<(Decimal, usize)>,
    // Largest -> Smallest
    asks: Vec<(Decimal, usize)>,

    price_levels: Slab<PriceLevel>,
    orders: Slab<Order>,

    order_map: OrderMap,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: Vec::with_capacity(1000),
            asks: Vec::with_capacity(1000),
            price_levels: Slab::with_capacity(2000),
            orders: Slab::with_capacity(50_000_000),
            order_map: OrderMap::new(50_000_000),
        }
    }

    pub fn meta(&self) -> (usize, usize, usize, usize) {
        (
            self.bids.len(),
            self.asks.len(),
            self.price_levels.len(),
            self.orders.len(),
        )
    }

    pub fn best_bid(&self) -> Option<Decimal> {
        let highest_bid_idx = self.bids.last()?;
        let highest_bid = self.price_levels.get(highest_bid_idx.1)?;
        Some(highest_bid.price)
    }

    pub fn best_ask(&self) -> Option<Decimal> {
        let lowest_ask_idx = self.asks.last()?;
        let lowest_ask = self.price_levels.get(lowest_ask_idx.1)?;
        Some(lowest_ask.price)
    }

    pub fn spread(&self) -> Option<Decimal> {
        let lowest_ask_idx = self.asks.last()?;
        let highest_bid_idx = self.bids.last()?;

        let lowest_ask = self.price_levels.get(lowest_ask_idx.1)?;
        let highest_bid = self.price_levels.get(highest_bid_idx.1)?;

        lowest_ask.price.checked_sub(highest_bid.price)
    }

    // volume and depth for a plevel
    // volume and depth between plevels

    pub fn add_order(&mut self, id: u64, price: Decimal, volume: u64, side: OrderSide) {
        let list = if side == OrderSide::Sell {
            &mut self.asks
        } else {
            &mut self.bids
        };

        let mut found = false;
        let mut insertion_idx = 0;

        for (idx, (plevel_price, _)) in list.iter().enumerate().rev() {
            if price.eq(plevel_price) {
                insertion_idx = idx;
                found = true;
                break;
            }

            match side {
                OrderSide::Sell => {
                    if price.lt(plevel_price) {
                        insertion_idx = idx + 1;
                        break;
                    }
                }
                OrderSide::Buy => {
                    if price.gt(plevel_price) {
                        insertion_idx = idx + 1;
                        break;
                    }
                }
            }
        }

        let entry = self.orders.vacant_entry();
        let mut order = Order {
            id,
            price,
            volume,
            side,
            price_level: 0,
        };
        self.order_map.reserve(id);
        self.order_map.put(id, entry.key());

        if found {
            let plevel_idx = list[insertion_idx];
            let plevel = self.price_levels.get_mut(plevel_idx.1).unwrap();
            plevel.depth += 1;
            plevel.volume += volume;
            order.price_level = plevel_idx.1;
        } else {
            let new_plevel_idx = self.price_levels.insert(PriceLevel {
                price,
                depth: 1,
                volume,
            });

            order.price_level = new_plevel_idx;
            list.insert(insertion_idx, (price, new_plevel_idx));
        }

        entry.insert(order);
    }

    pub fn execute_order(&mut self, order_id: u64, volume: u64) {
        let order_slab_idx = self.order_map.get(order_id).unwrap();
        let order = self.orders.get_mut(*order_slab_idx).unwrap();
        order.volume -= volume;

        let plevel_slab_idx = order.price_level;
        let plevel = self.price_levels.get_mut(plevel_slab_idx).unwrap();
        plevel.volume -= volume;

        if order.volume == 0 {
            plevel.depth -= 1;
            self.orders.remove(*order_slab_idx);
        }

        //if plevel.volume == 0 {
        //    self.price_levels.remove(plevel_slab_idx);
        //}
    }

    pub fn cancel_order(&mut self, order_id: u64, volume: u64) {
        let order_slab_idx = self.order_map.get(order_id).unwrap();
        let order = self.orders.get_mut(*order_slab_idx).unwrap();
        order.volume -= volume;

        let plevel_slab_idx = order.price_level;
        let plevel = self.price_levels.get_mut(plevel_slab_idx).unwrap();
        plevel.volume -= volume;

        if order.volume == 0 {
            plevel.depth -= 1;

            self.orders.remove(*order_slab_idx);
        }

        //if plevel.volume == 0 {
        //    self.price_levels.remove(plevel_slab_idx);
        //}
    }

    pub fn delete_order(&mut self, order_id: u64) {
        let order_slab_idx = self.order_map.get(order_id).unwrap();
        let order = self.orders.get_mut(*order_slab_idx).unwrap();

        let plevel_slab_idx = order.price_level;
        let plevel = self.price_levels.get_mut(plevel_slab_idx).unwrap();
        plevel.volume -= order.volume;
        plevel.depth -= 1;

        self.orders.remove(*order_slab_idx);

        //if plevel.volume == 0 {
        //    self.price_levels.remove(plevel_slab_idx);
        //}
    }

    pub fn replace_order(
        &mut self,
        old_order_id: u64,
        new_order_id: u64,
        price: Decimal,
        volume: u64,
    ) {
        let order_slab_idx = self.order_map.get(old_order_id).unwrap();
        let order = self.orders.get(*order_slab_idx).unwrap();
        let side = order.side.clone();

        self.delete_order(old_order_id);
        self.add_order(new_order_id, price, volume, side);
    }
}
