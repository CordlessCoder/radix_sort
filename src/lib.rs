#![cfg_attr(not(test), no_std)]
extern crate alloc;
use alloc::vec::Vec;
use core::{hint::assert_unchecked, mem::MaybeUninit};
mod impls;

const WIDTH: usize = 16;
const BUCKETS: usize = 2usize.pow(WIDTH as u32);

pub trait RadixSortable: Copy + PartialOrd {
    const BITS: usize;
    const SIGNED: bool = false;
    const FLOAT: bool = false;
    fn offset_bits<const WIDTH: usize>(&self, offset: usize) -> usize;
    #[inline(always)]
    fn is_negative(&self) -> bool {
        false
    }
}

#[inline(always)]
fn radix_sort_pass<const OFFSET: usize, T: RadixSortable>(
    counters: &mut [usize; BUCKETS],
    values: &[T],
    out: &mut [MaybeUninit<T>],
) {
    counters.iter_mut().for_each(|c| *c = 0);
    for &value in values {
        let v = counters.get_mut(value.offset_bits::<WIDTH>(OFFSET) & (BUCKETS - 1));
        unsafe {
            *v.unwrap_unchecked() += 1;
        }
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
        let offset = &mut offsets[value.offset_bits::<WIDTH>(OFFSET) & (BUCKETS - 1)];
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

fn as_uninit_slice<T>(s: &mut [T]) -> &mut [MaybeUninit<T>] {
    unsafe { core::mem::transmute(s) }
}

#[inline(always)]
#[expect(clippy::erasing_op)]
#[expect(clippy::identity_op)]
/// The final result will be in values if ([T::BITS] / 8) % 2 == 0.
/// It will be in buf otherwise.
fn radix_sort_unsigned<T: RadixSortable>(values: &mut [T], buf: &mut [MaybeUninit<T>]) {
    let mut counters: [usize; BUCKETS] = [0; BUCKETS];
    assert_eq!(values.len(), buf.len());
    if T::BITS >= WIDTH * 1 {
        radix_sort_pass::<{ WIDTH * 0 }, T>(&mut counters, values, buf);
    }
    // SAFETY: Buf has been initialized by the first call to radix_sort_unsigned
    let buf: &mut [T] = unsafe { core::mem::transmute(buf) };
    macro_rules! sort_offset {
        ($($offset:expr),*) => {
            $(
                if $offset % 2 == 0 {
                    if T::BITS >= WIDTH * ($offset + 1) {
                        radix_sort_pass::<{ WIDTH * $offset }, T>(&mut counters, values, as_uninit_slice(buf));
                    }
                } else {
                    if T::BITS >= WIDTH * ($offset + 1) {
                        radix_sort_pass::<{ WIDTH * $offset }, T>(&mut counters, buf, as_uninit_slice(values));
                    }
                }
            )*
        };
    }
    sort_offset!(1, 2, 3, 4, 5, 6, 7);
}

pub fn radix_sort<T: RadixSortable>(values: &mut [T], buf: &mut [MaybeUninit<T>]) {
    radix_sort_unsigned(values, buf);
    // Unsigned int, result already in values
    if !T::SIGNED && (T::BITS / 8) % 2 == 0 {
        return;
    }
    // SAFETY: buf must've been initialized by the call to radix_sort_unsigned
    let buf: &mut [T] = unsafe { core::mem::transmute(buf) };
    // Unsigned int, result in buf
    if !T::SIGNED && (T::BITS / 8) % 2 == 1 {
        values.copy_from_slice(buf);
        return;
    }
    // Signed int, result in values - put it into buf
    if (T::BITS / 8) % 2 == 0 {
        buf.copy_from_slice(values);
    }
    // Write back into values, fixing order of negative values
    let first_neg = buf.iter().copied().take_while(|v| !v.is_negative()).count();
    let neg_count = buf.len() - first_neg;
    values[..neg_count].copy_from_slice(&buf[first_neg..]);
    if T::FLOAT {
        values[..neg_count].reverse();
    }
    values[neg_count..].copy_from_slice(&buf[..first_neg]);
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
