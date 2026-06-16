use std::{marker::PhantomData, sync::Arc};

use wgpu::{TextureView, util::DeviceExt};




pub type RenderPass<D: RenderDispatch, Z: DepthTrait> = RenderPassCommon<D, NoTexture, Z>;
pub type RenderPassTexture<D: RenderDispatch, Z: DepthTrait> = RenderPassCommon<D, Texture, Z>;

impl<D: RenderDispatch, Z: DepthTrait> RenderPass<D,Z> {
    pub fn new(device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>, 
        bindings: &[wgpu::BindGroupLayoutEntry],
        vs: &wgpu::ShaderModule,
        fs: &wgpu::ShaderModule,
        format: wgpu::TextureFormat) -> Self {
        let t = NoTexture {};
        return Self::new_render_pass(device, queue, bindings, vs, fs, format, t);
    }
}
impl<D: RenderDispatch, Z: DepthTrait> RenderPassTexture<D,Z> {
    pub fn new(device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>, 
        bindings: &[wgpu::BindGroupLayoutEntry],
        vs: &wgpu::ShaderModule,
        fs: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        texture: wgpu::Texture
    ) -> Self {
        let t = Texture::new(&device, texture);
        return Self::new_render_pass(device, queue, bindings, vs, fs, format, t);
    }
}



pub struct ComputePassCommon<D: ComputeDispatch> {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    bind_group_layout: wgpu::BindGroupLayout, 
    pipeline: wgpu::ComputePipeline,
    dispatch: D,
}

impl<D: ComputeDispatch> ComputePassCommon<D> {
    pub fn new( device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>, 
        bindings: &[wgpu::BindGroupLayoutEntry],
        cs: &wgpu::ShaderModule) -> Self {
        let mut s = Self::new_compute_pass(device, queue, bindings, cs);
        return s;
    }
}

impl<D: ComputeDispatch> ComputePassCommon<D> {
    fn bind_group(&self, buffers: &[&wgpu::Buffer]) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &(self.bind_group_layout),
            entries: &buffers
                .iter()
                .enumerate()
                .map(|(i, buffer)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: buffer.as_entire_binding(),
                })
                .collect::<Vec<_>>(),
            label: None,
        })
    }

    fn new_compute_pass(device: Arc<wgpu::Device>,
            queue: Arc<wgpu::Queue>, 
            bindings: &[wgpu::BindGroupLayoutEntry],
            cs: &wgpu::ShaderModule
        ) -> Self {
        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("RenderPass BindGroupLayout"),
                entries: bindings,
            });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("Compute Pipeline"),
            layout: Some(&device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[Some(&bind_group_layout)],
                immediate_size: 0
            })),
            module: &cs,
            entry_point: Some("cs_main"),
            compilation_options: wgpu::PipelineCompilationOptions::default(),
            cache: None,
        });

        
        let dispatch = D::new(&device);

        Self { device, queue, bind_group_layout, pipeline, dispatch }
    }

    pub fn do_pass(
        &self,
        buffers: &[&wgpu::Buffer],
    ) {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Compute Encoder"),
        });


        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("Compute Pass"),
            timestamp_writes: None,
        });

        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group(buffers), &[]);
        self.dispatch.do_dispatch(&mut pass);

        drop(pass);

        self.queue.submit(std::iter::once(encoder.finish()));
    }
}


pub struct RenderPassCommon<D: RenderDispatch, T: TextureTrait, Z: DepthTrait> {
    device: Arc<wgpu::Device>,
    queue: Arc<wgpu::Queue>,
    bind_group_layout: wgpu::BindGroupLayout, 
    pipeline: wgpu::RenderPipeline,
    dispatch: D,
    tex: T,
    _depth: PhantomData<Z>,
}
impl<D: RenderDispatch, T: TextureTrait, Z: DepthTrait> RenderPassCommon<D,T,Z> {
    fn bind_group(&self, buffers: &[&wgpu::Buffer]) -> wgpu::BindGroup {
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &(self.bind_group_layout),
            entries: &buffers
                .iter()
                .enumerate()
                .map(|(i, buffer)| wgpu::BindGroupEntry {
                    binding: i as u32,
                    resource: buffer.as_entire_binding(),
                })
                .collect::<Vec<_>>(),
            label: None,
        })
    }

    fn new_render_pass(device: Arc<wgpu::Device>,
        queue: Arc<wgpu::Queue>, 
        bindings: &[wgpu::BindGroupLayoutEntry],
        vs: &wgpu::ShaderModule,
        fs: &wgpu::ShaderModule,
        format: wgpu::TextureFormat,
        t: T
        ) -> Self {
            let bind_group_layout =
                device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: Some("RenderPass BindGroupLayout"),
                    entries: bindings,
                });

            let mut bind_group_layouts: Vec<Option<&wgpu::BindGroupLayout>> = vec![Some(&bind_group_layout)];
            for l in t.extra_group_layouts() {
                bind_group_layouts.push(Some(l));
            }

            let pipeline_layout =
                device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("RenderPass PipelineLayout"),
                    bind_group_layouts: &bind_group_layouts,
                    immediate_size: 0,
                });

            let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some("RenderPass Pipeline"),
                layout: Some(&pipeline_layout),

                vertex: wgpu::VertexState {
                    module: vs,
                    entry_point: Some("vs_main"),
                    buffers: &[],
                    compilation_options: Default::default(),
                },

                fragment: Some(wgpu::FragmentState {
                    module: fs,
                    entry_point: Some("fs_main"),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState {
                            color: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::SrcAlpha,
                                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                                operation: wgpu::BlendOperation::Add,
                            },
                            alpha: wgpu::BlendComponent {
                                src_factor: wgpu::BlendFactor::One,
                                dst_factor: wgpu::BlendFactor::Zero,
                                operation: wgpu::BlendOperation::Add,
                            },
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),

                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },

                depth_stencil: Z::depth_stenctil(),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None
            });

            let dispatch = D::new(&device);

            Self { device, queue, bind_group_layout, pipeline, dispatch: dispatch, tex: t, _depth: PhantomData}
        }

    pub fn do_pass(
        &self,
        surface: &wgpu::TextureView,
        buffers: &[&wgpu::Buffer],
        depth: Z
    ) {
        let mut encoder = self.device.create_command_encoder(&Default::default());
        self.dispatch.pre_pass(&mut encoder, buffers);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: surface,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None
            })],
            depth_stencil_attachment: depth.stencil_attachment(),
            label: None,
            occlusion_query_set: None,
            timestamp_writes: None,
            multiview_mask: None
        });

        pass.set_pipeline(&self.pipeline);

        pass.set_bind_group(0, &self.bind_group(buffers), &[]);
        self.tex.set_texture_bind_group(buffers, &mut pass);

        self.dispatch.do_dispatch(&mut pass);

        drop(pass);

        // self.base.device.queue().submit(Some(encoder.finish()));
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}






pub struct Dispatch;
pub struct SelfDispatch {
    dispatch_buffer: wgpu::Buffer
}

pub trait DispatchTrait {
    fn new(device: &Arc<wgpu::Device>) -> Self;
    fn pre_pass(&self, encoder: &mut wgpu::CommandEncoder, buffers: &[&wgpu::Buffer]);
}

pub trait RenderDispatch: DispatchTrait {
    fn do_dispatch(&self, pass: &mut wgpu::RenderPass);
}
pub trait ComputeDispatch: DispatchTrait {
    fn do_dispatch(&self, pass: &mut wgpu::ComputePass);
}

impl DispatchTrait for Dispatch {
    fn new(_: &Arc<wgpu::Device>) -> Self { Self }
    fn pre_pass(&self, _: &mut wgpu::CommandEncoder, _: &[&wgpu::Buffer]) {  }
}

impl DispatchTrait for SelfDispatch {
    fn new(device: &Arc<wgpu::Device>) -> Self {
        let dispatch_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Dispatch Buffer"),
            contents: bytemuck::cast_slice(&[4,1,0,0]),
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::COPY_DST,
        });
        Self { dispatch_buffer }
    }
    fn pre_pass(&self, encoder: &mut wgpu::CommandEncoder, buffers: &[&wgpu::Buffer]) {
        encoder.copy_buffer_to_buffer(&buffers[0], 0, &self.dispatch_buffer, 4, 4);
    }
}

impl RenderDispatch for Dispatch {
    fn do_dispatch(&self, pass: &mut wgpu::RenderPass) {
        pass.draw(0..4, 0..1);
    }
}

impl ComputeDispatch for Dispatch {
    fn do_dispatch(&self, pass: &mut wgpu::ComputePass) {
        pass.dispatch_workgroups(1, 1, 1);
    }
}


impl RenderDispatch for SelfDispatch {
    fn do_dispatch(&self, pass: &mut wgpu::RenderPass) {
        pass.draw_indirect(&self.dispatch_buffer, 0);
    }
}

impl ComputeDispatch for SelfDispatch {
    fn do_dispatch(&self, pass: &mut wgpu::ComputePass) {
        pass.dispatch_workgroups_indirect(&self.dispatch_buffer, 0);
    }
}





trait DepthTrait {
    fn stencil_attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachment<'_>> { None }
    fn depth_stenctil() -> Option<wgpu::DepthStencilState> { None }

}
pub struct NoDepth;
impl DepthTrait for NoDepth { }

#[derive(Clone,Copy)]
pub struct Depth<'a> {
    pub depth_view: &'a TextureView
}
impl DepthTrait for Depth<'_> {
    fn stencil_attachment(&self) -> Option<wgpu::RenderPassDepthStencilAttachment<'_>> {
        let v = Some(wgpu::RenderPassDepthStencilAttachment {
            view: &self.depth_view, // 👈 YOU ADD THIS
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        });
        return v;
    }
    
    fn depth_stenctil() -> Option<wgpu::DepthStencilState> {
        Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth24Plus,
            depth_write_enabled: Some(true),
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: Default::default(),
            bias: Default::default(),
        })
    }
}



pub struct Texture {
    texture_bindgroup: wgpu::BindGroup,
    texture_bind_group_layout: wgpu::BindGroupLayout
}
pub struct NoTexture;

trait TextureTrait {
    fn new(device: &Arc<wgpu::Device>, texture: wgpu::Texture) -> Self;
    fn extra_group_layouts(&self) -> Vec<&wgpu::BindGroupLayout>;
    fn set_texture_bind_group(&self, buffers: &[&wgpu::Buffer], pass: &mut wgpu::RenderPass);
}

impl TextureTrait for NoTexture {
    fn new(_: &Arc<wgpu::Device>, _: wgpu::Texture) -> Self { Self }
    fn extra_group_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        vec![]
    }
    fn set_texture_bind_group(&self, _: &[&wgpu::Buffer], _: &mut wgpu::RenderPass) {
    }
}

impl TextureTrait for Texture {
    fn new(device: &Arc<wgpu::Device>, texture: wgpu::Texture) -> Self {
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            ..Default::default()
        });

        let texture_bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::NonFiltering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let texture_bindgroup = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture.create_view(&Default::default())),
                },
            ],
            label: None,
        });

        Self { texture_bindgroup, texture_bind_group_layout }
    }

    fn set_texture_bind_group(&self, _: &[&wgpu::Buffer], pass: &mut wgpu::RenderPass) {
        pass.set_bind_group(1, &self.texture_bindgroup, &[]);
    }

    fn extra_group_layouts(&self) -> Vec<&wgpu::BindGroupLayout> {
        vec![&self.texture_bind_group_layout]
    }
}