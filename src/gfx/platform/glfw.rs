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

extern crate glfw;

use self::glfw::Context;
use device;


struct Wrap<'a>(&'a glfw::Glfw);

impl<'a> device::GlProvider for Wrap<'a> {
    fn get_proc_address(&self, name: &str) -> *const ::libc::c_void {
        let Wrap(provider) = *self;
        provider.get_proc_address(name)
    }
    fn is_extension_supported(&self, name: &str) -> bool {
        let Wrap(provider) = *self;
        provider.extension_supported(name)
    }
}


pub struct GlfwGraphicsContext<C> {
    pub context: C,
}

impl<C: Context> GlfwGraphicsContext<C> {
    #[allow(visible_private_types)]
    pub fn new<'a>(context: C, provider: &'a glfw::Glfw) -> (GlfwGraphicsContext<C>, Wrap<'a>)  {
        context.make_current();
        (GlfwGraphicsContext { context: context }, Wrap(provider))
    }
}

impl<C: Context> device::GraphicsContext<super::GlApi> for GlfwGraphicsContext<C> {
    fn make_current(&self) {
        self.context.make_current();
    }

    fn swap_buffers(&self) {
        self.context.swap_buffers();
    }
}
