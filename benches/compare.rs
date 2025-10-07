use gungraun::{library_benchmark, library_benchmark_group, main};
use radix_sort::{RadixSortable, make_buf, radix_sort};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use std::{cell::RefCell, hint::black_box, mem::MaybeUninit};

type InputPair<T> = (Vec<T>, Vec<MaybeUninit<T>>);

fn ascending_u32(idx: usize) -> u32 {
    idx as u32
}

fn ascending_i32(idx: usize) -> i32 {
    idx as i32
}

fn reverse_u32(idx: usize) -> u32 {
    u32::MAX - idx as u32
}

fn reverse_i32(idx: usize) -> i32 {
    i32::MAX - idx as i32
}

thread_local! {
    static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_os_rng());
}
fn random_u32(_: usize) -> u32 {
    RNG.with(|r| r.borrow_mut().random())
}
fn random_i32(_: usize) -> i32 {
    RNG.with(|r| r.borrow_mut().random())
}

fn u32_with_size(size: usize) -> impl Iterator<Item = (impl FnMut(usize) -> u32, usize)> {
    [ascending_u32, reverse_u32, random_u32]
        .into_iter()
        .map(move |cb| (cb, size))
}

fn i32_with_size(size: usize) -> impl Iterator<Item = (impl FnMut(usize) -> i32, usize)> {
    [ascending_i32, reverse_i32, random_i32]
        .into_iter()
        .map(move |cb| (cb, size))
}

fn confirm_sorted<T: RadixSortable>((values, _): InputPair<T>) {
    assert!(values.is_sorted())
}

#[track_caller]
fn gen_input<T: RadixSortable>(
    (mut cb, len): (impl FnMut(usize) -> T, usize),
) -> (Vec<T>, Vec<MaybeUninit<T>>) {
    let numbers: Vec<T> = (0..len).map(|idx| black_box(cb(idx))).collect();
    let buf = make_buf(numbers.len());
    (black_box(numbers), buf)
}

#[library_benchmark]
#[benches::small_u32(
    iter = u32_with_size(1024),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::large_u32(
    iter = u32_with_size(1024 * 64),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::huge_u32(
    iter = u32_with_size(1024 * 1024 * 4),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::small_i32(
    iter = i32_with_size(1024),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::large_i32(
    iter = i32_with_size(1024 * 64),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::huge_i32(
    iter = i32_with_size(1024 * 1024 * 4),
    setup = gen_input,
    teardown = confirm_sorted
)]
fn radix<T: RadixSortable>((mut values, mut buf): InputPair<T>) -> InputPair<T> {
    radix_sort(black_box(&mut values), black_box(&mut buf));
    (values, buf)
}

#[library_benchmark]
#[benches::small_u32(
    iter = u32_with_size(1024),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::large_u32(
    iter = u32_with_size(1024 * 64),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::huge_u32(
    iter = u32_with_size(1024 * 1024 * 4),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::small_i32(
    iter = i32_with_size(1024),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::large_i32(
    iter = i32_with_size(1024 * 64),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::huge_i32(
    iter = i32_with_size(1024 * 1024 * 4),
    setup = gen_input,
    teardown = confirm_sorted
)]
fn stdsort<T: RadixSortable + Ord>((mut values, buf): InputPair<T>) -> InputPair<T> {
    values.sort();
    (values, buf)
}

library_benchmark_group!(
    name = bench_sort_group;
    compare_by_id = true;
    benchmarks = stdsort,radix
);

main!(library_benchmark_groups = bench_sort_group);
