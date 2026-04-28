use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use sushi::reconciler::fiber::WorkTag;
use sushi::{use_state, ChildSpec, DebugHost, Props, Reconciler, Setter};

fn main() -> Result<(), String> {
    println!("⚛️  sushi — Sovereign React (Pure Rust Reconciler)\n");

    let mut r = Reconciler::new();
    let root = r.create_host_root();

    let setter_slot: Arc<Mutex<Option<Setter<i32>>>> = Arc::new(Mutex::new(None));
    let slot = setter_slot.clone();

    let counter = r.create_function_fiber("Counter", move || {
        let (n, set_n) = use_state(0_i32);
        *slot.lock().unwrap() = Some(set_n);
        vec![ChildSpec::Text(format!("count = {n}"))]
    });
    r.append_child(root, counter);

    let mut host = DebugHost::new();

    println!("--- [PHASE 1: INITIAL RENDER] ---");
    r.schedule_update(root);
    r.work_loop_with_deadline(None);
    r.finish_commit(&mut host)?;

    println!("\n--- [PHASE 2: STATE UPDATE via setter ] ---");
    if let Some(s) = setter_slot.lock().unwrap().as_ref() {
        s.set(42);
    }
    r.pump(&mut host, Duration::from_secs(1));

    println!("\n--- [PHASE 3: SECOND UPDATE — only diff is committed ] ---");
    if let Some(s) = setter_slot.lock().unwrap().as_ref() {
        s.set(43);
    }
    r.pump(&mut host, Duration::from_secs(1));

    println!("\n--- [PHASE 4: NO-OP UPDATE — same value, no commit ] ---");
    if let Some(s) = setter_slot.lock().unwrap().as_ref() {
        s.set(43);
    }
    let log_len_before = host.log.len();
    r.pump(&mut host, Duration::from_secs(1));
    println!("  host log grew by {}", host.log.len() - log_len_before);

    println!("\n--- [PHASE 5: INTERRUPTIBLE WORK LOOP ] ---");
    let big = r.create_host_root();
    for i in 0..2_000 {
        let id = r.create_fiber(WorkTag::HostComponent, Props::Text(format!("item {i}")));
        r.append_child(big, id);
    }
    r.schedule_update(big);
    let started = Instant::now();
    r.work_loop_with_deadline(Some(Duration::from_micros(500)));
    let elapsed = started.elapsed();
    println!(
        "  yielded after {:?}; remaining unit_of_work: {:?}; render_complete = {}",
        elapsed,
        r.next_unit_of_work,
        r.is_render_complete()
    );

    println!("\n--- [PHASE 6: RESUME — finish the rest in unbounded run ] ---");
    let started = Instant::now();
    r.work_loop_with_deadline(None);
    println!(
        "  finished remaining work in {:?}; render_complete = {}",
        started.elapsed(),
        r.is_render_complete()
    );

    println!("\n✅ Reconciler upgraded.");
    Ok(())
}
