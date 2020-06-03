#[path = "../vendor/wgpu-rs/examples/framework.rs"]
mod framework;

use bytemuck::{Pod, Zeroable};

use blend::{Blend, Instance};
use libflate::gzip::Decoder;
use std::{
    env,
    fs::File,
    io::{self, BufReader, BufWriter, Read, Write},
    path::{self, PathBuf},
};

type BlenderVertex = (
    [f32; 3],   //  頂点
    [f32; 3],   //  法線
    [f32; 2]    //  UV
);
type BlenderFace = [BlenderVertex; 3];

#[derive(Debug)]
struct BlenderMesh {
    faces: Vec<BlenderFace>,
}

#[derive(Debug)]
struct Object {
    name: String,
    location: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
    mesh: BlenderMesh,
}

// This is only valid for meshes with triangular faces
fn instance_to_mesh(mesh: Instance) -> Option<BlenderMesh> {
    if !mesh.is_valid("mpoly")
        || !mesh.is_valid("mloop")
        || !mesh.is_valid("mloopuv")
        || !mesh.is_valid("mvert")
    {
        return None;
    }

    let faces = mesh.get_iter("mpoly").collect::<Vec<_>>();
    let loops = mesh.get_iter("mloop").collect::<Vec<_>>();
    let uvs = mesh.get_iter("mloopuv").collect::<Vec<_>>();
    let verts = mesh.get_iter("mvert").collect::<Vec<_>>();

    let mut index_count = 0;
    let mut face_indice_count = 0;
    for face in &faces {
        let len = face.get_i32("totloop");
        let mut indexi = 1;

        while indexi < len {
            face_indice_count += 3;
            indexi += 2;
        }
    }

    let mut uv_buffer = vec![0f32; face_indice_count * 2];
    let mut normal_buffer = vec![0f32; face_indice_count * 3];
    let mut verts_array_buff = vec![0f32; face_indice_count * 3];

    for face in &faces {
        let len = face.get_i32("totloop");
        let start = face.get_i32("loopstart");
        let mut indexi = 1;

        while indexi < len {
            let mut index;

            for l in 0..3 {
                if (indexi - 1) + l < len {
                    index = start + (indexi - 1) + l;
                } else {
                    index = start;
                }

                let v = loops[index as usize].get_i32("v");
                let vert = &verts[v as usize];

                let co = vert.get_f32_vec("co");
                verts_array_buff[index_count * 3] = co[0];
                verts_array_buff[index_count * 3 + 1] = co[1];
                verts_array_buff[index_count * 3 + 2] = co[2];

                //Normals are compressed into 16 bit integers
                let no = vert.get_i16_vec("no");
                normal_buffer[index_count * 3] = f32::from(no[0]) / 32767.0;
                normal_buffer[index_count * 3 + 1] = f32::from(no[1]) / 32767.0;
                normal_buffer[index_count * 3 + 2] = f32::from(no[2]) / 32767.0;

                let uv = uvs[index as usize].get_f32_vec("uv");
                let uv_x = uv[0];
                let uv_y = uv[1];
                uv_buffer[index_count * 2] = uv_x;
                uv_buffer[index_count * 2 + 1] = uv_y;

                index_count += 1;
            }

            indexi += 2;
        }
    }

    let faces: Vec<_> = (&verts_array_buff[..])
        .chunks(3)
        .enumerate()
        .map(|(i, pos)| {
            (
                [pos[0], pos[1], pos[2]],
                [
                    normal_buffer[i * 3],
                    normal_buffer[i * 3 + 1],
                    normal_buffer[i * 3 + 2],
                ],
                [uv_buffer[i * 2], uv_buffer[i * 2 + 1]],
            )
        })
        .collect::<Vec<BlenderVertex>>();

    let faces: Vec<_> = faces.chunks(3).map(|f| [f[0], f[1], f[2]]).collect();

    Some(BlenderMesh { faces })
}

fn read_mesh(file_name: impl AsRef<str>) -> Vec<Object> {
    let file_name = file_name.as_ref();
    let base_path = path::PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("could not find cargo manifest dir"),
    );
    let blend_path = base_path.join(format!("{}", file_name));
    let blend = Blend::from_path(blend_path);

    let mut objects = Vec::new();

    for obj in blend.get_by_code(*b"OB") {
        if obj.is_valid("data") && obj.get("data").code()[0..=1] == *b"ME" {
            let loc = obj.get_f32_vec("loc");
            let rot = obj.get_f32_vec("rot");
            let size = obj.get_f32_vec("size");
            let data = obj.get("data");

            if let Some(mesh) = instance_to_mesh(data) {
                objects.push(Object {
                    name: obj.get("id").get_string("name"),
                    location: [loc[0], loc[1], loc[2]],
                    rotation: [rot[0], rot[1], rot[2]],
                    scale: [size[0], size[1], size[2]],
                    mesh,
                });
            }
        }
    }

    // println!("{:#?}", objects);
    return objects;
}

fn print_blend(file_name: impl AsRef<str>, output_name: impl AsRef<str>) -> Result<(), io::Error> {
    let file_name = file_name.as_ref();
    let output_name = output_name.as_ref();
    let base_path = path::PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("could not find cargo manifest dir"),
    );

    let blend_path = base_path.join(format!("{}", file_name));
    let output_path = base_path.join(format!("{}", output_name));

    println!("{}", blend_path.display());
    let mut file = File::open(blend_path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;

    if data[0..7] != *b"BLENDER" {
        let mut decoder = Decoder::new(&data[..])?;
        let mut gzip_data = Vec::new();
        decoder.read_to_end(&mut gzip_data)?;

        data = gzip_data;
    }

    let blend = Blend::new(&data[..]);
    let mut output_path_without_file = PathBuf::from(&output_path);
    output_path_without_file.pop();
    std::fs::create_dir_all(&output_path_without_file)?;
    let mut buffer = BufWriter::new(File::create(output_path)?);

    for o in blend.get_all_root_blocks() {
        write!(buffer, "{}", o)?;
    }

    writeln!(buffer)?;
    buffer.flush()?;

    println!("done: {}", file_name);

    Ok(())
}

fn print_name_position(file_name: impl AsRef<str>) {
    let file_name = file_name.as_ref();
    let base_path = path::PathBuf::from(
        env::var_os("CARGO_MANIFEST_DIR").expect("could not find cargo manifest dir"),
    );
    let blend_path = base_path.join(format!("{}", file_name));
    let blend = Blend::from_path(blend_path);

    for obj in blend.get_by_code(*b"OB") {
        let loc = obj.get_f32_vec("loc");
        let name = obj.get("id").get_string("name");

        println!("\"{}\" at {:?}", name, loc);
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct Vertex {
    _pos: [f32; 4],
    _normal: [f32; 4],
    _tex_coord: [f32; 2],
}

unsafe impl Pod for Vertex {}
unsafe impl Zeroable for Vertex {}

fn vertex(pos: [f32; 3], normal: [f32; 3], tc: [f32; 2]) -> Vertex {
    Vertex {
        _pos: [pos[0] as f32, pos[1] as f32, pos[2] as f32, 1.0],
        _normal: [normal[0] as f32, normal[1] as f32, normal[2] as f32, 1.0],
        _tex_coord: [tc[0] as f32, tc[1] as f32],
    }
}

fn create_vertices() -> (Vec<Vertex>, Vec<u16>) {
    let filepath = "assets/model/box/box.blend";
    let objects = read_mesh(filepath);

    let mut index: u16 = 0;
    let mut vertex_data: Vec<Vertex> = Vec::new();
    let mut index_data: Vec<u16> = Vec::new();

    for object in objects {
        for face in object.mesh.faces {
            let position = face[0].0;
            let normal = face[0].1;
            let uvs = face[0].2;
            vertex_data.push(vertex(position, normal, uvs));
            index_data.push(index);

            let position = face[1].0;
            let normal = face[1].1;
            let uvs = face[1].2;
            vertex_data.push(vertex(position, normal, uvs));
            index_data.push(index+1);

            let position = face[2].0;
            let normal = face[2].1;
            let uvs = face[2].2;
            vertex_data.push(vertex(position, normal, uvs));
            index_data.push(index+2);

            index += 3;
        }
    }

    (vertex_data.to_vec(), index_data.to_vec())
}

fn create_texels(size: usize) -> Vec<u8> {
    use std::iter;

    (0..size * size)
        .flat_map(|id| {
            // get high five for recognizing this ;)
            let cx = 3.0 * (id % size) as f32 / (size - 1) as f32 - 2.0;
            let cy = 2.0 * (id / size) as f32 / (size - 1) as f32 - 1.0;
            let (mut x, mut y, mut count) = (cx, cy, 0);
            while count < 0xFF && x * x + y * y < 4.0 {
                let old_x = x;
                x = x * x - y * y + cx;
                y = 2.0 * old_x * y + cy;
                count += 1;
            }
            iter::once(0xFF - (count * 5) as u8)
                .chain(iter::once(0xFF - (count * 15) as u8))
                .chain(iter::once(0xFF - (count * 50) as u8))
                .chain(iter::once(1))
        })
        .collect()
}

struct Example {
    vertex_buf: wgpu::Buffer,
    index_buf: wgpu::Buffer,
    index_count: usize,
    bind_group: wgpu::BindGroup,
    uniform_buf: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
}

impl Example {
    fn generate_matrix(aspect_ratio: f32) -> cgmath::Matrix4<f32> {
        let mx_projection = cgmath::perspective(cgmath::Deg(45f32), aspect_ratio, 1.0, 10.0);
        let mx_view = cgmath::Matrix4::look_at(
            cgmath::Point3::new(1.5f32, -5.0, 3.0),
            cgmath::Point3::new(0f32, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );
        let mx_correction = framework::OPENGL_TO_WGPU_MATRIX;
        mx_correction * mx_projection * mx_view
    }
}

impl framework::Example for Example {
    fn init(
        sc_desc: &wgpu::SwapChainDescriptor,
        device: &wgpu::Device,
    ) -> (Self, Option<wgpu::CommandBuffer>) {
        use std::mem;

        let mut init_encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });

        // Create the vertex and index buffers
        let vertex_size = mem::size_of::<Vertex>();
        let (vertex_data, index_data) = create_vertices();

        let vertex_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(&vertex_data),
            wgpu::BufferUsage::VERTEX,
        );

        let index_buf = device
            .create_buffer_with_data(bytemuck::cast_slice(&index_data), wgpu::BufferUsage::INDEX);

        // Create pipeline layout
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            bindings: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStage::VERTEX,
                    ty: wgpu::BindingType::UniformBuffer { dynamic: false },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::SampledTexture {
                        multisampled: false,
                        component_type: wgpu::TextureComponentType::Float,
                        dimension: wgpu::TextureViewDimension::D2,
                    },
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStage::FRAGMENT,
                    ty: wgpu::BindingType::Sampler { comparison: false },
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
        });

        // Create the texture
        let size = 256u32;
        let texels = create_texels(size as usize);
        let texture_extent = wgpu::Extent3d {
            width: size,
            height: size,
            depth: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: texture_extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsage::SAMPLED | wgpu::TextureUsage::COPY_DST,
        });
        let texture_view = texture.create_default_view();
        let temp_buf =
            device.create_buffer_with_data(texels.as_slice(), wgpu::BufferUsage::COPY_SRC);
        init_encoder.copy_buffer_to_texture(
            wgpu::BufferCopyView {
                buffer: &temp_buf,
                offset: 0,
                bytes_per_row: 4 * size,
                rows_per_image: 0,
            },
            wgpu::TextureCopyView {
                texture: &texture,
                mip_level: 0,
                array_layer: 0,
                origin: wgpu::Origin3d::ZERO,
            },
            texture_extent,
        );

        // Create other resources
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 100.0,
            compare: wgpu::CompareFunction::Undefined,
        });
        let mx_total = Self::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        let uniform_buf = device.create_buffer_with_data(
            bytemuck::cast_slice(mx_ref),
            wgpu::BufferUsage::UNIFORM | wgpu::BufferUsage::COPY_DST,
        );

        // Create bind group
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &bind_group_layout,
            bindings: &[
                wgpu::Binding {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(uniform_buf.slice(..)),
                },
                wgpu::Binding {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::Binding {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: None,
        });

        // Create the render pipeline
        let vs_bytes = include_bytes!("../vendor/wgpu-rs/examples/cube/shader.vert.spv");
        let fs_bytes = include_bytes!("../vendor/wgpu-rs/examples/cube/shader.frag.spv");
        let vs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&vs_bytes[..])).unwrap());
        let fs_module = device
            .create_shader_module(&wgpu::read_spirv(std::io::Cursor::new(&fs_bytes[..])).unwrap());

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            layout: &pipeline_layout,
            vertex_stage: wgpu::ProgrammableStageDescriptor {
                module: &vs_module,
                entry_point: "main",
            },
            fragment_stage: Some(wgpu::ProgrammableStageDescriptor {
                module: &fs_module,
                entry_point: "main",
            }),
            rasterization_state: Some(wgpu::RasterizationStateDescriptor {
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: wgpu::CullMode::Back,
                depth_bias: 0,
                depth_bias_slope_scale: 0.0,
                depth_bias_clamp: 0.0,
            }),
            primitive_topology: wgpu::PrimitiveTopology::TriangleList,
            color_states: &[wgpu::ColorStateDescriptor {
                format: sc_desc.format,
                color_blend: wgpu::BlendDescriptor::REPLACE,
                alpha_blend: wgpu::BlendDescriptor::REPLACE,
                write_mask: wgpu::ColorWrite::ALL,
            }],
            depth_stencil_state: None,
            vertex_state: wgpu::VertexStateDescriptor {
                index_format: wgpu::IndexFormat::Uint16,
                vertex_buffers: &[wgpu::VertexBufferDescriptor {
                    stride: vertex_size as wgpu::BufferAddress,
                    step_mode: wgpu::InputStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float4,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttributeDescriptor {
                            format: wgpu::VertexFormat::Float2,
                            offset: 4 * 4,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            sample_count: 1,
            sample_mask: !0,
            alpha_to_coverage_enabled: false,
        });

        // Done
        let this = Example {
            vertex_buf,
            index_buf,
            index_count: index_data.len(),
            bind_group,
            uniform_buf,
            pipeline,
        };
        (this, Some(init_encoder.finish()))
    }

    fn update(&mut self, _event: winit::event::WindowEvent) {
        //empty
    }

    fn resize(
        &mut self,
        sc_desc: &wgpu::SwapChainDescriptor,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) {
        let mx_total = Self::generate_matrix(sc_desc.width as f32 / sc_desc.height as f32);
        let mx_ref: &[f32; 16] = mx_total.as_ref();
        queue.write_buffer(bytemuck::cast_slice(mx_ref), &self.uniform_buf, 0);
    }

    fn render(
        &mut self,
        frame: &wgpu::SwapChainOutput,
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
    ) -> wgpu::CommandBuffer {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &frame.view,
                    resolve_target: None,
                    load_op: wgpu::LoadOp::Clear,
                    store_op: wgpu::StoreOp::Store,
                    clear_color: wgpu::Color {
                        r: 0.1,
                        g: 0.2,
                        b: 0.3,
                        a: 1.0,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&self.pipeline);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_index_buffer(self.index_buf.slice(..));
            rpass.set_vertex_buffer(0, self.vertex_buf.slice(..));
            rpass.draw_indexed(0..self.index_count as u32, 0, 0..1);
        }

        encoder.finish()
    }
}

fn main() {
    framework::run::<Example>("cube");
}
