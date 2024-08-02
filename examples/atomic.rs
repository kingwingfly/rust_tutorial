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
            x1.store(1, Ordering::Release);
            y1.store(4, Ordering::Relaxed);
        });
        let jh2 = thread::spawn(move || {
            if x2.load(Ordering::Acquire) == 1 {
                let old = y2.load(Ordering::Relaxed);
                y2.store(old * 2, Ordering::Relaxed);
            }
        });
        jh1.join().unwrap();
        jh2.join().unwrap();
        let y = y.load(Ordering::Relaxed);
        assert!(y == 4 || y == 8 || y == 6, "y = {}", y);
    })
}
