use wasm_bindgen::JsCast;
use web_sys::{
    Document, HtmlCanvasElement, WebGlProgram, WebGlRenderingContext, WebGlShader, Window,
};

static VERTEX_SHADER: &'static str = r#"
    attribute vec2 a_coords;
    attribute vec3 a_color;
    varying vec3 v_color;
    uniform float u_pointsize;
    uniform float u_width;
    uniform float u_height;
    void main() {
       float x = -1.0 + 2.0*(a_coords.x / u_width);
       float y = 1.0 - 2.0*(a_coords.y / u_height);
       gl_Position = vec4(x, y, 0.0, 1.0);
       v_color = a_color;
       gl_PointSize = u_pointsize;
    }
"#;

static FRAGMENT_SHADER: &'static str = r#"
    precision mediump float;
    varying vec3 v_color;
    void main() {
       float distanceFromCenter = distance( gl_PointCoord, vec2(0.5,0.5) );
       if ( distanceFromCenter >= 0.5 ) {
           discard;  // don't draw this pixel!
       }
       gl_FragColor = vec4(v_color, 1.0);
    }
"#;

pub fn window() -> Option<Window> {
    web_sys::window()
}

pub fn document() -> Option<Document> {
    window().and_then(|w| w.document())
}

pub fn canvas(id: &str) -> Option<HtmlCanvasElement> {
    document()
        .and_then(|d| d.get_element_by_id(id))
        .and_then(|el| el.dyn_into::<HtmlCanvasElement>().ok())
}

pub fn get_webgl_context_by_id(id: &str, width: u32, height: u32) -> Option<WebGlRenderingContext> {
    canvas(id)
        .and_then(|c| c.get_context("webgl").ok())
        .and_then(|c| c.unwrap().dyn_into::<WebGlRenderingContext>().ok())
        .and_then(|c| {
            c.viewport(0, 0, width as i32, height as i32);
            Some(c)
        })
}

pub fn get_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Option<WebGlShader> {
    let shader = context.create_shader(shader_type)?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);
    let compile_is_success = context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()?;
    if !compile_is_success {
        panic!("failed to compile.");
    }
    Some(shader)
}

pub fn create_program(context: &WebGlRenderingContext) -> Option<WebGlProgram> {
    let fragment_shader = get_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        FRAGMENT_SHADER,
    )?;
    let vertex_shader = get_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        VERTEX_SHADER,
    )?;
    let shader_program = context.create_program()?;

    context.attach_shader(&shader_program, &vertex_shader);
    context.attach_shader(&shader_program, &fragment_shader);
    context.link_program(&shader_program);

    let shader_is_created = context
        .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()?;
    if !shader_is_created {
        panic!("failed to create shader.");
    }
    context.use_program(Some(&shader_program));
    let vertex_position_attribute = context.get_attrib_location(&shader_program, "aVertexPosition");
    context.enable_vertex_attrib_array(vertex_position_attribute as u32);
    Some(shader_program)
}
