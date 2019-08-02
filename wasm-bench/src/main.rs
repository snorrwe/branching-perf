#![recursion_limit = "128"]
#![feature(test)]

extern crate test;
#[macro_use]
extern crate stdweb;

use stdweb::unstable::TryInto;

/// The common interface defining a task
pub trait Task {
    fn with_prereq(self, prereq: i32) -> Self;
    fn tick(&self, inp: i32, prereq: i32) -> i32;
}

/// Holds callable tasks and prerequisites
pub struct TaskV1 {
    task: Box<dyn Fn(i32, i32) -> i32>,
    prereq: Option<i32>,
}

impl TaskV1 {
    pub fn new(task: Box<dyn Fn(i32, i32) -> i32>) -> Self {
        Self { task, prereq: None }
    }
}

impl Task for TaskV1 {
    fn with_prereq(mut self, prereq: i32) -> Self {
        self.prereq = Some(prereq);
        self
    }

    fn tick(&self, inp: i32, prereq: i32) -> i32 {
        if self.prereq.map(|p| p > prereq).unwrap_or(true) {
            (*self.task)(inp, prereq)
        } else {
            0
        }
    }
}

/// Holds callable tasks
pub struct TaskV2 {
    task: Box<dyn Fn(i32, i32) -> i32>,
}

impl TaskV2 {
    pub fn new(task: Box<dyn Fn(i32, i32) -> i32>) -> Self {
        Self { task }
    }
}

impl Task for TaskV2 {
    /// Wraps the original task with a precondition check
    fn with_prereq(self, prereq: i32) -> Self {
        Self::new(Box::new(
            move |i, p| if prereq <= p { (*self.task)(i, p) } else { 0 },
        ))
    }

    fn tick(&self, inp: i32, prereq: i32) -> i32 {
        (*self.task)(inp, prereq)
    }
}

/// Holding function pointers instead of dyn Fn
/// This has the limitation of holding only stateless tasks
/// But has the benefit of avoiding Boxes
pub struct TaskV3 {
    task: fn(i32, i32) -> i32,
    prereq: Option<i32>,
}

impl TaskV3 {
    pub fn new(task: fn(i32, i32) -> i32) -> Self {
        Self { task, prereq: None }
    }
}

impl Task for TaskV3 {
    fn with_prereq(mut self, prereq: i32) -> Self {
        self.prereq = Some(prereq);
        self
    }

    fn tick(&self, inp: i32, prereq: i32) -> i32 {
        if self.prereq.map(|p| p > prereq).unwrap_or(true) {
            (self.task)(inp, prereq)
        } else {
            0
        }
    }
}

pub fn prepare_v1() -> Vec<TaskV1> {
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

pub fn prepare_v2() -> Vec<TaskV2> {
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

pub fn prepare_v3() -> Vec<TaskV3> {
    (0..1_000_000)
        .map(|i| {
            if i % 128 == 0 {
                TaskV3::new(|i, _| i).with_prereq(32)
            } else {
                TaskV3::new(|i, _| i * 2)
            }
        })
        .collect::<Vec<_>>()
}

macro_rules! js_benchmark {
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

fn bench_v1() -> f32 {
    let tasks = prepare_v1();

    js_benchmark!(tasks)
}

fn bench_v2() -> f32 {
    let tasks = prepare_v2();

    js_benchmark!(tasks)
}

fn bench_v3() -> f32 {
    let tasks = prepare_v3();

    js_benchmark!(tasks)
}

fn main() {
    stdweb::initialize();

    let v1 = bench_v1();
    let v2 = bench_v2();
    let v3 = bench_v3();

    js! {
        const v1 = @{v1};
        const v2 = @{v2};
        const v3 = @{v3};
        console.log("V1: ", v1, "ms");
        console.log("V2: ", v2, "ms");
        console.log("V3: ", v3, "ms");
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use test::Bencher;

    #[bench]
    fn v1(b: &mut Bencher) {
        let tasks = prepare_v1();

        b.iter(|| {
            tasks.iter().enumerate().for_each(|(i, t)| {
                t.tick(1, i as i32);
            })
        });
    }

    #[bench]
    fn v2(b: &mut Bencher) {
        let tasks = prepare_v2();

        b.iter(|| {
            tasks.iter().enumerate().for_each(|(i, t)| {
                t.tick(1, i as i32);
            })
        });
    }

    #[bench]
    fn v3(b: &mut Bencher) {
        let tasks = prepare_v3();

        b.iter(|| {
            tasks.iter().enumerate().for_each(|(i, t)| {
                t.tick(1, i as i32);
            })
        });
    }

}

