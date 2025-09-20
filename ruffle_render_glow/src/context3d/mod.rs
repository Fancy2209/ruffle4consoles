use glow::HasContext;
use ruffle_render::{
    backend::{
        BufferUsage, Context3D, Context3DBlendFactor, Context3DCommand, Context3DCompareMode,
        Context3DProfile, Context3DTextureFormat, Context3DVertexBufferFormat, IndexBuffer,
        ProgramType, VertexBuffer,
    },
    bitmap::BitmapHandle,
};

use ruffle_render::error::Error;
use std::cell::Cell;
use std::{any::Any, rc::Rc, sync::Arc};
use swf::{Rectangle, Twips};

use crate::{as_registry_data, RegistryData};

pub const COLOR:   u32 = 1 << 0;
pub const DEPTH:   u32 = 1 << 1;
pub const STENCIL: u32 = 1 << 2;
pub const ALL:     u32 = COLOR | DEPTH | STENCIL;

struct GlowContext3D {
    gl: Arc<glow::Context>,
    profile: Context3DProfile,

    back_buffer: Option<glow::Framebuffer>,
    back_buffer_depth_stencil: Option<glow::Texture>,

    front_buffer_texture: Option<glow::Texture>,

    back_buffer_raw_texture_handle: BitmapHandle,
    front_buffer_raw_texture_handle: BitmapHandle,

    // After a call to 'present()', the Context3D API requires a call to 'clear'
    // before any new calls to 'drawTriangles'. This tracks whether we've
    // seen a `Context3DCommand::Clear` so far. Note that this is separate from
    // `clear_color`, which may be `None` even if we've seen a `Clear` command.
    seen_clear_command: bool,

    scissor_rectangle: Option<Rectangle<Twips>>,
    curr_vao: Option<glow::VertexArray>,
}

impl GlowContext3D {
    fn new(gl: Arc<glow::Context>, profile: Context3DProfile) -> Self {
        let make_dummy_handle = || {
            let dummy_texture = unsafe { gl.create_texture().unwrap() };

            BitmapHandle(Arc::new(RegistryData {
                gl: gl.clone(),
                width: 1,
                height: 1,
                texture: dummy_texture,
            }))
        };

        let back_buffer_raw_texture_handle = make_dummy_handle();
        let front_buffer_raw_texture_handle = make_dummy_handle();

        Self {
            gl: gl.clone(),
            profile,

            back_buffer: None,
            back_buffer_depth_stencil: None,
            //back_buffer_texture: None,
            front_buffer_texture: None,
            back_buffer_raw_texture_handle,
            front_buffer_raw_texture_handle,
            seen_clear_command: false,
            scissor_rectangle: None,
            curr_vao: None,
        }
    }
}

pub struct IndexBufferWrapper {
    pub buffer: glow::Buffer,
    pub usage: u32,
    pub num_indices: u32,
}

#[derive(Debug)]
pub struct VertexBufferWrapper {
    pub buffer: glow::Buffer,
    pub usage: u32,
    pub num_vertices: u32,
    //pub data_32_per_vertex: u8,
}

#[derive(Debug)]
pub struct TextureWrapper {
    texture: glow::Texture,
    width: u32,
    height: u32,
    internal_format: u32,
    format: u32,
    target: u32,
}

impl IndexBuffer for IndexBufferWrapper {}
impl VertexBuffer for VertexBufferWrapper {}
impl ruffle_render::backend::Texture for TextureWrapper {
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
}

impl Context3D for GlowContext3D {
    fn profile(&self) -> Context3DProfile {
        self.profile
    }
    // The BitmapHandle for the texture we're rendering to
    fn bitmap_handle(&self) -> BitmapHandle {
        self.front_buffer_raw_texture_handle.clone()
    }
    // Whether or not we should actually render the texture
    // as part of stage rendering
    fn should_render(&self) -> bool {
        self.seen_clear_command
    }

    // Get a 'disposed' handle - this is what we store in all IndexBuffer3D
    // objects after dispose() has been called.
    fn disposed_index_buffer_handle(&self) -> Rc<dyn IndexBuffer> {
        todo!()
    }

    // Get a 'disposed' handle - this is what we store in all VertexBuffer3D
    // objects after dispose() has been called.
    fn disposed_vertex_buffer_handle(&self) -> Rc<dyn VertexBuffer> {
        todo!()
    }

    fn create_index_buffer(
        &mut self,
        usage: BufferUsage,
        num_indices: u32,
    ) -> Box<dyn IndexBuffer> {
        Box::new(IndexBufferWrapper {
            buffer: unsafe { self.gl.create_buffer().unwrap() },
            usage: match usage {
                BufferUsage::DynamicDraw => glow::DYNAMIC_DRAW,
                BufferUsage::StaticDraw => glow::STATIC_DRAW,
            },
            num_indices,
        })
    }

    fn create_vertex_buffer(
        &mut self,
        usage: BufferUsage,
        num_vertices: u32,
        _data_32_per_vertex: u8,
    ) -> Rc<dyn VertexBuffer> {
        Rc::new(VertexBufferWrapper {
            buffer: unsafe { self.gl.create_buffer().unwrap() },
            usage: match usage {
                BufferUsage::DynamicDraw => glow::DYNAMIC_DRAW,
                BufferUsage::StaticDraw => glow::STATIC_DRAW,
            },
            num_vertices,
            //data_32_per_vertex,
        })
    }

    fn create_texture(
        &mut self,
        width: u32,
        height: u32,
        format: Context3DTextureFormat,
        _optimize_for_render_to_texture: bool,
        streaming_levels: u32,
    ) -> Result<Rc<dyn ruffle_render::backend::Texture>, Error> {
        let (internal_format, format, target) =
            context3d_texture_format_to_gl_format_and_target(format);
        if streaming_levels != 0 {
            return Err(Error::Unimplemented(
                format!("streamingLevels={streaming_levels}").into(),
            ));
        }
        Ok(Rc::new(TextureWrapper {
            texture: unsafe { self.gl.create_texture().unwrap() },
            width,
            height,
            internal_format,
            format,
            target,
        }))
    }

    fn create_cube_texture(
        &mut self,
        size: u32,
        format: Context3DTextureFormat,
        _optimize_for_render_to_texture: bool,
        streaming_levels: u32,
    ) -> Result<Rc<dyn ruffle_render::backend::Texture>, Error> {
        let (internal_format, format, target) =
            context3d_texture_format_to_gl_format_and_target(format);
        if streaming_levels != 0 {
            return Err(Error::Unimplemented(
                format!("streamingLevels={streaming_levels}").into(),
            ));
        }
        Ok(Rc::new(TextureWrapper {
            texture: unsafe { self.gl.create_texture().unwrap() },
            width: size,
            height: size,
            internal_format,
            format,
            target,
        }))
    }

    fn process_command(&mut self, command: Context3DCommand<'_>) {
        unsafe {
            match command {
                Context3DCommand::Clear {
                    red,
                    green,
                    blue,
                    alpha,
                    depth,
                    stencil,
                    mask,
                } => {
                let mut gl_mask:u32 = 0;
                    if mask & COLOR != 0 {
                        self.gl.clear_color(red as f32, green as f32, blue as f32, alpha as f32);
                        gl_mask |= glow::COLOR_BUFFER_BIT
                    }
                    if mask & DEPTH != 0 {
                        self.gl.clear_depth_f64(depth);
                        gl_mask |= glow::DEPTH_BUFFER_BIT
                    }
                    if mask & STENCIL != 0 {
                        self.gl.clear_stencil(stencil as i32);
                        gl_mask |= glow::STENCIL_BUFFER_BIT
                    }
                    self.gl.clear(gl_mask);
                }
                Context3DCommand::ConfigureBackBuffer {
                    width,
                    height,
                    anti_alias,
                    depth_and_stencil,
                    wants_best_resolution: _,
                    wants_best_resolution_on_browser_zoom: _,
                } => {
                    if let Some(back_buffer) = self.back_buffer.take() {
                        self.gl.delete_framebuffer(back_buffer);
                    }
                    if let Some(back_buffer_depth_stencil) = self.back_buffer_depth_stencil.take() {
                        self.gl.delete_texture(back_buffer_depth_stencil);
                    }
                    let backbuffer_reg_data =
                        as_registry_data(&self.back_buffer_raw_texture_handle);
                    self.back_buffer = Some(self.gl.create_framebuffer().unwrap());
                    self.gl.bind_framebuffer(glow::TEXTURE_2D, self.back_buffer);
                    self.gl
                        .bind_texture(glow::TEXTURE_2D, Some(backbuffer_reg_data.texture));
                    self.gl.tex_image_2d_multisample(
                        glow::TEXTURE_2D,
                        anti_alias as i32,
                        glow::RGBA8 as i32,
                        width as i32,
                        height as i32,
                        true,
                    );
                    self.gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MAG_FILTER,
                        glow::NEAREST as i32,
                    );
                    self.gl.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MIN_FILTER,
                        glow::NEAREST as i32,
                    );
                    self.gl.bind_texture(glow::TEXTURE_2D, None);
                    self.gl.framebuffer_texture_2d(
                        glow::FRAMEBUFFER,
                        glow::COLOR_ATTACHMENT0,
                        glow::TEXTURE_2D,
                        Some(backbuffer_reg_data.texture),
                        0,
                    );
                    if depth_and_stencil {
                        self.back_buffer_depth_stencil = Some(self.gl.create_texture().unwrap());
                        self.gl
                            .bind_texture(glow::TEXTURE_2D, self.back_buffer_depth_stencil);
                        self.gl.tex_image_2d(
                            glow::TEXTURE_2D,
                            0,
                            glow::DEPTH24_STENCIL8 as i32,
                            width as i32,
                            height as i32,
                            0,
                            glow::DEPTH_STENCIL,
                            glow::UNSIGNED_INT_24_8,
                            glow::PixelUnpackData::Slice(None),
                        );
                        self.gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_MAG_FILTER,
                            glow::NEAREST as i32,
                        );
                        self.gl.tex_parameter_i32(
                            glow::TEXTURE_2D,
                            glow::TEXTURE_MIN_FILTER,
                            glow::NEAREST as i32,
                        );
                        self.gl.bind_texture(glow::TEXTURE_2D, None);
                        self.gl.framebuffer_texture_2d(
                            glow::FRAMEBUFFER,
                            glow::DEPTH_STENCIL_ATTACHMENT,
                            glow::TEXTURE_2D,
                            self.back_buffer_depth_stencil,
                            0,
                        );
                    }
                }
                Context3DCommand::SetRenderToTexture {
                    texture,
                    enable_depth_and_stencil,
                    anti_alias,
                    surface_selector,
                } => todo!(),
                Context3DCommand::SetRenderToBackBuffer => self.gl.bind_framebuffer(glow::TEXTURE_2D, self.back_buffer),
                Context3DCommand::UploadToIndexBuffer {
                    buffer,
                    start_offset,
                    data,
                } => {
                    //self.gl.bind_buffer(buffer.);
                }
                Context3DCommand::UploadToVertexBuffer {
                    buffer,
                    start_vertex,
                    data32_per_vertex,
                    data,
                } => todo!(),
                Context3DCommand::DrawTriangles {
                    index_buffer,
                    first_index,
                    num_triangles,
                } => todo!(),
                Context3DCommand::SetVertexBufferAt {
                    index,
                    buffer,
                    buffer_offset,
                } => todo!(),
                Context3DCommand::UploadShaders {
                    module,
                    vertex_shader_agal,
                    fragment_shader_agal,
                } => todo!(),
                Context3DCommand::SetShaders { module } => todo!(),
                Context3DCommand::SetProgramConstantsFromVector {
                    program_type,
                    first_register,
                    matrix_raw_data_column_major,
                } => todo!(),
                Context3DCommand::SetCulling { face } => todo!(),
                Context3DCommand::CopyBitmapToTexture {
                    source,
                    source_width,
                    source_height,
                    dest,
                    layer,
                } => todo!(),
                Context3DCommand::SetTextureAt {
                    sampler,
                    texture,
                    cube,
                } => todo!(),
                Context3DCommand::SetColorMask {
                    red,
                    green,
                    blue,
                    alpha,
                } => todo!(),
                Context3DCommand::SetDepthTest {
                    depth_mask,
                    pass_compare_mode,
                } => todo!(),
                Context3DCommand::SetBlendFactors {
                    source_factor,
                    destination_factor,
                } => todo!(),
                Context3DCommand::SetSamplerStateAt {
                    sampler,
                    wrap,
                    filter,
                } => todo!(),
                Context3DCommand::SetScissorRectangle { rect } => {
                    todo!()
                }
            }
        }
    }

    fn present(&mut self) {
        todo!()
    }
}

fn context3d_texture_format_to_gl_format_and_target(
    fmt: Context3DTextureFormat,
) -> (u32, u32, u32) {
    match fmt {
        Context3DTextureFormat::Bgra => (glow::RGBA8, glow::BGRA, glow::UNSIGNED_SHORT),
        Context3DTextureFormat::BgraPacked => {
            (glow::RGBA4, glow::BGRA, glow::UNSIGNED_SHORT_4_4_4_4)
        }
        Context3DTextureFormat::BgrPacked => (glow::RGB565, glow::BGR, glow::UNSIGNED_SHORT_5_6_5),
        Context3DTextureFormat::Compressed => (glow::COMPRESSED_RGBA_S3TC_DXT1_EXT, 0, 0),
        Context3DTextureFormat::CompressedAlpha => (glow::COMPRESSED_RGBA_S3TC_DXT5_EXT, 0, 0),
        #[cfg(not(target_os = "vita"))]
        Context3DTextureFormat::RgbaHalfFloat => (glow::RGBA16F, glow::RGBA, glow::HALF_FLOAT),
        #[cfg(target_os = "vita")] // WebGL1/GLES2 use an OES extension with a different value
        Context3DTextureFormat::RgbaHalfFloat => (glow::RGBA16F, glow::RGBA, glow::HALF_FLOAT_OES),
    }
}
