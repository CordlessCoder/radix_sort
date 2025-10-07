use gungraun::{library_benchmark, library_benchmark_group, main};
use radix_sort::{RadixSortable, make_buf, radix_sort};
use rand::{RngCore, SeedableRng, rngs::SmallRng};
use std::{cell::RefCell, hint::black_box, mem::MaybeUninit};

type InputPair<T> = (Vec<T>, Vec<MaybeUninit<T>>);

fn ascending_u32(idx: usize) -> u32 {
    idx as u32
}

fn reverse_u32(idx: usize) -> u32 {
    u32::MAX - idx as u32
}

fn random_u32(_: usize) -> u32 {
    thread_local! {
        static RNG: RefCell<SmallRng> = RefCell::new(SmallRng::from_os_rng());
    }
    RNG.with(|r| r.borrow_mut().next_u32())
}

fn with_size(size: usize) -> impl Iterator<Item = (impl FnMut(usize) -> u32, usize)> {
    [ascending_u32, reverse_u32, random_u32]
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
#[benches::small(
    iter = with_size(1024),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::large(
    iter = with_size(1024 * 64),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::huge(
    iter = with_size(1024 * 1024 * 4),
    setup = gen_input,
    teardown = confirm_sorted
)]
fn rdxsort((mut values, buf): InputPair<u32>) -> InputPair<u32> {
    ::rdxsort::RdxSort::rdxsort(&mut values);
    (values, buf)
}

#[library_benchmark]
#[benches::small(
    iter = with_size(1024),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::large(
    iter = with_size(1024 * 64),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::huge(
    iter = with_size(1024 * 1024 * 4),
    setup = gen_input,
    teardown = confirm_sorted
)]
fn radix<T: RadixSortable>((mut values, mut buf): InputPair<T>) -> InputPair<T> {
    radix_sort(black_box(&mut values), black_box(&mut buf));
    (values, buf)
}

#[library_benchmark]
#[benches::small(
    iter = with_size(1024),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::large(
    iter = with_size(1024 * 64),
    setup = gen_input,
    teardown = confirm_sorted
)]
#[benches::huge(
    iter = with_size(1024 * 1024 * 4),
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
    benchmarks = stdsort,rdxsort,radix
);

main!(library_benchmark_groups = bench_sort_group);
