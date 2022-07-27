mod dom_utils;
mod utils;

use rand::Rng;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console::log_1;
use web_sys::{WebGlBuffer, WebGlRenderingContext, WebGlUniformLocation};

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

fn log(s: &String) {
    log_1(&JsValue::from(s));
}

#[wasm_bindgen]
pub fn output_log(s: &str) {
    log(&format!("Hello {}", s));
}

#[derive(Clone, Copy, Debug)]
pub struct Disk {
    x: f64,   // x-coordinate
    y: f64,   // y-coordinate
    cos: f64, // moving velocity-cos
    sin: f64, // moving velocity-sin
}

impl Disk {
    fn new(x: f64, y: f64, cos: f64, sin: f64) -> Self {
        Self { x, y, cos, sin }
    }
}

/**
 * ディスクのベクタを初期化する
 */
fn init_disks(disk_num: u32, bound_x: u32, bound_y: u32) -> Vec<Box<Disk>> {
    let mut disks_buffer: Vec<Box<Disk>> = Vec::with_capacity(disk_num as usize);

    let mut rand = rand::thread_rng();
    for i in 0..disk_num {
        let random = rand.gen_range(0., 1.);
        let velocity = 1. + 3. * random;
        let angle = std::f64::consts::PI * (0.1 * (i as f64) * random);
        let disk = Box::new(Disk::new(
            (bound_x as f64) / 2.,
            (bound_y as f64) / 2.,
            velocity * angle.cos(),
            velocity * angle.sin(),
        ));
        disks_buffer.push(disk);
    }
    disks_buffer
}

#[derive(Debug)]
#[wasm_bindgen]
pub struct Screen {
    gl: WebGlRenderingContext,
    uniform_point_size: WebGlUniformLocation,
    buffer_coords: WebGlBuffer,

    attrib_coords: i32,
    attrib_color: i32,
    width: u32,
    height: u32,
    disk_num: u32,
    disk_size: f64,

    disks: Vec<Box<Disk>>,
}

#[wasm_bindgen]
impl Screen {
    /**
     * 1イテレーションごとの座標計算
     */
    fn on_animation_frame(&mut self) -> () {
        let size = self.disk_size as f64;
        let width = self.width as f64;
        let height = self.height as f64;
        for disk in self.disks.iter_mut() {
            disk.x += disk.cos;
            disk.y += disk.sin;
            if disk.x - size < 0. {
                disk.x = size - (disk.x - size);
                disk.cos = disk.cos.abs();
            } else if disk.x + size > width {
                disk.x = width - (disk.x + size - width) - size;
                disk.cos = -disk.cos.abs();
            }
            if disk.y - size < 0. {
                disk.y = size - (disk.y - size);
                disk.sin = disk.sin.abs();
            } else if disk.y + size > height {
                disk.y = height - (disk.y + size - height) - size;
                disk.sin = -disk.sin.abs();
            }
        }
    }

    /**
     * 各アニメーションフレームごとの処理
     */
    pub fn do_frame(&mut self) -> () {
        self.on_animation_frame();
        self.draw();
    }

    /**
     * レンダリング処理
     */
    fn draw(&self) -> () {
        self.gl.clear_color(0., 0., 0., 1.);
        self.gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

        self.gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&self.buffer_coords),
        );
        let buff_vec = self
            .disks
            .iter()
            .flat_map(|d| {
                let cloned = d.as_ref().clone();
                vec![cloned.x as f32, cloned.y as f32]
            })
            .collect::<Vec<f32>>();
        unsafe {
            self.gl.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &js_sys::Float32Array::view(buff_vec.as_slice()), //
                WebGlRenderingContext::STREAM_DRAW,
            )
        }
        self.gl.vertex_attrib_pointer_with_f64(
            self.attrib_coords as u32,
            2,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0.,
        );
        self.gl
            .enable_vertex_attrib_array(self.attrib_coords as u32);

        self.gl.enable_vertex_attrib_array(self.attrib_color as u32);
        self.gl
            .vertex_attrib3f(self.attrib_color as u32, 1., 0., 0.);

        self.gl
            .uniform1f(Some(&self.uniform_point_size), self.disk_size as f32);

        self.gl
            .draw_arrays(WebGlRenderingContext::POINTS, 0, self.disk_num as i32);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Options {
    pub canvas_id: String,
    pub disk_num: Option<u32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub disk_size: Option<f64>,
    pub collision: Option<bool>,
}

/**
 * WebGLContextの初期化処理
 */
#[wasm_bindgen]
pub fn init_gl(option_input: JsValue) -> Screen {
    let options: Options = option_input.into_serde().unwrap();
    let canvas_id = options.canvas_id;
    let width = options.width.unwrap_or(500);
    let height = options.height.unwrap_or(500);
    let disk_num = options.disk_num.unwrap_or(100);
    let disk_size = options.disk_size.unwrap_or(32.);

    let context = dom_utils::get_webgl_context_by_id(canvas_id.as_str(), width, height).unwrap();
    let program = dom_utils::create_program(&context).unwrap();
    context.use_program(Some(&program));

    let disks = init_disks(disk_num, width, height);
    let attrib_coords = context.get_attrib_location(&program, "a_coords");
    let buffer_coords = context.create_buffer().unwrap();
    let attrib_color = context.get_attrib_location(&program, "a_color");
    let buffer_color = context.create_buffer().unwrap();
    let uniform_height = context.get_uniform_location(&program, "u_height").unwrap();
    let uniform_width = context.get_uniform_location(&program, "u_width").unwrap();
    let uniform_point_size = context
        .get_uniform_location(&program, "u_pointsize")
        .unwrap();
    context.uniform1f(Some(&uniform_height), width as f32);
    context.uniform1f(Some(&uniform_width), height as f32);

    // ランダム生成した浮動小数点値を1diskあたりに3値(rgb)割り当てる
    let mut random = rand::thread_rng();
    let color_buffer_array = (0..(disk_num * 3))
        .into_iter()
        .map(|_| random.gen_range(0., 1.) as f32)
        .collect::<Vec<f32>>();
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer_color));
    unsafe {
        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &js_sys::Float32Array::view(color_buffer_array.as_slice()), //
            WebGlRenderingContext::STREAM_DRAW,
        )
    }
    context.vertex_attrib_pointer_with_f64(
        attrib_color as u32,
        3,
        WebGlRenderingContext::FLOAT,
        false,
        0,
        0.,
    );

    Screen {
        gl: context,
        disks,
        disk_size,
        disk_num,
        width,
        height,
        uniform_point_size,
        attrib_coords,
        buffer_coords,
        attrib_color,
    }
}
