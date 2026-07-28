#![cfg(not(target_arch = "wasm32"))]

use compositor::{
    CanvasClearDescriptor, Compositor, FrameDescriptor, FrameItemDescriptor, LayerDescriptor,
    QuadTransformDescriptor,
};
use gpu::GpuContext;

fn block_on<F: Future>(future: F) -> F::Output {
    use std::sync::Arc;
    use std::task::{Context, Poll, Wake, Waker};

    struct ThreadWaker(std::thread::Thread);

    impl Wake for ThreadWaker {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }

    let waker = Waker::from(Arc::new(ThreadWaker(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = Box::pin(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::park(),
        }
    }
}

#[test]
fn renders_solid_layer_offscreen() {
    let Some(context) = block_on(GpuContext::new()).ok() else {
        eprintln!("skipping: no GPU adapter available");
        return;
    };
    let mut compositor = Compositor::new(&context);

    let width = 64;
    let height = 64;
    let red = [255u8, 0, 0, 255];
    let pixels: Vec<u8> = red.repeat((width * height) as usize);
    let texture = context.create_render_texture(width, height, "test-source");
    context.queue().write_texture(
        texture.as_image_copy(),
        &pixels,
        gpu::wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(width * 4),
            rows_per_image: Some(height),
        },
        texture.size(),
    );
    compositor.upsert_texture("source".to_string(), texture);

    let frame = FrameDescriptor {
        width,
        height,
        clear: CanvasClearDescriptor {
            color: [0.0, 0.0, 0.0, 1.0],
        },
        items: vec![FrameItemDescriptor::Layer(LayerDescriptor {
            texture_id: "source".to_string(),
            transform: QuadTransformDescriptor {
                center_x: 32.0,
                center_y: 32.0,
                width: width as f32,
                height: height as f32,
                rotation_degrees: 0.0,
                flip_x: false,
                flip_y: false,
            },
            opacity: 1.0,
            blend_mode: compositor::BlendMode::Normal,
            effect_pass_groups: Vec::new(),
            mask: None,
        })],
    };

    let output = compositor
        .render_frame_to_texture(&context, &frame)
        .expect("render");
    let rendered = block_on(context.read_texture(&output)).expect("readback");
    assert_eq!(rendered.len(), (width * height * 4) as usize);

    let red_count = rendered
        .chunks_exact(4)
        .filter(|px| px[0] > 200 && px[1] < 60 && px[2] < 60)
        .count();
    let total = (width * height) as usize;
    assert!(
        red_count * 100 / total > 90,
        "expected mostly red frame, got {red_count}/{total} red pixels"
    );
}
