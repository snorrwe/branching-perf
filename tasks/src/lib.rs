//! Tasks have 2 inputs: and actual input and a prereq
//! prereq represents a prerequisite number that the task optionally has to meet or can ignore it

#![feature(test)]

extern crate test;
#[cfg(test)]
use test::Bencher;

pub struct TaskV1 {
    task: Box<dyn Fn(i32, i32) -> i32>,
    prereq: Option<i32>,
}

impl TaskV1 {
    pub fn new(task: Box<dyn Fn(i32, i32) -> i32>) -> Self {
        Self { task, prereq: None }
    }

    pub fn with_prereq(mut self, prereq: i32) -> Self {
        self.prereq = Some(prereq);
        self
    }

    pub fn tick(&self, inp: i32, prereq: i32) -> i32 {
        if self.prereq.map(|p| p > prereq).unwrap_or(true) {
            (*self.task)(inp, prereq)
        } else {
            0
        }
    }
}

pub struct TaskV2 {
    task: Box<dyn Fn(i32, i32) -> i32>,
}

impl TaskV2 {
    pub fn new(task: Box<dyn Fn(i32, i32) -> i32>) -> Self {
        Self { task }
    }

    pub fn with_prereq(self, prereq: i32) -> Self {
        Self::new(Box::new(
            move |i, p| if prereq <= p { (*self.task)(i, p) } else { 0 },
        ))
    }

    pub fn tick(&self, inp: i32, prereq: i32) -> i32 {
        (*self.task)(inp, prereq)
    }
}

#[bench]
fn v1(b: &mut Bencher) {
    let tasks = (0..1_000_000)
        .map(|i| {
            if i % 128 == 0 {
                TaskV1::new(Box::new(|i, _| i)).with_prereq(32)
            } else {
                TaskV1::new(Box::new(|i, _| i * 2))
            }
        })
        .collect::<Vec<_>>();

    b.iter(|| {
        tasks.iter().enumerate().for_each(|(i, t)| {
            t.tick(1, i as i32);
        })
    });
}

#[bench]
fn v2(b: &mut Bencher) {
    let tasks = (0..1_000_000)
        .map(|i| {
            if i % 128 == 0 {
                TaskV2::new(Box::new(|i, _| i)).with_prereq(32)
            } else {
                TaskV2::new(Box::new(|i, _| i * 2))
            }
        })
        .collect::<Vec<_>>();

    b.iter(|| {
        tasks.iter().enumerate().for_each(|(i, t)| {
            t.tick(1, i as i32);
        })
    });
}

