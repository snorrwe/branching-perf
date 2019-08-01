class TaskV1 {
    constructor(task) {
        this.task = task;
        this.prereq = null;
    }

    withPrereq(prereq) {
        this.prereq = prereq;
        return this;
    }

    tick(inp, prereq) {
        if (this.prereq == null || this.prereq > prereq) {
            return this.task(inp, prereq);
        }
        return 0;
    }
}

class TaskV2 {
    constructor(task) {
        this.task = task;
    }

    withPrereq(prereq) {
        let task = this.task;
        this.task = (i, p) => {
            if (this.prereq == null || this.prereq > prereq) {
                return task(i, p);
            }
            return 0;
        };
        return this;
    }

    tick(inp, prereq) {
        return this.task(inp, prereq);
    }
}

function createV1() {
    const tasks = [];
    for (let i = 0; i < 1000 * 1000; ++i) {
        if (i % 128 === 0) {
            tasks.push(new TaskV1((i, _p) => i).withPrereq(32));
        } else {
            tasks.push(new TaskV1((i, _p) => i * 2));
        }
    }

    return tasks;
}

function createV2() {
    const tasks = [];
    for (let i = 0; i < 1000 * 1000; ++i) {
        if (i % 128 === 0) {
            tasks.push(new TaskV2((i, _p) => i).withPrereq(32));
        } else {
            tasks.push(new TaskV2((i, _p) => i * 2));
        }
    }

    return tasks;
}

function bench(tasks) {
    const results = [];

    for (let i = 0; i < 1000; ++i) {
        let start = Date.now();

        for (let a of tasks.entries()) {
            let [i, t] = a;
            t.tick(0, i);
        }

        let dur = Date.now() - start;

        results.push(dur);
    }

    return results.reduce((sum, i) => (sum += i), 0) / results.length;
}

const tasksv1 = createV1();
const tasksv2 = createV2();

const v1 = bench(tasksv1);
const v2 = bench(tasksv2);

console.log("V1: ", v1, "ms");
console.log("V2: ", v2, "ms");