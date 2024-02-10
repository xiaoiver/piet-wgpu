// use font_kit::source::SystemSource;
// use lyon::lyon_tessellation::{
//     BuffersBuilder, FillOptions, FillTessellator, FillVertex, StrokeOptions, StrokeTessellator,
//     StrokeVertex, VertexBuffers,
// };
use piet::kurbo::Line;
use piet::Color;
use piet::{
    kurbo::{Point, Size},
    FontFamily, FontStyle, FontWeight, HitTestPoint, HitTestPosition, LineMetric, Text,
    TextAttribute, TextLayout, TextLayoutBuilder, TextStorage,
};
use std::{ops::Range, rc::Rc};
use unicode_width::UnicodeWidthChar;

use crate::context::WgpuRenderContext;

#[derive(Clone)]
pub struct WgpuText {}

impl WgpuText {
    pub(crate) fn new() -> Self {
        Self {}
    }
}

#[derive(Clone)]
pub struct WgpuTextLayout {
    state: WgpuText,
    text: String,
    width: f64,
    attrs: Rc<Attributes>,
}

impl WgpuTextLayout {
    pub fn new(text: String, state: WgpuText) -> Self {
        let char_number = text.chars().count();
        let num_vertices = char_number * 4;
        let num_indices = char_number * 6;
        Self {
            state,
            text,
            width: f64::MAX,
            attrs: Rc::new(Attributes::default()),
        }
    }

    fn set_width(&mut self, width: f64) {
        self.width = width;
    }

    fn set_attrs(&mut self, attrs: Attributes) {
        self.attrs = Rc::new(attrs);
    }

    pub fn set_color(&self, color: &Color) {
        todo!()
    }

    pub(crate) fn rebuild(&self, is_mono: bool, tab_width: usize, bounds: Option<[f64; 2]>) {
        todo!()
    }

    pub(crate) fn draw_text(&self, ctx: &mut WgpuRenderContext, translate: [f32; 2]) {
        todo!()
    }

    pub fn cursor_line_for_text_position(&self, text_pos: usize) -> Line {
        todo!()
    }
}

pub struct WgpuTextLayoutBuilder {
    width: f64,
    state: WgpuText,
    text: String,
    attrs: Attributes,
}

impl WgpuTextLayoutBuilder {
    pub(crate) fn new(text: impl TextStorage, state: WgpuText) -> Self {
        Self {
            width: f64::MAX,
            text: text.as_str().to_string(),
            attrs: Default::default(),
            state,
        }
    }

    fn add(&mut self, attr: TextAttribute, range: Range<usize>) {
        self.attrs.add(range, attr);
    }

    pub fn build_with_info(
        self,
        is_mono: bool,
        tab_width: usize,
        bounds: Option<[f64; 2]>,
    ) -> WgpuTextLayout {
        todo!()
    }

    pub fn build_with_bounds(self, bounds: [f64; 2]) -> WgpuTextLayout {
        todo!()
    }
}

impl Text for WgpuText {
    type TextLayoutBuilder = WgpuTextLayoutBuilder;
    type TextLayout = WgpuTextLayout;

    fn font_family(&mut self, family_name: &str) -> Option<FontFamily> {
        todo!()
    }

    fn load_font(&mut self, data: &[u8]) -> Result<piet::FontFamily, piet::Error> {
        todo!()
    }

    fn new_text_layout(&mut self, text: impl piet::TextStorage) -> Self::TextLayoutBuilder {
        let state = self.clone();
        Self::TextLayoutBuilder::new(text, state)
    }
}

impl TextLayoutBuilder for WgpuTextLayoutBuilder {
    type Out = WgpuTextLayout;

    fn max_width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    fn alignment(self, alignment: piet::TextAlignment) -> Self {
        self
    }

    fn default_attribute(mut self, attribute: impl Into<piet::TextAttribute>) -> Self {
        let attribute = attribute.into();
        self.attrs.defaults.set(attribute);
        self
    }

    fn range_attribute(
        mut self,
        range: impl std::ops::RangeBounds<usize>,
        attribute: impl Into<piet::TextAttribute>,
    ) -> Self {
        let range = piet::util::resolve_range(range, self.text.len());
        let attribute = attribute.into();
        self.add(attribute, range);
        self
    }

    fn build(self) -> Result<Self::Out, piet::Error> {
        let state = self.state.clone();
        let mut text_layout = WgpuTextLayout::new(self.text, state);
        text_layout.set_attrs(self.attrs);
        text_layout.set_width(self.width);
        text_layout.rebuild(false, 8, None);
        Ok(text_layout)
    }
}

impl TextLayout for WgpuTextLayout {
    fn size(&self) -> Size {
        todo!()
    }

    fn trailing_whitespace_width(&self) -> f64 {
        0.0
    }

    fn image_bounds(&self) -> piet::kurbo::Rect {
        Size::ZERO.to_rect()
    }

    fn text(&self) -> &str {
        &self.text
    }

    fn line_text(&self, line_number: usize) -> Option<&str> {
        Some(&self.text)
    }

    fn line_metric(&self, line_number: usize) -> Option<LineMetric> {
        todo!()
    }

    fn line_count(&self) -> usize {
        0
    }

    fn hit_test_point(&self, point: Point) -> HitTestPoint {
        todo!()
    }

    fn hit_test_text_position(&self, idx: usize) -> HitTestPosition {
        todo!()
    }
}

#[derive(Default)]
struct Attributes {
    defaults: piet::util::LayoutDefaults,
    color: Vec<Span<Color>>,
    font: Vec<Span<FontFamily>>,
    size: Vec<Span<f64>>,
    weight: Vec<Span<FontWeight>>,
    style: Option<Span<FontStyle>>,
}

/// during construction, `Span`s represent font attributes that have been applied
/// to ranges of the text; these are combined into coretext font objects as the
/// layout is built.
struct Span<T> {
    payload: T,
    range: Range<usize>,
}

impl<T> Span<T> {
    fn new(payload: T, range: Range<usize>) -> Self {
        Span { payload, range }
    }

    fn range_end(&self) -> usize {
        self.range.end
    }
}

impl Attributes {
    fn add(&mut self, range: Range<usize>, attr: TextAttribute) {
        match attr {
            TextAttribute::TextColor(color) => self.color.push(Span::new(color, range)),
            TextAttribute::Weight(weight) => self.weight.push(Span::new(weight, range)),
            _ => {}
        }
    }

    fn color(&self, index: usize) -> &Color {
        for r in &self.color {
            if r.range.contains(&index) {
                return &r.payload;
            }
        }
        &self.defaults.fg_color
    }

    fn size(&self, index: usize) -> f64 {
        for r in &self.size {
            if r.range.contains(&index) {
                return r.payload;
            }
        }
        self.defaults.font_size
    }

    fn italic(&self) -> bool {
        matches!(
            self.style
                .as_ref()
                .map(|t| t.payload)
                .unwrap_or(self.defaults.style),
            FontStyle::Italic
        )
    }

    fn font(&self, index: usize) -> FontFamily {
        for r in &self.font {
            if r.range.contains(&index) {
                return r.payload.clone();
            }
        }
        self.defaults.font.clone()
    }

    fn font_weight(&self, index: usize) -> FontWeight {
        for r in &self.weight {
            if r.range.contains(&index) {
                return r.payload;
            }
        }
        self.defaults.weight
    }
}
