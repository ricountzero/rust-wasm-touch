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
        .dyn_into::<WebGlRenderingContext>()?;

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
        let log = gl.get_shader_info_log(&shader).unwrap_or_else(|| "Unknown error creating shader".into());
        web_sys::console::log_1(&JsValue::from_str(&log));
        Err(JsValue::from_str(&log))
    }
}

fn setup_shaders(gl: &WebGlRenderingContext) -> Result<WebGlProgram, JsValue> {
    let vertex_shader_source = "
        attribute vec3 coordinates;
        attribute vec3 normal;
        uniform mat4 modelViewMatrix;
        uniform mat4 projectionMatrix;
        varying vec3 vNormal;
        varying vec3 vPosition;
        void main(void) {
            vNormal = normal;
            vPosition = vec3(modelViewMatrix * vec4(coordinates, 1.0));
            gl_Position = projectionMatrix * modelViewMatrix * vec4(coordinates, 1.0);
        }
    ";

    let fragment_shader_source = "
        precision mediump float;
        uniform vec4 fragColor;
        uniform vec3 lightPosition;
        varying vec3 vNormal;
        varying vec3 vPosition;
        void main(void) {
            vec3 lightDir = normalize(lightPosition - vPosition);
            float diff = max(dot(vNormal, lightDir), 0.0);
            vec3 diffuse = diff * vec3(fragColor);
            gl_FragColor = vec4(diffuse, fragColor.a);
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
        let log = gl.get_program_info_log(&shader_program).unwrap_or_else(|| "Unknown error linking program".into());
        web_sys::console::log_1(&JsValue::from_str(&log));
        Err(JsValue::from_str(&log))
    }
}

fn generate_sphere(radius: f32, lat_bands: u32, long_bands: u32) -> (Vec<f32>, Vec<f32>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=lat_bands {
        let theta = lat as f32 * std::f32::consts::PI / lat_bands as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for long in 0..=long_bands {
            let phi = long as f32 * 2.0 * std::f32::consts::PI / long_bands as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = cos_phi * sin_theta;
            let y = cos_theta;
            let z = sin_phi * sin_theta;

            normals.push(x);
            normals.push(y);
            normals.push(z);
            vertices.push(radius * x);
            vertices.push(radius * y);
            vertices.push(radius * z);
        }
    }

    for lat in 0..lat_bands {
        for long in 0..long_bands {
            let first = (lat * (long_bands + 1) + long) as u16;
            let second = first + long_bands as u16 + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);

            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }

    (vertices, normals, indices)
}

fn setup_sphere_buffers(
    gl: &WebGlRenderingContext,
    vertices: &[f32],
    normals: &[f32],
    indices: &[u16],
    shader_program: &WebGlProgram,
) {
    let vertices_array = unsafe { js_sys::Float32Array::view(vertices) };
    let normals_array = unsafe { js_sys::Float32Array::view(normals) };
    let indices_array = unsafe { js_sys::Uint16Array::view(indices) };

    let vertex_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&vertex_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &vertices_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    let normal_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&normal_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &normals_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    let index_buffer = gl.create_buffer().unwrap();
    gl.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        &indices_array,
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

    let normal_location = gl.get_attrib_location(&shader_program, "normal");
    gl.vertex_attrib_pointer_with_i32(
        normal_location as u32,
        3,
        WebGlRenderingContext::FLOAT,
        false,
        0,
        0,
    );
    gl.enable_vertex_attrib_array(normal_location as u32);
}

fn get_projection_matrix(aspect: f32) -> [f32; 16] {
    let fov: f32 = 45.0;
    let near = 0.1;
    let far = 100.0;
    let f = 1.0 / (fov.to_radians() / 2.0).tan();
    [
        f / aspect, 0.0, 0.0, 0.0,
        0.0, f, 0.0, 0.0,
        0.0, 0.0, (far + near) / (near - far), -1.0,
        0.0, 0.0, (2.0 * far * near) / (near - far), 0.0,
    ]
}

fn get_model_view_matrix() -> [f32; 16] {
    [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, -6.0, 1.0,
    ]
}

#[wasm_bindgen]
pub fn draw_sphere(
    canvas_id: &str,
    selected_color: Option<Vec<f32>>,
) -> Result<WebGlRenderingContext, JsValue> {
    let gl: WebGlRenderingContext = init_webgl_context(canvas_id)?;
    let shader_program: WebGlProgram = setup_shaders(&gl)?;

    let (vertices, normals, indices) = generate_sphere(1.0, 30, 30);

    setup_sphere_buffers(&gl, &vertices, &normals, &indices, &shader_program);

    let color = selected_color.unwrap_or_else(|| vec![1.0, 0.0, 0.0, 1.0]);
    let color_location = gl
        .get_uniform_location(&shader_program, "fragColor")
        .unwrap();
    gl.uniform4fv_with_f32_array(Some(&color_location), &color);

    let light_position = vec![5.0, 5.0, 5.0];
    let light_position_location = gl
        .get_uniform_location(&shader_program, "lightPosition")
        .unwrap();
    gl.uniform3fv_with_f32_array(Some(&light_position_location), &light_position);

    let aspect = gl.drawing_buffer_width() as f32 / gl.drawing_buffer_height() as f32;
    let projection_matrix = get_projection_matrix(aspect);
    let model_view_matrix = get_model_view_matrix();

    let projection_matrix_location = gl
        .get_uniform_location(&shader_program, "projectionMatrix")
        .unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&projection_matrix_location), false, &projection_matrix);

    let model_view_matrix_location = gl
        .get_uniform_location(&shader_program, "modelViewMatrix")
        .unwrap();
    gl.uniform_matrix4fv_with_f32_array(Some(&model_view_matrix_location), false, &model_view_matrix);

    gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);
    gl.draw_elements_with_i32(
        WebGlRenderingContext::TRIANGLES,
        indices.len() as i32,
        WebGlRenderingContext::UNSIGNED_SHORT,
        0,
    );

    web_sys::console::log_1(&JsValue::from_str("Sphere drawn successfully"));

    Ok(gl)
}