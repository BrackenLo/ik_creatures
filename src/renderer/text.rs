use glyphon::{
    Attrs, Buffer, Cache, Color, FontSystem, Metrics, Resolution, Shaping, SwashCache, TextArea,
    TextAtlas, TextBounds, TextRenderer, Viewport,
};

use super::tools::{Pipeline, PipelineUpdate};

pub struct TextPipeline {
    renderer: TextRenderer,
    font_system: FontSystem,
    swash_cache: SwashCache,
    atlas: TextAtlas,
    viewport: Viewport,

    default_buffer: Buffer,
    buffers: Vec<Buffer>,
}

impl Pipeline for TextPipeline {
    fn new(core: &super::Core, _uniques: &mut super::uniques::Uniques) -> Self
    where
        Self: Sized,
    {
        let cache = Cache::new(core.device());
        let mut font_system = FontSystem::new();
        let swash_cache = SwashCache::new();
        let mut atlas = TextAtlas::new(core.device(), core.queue(), &cache, core.config.format);
        let viewport = Viewport::new(core.device(), &cache);

        let renderer = TextRenderer::new(
            &mut atlas,
            core.device(),
            wgpu::MultisampleState::default(),
            None,
        );

        let text_buffer = Buffer::new(&mut font_system, Metrics::new(30., 42.));

        Self {
            renderer,
            font_system,
            swash_cache,
            atlas,
            viewport,
            default_buffer: text_buffer,
            buffers: vec![],
        }
    }

    fn resize(&mut self, core: &super::Core, width: u32, height: u32) {
        self.viewport
            .update(core.queue(), Resolution { width, height });
    }

    fn render<'pass>(
        &'pass mut self,
        pass: &mut wgpu::RenderPass<'pass>,
        _uniques: &super::uniques::Uniques,
    ) {
        self.renderer
            .render(&self.atlas, &self.viewport, pass)
            .unwrap();
    }
}

impl TextPipeline {
    pub fn trim(&mut self) {
        self.atlas.trim();
    }
}

pub struct TextData {
    pub text: String,
    pub pos: (f32, f32),
    pub color: [u8; 3],
}

impl PipelineUpdate<&[TextData]> for TextPipeline {
    fn update(&mut self, core: &super::Core, data: &[TextData]) {
        if data.is_empty() {
            self.buffers.clear();
        }

        if data.len() > self.buffers.len() {
            println!("Updating size");

            (0..data.len() - self.buffers.len())
                .for_each(|_| self.buffers.push(self.default_buffer.clone()));
        }

        let data = data
            .iter()
            .zip(self.buffers.iter_mut())
            .map(|(val, buffer)| {
                buffer.set_text(
                    &mut self.font_system,
                    &val.text,
                    // Attrs::new().family(Family::Monospace),
                    Attrs::new(),
                    Shaping::Advanced,
                );

                TextArea {
                    buffer,
                    left: val.pos.0,
                    top: val.pos.1,
                    scale: 1.,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 600,
                        bottom: 160,
                    },
                    default_color: Color::rgb(val.color[0], val.color[1], val.color[2]),
                }
            })
            .collect::<Vec<_>>();

        self.renderer
            .prepare(
                core.device(),
                core.queue(),
                &mut self.font_system,
                &mut self.atlas,
                &self.viewport,
                data,
                &mut self.swash_cache,
            )
            .unwrap();
    }
}
