use rand::Rng;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext, WebGlShader, WebGlProgram};
extern crate js_sys;

fn init_webgl_context(canvas_id: &str) -> Result<WebGlRenderingContext, JsValue> {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id(canvas_id).unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl: WebGlRenderingContext = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()
        .unwrap();

    gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
    Ok(gl)
}

fn create_shader(
    gl: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Unable to create shader object"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if gl
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(JsValue::from_str(
            &gl.get_shader_info_log(&shader)
                .unwrap_or_else(|| "Unknown error creating shader".into()),
        ))
    }
}

fn setup_shaders(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
    let vertex_shader_source = "
        attribute vec3 coordinates;
        void main(void) {
            gl_Position = vec4(coordinates, 1.0);
        }
    ";

    let fragment_shader_source = "
        precision mediump float;
        uniform vec4 fragColor;
        void main(void) {
            gl_FragColor = fragColor;
        }
    ";

    let vertex_shader = create_shader(
        &gl,
        WebGlRenderingContext::VERTEX_SHADER,
        vertex_shader_source,
    )?;
    let fragment_shader = create_shader(
        &gl,
        WebGlRenderingContext::FRAGMENT_SHADER,
        fragment_shader_source,
    )?;

    let shader_program = gl.create_program().unwrap();
    gl.attach_shader(&shader_program, &vertex_shader);
    gl.attach_shader(&shader_program, &fragment_shader);
    gl.link_program(&shader_program);

    if gl
        .get_program_parameter(&shader_program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        gl.use_program(Some(&shader_program));
        Ok(shader_program)
    } else {
        Err(JsValue::from_str(
            &gl.get_program_info_log(&shader_program)
                .unwrap_or_else(|| "Unknown error linking program".into()),
        ))
    }
}

fn setup_vertices(gl: &WebGlRenderingContext, vertices: &[f32], shader_program: &WebGlProgram) {
    let vertices_array = unsafe { js_sys::Float32Array::view(vertices) };
    let vertex_buffer = gl.create_buffer().unwrap();

    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vertices_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    let coordinates_location = gl.get_attrib_location(&shader_program, "coordinates");
    gl.vertex_attrib_pointer_with_i32(
        coordinates_location as u32,
        3,
        WebGlRenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(coordinates_location as u32);
}

fn draw_triangles_with_color(
    gl: &WebGlRenderingContext,
    shader_program: &WebGlProgram,
    vertices: &[f32],
    color: Vec<f32>,
) {
    setup_vertices(&gl, &vertices, &shader_program);

    let color_location = gl
        .get_uniform_location(&shader_program, "fragColor")
        .unwrap();
    gl.uniform4fv_with_f32_array(Some(&color_location), &color);

    gl.draw_arrays(
        WebGlRenderingContext::TRIANGLES,
        0,
        (vertices.len() / 3) as i32,
    );
}

#[wasm_bindgen]
pub fn draw_triangle(
    canvas_id: &str,
    selected_color: Option<Vec<f32>>,
) -> Result<WebGlRenderingContext, JsValue> {
    let gl: WebGlRenderingContext = init_webgl_context(canvas_id)?;
    let shader_program: WebGlProgram = setup_shaders(&gl)?;

    let vertices: [f32; 9] = [
        0.0, 0.5, 0.0,  // top
        -0.5, -0.5, 0.0, // bottom left
        0.5, -0.5, 0.0,  // bottom right
    ];

    let color = selected_color.unwrap_or_else(|| vec![1.0, 0.0, 0.0, 1.0]);
    draw_triangles_with_color(&gl, &shader_program, &vertices, color);

    Ok(gl)
}

#[wasm_bindgen]
pub fn draw_triangles(
    canvas_id: &str,
    selected_color: Option<Vec<f32>>,
) -> Result<WebGlRenderingContext, JsValue> {
    let gl: WebGlRenderingContext = init_webgl_context(canvas_id)?;
    let shader_program: WebGlProgram = setup_shaders(&gl)?;

    // Define 100 triangles (300 vertices)
    let vertices: Vec<f32> = (0..100)
        .flat_map(|_| {
            let mut rng = rand::thread_rng();

            // Generate random offsets for each triangle's position
            let x_offset = rng.gen_range(-0.5..0.5); // Random x position within canvas range
            let y_offset = rng.gen_range(-0.5..0.5); // Random y position within canvas range

            vec![
                0.0 + x_offset, 0.05 + y_offset, 0.0,     // top
                -0.05 + x_offset, -0.05 + y_offset, 0.0,   // bottom left
                0.05 + x_offset, -0.05 + y_offset, 0.0,    // bottom right
            ]
        })
        .collect();

    let color = selected_color.unwrap_or_else(|| vec![1.0, 0.0, 0.0, 1.0]);
    draw_triangles_with_color(&gl, &shader_program, &vertices, color);

    Ok(gl)
}
