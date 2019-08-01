extern crate tasks;
#[macro_use]
extern crate stdweb;

use stdweb::unstable::TryInto;
use tasks::*;

macro_rules! benchmark_tasks {
    ($tasks: ident) => {{
        let results = (0..1_000)
            .map(|_| {
                let start = js! {return Date.now();};

                $tasks.iter().enumerate().for_each(|(i, t)| {
                    t.tick(0, i as i32);
                });

                let duration = js! {
                    const start = @{start};
                    const dur = Date.now() - start;
                    return dur;
                };

                duration.try_into().unwrap()
            })
            .collect::<Vec<f64>>();

        let l = results.len();
        let sum: f64 = results.into_iter().sum();

        let avg = sum as f32 / l as f32;
        avg
    }};
}

fn prepare_taskv1() -> Vec<TaskV1> {
    (0..1_000_000)
        .map(|i| {
            if i % 128 == 0 {
                TaskV1::new(Box::new(|i, _| i)).with_prereq(32)
            } else {
                TaskV1::new(Box::new(|i, _| i * 2))
            }
        })
        .collect::<Vec<_>>()
}

fn prepare_taskv2() -> Vec<TaskV2> {
    (0..1_000_000)
        .map(|i| {
            if i % 128 == 0 {
                TaskV2::new(Box::new(|i, _| i)).with_prereq(32)
            } else {
                TaskV2::new(Box::new(|i, _| i * 2))
            }
        })
        .collect::<Vec<_>>()
}

fn bench_v1() -> f32 {
    let tasks = prepare_taskv1();

    benchmark_tasks!(tasks)
}

fn bench_v2() -> f32 {
    let tasks = prepare_taskv2();

    benchmark_tasks!(tasks)
}

fn main() {
    stdweb::initialize();

    let v1 = bench_v1();
    let v2 = bench_v2();

    js! {
        const v1 = @{v1};
        const v2 = @{v2};
        console.log("V1: ", v1, "ms");
        console.log("V2: ", v2, "ms");
    };
}

