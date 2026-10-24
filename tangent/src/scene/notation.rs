use tangent_core::render::{QuadInstance, QuadRenderer, TextRenderer};

pub(crate) fn quad(out: &mut QuadRenderer, x: f32, y: f32, w: f32, h: f32, color: [f32; 4]) {
    out.push(QuadInstance {
        position: [x, y],
        size: [w, h],
        color,
        ..Default::default()
    });
}
pub(crate) fn label(ui: &mut nuon::Ui, x: f32, y: f32, w: f32, h: f32, size: f32, text: String) {
    nuon::label()
        .pos(x, y)
        .size(w, h)
        .font_size(size)
        .text(text)
        .text_justify(nuon::TextAlign::Center)
        .build(ui);
}
pub(crate) fn glyph(
    out: &mut TextRenderer,
    x: f32,
    baseline: f32,
    size: f32,
    text: &str,
    color: [u8; 3],
) {
    let buffer = TextRenderer::gen_buffer_with_attr(
        size,
        text,
        cosmic_text::Attrs::new()
            .family(cosmic_text::Family::Name("Leland"))
            .color(cosmic_text::Color::rgb(color[0], color[1], color[2])),
    );
    let line_y = buffer.layout_runs().next().map(|r| r.line_y).unwrap_or(0.0);
    out.queue_buffer(x, baseline - line_y, buffer);
}
/// Reuse shaped music glyphs in timeline views rather than reshaping hundreds
/// of identical noteheads every frame. Zoom changes invalidate this small cache.
#[derive(Default)]
pub(crate) struct MusicGlyphs {
    size: f32,
    buffers: std::collections::HashMap<(char, [u8; 3]), (cosmic_text::Buffer, f32)>,
}
impl MusicGlyphs {
    pub(crate) fn draw(
        &mut self,
        out: &mut TextRenderer,
        x: f32,
        baseline: f32,
        size: f32,
        symbol: char,
        color: [u8; 3],
    ) {
        if self.size.to_bits() != size.to_bits() {
            self.buffers.clear();
            self.size = size;
        }
        let (buffer, line_y) = self.buffers.entry((symbol, color)).or_insert_with(|| {
            let buffer = TextRenderer::gen_buffer_with_attr(
                size,
                &symbol.to_string(),
                cosmic_text::Attrs::new()
                    .family(cosmic_text::Family::Name("Leland"))
                    .color(cosmic_text::Color::rgb(color[0], color[1], color[2])),
            );
            let line_y = buffer.layout_runs().next().map(|r| r.line_y).unwrap_or(0.0);
            (buffer, line_y)
        });
        out.queue_buffer(x, baseline - *line_y, buffer.clone());
    }
}
