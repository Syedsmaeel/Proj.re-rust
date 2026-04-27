use sushi::reconciler::{Reconciler, fiber::{WorkTag, flags}};

fn main() -> Result<(), String> {
    println!("⚛️  sushi — Sovereign React (Pure Rust Reconciler)\n");

    let mut reconciler = Reconciler::new();

    // -- 1. THE INITIAL RENDER PHASE --
    println!("--- [PHASE 1: INITIAL RENDER] ---");
    // Create the root of our application
    let root_id = reconciler.create_fiber(WorkTag::HostRoot, Box::new("Root"));
    
    // Create a child (e.g. a Button)
    let button_id = reconciler.create_fiber(WorkTag::HostComponent, Box::new("Neon Button"));
    
    // Setup relationship & set 'Placement' flag (Initial Render)
    if let Some(root) = reconciler.fibers.get_mut(&root_id) {
        root.child_id = Some(button_id);
    }
    if let Some(btn) = reconciler.fibers.get_mut(&button_id) {
        btn.return_id = Some(root_id);
        btn.flags |= flags::PLACEMENT; // Mark for creation
    }

    // Run 'Begin Work' on the tree
    reconciler.begin_work(root_id)?;
    reconciler.begin_work(button_id)?;

    // -- 2. THE COMMIT PHASE --
    println!("\n--- [PHASE 2: COMMIT] ---");
    reconciler.commit_root(root_id)?;

    // -- 3. AN UPDATE (e.g. state change) --
    println!("\n--- [PHASE 3: STATE UPDATE] ---");
    if let Some(btn) = reconciler.fibers.get_mut(&button_id) {
        btn.flags |= flags::UPDATE; // Mark for refresh
        println!("State changed: Button needs to be re-rendered.");
    }
    
    reconciler.begin_work(button_id)?;
    reconciler.commit_root(root_id)?;

    println!("\n✅ Success! The Sushi Reconciler managed the lifecycle perfectly.");
    Ok(())
}
