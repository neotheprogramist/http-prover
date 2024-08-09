use core::array::ArrayTrait;
fn main(mut n: Array<felt252>) -> Array<felt252> {
    let r = fib(n.pop_front().unwrap());
    array![r.into()]
}

fn fib(n: felt252) -> felt252 {
    if n == 1 || n == 0 {
        return n;
    }

    fib(n - 1) + fib(n - 2)
}