# Atomic

See the example below.

```rust
use loom::sync::atomic::{AtomicUsize, Ordering};
use loom::sync::Arc;
use loom::thread;

fn main() {
    loom::model(|| {
        let x = Arc::new(AtomicUsize::new(0));
        let y = Arc::new(AtomicUsize::new(1));
        let x1 = x.clone();
        let y1 = y.clone();
        let x2 = x.clone();
        let y2 = y.clone();
        let jh1 = thread::spawn(move || {
            y1.store(3, Ordering::Relaxed);
            x1.store(1, Ordering::Relaxed);
        });
        let jh2 = thread::spawn(move || {
            if x2.load(Ordering::Relaxed) == 1 {
                let old = y2.load(Ordering::Relaxed);
                y2.store(old * 2, Ordering::Relaxed);
            }
        });
        jh1.join().unwrap();
        jh2.join().unwrap();
        let y = y.load(Ordering::Acquire);
        assert!(y == 3 || y == 6, "y = {}", y);
    })
}
```

This example is copied from `the Rustonomicon`, and all the atomic ops are `Ordering::Relaxed`.

```shell
initial state: x = 0, y = 1

THREAD 1        THREAD 2
y = 3;          if x == 1 {
x = 1;              y *= 2;
                }
```

Easily, we can guess there two output:

- thread2 sees `x==1`, then it sees `y==3`, so the output is `y==6`.
- or, thread 2 doesn’t see `x==1`, so `y*=2` doesn’t run, the output must be `y==3`.

However, due to cache, CPU disordering, compiler or something advanced else, here’s a annoying situation:

- thread2 sees `x==1`, but it also sees `y==1`, so the output is `y==2`.



This brings us into `atomic` to avoid the third situation.

I have to say, the `Ordering` of atomic ops are quite hard to understand, so let’s struggle on it anyway.



> Code can be found at examples in this tutorial repo.
>
> Run
>
> ```rust
> LOOM_LOG=debug \
> LOOM_LOCATION=1 \
> LOOM_CHECKPOINT_INTERVAL=1 \
> LOOM_CHECKPOINT_FILE=loom.json \
> cargo run --example atomic --features atomic --release -- --nocapture
> ```
>
> to see the results

## The tool we use to find such magical `y==2` state

The tool we use is [`loom`](https://github.com/tokio-rs/loom).

If we run the example

```rust
cargo run --example atomic --features atomic --release -- --nocapture
```

output in stdio is

```shell
INFO loom::model:
INFO loom::model:  ================== Iteration 1 ==================
INFO loom::model:
INFO iter{1}:thread{id=1}: loom::rt::execution: ~~~~~~~~ THREAD 1 ~~~~~~~~
INFO iter{1}:thread{id=0}: loom::rt::execution: ~~~~~~~~ THREAD 0 ~~~~~~~~
INFO iter{1}:thread{id=2}: loom::rt::execution: ~~~~~~~~ THREAD 2 ~~~~~~~~
INFO iter{1}:thread{id=0}: loom::rt::execution: ~~~~~~~~ THREAD 0 ~~~~~~~~
thread 'main' panicked at examples/atomic.rs:26:9:
y = 2
```

`y=2` is output by this `assert!(y == 3 || y == 6, "y = {}", y);`

So we found such a strange state posted by loom…

## Fix this

### But how?

No matter what leads to this, what we are going to do is to ensure:

- if thread2 sees `x==1`, it **must** see `y==3`(not 1), and result in `y==6`
- if thread2 sees `x!=1`, the result is always `y==3`, we do not care this situation (`y*=2` never run)

This is `Ordering`'s show.

Let’s modify the way we talk about the first situation above:

- If thread2 `acquired` x\=\=1, which means thread1 `released` x=1, which means thread1 `released` y=3, then thread2 must `acquired` y\=\=3, resulting in y=6, no other output in this branch.

No matter what `Acquire` `Release` in Rust or Atomic, let’s understand it as English.

- `Release`, release values to others, so that others can acquire the values
- `Acquire`, acquire the released values from others

With above in mind, we change code to this:

```rust
let jh1 = thread::spawn(move || {
    y1.store(3, Ordering::Relaxed);
    x1.store(1, Ordering::Release);	// release value to others
});
let jh2 = thread::spawn(move || {
    if x2.load(Ordering::Acquire) == 1 {  // acquire released value
        let old = y2.load(Ordering::Relaxed);
        y2.store(old * 2, Ordering::Relaxed);
    }
});
```

We guess it’s Ok, the truth also makes us happy:

```shell
...
INFO loom::model: Completed in 105 iterations
```

No assert fails, nice.

## Actual semantics of Ordering

Noticing many `Relexed` still remained in the code, how can we assert it’s right without tools like `loom`?

So we have to properly understand `Release` and `Acquire`.

Actually, every atomic op has its Ordering,

- `Release`, ops in the same thread before me stay before me in others' eyes.
- `Acquire`, ops in the same thread after me stay after me in others' eyes.

More concrete, “ops before me stay before me”:

- In thread1, we store x=1 in ordering release.

- In thread2, we load x==1 in ordering acquire

- Synchronization happens

- In thread2’s eyes, storing y=3 stays before storing x=1.


The same as acquire, "ops after me stay after me":

- In thread2, we load x in ordering acquire.
- In thread1, we store x=1 in ordering release.
- Synchronization happens
- In thread1’s eyes, loading y\=\=3 stays after loading x\=\=1.

Let’s copy code down here:

```rust
let jh1 = thread::spawn(move || {
    y1.store(3, Ordering::Relaxed);
    x1.store(1, Ordering::Release);	// release value to others
});
let jh2 = thread::spawn(move || {
    if x2.load(Ordering::Acquire) == 1 {  // acquire released value
        let old = y2.load(Ordering::Relaxed);
        y2.store(old * 2, Ordering::Relaxed);
    }
});
```

What magic result happened with those two changes?

- The order become certain: `store y=3 then store x=1 then load x==1 then load y==3`
- output y==6

### Other details

1. ```rust
   let old = y2.load(Ordering::Relaxed);
   y2.store(old * 2, Ordering::Relaxed);	// this storing depends on old, so order is constrained
   ```

2. ```rust
   ...
   jh1.join().unwrap();
   jh2.join().unwrap();
   let y = y.load(Ordering::Relaxed);	// this Relaxed since all threads joined, it's really relaxed
   assert!(y == 3 || y == 6, "y = {}", y);
   ```

3. ```rust
   let jh1 = thread::spawn(move || {
       y1.store(4, Ordering::Relaxed);	// line 2
       y1.store(3, Ordering::Relaxed); // line 3
       x1.store(1, Ordering::Release);
   });
   let jh2 = thread::spawn(move || {
       if x2.load(Ordering::Acquire) == 1 {
           let old = y2.load(Ordering::Relaxed);
           y2.store(old * 2, Ordering::Relaxed);
       }
   });
   ...
   assert!(y == 3 || y == 6, "y = {}", y);
   // Within the same thread, line2 and line3 follow the `program order within a single thread`
   // line3 won't happen before line2
   ```

4. ```rust
   let jh1 = thread::spawn(move || {
       y1.store(3, Ordering::Relaxed); // line2
       x1.store(1, Ordering::Release); // line3
       y1.store(4, Ordering::Relaxed);	// line4
   });
   let jh2 = thread::spawn(move || {
       if x2.load(Ordering::Acquire) == 1 {
           let old = y2.load(Ordering::Relaxed); // line8
           y2.store(old * 2, Ordering::Relaxed);
       }
   });
   ...
   let y = y.load(Ordering::Relaxed);
   assert!(y == 4 || y == 8 || y == 6, "y = {}", y);
   // line4 can be seen by line8, since release--the before stay before, the after are free
   // However, line4 can never happen before line2
   ```
