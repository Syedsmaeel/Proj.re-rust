use sushi::Framebuffer;

fn main() -> Result<(), String> {
    let mut fb = Framebuffer::open()?;
    
    // Draw a single "Sushi Orange" pixel at the center
    fb.set_pixel(960, 540, 0xFFFF7800); 
    
    println!("✅ Drawn a pixel directly to video RAM.");
    Ok(())
}
