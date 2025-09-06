#[derive(Debug, Default)]
pub struct OrderMap {
    // Vec/Map of slab indexes
    orders: Vec<usize>,
}

impl OrderMap {
    pub fn new(size: usize) -> Self {
        OrderMap {
            orders: vec![0; size],
        }
    }

    pub fn reserve(&mut self, id: u64) {
        if (id as usize) < self.orders.len() {
            return;
        }

        self.orders.resize(id as usize + 1, 0);
    }

    pub fn get(&self, id: u64) -> Option<&usize> {
        self.orders.get(id as usize)
    }

    pub fn put(&mut self, order_id: u64, slab_index: usize) {
        self.orders[order_id as usize] = slab_index;
    }
}
