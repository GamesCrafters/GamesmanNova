use std::collections::BTreeMap;

pub const PAGE_BLOCK_SIZE: usize = 0xffff;

pub struct Page {
    data: BTreeMap<usize, [u8; PAGE_BLOCK_SIZE + 1]>,
}

impl Page {
    pub fn new() -> Self {
        Page {
            data: BTreeMap::new(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&u8> {
        let upper = index & !PAGE_BLOCK_SIZE;
        let lower = index & PAGE_BLOCK_SIZE;
        if let Some(block) = self.data.get(&upper) {
            Some(&block[lower])
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        let upper = index & !PAGE_BLOCK_SIZE;
        let lower = index & PAGE_BLOCK_SIZE;
        if let Some(block) = self.data.get_mut(&upper) {
            Some(&mut block[lower])
        } else {
            None
        }
    }

    pub fn insert(&mut self, index: usize, value: u8) {
        let upper = index & !PAGE_BLOCK_SIZE;
        let lower = index & PAGE_BLOCK_SIZE;
        match self.data.get_mut(&upper) {
            Some(mut block) => {
                block[lower] = value;
            }
            None => {
                let mut new_block = [0; PAGE_BLOCK_SIZE + 1];
                new_block[lower] = value;
                self.data.insert(upper, new_block);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic]
    fn test_page_insert_value_above_255_panics() {
        let mut p = Page::new();
        p.insert(5, 256);
    }

    #[test]
    fn test_page_insert_and_get() {
        let mut p = Page::new();

        assert_eq!(None, p.get(10).copied());

        p.insert(10, 20);
        p.insert(0, 160);
        assert_eq!(Some(20), p.get(10).copied());
        assert_eq!(Some(160), p.get(0).copied());
        assert_eq!(Some(20), p.get_mut(10).copied());
        assert_eq!(Some(160), p.get_mut(0).copied());

        p.insert(10, 30);
        p.insert(0, 200);
        assert_eq!(Some(30), p.get(10).copied());
        assert_eq!(Some(200), p.get(0).copied());
        assert_eq!(Some(30), p.get_mut(10).copied());
        assert_eq!(Some(200), p.get_mut(0).copied());
    }
}
