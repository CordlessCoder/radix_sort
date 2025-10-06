use crate::RadixSortable;

impl RadixSortable for u8 {
    const BITS: usize = 8;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
}
impl RadixSortable for u16 {
    const BITS: usize = 16;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
}
impl RadixSortable for u32 {
    const BITS: usize = 32;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
}
impl RadixSortable for u64 {
    const BITS: usize = 64;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
}
impl RadixSortable for i8 {
    const BITS: usize = 8;
    const SIGNED: bool = true;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
    #[inline(always)]
    fn is_negative(&self) -> bool {
        i8::is_negative(*self)
    }
}
impl RadixSortable for i16 {
    const BITS: usize = 16;
    const SIGNED: bool = true;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
    #[inline(always)]
    fn is_negative(&self) -> bool {
        i16::is_negative(*self)
    }
}
impl RadixSortable for i32 {
    const BITS: usize = 32;
    const SIGNED: bool = true;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
    #[inline(always)]
    fn is_negative(&self) -> bool {
        i32::is_negative(*self)
    }
}
impl RadixSortable for i64 {
    const BITS: usize = 64;
    const SIGNED: bool = true;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
    #[inline(always)]
    fn is_negative(&self) -> bool {
        i64::is_negative(*self)
    }
}
impl RadixSortable for f32 {
    const BITS: usize = 32;
    const SIGNED: bool = true;
    const FLOAT: bool = true;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self.to_bits() >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
    #[inline(always)]
    fn is_negative(&self) -> bool {
        self.is_sign_negative()
    }
}
impl RadixSortable for f64 {
    const BITS: usize = 64;
    const SIGNED: bool = true;
    const FLOAT: bool = true;
    #[inline(always)]
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize {
        let shifted = (self.to_bits() >> offset) as usize;
        shifted & ((1 << WIDTH) - 1)
    }
    #[inline(always)]
    fn is_negative(&self) -> bool {
        self.is_sign_negative()
    }
}
