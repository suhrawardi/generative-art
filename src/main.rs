use nannou::prelude::*;
use rand::distributions::{Distribution, Uniform};
use rand_distr::{LogNormal};
use rand::seq::SliceRandom;


const COLORS: [Rgb<u8>; 5] = [
    OLIVEDRAB,
    DARKOLIVEGREEN,
    OLIVE,
    OLIVE,
    OLIVE
];

fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

struct Model {
    texture: wgpu::Texture,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    texture_capturer: wgpu::TextureCapturer,
    texture_reshaper: wgpu::TextureReshaper,
    p: Vector2,
    s: Vector2
}

fn model(app: &App) -> Model {
    // 4K UHD texture
    let texture_size = [3_840, 2_160];

    let [win_w, win_h] = [texture_size[0] / 4, texture_size[1] / 4];
    let w_id = app
        .new_window()
        .size(win_w, win_h)
        .title("second")
        .view(view)
        .build()
        .unwrap();
    let window = app.window(w_id).unwrap();

    let device = window.swap_chain_device();

    let sample_count = window.msaa_samples();
    let texture = wgpu::TextureBuilder::new()
        .size(texture_size)
        .usage(wgpu::TextureUsage::OUTPUT_ATTACHMENT | wgpu::TextureUsage::SAMPLED)
        .sample_count(sample_count)
        .format(wgpu::TextureFormat::Rgba16Float)
        .build(device);

    let draw = nannou::Draw::new();
    let descriptor = texture.descriptor();
    let renderer = nannou::draw::RendererBuilder::new()
        .build_from_texture_descriptor(device, descriptor);

    let texture_capturer = wgpu::TextureCapturer::default();

    let texture_view = texture.create_default_view();
    let texture_component_type = texture.component_type();
    let dst_format = Frame::TEXTURE_FORMAT;
    let texture_reshaper = wgpu::TextureReshaper::new(
        device,
        &texture_view,
        sample_count,
        texture_component_type,
        sample_count,
        dst_format,
    );

    std::fs::create_dir_all(&capture_directory(app)).unwrap();

    Model {
        texture,
        draw,
        renderer,
        texture_capturer,
        texture_reshaper,
        p:vec2(0.0,0.0),
        s:vec2(0.0,0.0)
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
    let draw = &model.draw;
    draw.reset();

    let [w, h] = model.texture.size();
    // let r = geom::Rect::from_w_h(w as f32, h as f32);

    let elapsed_frames = app.main_window().elapsed_frames();
    // let t = elapsed_frames as f32 / 60.0;

    model.p = model.s;
    model.s = next_point(model.s, w as f32, h as f32);

    if elapsed_frames == 0 {
        draw.background().color(BLACK);
    }

    let log_normal = LogNormal::new(2.0, 3.0).unwrap();
    let i: f32 = log_normal.sample(&mut rand::thread_rng()) % 500.0;

    if i > 499.0 {
        draw.rect()
            .x_y(0.0,0.0)
            .w_h(w as f32, h as f32)
            .color(hsla(0.0,0.0,0.0,0.005));
    }

    let j: f32 = log_normal.sample(&mut rand::thread_rng()) % 300.0;
    if j > 298.0 {
        draw.line()
            .start(model.p)
            .end(model.s)
            .weight(2.0)
            .color(SIENNA);
    } else {
        let mut rng = rand::thread_rng();
        let color_obj: Rgb<u8> = *COLORS.choose(&mut rng).unwrap();
        let color = Srgb::<f32>::from_format(color_obj).into_linear();
        draw.line()
            .start(model.p)
            .end(model.s)
            .weight(1.0)
            .color(color);
    }

    let window = app.main_window();
    let device = window.swap_chain_device();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);
    model
        .renderer
        .render_to_texture(device, &mut encoder, draw, &model.texture);

    let snapshot = model
        .texture_capturer
        .capture(device, &mut encoder, &model.texture);

    window.swap_chain_queue().submit(&[encoder.finish()]);

    if elapsed_frames % 1000 == 0 {
        let path = capture_directory(app)
            .join(elapsed_frames.to_string())
            .with_extension("png");
        snapshot
            .read(move |result| {
                let image = result.expect("failed to map texture memory");
                image
                    .save(&path)
                    .expect("failed to save texture to png image");
            })
            .unwrap();
    }
}

fn view(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();
    model
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}

fn exit(app: &App, model: Model) {
    println!("Waiting for PNG writing to complete...");
    let window = app.main_window();
    let device = window.swap_chain_device();
    model
        .texture_capturer
        .await_active_snapshots(&device)
        .unwrap();
    println!("Done!");
}

fn next_point(point: Vector2, w: f32, h: f32) -> Vector2 {
    let mut rng = rand::thread_rng();
    let die = Uniform::from(1..9);
    let throw = die.sample(&mut rng);

    if throw == 1 {
        return pt2(next_plus(point.x, w), point.y);
    } else if throw == 2 {
        return pt2(point.x, next_plus(point.y, h));
    } else if throw == 3 {
        return pt2(next_min(point.x, w), point.y);
    } else if throw == 4 {
        return pt2(point.x, next_min(point.y, h));
    } else if throw == 5 {
        return pt2(next_plus(point.x, w), next_plus(point.y, h));
    } else if throw == 6 {
        return pt2(next_min(point.x, w), next_plus(point.y, h));
    } else if throw == 7 {
        return pt2(next_plus(point.x, w), next_min(point.y, h));
    } else {
        return pt2(next_min(point.x, w), next_min(point.y, h));
    }
}

fn prob(coord: f32) -> bool {
    let log_normal = LogNormal::new(2.0, 3.0).unwrap();
    let i: f32 = log_normal.sample(&mut rand::thread_rng()) % 300.0;
    let prob: bool = (i - (300.0 - coord.abs())) > 300.0;
    return prob;
}

fn step() -> f32 {
    let log_normal = LogNormal::new(2.0, 3.0).unwrap();
    let i: f32 = log_normal.sample(&mut rand::thread_rng()) % 300.0;

    if i > 299.0 {
        return 3.0;
    } else if i > 295.0 {
        return 2.0;
    } else {
        return 1.0;
    }
}

fn next_min(coord: f32, s: f32) -> f32 {
    if coord <= -(s / 2.0) {
        return coord + 1.0;
    } else {
        if coord > 0.0 && coord < (s / 2.0 - 5.0) && prob(coord) {
            return coord + step();
        } else {
            return coord - 1.0;
        }
    }
}

fn next_plus(coord: f32, s: f32) -> f32 {
    if coord >= s / 2.0 {
        return coord - 1.0;
    } else {
        if coord < 0.0 && coord > -(s / 2.0 - 5.0) && prob(coord) {
            return coord - step();
        } else {
            return coord + 1.0;
        }
    }
}

fn capture_directory(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("could not locate project_path")
        .join(app.exe_name().unwrap())
}
