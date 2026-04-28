use std::sync::{Arc, Mutex};
use std::time::Duration;

use sushi::{use_state, ChildSpec, Reconciler, Setter, TuiHost};

fn main() {
    let mut r = Reconciler::new();
    let root = r.create_host_root();

    let setter_slot: Arc<Mutex<Option<Setter<i32>>>> = Arc::new(Mutex::new(None));
    let slot = setter_slot.clone();

    let counter = r.create_function_fiber("Counter", move || {
        let (n, set_n) = use_state(0_i32);
        *slot.lock().unwrap() = Some(set_n);
        vec![
            ChildSpec::Host {
                tag: "label".into(),
                text: format!("counter = {n}"),
            },
            ChildSpec::Host {
                tag: "hint".into(),
                text: format!("(parity: {})", if n % 2 == 0 { "even" } else { "odd" }),
            },
        ]
    });
    r.append_child(root, counter);

    let mut host = TuiHost::new();
    r.schedule_update(root);
    r.pump(&mut host, Duration::from_secs(1));

    println!("After initial render:\n{}\n", host.render_text());

    for i in 1..=5 {
        if let Some(s) = setter_slot.lock().unwrap().as_ref() {
            s.set(i);
        }
        r.pump(&mut host, Duration::from_millis(16));
        println!("After set({i}):\n{}\n", host.render_text());
    }
}
