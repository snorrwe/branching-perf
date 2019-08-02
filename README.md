# Benchmarking Task prerequisite performance between 2 implementations

__TBA__: add v3 description and results

## The task

Implement a data structure that can hold tasks which are callables accepting 2 parameters: `input` which is the input to the task and `prerequisite` which is a number representing some precondition

Tasks may or may not have a prerequisite if they do not have one the precondition is considered met by default.

We want to be able to execute a collection of heterogeneous tasks.

## Implementation

We'll consider two implementations:

- The first one ( __V1__ ) has an optional data member called `prereq` which is checked before executing the callable member (see the `tick` methods in tasks/src/lib.rs or js-bench/main.js)
- The second implementation ( __V2__ ) replaces the internal task with one that checks for the precondition if a precondition is required (see the `tick` methods in tasks/src/lib.rs or js-bench/main.js). This way the condition check is omitted altogether if the task has no preconditions

## Benchmarks

We'll iterate over a list of a *million* tasks that are created by the following rules:

    - If the index of the task modulo 128 is 0 then the task has a prerequisite of 32 and returns the input
    - Else the task has no precondition and returns `input * 2`

We'll use fix input while iterating, all tasks are called with `input == 1` and `prerequisite == index of the task`
We'll measure the average time to iterate and execute all tasks

## Structure

```
+ js-bench/   # The JavaScript benchmark
    main.js

+ wasm-bench/ # The Rust WASM benchmarks and the native benchmarks
  ~ src/
      main.rs 
    Cargo.toml
    Web.toml
```

## Environment

The benchmarks were ran on a Intel Code i7-8700 processor<br>
The Rust source was compiled using `cargo 1.37.0-nightly (4c1fa54d1 2019-06-24)` and `cargo-web 0.6.25`<br>
The WASM and JS tests were ran using Node.JS 10.15.1<br>

## Running

- Native: `cd wasm-bench` then `cargo bench`
- WASM: `cd wasm-bench` then `cargo web build --release` then `node target/wasm32-unknown-unknown/release/wasm-bench.js`
- JS:`cd js-bench` then `node main.js`

## Results


- Running `cargo bench` (note 1.8 million ns == 1.8 ms):

    - test v1 ... bench:   1,873,860 ns/iter (+/- 98,513)
    - test v2 ... bench:   1,540,270 ns/iter (+/- 56,746)

- Running `cargo web build --release` then `node wasm-bench.js` in the target directory to execute the wasm code:

    - V1:  5.364999771118164 ms
    - V2:  5.193999767303467 ms

- Running `node main.js` in the js-bench directory:

    - V1:  15.043 ms
    - V2:  14.371 ms


The native code gained an about 20% speedup by the V2 implementation. While the WASM performance gains are negligible. The JavaScript implementation gained about 5% while the WASM implementation gained about 3%.

WASM overall was still 3 times faster than the JavaScript implementation. While the native Rust code ran 3 times faster than the WASM.

## Conclusion

Avoiding the unnecessary checks benefits native code execution greatly, supposedly because of the branch predictor of the CPU. However the same could be barely said about the WASM and JavaScript implementations. What I learned from this experiment is that you should not expect that the same techinques used doing CPU optimizations will work the same when compiling to WASM. 
