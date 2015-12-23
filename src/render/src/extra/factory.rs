// Copyright 2014 The Gfx-rs Developers.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Factory extension. Provides resource construction shortcuts.

use gfx_core::{handle, tex};
use gfx_core::{Primitive, Resources, ShaderSet, VertexCount};
use gfx_core::factory::{BufferRole, Factory};
use gfx_core::pso::{CreationError, Descriptor};
use gfx_core::shade::{CreateShaderError, CreateProgramError};
use gfx_core::state::Rasterizer;
use extra::shade::*;
use mesh::{Mesh, VertexFormat};
use pso;

/// Error creating a PipelineState
#[derive(Clone, PartialEq, Debug)]
pub enum PipelineStateError<R: Resources> {
    /// Shader program failed to link, providing an error string.
    ProgramLink(CreateProgramError),
    /// Unable to create PSO descriptor due to mismatched formats.
    DescriptorInit(pso::InitError, handle::Program<R>),
    /// Device failed to create the handle give the descriptor.
    DeviceCreate(CreationError),
}


/// Factory extension trait
pub trait FactoryExt<R: Resources>: Factory<R> {
    /// Create a new mesh from the given vertex data.
    fn create_mesh<T: VertexFormat>(&mut self, data: &[T]) -> Mesh<R> {
        let nv = data.len();
        //debug_assert!(nv <= self.get_capabilities().max_vertex_count);
        let buf = self.create_buffer_static(data, BufferRole::Vertex);
        Mesh::from_format(buf, nv as VertexCount)
    }

    /// Create a simple program given a vertex shader with a pixel one.
    fn link_program(&mut self, vs_code: &[u8], ps_code: &[u8])
                    -> Result<handle::Program<R>, ProgramError> {

        let vs = match self.create_shader_vertex(vs_code) {
            Ok(s) => s,
            Err(e) => return Err(ProgramError::Vertex(e)),
        };
        let ps = match self.create_shader_pixel(ps_code) {
            Ok(s) => s,
            Err(e) => return Err(ProgramError::Pixel(e)),
        };

        let set = ShaderSet::Simple(vs, ps);

        self.create_program(&set)
            .map_err(|e| ProgramError::Link(e))
    }

    /// Create a simple program given `ShaderSource` versions of vertex and
    /// pixel shaders, automatically picking available shader variant.
    fn link_program_source(&mut self, vs_src: ShaderSource, ps_src: ShaderSource)
                            -> Result<handle::Program<R>, ProgramError> {
        let model = self.get_capabilities().shader_model;

        match (vs_src.choose(model), ps_src.choose(model)) {
            (Ok(vs_code), Ok(ps_code)) => self.link_program(vs_code, ps_code),
            (Err(_), Ok(_)) => Err(ProgramError::Vertex(CreateShaderError::ModelNotSupported)),
            (_, Err(_)) => Err(ProgramError::Pixel(CreateShaderError::ModelNotSupported)),
        }
    }

    /// Create a strongly-typed Pipeline State.
    fn create_pipeline_state<I: pso::PipelineInit>(&mut self, shaders: &ShaderSet<R>,
                             primitive: Primitive, rasterizer: Rasterizer, init: &I)
                             -> Result<pso::PipelineState<R, I::Meta>, PipelineStateError<R>>
    {
        let program = match self.create_program(shaders) {
            Ok(p) => p,
            Err(e) => return Err(PipelineStateError::ProgramLink(e)),
        };
        let mut descriptor = Descriptor::new(primitive, rasterizer);
        let meta = match init.link_to(&mut descriptor, program.get_info()) {
            Ok(m) => m,
            Err(e) => return Err(PipelineStateError::DescriptorInit(e, program)),
        };
        let raw = match self.create_pipeline_state_raw(&program, &descriptor) {
            Ok(raw) => raw,
            Err(e) => return Err(PipelineStateError::DeviceCreate(e)),
        };

        Ok(pso::PipelineState::new(raw, primitive, meta))
    }

    /// Create a simple RGBA8 2D texture.
    fn create_texture_rgba8(&mut self, width: u16, height: u16)
                            -> Result<handle::Texture<R>, tex::TextureError> {
        self.create_texture(tex::TextureInfo {
            width: width,
            height: height,
            depth: 1,
            levels: 1,
            kind: tex::Kind::D2(tex::AaMode::Single),
            format: tex::RGBA8,
        })
    }

    /// Create RGBA8 2D texture with given contents and mipmap chain.
    fn create_texture_rgba8_static(&mut self, width: u16, height: u16, data: &[u32])
                                   -> Result<handle::Texture<R>, tex::TextureError> {
        let info = tex::TextureInfo {
            width: width,
            height: height,
            depth: 1,
            levels: 99,
            kind: tex::Kind::D2(tex::AaMode::Single),
            format: tex::RGBA8,
        };
        match self.create_texture_static(info, data) {
            Ok(handle) => {
                self.generate_mipmap(&handle);
                Ok(handle)
            },
            Err(e) => Err(e),
        }
    }

    /// Create a simple depth+stencil 2D texture.
    fn create_texture_depth_stencil(&mut self, width: u16, height: u16)
                                    -> Result<handle::Texture<R>, tex::TextureError> {
        self.create_texture(tex::TextureInfo {
            width: width,
            height: height,
            depth: 0,
            levels: 1,
            kind: tex::Kind::D2(tex::AaMode::Single),
            format: tex::Format::DEPTH24_STENCIL8,
        })
    }

    /// Create a linear sampler with clamping to border.
    fn create_sampler_linear(&mut self) -> handle::Sampler<R> {
        self.create_sampler(tex::SamplerInfo::new(
            tex::FilterMethod::Trilinear,
            tex::WrapMode::Clamp,
        ))
    }
}

impl<R: Resources, F: Factory<R>> FactoryExt<R> for F {}
