mod ordermap;

use ordermap::OrderMap;
use rust_decimal::Decimal;
use slotmap::{DefaultKey, SlotMap};

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
pub enum OrderSide {
    Buy,
    Sell,
}

#[derive(Debug)]
struct PriceLevel {
    depth: usize,
    volume: u32,
    side: OrderSide,
}

#[derive(Debug, Default)]
pub struct OrderBook {
    // Tuple of price and pricelevel slotmap index
    // Smallest -> Largest
    bids: Vec<(u32, DefaultKey)>,
    // Largest -> Smallest
    asks: Vec<(u32, DefaultKey)>,

    price_levels: SlotMap<DefaultKey, PriceLevel>,
    order_map: OrderMap,
}

impl OrderBook {
    pub fn new() -> Self {
        OrderBook {
            bids: Vec::with_capacity(10_000),
            asks: Vec::with_capacity(10_000),
            price_levels: SlotMap::with_capacity(10_000),
            order_map: OrderMap::new(2_000_000),
        }
    }

    pub fn meta(&self) -> (usize, usize, usize) {
        (self.bids.len(), self.asks.len(), self.price_levels.len())
    }

    pub fn best_bid(&self) -> Option<Decimal> {
        let highest_bid = self.bids.last()?.0;
        Some(Decimal::from(highest_bid) / Decimal::from(10_000))
    }

    pub fn best_ask(&self) -> Option<Decimal> {
        let lowest_ask = self.asks.last()?.0;
        Some(Decimal::from(lowest_ask) / Decimal::from(10_000))
    }

    pub fn spread(&self) -> Option<Decimal> {
        let lowest_ask = self.best_ask()?;
        let highest_bid = self.best_bid()?;
        Some((lowest_ask - highest_bid) / Decimal::from(10_000))
    }

    pub fn add_order(&mut self, id: u64, price: u32, volume: u32, side: OrderSide) {
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

        let plevel_idx;
        if found {
            plevel_idx = list[insertion_idx].1;
            let plevel = self.price_levels.get_mut(plevel_idx).unwrap();
            plevel.depth += 1;
            plevel.volume += volume;
        } else {
            plevel_idx = self.price_levels.insert(PriceLevel {
                depth: 1,
                volume,
                side,
            });
            list.insert(insertion_idx, (price, plevel_idx));
        }

        self.order_map.put(id, (plevel_idx, volume));
    }

    pub fn execute_order(&mut self, order_id: u64, volume: u32) {
        let (plevel_idx, _) = self.order_map.get(order_id).unwrap();

        let plevel = self.price_levels.get_mut(*plevel_idx).unwrap();
        let side = plevel.side;
        plevel.volume -= volume;

        if plevel.volume == 0 {
            self.remove_price_level(*plevel_idx, side);
        }

        self.order_map.reduce_volume(order_id, volume);
    }

    pub fn cancel_order(&mut self, order_id: u64, volume: u32) {
        let (plevel_idx, _) = self.order_map.get(order_id).unwrap();

        let plevel = self.price_levels.get_mut(*plevel_idx).unwrap();
        let side = plevel.side;
        plevel.volume -= volume;

        if plevel.volume == 0 {
            self.remove_price_level(*plevel_idx, side);
        }

        self.order_map.reduce_volume(order_id, volume);
    }

    pub fn delete_order(&mut self, order_id: u64) {
        let (plevel_idx, order_volume) = self.order_map.get(order_id).unwrap();

        let plevel = self.price_levels.get_mut(*plevel_idx).unwrap();
        let side = plevel.side;
        plevel.volume -= order_volume;
        plevel.depth -= 1;

        if plevel.volume == 0 {
            self.remove_price_level(*plevel_idx, side);
        }
    }

    pub fn replace_order(&mut self, old_order_id: u64, new_order_id: u64, price: u32, volume: u32) {
        let (plevel_idx, _) = self.order_map.get(old_order_id).unwrap();

        let plevel = self.price_levels.get_mut(*plevel_idx).unwrap();
        let side = plevel.side;

        self.delete_order(old_order_id);
        self.add_order(new_order_id, price, volume, side);
    }

    fn remove_price_level(&mut self, plevel_slab_idx: DefaultKey, side: OrderSide) {
        let list = if side == OrderSide::Sell {
            &mut self.asks
        } else {
            &mut self.bids
        };

        for (idx, (_, slab_idx)) in list.iter_mut().enumerate().rev() {
            if *slab_idx != plevel_slab_idx {
                continue;
            }

            list.remove(idx);
            break;
        }
        self.price_levels.remove(plevel_slab_idx);
    }
}
