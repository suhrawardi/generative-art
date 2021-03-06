use nannou::prelude::*;
use rand::distributions::{Distribution, Uniform};
use rand_distr::{LogNormal};
use nannou::noise::{NoiseFn, Perlin, Seedable};
// use palette::Rgba;


const RULE: i32 = 5;


fn main() {
    nannou::app(model).update(update).exit(exit).run();
}

struct Ca {
    w: f32,
    h: f32,
    size: f32,
    cells: Vec<i32>,
    rule_set: Vec<i32>,
}


impl Ca {
    fn new(w: f32, h: f32, size: f32, rule_set: Vec<i32>) -> Self {
        let w = w;
        let h = h;
//      let size = 4.0;
        let size = size;
        let cells = vec![0];
//      let mut cells = vec![0; (rect.w() / w) as usize];
//       cells[0] = 1; // We arbitrarily start with just the middle cell having a state of "1"
        Ca {
            w,
            h,
            size,
            cells,
            rule_set,
        }
    }

    // The process of creating the new generation
    fn generate(&mut self) {
        // First we create an empty array for the new values
        let mut next_gen = vec![0; self.cells.len()];

        // For every spot, determine new state by examing current state, and neighbor states
        // Ignore edges that only have one neighor
//      for i in self.cells_range.clone() {
//          let left = self.cells[i - 1]; // Left neighbor state
//          let me = self.cells[i]; // Current state
//          let right = self.cells[i + 1]; // Right beighbor state
//          next_gen[i] = self.rule(left, me, right);
//          // Compute next generation state based on ruleset
//      }
        // The current generation is the new generation
        self.cells = next_gen;
    }

    fn display(&self, draw: &Draw, noises: Vec<Perlin>, k: f64) {
        for i in (0..self.h as u32).step_by(self.size as usize) {
            for j in (0..self.w as u32).step_by(self.size as usize) {

                let log_normal = LogNormal::new(2.0, 3.0).unwrap();
                let fill: f32 = log_normal.sample(&mut rand::thread_rng()) % 300.0;
                let opa: f32 = log_normal.sample(&mut rand::thread_rng()) % 1.0;

                let y: f32 = i as f32 - self.h / 2.0 + self.size / 2.0;
                let x: f32 = j as f32 - self.w / 2.0 + self.size / 2.0;

                let n1 = noises[0].get([x as f64 / 300.0, y as f64 / 300.0, k as f64]);
                let n2 = noises[1].get([x as f64 / 300.0, y as f64 / 300.0, k as f64]);
                let n3 = noises[2].get([x as f64 / 300.0, y as f64 / 300.0, k as f64]);
                let n4 = noises[3].get([(x / fill) as f64, (y / fill) as f64, k as f64]);
                let nabs1 = n1.abs() as f32 % 1.0;
                let nabs2 = n2.abs() as f32 % 1.0;
                let nabs3 = n3.abs() as f32 % 1.0;
                let nabs4 = n4.abs() as f32 % opa;

                let c = rgba(nabs1, nabs2, nabs3, nabs4);

                draw.ellipse()
                    .color(c)
                    .w(self.size)
                    .h(self.size)
                    .x_y(x, y);
            }
        }
    }

    fn rule(&self, a: i32, b: i32, c: i32) -> i32 {
        let bstr = format!("{}{}{}", a, b, c);
        let int = isize::from_str_radix(&bstr, 2).unwrap();
        return self.rule_set[int as usize];
    }
}

struct Model {
    texture: wgpu::Texture,
    draw: nannou::Draw,
    renderer: nannou::draw::Renderer,
    texture_capturer: wgpu::TextureCapturer,
    texture_reshaper: wgpu::TextureReshaper,
    ca: Ca,
    noise_seeds: Vec<u32>,
}

fn model(app: &App) -> Model {
    app.set_loop_mode(LoopMode::rate_fps(2.0));

    // 4K UHD texture
    let w: f32 = 3_840.0;
    let h: f32 = 2_160.0;
    let size: f32 = 6.0;
    let texture_size = [w as u32, h as u32];

    let w_id = app
        .new_window()
        .size(w as u32, h as u32)
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
    &draw.background().color(LIGHTSLATEGRAY);
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

    let rule_set = match RULE {
        1 => vec![0, 1, 1, 1, 1, 0, 1, 1], // Rule 222
        2 => vec![0, 1, 1, 1, 1, 1, 0, 1], // Rule 190
        3 => vec![0, 1, 1, 1, 1, 0, 0, 0], // Rule 30
        4 => vec![0, 1, 1, 1, 0, 1, 1, 0], // Rule 110
        5 => vec![0, 1, 0, 1, 1, 0, 1, 0], // Rule 90
        _ => vec![0, 0, 0, 0, 0, 0, 0, 0],
    };

    let ca = Ca::new(w, h, size, rule_set);

    Model {
        texture,
        draw,
        renderer,
        texture_capturer,
        texture_reshaper,
        ca: ca,
        noise_seeds: vec![12, 17, 14, 100],
    }
}

fn update(app: &App, model: &mut Model, _update: Update) {
//  model.ca.generate();
    let draw = &model.draw;

    let noise1 = Perlin::new().set_seed(model.noise_seeds[0]);
    let noise2 = Perlin::new().set_seed(model.noise_seeds[1]);
    let noise3 = Perlin::new().set_seed(model.noise_seeds[2]);
    let noise4 = Perlin::new().set_seed(model.noise_seeds[3]);

    let elapsed_frames = app.main_window().elapsed_frames() as f64;
    model.ca.display(&draw, vec![noise1, noise2, noise3, noise4], elapsed_frames);

    let window = app.main_window();
    let device = window.swap_chain_device();
    let ce_desc = wgpu::CommandEncoderDescriptor {
        label: Some("texture renderer"),
    };
    let mut encoder = device.create_command_encoder(&ce_desc);
    model
        .renderer
        .render_to_texture(device, &mut encoder, &draw, &model.texture);

    window.swap_chain_queue().submit(&[encoder.finish()]);

    maybe_mk_screenshot(&app);
}

fn view(_app: &App, model: &Model, frame: Frame) {
    let mut encoder = frame.command_encoder();
    model
        .texture_reshaper
        .encode_render_pass(frame.texture_view(), &mut *encoder);
}

fn exit(app: &App, _model: Model) {
    println!("Waiting for PNG writing to complete...");
    maybe_mk_screenshot(&app);
    println!("Done!");
}

fn maybe_mk_screenshot(app: &App) {
    let window = app.main_window();
    let elapsed_frames = window.elapsed_frames();
    let path = capture_directory(app)
        .join(format!("{:04}", elapsed_frames).to_string())
        .with_extension("png");
    window.capture_frame(path);
}

fn capture_directory(app: &App) -> std::path::PathBuf {
    app.project_path()
        .expect("could not locate project_path")
        .join(app.exe_name().unwrap())
        .join("vijf")
}
