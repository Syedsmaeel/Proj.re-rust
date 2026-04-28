use boa_engine::{Context, JsValue, NativeFunction};
use sushi::Canvas;

pub fn register_sushi_bridge(ctx: &mut Context, canvas: &mut Canvas) {
    // We bind the drawRect function to JS
    let draw_rect = NativeFunction::from_fn_ptr(|_this, args, context| {
        let x = args.get(0).unwrap().as_number().unwrap() as f32;
        let y = args.get(1).unwrap().as_number().unwrap() as f32;
        let w = args.get(2).unwrap().as_number().unwrap() as f32;
        let h = args.get(3).unwrap().as_number().unwrap() as f32;
        
        println!("🖌️ JS called drawRect: {} {} {} {}", x, y, w, h);
        Ok(JsValue::Boolean(true))
    });

    ctx.register_global_function("drawRect", 4, draw_rect).unwrap();
}
