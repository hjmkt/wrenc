use std::cmp::PartialEq;

pub struct Bins {
    pub head: u64,
    pub tail: Vec<u64>,
    pub num_bins: usize,
}

impl Bins {
    #[inline(always)]
    pub fn new() -> Bins {
        Bins {
            head: 0,
            tail: vec![],
            num_bins: 0,
        }
    }

    #[inline(always)]
    pub fn push_initial_bin(&mut self, bin: bool) {
        debug_assert!(self.num_bins == 0);
        self.head = (self.head << 1) | bin as u64;
        self.num_bins = 1;
    }

    #[inline(always)]
    pub fn push_bin(&mut self, bin: bool) {
        //debug_assert!(self.num_bins > 0);
        if self.num_bins % 64 == 0 {
            self.tail.push(self.head);
            self.head = bin as u64;
        } else {
            self.head = (self.head << 1) | bin as u64;
        }
        self.num_bins += 1;
    }

    #[inline(always)]
    pub fn push_bin_with_initial_check(&mut self, bin: bool) {
        if self.num_bins % 64 == 0 && self.num_bins > 0 {
            self.tail.push(self.head);
            self.head = bin as u64;
        } else {
            self.head = (self.head << 1) | bin as u64;
        }
        self.num_bins += 1;
    }

    #[inline(always)]
    pub fn push_initial_bins_with_size(&mut self, bins: u64, size: usize) {
        debug_assert!(self.num_bins == 0);
        debug_assert!(size <= 64);
        self.head = bins;
        self.num_bins = size;
    }

    #[inline(always)]
    pub fn push_bins_with_size(&mut self, bins: u64, size: usize) {
        //debug_assert!(self.num_bins > 0);
        debug_assert!(size <= 64);
        let r = if self.num_bins > 0 {
            (self.num_bins - 1) % 64 + 1
        } else {
            0
        };
        if r + size > 64 {
            let n0 = 64 - r;
            let n1 = r + size - 64;
            let bins0 = bins >> n1;
            let bins1 = bins - (bins0 << n1);
            self.tail.push((self.head << n0) | bins0);
            self.head = bins1;
        } else {
            self.head = (self.head << size) | bins;
        }
        self.num_bins += size;
    }

    //#[inline(always)]
    //pub fn push_same_bins(&mut self, bin: bool, size: usize) {
    //debug_assert!(self.num_bins > 0);
    //let mut r = (self.num_bins - 1) % 64 + 1;
    //if r + size > 64 {
    //let n0 = 64 - r;
    //if n0 > 0 {
    //let bins = if bin { (1 << n0) - 1 } else { 0 };
    //self.head = (self.head << n0) + bins;
    //}
    //self.tail.push(self.head);
    //self.head = 0;
    //let mut n = size - n0;
    //let bins64 = if bin {
    //if size == 64 {
    //0xffffffffffffffff
    //} else {
    //(1 << size) - 1
    //}
    //} else {
    //0
    //};
    //while n > 64 {
    //self.tail.push(bins64);
    //n -= 64;
    //}
    //if n == 64 {
    //self.head = bins64;
    //} else if n > 0 && bin {
    //self.head = (1 << n) - 1;
    //}
    //} else {
    //let bins = if bin {
    //if size == 64 {
    //0xffffffffffffffff
    //} else {
    //(1 << size) - 1
    //}
    //} else {
    //0
    //};
    //self.head = (self.head << size) + bins;
    //}
    //self.num_bins += size;
    //}

    #[inline(always)]
    pub fn byte_align(&mut self) {
        if self.num_bins % 8 > 0 {
            let r = 8 - self.num_bins % 8;
            self.head <<= r;
            self.num_bins += r;
        }
    }

    #[inline(always)]
    pub fn bytes(&self) -> BinsByteIterator {
        BinsByteIterator {
            bins: self,
            index: 0,
        }
    }
}

impl PartialEq for Bins {
    fn eq(&self, other: &Bins) -> bool {
        if self.head != other.head || self.num_bins != other.num_bins {
            false
        } else {
            std::iter::zip(self.tail.iter(), other.tail.iter()).all(|(x, y)| x == y)
        }
    }
}

pub struct BinsIntoIterator {
    bins: Bins,
    index: usize,
}

impl Iterator for BinsIntoIterator {
    type Item = bool;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.bins.num_bins {
            None
        } else if self.bins.num_bins > 64 {
            let offset = self.index / 64;
            let index = self.index % 64;
            let b = if offset == self.bins.tail.len() {
                Some((self.bins.head >> (self.bins.num_bins % 64 - index - 1)) & 1 > 0)
            } else {
                Some((self.bins.tail[offset] >> (64 - index - 1)) & 1 > 0)
            };
            self.index += 1;
            b
        } else {
            let index = self.bins.num_bins - self.index - 1;
            let b = Some((self.bins.head >> index) & 1 > 0);
            self.index += 1;
            b
        }
    }
}

pub struct BinsByteIterator<'a> {
    bins: &'a Bins,
    index: usize,
}

impl<'a> Iterator for BinsByteIterator<'a> {
    type Item = u8;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.bins.num_bins {
            None
        } else if self.bins.num_bins > 64 {
            let offset = self.index / 64;
            let index = self.index % 64;
            let byte = if offset == self.bins.tail.len() {
                Some(
                    ((self.bins.head >> ((self.bins.num_bins - 1) % 64 + 1 - index - 8)) & 0xff)
                        as u8,
                )
            } else {
                Some(((self.bins.tail[offset] >> (64 - index - 8)) & 0xff) as u8)
            };
            self.index += 8;
            byte
        } else {
            let index = self.bins.num_bins - self.index - 8;
            let byte = Some(((self.bins.head >> index) & 0xff) as u8);
            self.index += 8;
            byte
        }
    }
}

impl IntoIterator for Bins {
    type Item = bool;
    type IntoIter = BinsIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        BinsIntoIterator {
            bins: self,
            index: 0,
        }
    }
}
