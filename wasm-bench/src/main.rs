#![recursion_limit = "128"]
#![feature(test)]

extern crate test;
#[macro_use]
extern crate stdweb;

use stdweb::unstable::TryInto;

const DATA_SIZE: usize = 2048;

pub struct Auxiliary {
    pub data: [u8; DATA_SIZE],
}

impl Default for Auxiliary {
    fn default() -> Self {
        Self {
            data: [0; DATA_SIZE],
        }
    }
}

/// The common interface defining a task
pub trait Task {
    fn with_prereq(self, prereq: i32) -> Self;
    fn tick(&self, inp: &mut Auxiliary, prereq: i32) -> i32;
}

pub type TaskCallable = dyn Fn(&mut Auxiliary, i32) -> i32;
pub type TaskFnPtr = fn(&mut Auxiliary, i32) -> i32;

/// Holds callable tasks and prerequisites
pub struct TaskV1 {
    task: Box<TaskCallable>,
    prereq: Option<i32>,
}

impl TaskV1 {
    pub fn new(task: Box<TaskCallable>) -> Self {
        Self { task, prereq: None }
    }
}

impl Task for TaskV1 {
    fn with_prereq(mut self, prereq: i32) -> Self {
        self.prereq = Some(prereq);
        self
    }

    fn tick(&self, inp: &mut Auxiliary, prereq: i32) -> i32 {
        if self.prereq.map(|p| p > prereq).unwrap_or(true) {
            (*self.task)(inp, prereq)
        } else {
            0
        }
    }
}

/// Holds callable tasks
pub struct TaskV2 {
    task: Box<TaskCallable>,
}

impl TaskV2 {
    pub fn new(task: Box<TaskCallable>) -> Self {
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

    fn tick(&self, inp: &mut Auxiliary, prereq: i32) -> i32 {
        (*self.task)(inp, prereq)
    }
}

/// Holding function pointers instead of dyn Fn
/// This has the limitation of holding only stateless tasks
/// But has the benefit of avoiding Boxes
pub struct TaskV3 {
    task: TaskFnPtr,
    prereq: Option<i32>,
}

impl TaskV3 {
    pub fn new(task: TaskFnPtr) -> Self {
        Self { task, prereq: None }
    }
}

impl Task for TaskV3 {
    fn with_prereq(mut self, prereq: i32) -> Self {
        self.prereq = Some(prereq);
        self
    }

    fn tick(&self, inp: &mut Auxiliary, prereq: i32) -> i32 {
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
                TaskV1::new(Box::new(move |x, j| {
                    x.data[pseudo_random_ind(i + j)] as i32
                }))
                .with_prereq(32)
            } else {
                TaskV1::new(Box::new(move |x, j| {
                    x.data[pseudo_random_ind(i + j)] as i32 * 2
                }))
            }
        })
        .collect::<Vec<_>>()
}

pub fn prepare_v2() -> Vec<TaskV2> {
    (0..1_000_000)
        .map(|i| {
            if i % 128 == 0 {
                TaskV2::new(Box::new(move |x, j| {
                    x.data[pseudo_random_ind(i + j)] as i32
                }))
                .with_prereq(32)
            } else {
                TaskV2::new(Box::new(move |x, j| {
                    x.data[pseudo_random_ind(i + j)] as i32 * 2
                }))
            }
        })
        .collect::<Vec<_>>()
}

pub fn prepare_v3() -> Vec<TaskV3> {
    (0..1_000_000)
        .map(|i| {
            if i % 128 == 0 {
                TaskV3::new(|i, j| i.data[pseudo_random_ind(j)] as i32).with_prereq(32)
            } else {
                TaskV3::new(|i, j| i.data[pseudo_random_ind(j)] as i32 * 2)
            }
        })
        .collect::<Vec<_>>()
}

// Simulate random memory access
pub fn pseudo_random_ind(i: i32) -> usize {
    let i = match i & 255 {
        1 => i * 2,
        2 => i * 8,
        4 => i * 3,
        8 => i * 2,
        16 => i * 7,
        32 => i * 5,
        64 => i * 9,
        128 => i * 10,
        _ => i,
    };

    i as usize % DATA_SIZE
}

macro_rules! js_benchmark {
    ($tasks: ident) => {{
        let results = (0..1_000)
            .map(|_| {
                let mut aux = Auxiliary::default();
                let start = js! {return Date.now();};

                $tasks.iter().enumerate().for_each(|(i, t)| {
                    t.tick(&mut aux, i as i32);
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
    fn bench_index_creation(b: &mut Bencher) {
        b.iter(move || (0..1000).map(|i| pseudo_random_ind(i)).sum::<usize>());
    }

    #[bench]
    fn v1(b: &mut Bencher) {
        let tasks = prepare_v1();
        let mut aux = Auxiliary::default();

        b.iter(|| {
            tasks.iter().enumerate().for_each(|(i, t)| {
                t.tick(&mut aux, i as i32);
            })
        });
    }

    #[bench]
    fn v2(b: &mut Bencher) {
        let tasks = prepare_v2();
        let mut aux = Auxiliary::default();

        b.iter(|| {
            tasks.iter().enumerate().for_each(|(i, t)| {
                t.tick(&mut aux, i as i32);
            })
        });
    }

    #[bench]
    fn v3(b: &mut Bencher) {
        let tasks = prepare_v3();
        let mut aux = Auxiliary::default();

        b.iter(|| {
            tasks.iter().enumerate().for_each(|(i, t)| {
                t.tick(&mut aux, i as i32);
            })
        });
    }

}

