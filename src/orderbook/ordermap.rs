use slotmap::DefaultKey;

#[derive(Debug, Default)]
pub struct OrderMap {
    // Vec/Map of (slotmap indexes, order volume)
    orders: Vec<(DefaultKey, u32)>,
}

impl OrderMap {
    pub fn new(size: usize) -> Self {
        OrderMap {
            orders: vec![(DefaultKey::default(), 0); size],
        }
    }

    pub fn reserve(&mut self, id: u64) {
        if (id as usize) < self.orders.len() {
            return;
        }

        self.orders
            .resize(id as usize + 1, (DefaultKey::default(), 0));
    }

    pub fn get(&self, id: u64) -> Option<&(DefaultKey, u32)> {
        self.orders.get(id as usize)
    }

    pub fn put(&mut self, order_id: u64, data: (DefaultKey, u32)) {
        self.reserve(order_id);
        self.orders[order_id as usize] = data;
    }

    pub fn reduce_volume(&mut self, order_id: u64, volume: u32) {
        self.orders[order_id as usize].1 -= volume;
    }
}
