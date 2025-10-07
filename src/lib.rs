#![cfg_attr(not(test), no_std)]
extern crate alloc;
use alloc::vec::Vec;
use core::{hint::assert_unchecked, mem::MaybeUninit};
mod impls;

pub unsafe trait RadixSortable: Copy + PartialOrd {
    const BITS: usize;
    const SIGNED: bool = false;
    const FLOAT: bool = false;
    fn offset_bits<const WIDTH: usize, const OFFSET: usize>(&self) -> usize;
    #[inline(always)]
    fn is_negative(&self) -> bool {
        false
    }
}

/// Returns true if the level at the provided [OFFSET] was already sorted.
#[inline(always)]
fn update<const WIDTH: usize, const BUCKETS: usize, const OFFSET: usize, T: RadixSortable>(
    counters: &mut [usize; BUCKETS],
    values: &[T],
) -> bool {
    assert_eq!((1 << WIDTH), BUCKETS);
    counters.iter_mut().for_each(|c| *c = 0);
    let mut already_sorted = true;
    let mut last = 0usize;
    let mut values = values.iter();
    for value in &mut values {
        let idx = value.offset_bits::<WIDTH, OFFSET>();
        unsafe {
            *counters.get_unchecked_mut(idx) += 1;
        }
        if idx < last {
            already_sorted = false;
            break;
        }
        last = idx;
    }
    // The first loop captured all counts
    if already_sorted {
        return already_sorted;
    }
    for val in values {
        let idx = val.offset_bits::<WIDTH, OFFSET>();
        unsafe {
            *counters.get_unchecked_mut(idx) += 1;
        }
    }
    false
}
#[inline(always)]
fn radix_sort_pass<
    const WIDTH: usize,
    const BUCKETS: usize,
    const OFFSET: usize,
    T: RadixSortable,
>(
    counters: &mut [usize; BUCKETS],
    values: &[T],
    out: &mut [MaybeUninit<T>],
) {
    let already_sorted = update::<WIDTH, BUCKETS, OFFSET, T>(counters, values);
    if already_sorted {
        out.copy_from_slice(as_uninit_slice(values));
        return;
    }
    {
        let mut prefix = 0;
        counters.iter_mut().for_each(|counter| {
            let this = *counter;
            *counter = prefix;
            prefix += this;
        });
    }
    let offsets = counters;
    for &value in values {
        let offset = &mut offsets[value.offset_bits::<WIDTH, OFFSET>()];
        unsafe {
            assert_unchecked(*offset < out.len());
        }
        out[*offset] = MaybeUninit::new(value);
        *offset += 1;
    }
}

pub fn make_buf<T>(len: usize) -> Vec<MaybeUninit<T>> {
    (0..len).map(|_| MaybeUninit::uninit()).collect()
}

fn as_uninit_slice<T>(s: &[T]) -> &[MaybeUninit<T>] {
    unsafe { core::mem::transmute(s) }
}
fn as_uninit_slice_mut<T>(s: &mut [T]) -> &mut [MaybeUninit<T>] {
    unsafe { core::mem::transmute(s) }
}

#[doc(hidden)]
macro_rules! __radix_sort_unsigned_for_width_inner {
    (odd args ($counters:ident, $values:ident, $buf:ident), [ $first_offset:expr, $($offset:expr),* ]) => {{
        if T::BITS < WIDTH * ($first_offset + 1) {
            return;
        }
        radix_sort_pass::<WIDTH, BUCKETS, { WIDTH * $first_offset }, T>(
            &mut $counters,
            $values,
            as_uninit_slice_mut($buf),
        );
        __radix_sort_unsigned_for_width_inner!(even args ($counters, $values, $buf), [$($offset),*])
    }};
    (odd args ($counters:ident, $values:ident, $buf:ident), [ $first_offset:expr ]) => {{
        if T::BITS < WIDTH * ($first_offset + 1) {
            return;
        }
        radix_sort_pass::<WIDTH, BUCKETS, { WIDTH * $first_offset }, T>(
            &mut $counters,
            $values,
            as_uninit_slice_mut($buf),
        );
    }};
    (even args ($counters:ident, $values:ident, $buf:ident), [ $first_offset:expr ]) => {{
        if const { T::BITS < WIDTH * ($first_offset + 1) } {
            return;
        }
        radix_sort_pass::<WIDTH, BUCKETS, { WIDTH * $first_offset }, T>(&mut $counters, $buf, as_uninit_slice_mut($values));
    }};
    (even args ($counters:ident, $values:ident, $buf:ident), [ $first_offset:expr, $($offset:expr),* ]) => {{
        if const { T::BITS < WIDTH * ($first_offset + 1) } {
            return;
        }
        radix_sort_pass::<WIDTH, BUCKETS, { WIDTH * $first_offset }, T>(&mut $counters, $buf, as_uninit_slice_mut($values));
        __radix_sort_unsigned_for_width_inner!(odd args ($counters, $values, $buf), [$($offset),*])
    }};
}
macro_rules! radix_sort_unsigned_for_width {
    ($width:expr, [$($offset:expr),*]) => {{
        #[inline(always)]
        /// The final result will be in values if ([T::BITS] / 8) % 2 == 0.
        /// It will be in buf otherwise.
        fn radix_sort_unsigned_body<T: RadixSortable>(
            values: &mut [T],
            buf: &mut [MaybeUninit<T>],
        ) {
            const WIDTH: usize = $width;
            const BUCKETS: usize = 2u32.pow(WIDTH as u32) as usize;
            let mut counters = [0; BUCKETS];
            assert_eq!(values.len(), buf.len());
            radix_sort_pass::<WIDTH, BUCKETS, 0, T>(&mut counters, values, buf);
            // SAFETY: Buf has been initialized by the first call to radix_sort_pass
            let buf: &mut [T] = unsafe { core::mem::transmute(buf) };
            __radix_sort_unsigned_for_width_inner!(even args (counters, values, buf), [ $($offset),* ])
        }
        radix_sort_unsigned_body
    }};
}

#[inline(always)]
const fn should_use_small_algo<T: RadixSortable>(len: usize) -> bool {
    if core::mem::size_of::<T>() == 1 {
        return true;
    }
    // NOTE: The minimum length should be reconsidered.
    if len * core::mem::size_of::<T>() < 8096 {
        return true;
    }
    false
}

pub fn radix_sort<T: RadixSortable>(values: &mut [T], buf: &mut [MaybeUninit<T>]) {
    assert_eq!(values.len(), buf.len());
    if should_use_small_algo::<T>(values.len()) {
        radix_sort_unsigned_for_width!(8, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])(
            values, buf,
        );
    } else {
        radix_sort_unsigned_for_width!(16, [1, 2, 3, 4, 5, 6, 7])(values, buf);
    }
    // SAFETY: buf must've been initialized by the call to radix_sort_unsigned
    let buf: &mut [T] = unsafe { core::mem::transmute(buf) };
    // Unsigned int, result already in values
    if !T::SIGNED && (T::BITS / 8) % 2 == 0 {
        return;
    }
    // Unsigned int, result in buf
    if !T::SIGNED && (T::BITS / 8) % 2 == 1 {
        values.copy_from_slice(buf);
        return;
    }
    // Signed int, result in buf - put it into values
    if (T::BITS / 8) % 2 == 1 {
        values.copy_from_slice(buf);
    }
    // Fix order of negative values
    let first_neg = values
        .iter()
        .copied()
        .take_while(|v| !v.is_negative())
        .count();
    unsafe {
        assert_unchecked(first_neg <= values.len());
    }
    values.rotate_left(first_neg);
    if T::FLOAT {
        let neg_count = values.len() - first_neg;
        values[..neg_count].reverse();
    }
}

#[cfg(test)]
mod tests {
    use rand::{RngCore, SeedableRng};

    use crate::*;

    #[track_caller]
    fn test_with_generator<T: RadixSortable + core::fmt::Debug>(
        cb: impl FnMut(usize) -> T,
        len: usize,
    ) {
        let mut numbers: Vec<T> = (0..len).map(cb).collect();
        let mut buf = make_buf(numbers.len());
        radix_sort(&mut numbers, &mut buf);
        assert!(numbers.is_sorted(), "{numbers:?}");
    }

    #[test]
    fn ordered() {
        test_with_generator(|idx| idx as u32, 1024);
        test_with_generator(|idx| idx as f32, 1024);
        test_with_generator(|idx| idx as u8, 1024);
    }

    #[test]
    fn reversed() {
        test_with_generator(|idx| (1024 - idx - 1) as u32, 1024);
        test_with_generator(|idx| (1024 - idx - 1) as f32, 1024);
        test_with_generator(|idx| (1024 - idx - 1) as u8, 1024);
    }

    #[test]
    fn reversed_half_neg() {
        test_with_generator(
            |idx| {
                if idx % 2 == 0 {
                    (1024 - idx - 1) as i32 / 2
                } else {
                    -((1024 - idx - 1) as i32) / 2
                }
            },
            1024,
        );
        test_with_generator(
            |idx| {
                if idx % 2 == 0 {
                    (1024 - idx - 1) as f32 / 2.
                } else {
                    -((1024 - idx - 1) as f32) / 2.
                }
            },
            1024,
        );
    }

    #[test]
    fn uniformly_random() {
        use rand::rngs::SmallRng;
        let mut rng = SmallRng::from_os_rng();
        test_with_generator(|_| rng.next_u32(), 1024);
    }
}
