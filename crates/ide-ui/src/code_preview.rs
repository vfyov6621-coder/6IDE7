//! Code preview panel widget
//!
//! Displays generated code with syntax highlighting.

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};
use crate::theme::*;
use crate::codegen::{GeneratedCode, TargetLanguage};

/// Code preview state
#[derive(Data, Clone)]
pub struct CodePreviewState {
    /// Generated code
    pub code: Option<GeneratedCode>,
    /// Selected target language
    pub target_language: TargetLanguage,
    /// Scroll offset
    pub scroll_offset: f64,
    /// Whether auto-generate is enabled
    pub auto_generate: bool,
}

impl Default for CodePreviewState {
    fn default() -> Self {
        Self {
            code: None,
            target_language: TargetLanguage::Python,
            scroll_offset: 0.0,
            auto_generate: true,
        }
    }
}

impl CodePreviewState {
    /// Create with generated code
    pub fn with_code(code: GeneratedCode) -> Self {
        Self {
            code: Some(code),
            ..Default::default()
        }
    }
    
    /// Get the code text
    pub fn code_text(&self) -> &str {
        match &self.code {
            Some(gen) => &gen.code,
            None => "// No code generated yet",
        }
    }
    
    /// Get line count
    pub fn line_count(&self) -> usize {
        self.code_text().lines().count()
    }
}

/// Code preview panel widget
pub struct CodePreview {
    line_height: f64,
    char_width: f64,
    header_height: f64,
}

impl CodePreview {
    pub fn new() -> Self {
        Self {
            line_height: 18.0,
            char_width: 8.0,
            header_height: 36.0,
        }
    }
}

impl Widget<CodePreviewState> for CodePreview {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CodePreviewState, env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                // Language tabs
                let tab_width = 80.0;
                for (i, lang) in [TargetLanguage::Python, TargetLanguage::JavaScript, TargetLanguage::Rust].iter().enumerate() {
                    let tab_x = 12.0 + (i as f64) * tab_width;
                    if mouse.pos.x >= tab_x && mouse.pos.x < tab_x + tab_width
                        && mouse.pos.y >= 0.0 && mouse.pos.y < self.header_height {
                        data.target_language = *lang;
                        ctx.request_paint();
                        ctx.set_handled();
                        return;
                    }
                }
            }
            Event::Wheel(wheel) => {
                let content_height = (data.line_count() as f64) * self.line_height;
                let max_scroll = (content_height - ctx.size().height + self.header_height).max(0.0);
                data.scroll_offset = (data.scroll_offset - wheel.wheel_delta.y).clamp(0.0, max_scroll);
                ctx.request_paint();
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &CodePreviewState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &CodePreviewState, _data: &CodePreviewState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &CodePreviewState, _env: &Env) -> Size {
        Size::new(bc.max().width, bc.max().height.min(400.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CodePreviewState, env: &Env) {
        let size = ctx.size();
        
        // Background
        ctx.fill(size.to_rect(), &env.get(BACKGROUND));
        
        // Header with language tabs
        self.draw_header(ctx, data, env);
        
        // Code area
        self.draw_code(ctx, data, env);
        
        // Border
        ctx.stroke(size.to_rect(), &env.get(BORDER), 1.0);
    }
}

impl CodePreview {
    fn draw_header(&self, ctx: &mut PaintCtx, data: &CodePreviewState, env: &Env) {
        let size = ctx.size();
        
        // Header background
        ctx.fill(Size::new(size.width, self.header_height).to_rect(), &env.get(SURFACE));
        
        // Language tabs
        let tab_width = 80.0;
        for (i, lang) in [TargetLanguage::Python, TargetLanguage::JavaScript, TargetLanguage::Rust].iter().enumerate() {
            let tab_x = 12.0 + (i as f64) * tab_width;
            let is_active = data.target_language == *lang;
            let is_generated = data.code.as_ref().map(|c| c.language == *lang).unwrap_or(false);
            
            // Tab background
            if is_active {
                let tab_rect = Size::new(tab_width, self.header_height)
                    .to_rect()
                    .with_origin(Point::new(tab_x, 0.0));
                ctx.fill(tab_rect, &env.get(BACKGROUND));
                
                // Active indicator
                ctx.stroke(
                    druid::piet::Line::new(
                        Point::new(tab_x, self.header_height - 2.0),
                        Point::new(tab_x + tab_width, self.header_height - 2.0),
                    ),
                    &env.get(ACCENT),
                    2.0,
                );
            }
            
            // Tab label
            let label = lang.display_name();
            ctx.text()
                .new_text_layout(label.to_string())
                .font(druid::FontFamily::SYSTEM_UI, 11.0)
                .text_color(if is_active {
                    env.get(TEXT)
                } else if is_generated {
                    env.get(TEXT_SECONDARY)
                } else {
                    env.get(TEXT_MUTED)
                })
                .build()
                .map(|layout| {
                    let label_x = tab_x + (tab_width - layout.size().width) / 2.0;
                    ctx.draw_text(&layout, Point::new(label_x, (self.header_height - layout.size().height) / 2.0));
                })
                .ok();
        }
        
        // Line count indicator
        if let Some(code) = &data.code {
            let info = format!("{} lines", code.line_count());
            ctx.text()
                .new_text_layout(info)
                .font(druid::FontFamily::SYSTEM_UI, 10.0)
                .text_color(env.get(TEXT_MUTED))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(size.width - layout.size().width - 12.0, 12.0));
                })
                .ok();
        }
        
        // Bottom border
        ctx.stroke(
            druid::piet::Line::new(
                Point::new(0.0, self.header_height),
                Point::new(size.width, self.header_height),
            ),
            &env.get(BORDER),
            1.0,
        );
    }
    
    fn draw_code(&self, ctx: &mut PaintCtx, data: &CodePreviewState, env: &Env) {
        let size = ctx.size();
        let content_y = self.header_height;
        let code_area_height = size.height - content_y;
        
        // Clip to code area
        ctx.clip(Size::new(size.width, code_area_height)
            .to_rect()
            .with_origin(Point::new(0.0, content_y)));
        
        let code = data.code_text();
        let start_line = (data.scroll_offset / self.line_height) as usize;
        let y_offset = content_y - (data.scroll_offset % self.line_height);
        
        // Line number gutter width
        let gutter_width = 40.0;
        
        for (i, line) in code.lines().enumerate().skip(start_line) {
            let y = y_offset + (i - start_line) as f64 * self.line_height;
            
            if y > size.height {
                break;
            }
            
            // Line number
            let line_num = format!("{:4}", i + 1);
            ctx.text()
                .new_text_layout(line_num)
                .font(druid::FontFamily::MONOSPACE, 10.0)
                .text_color(env.get(TEXT_MUTED))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(8.0, y + 2.0));
                })
                .ok();
            
            // Highlighted code line (basic highlighting)
            let highlighted = self.highlight_line(line, &data.target_language);
            
            ctx.text()
                .new_text_layout(highlighted)
                .font(druid::FontFamily::MONOSPACE, 11.0)
                .text_color(env.get(TEXT))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(gutter_width, y + 2.0));
                })
                .ok();
        }
        
        // Gutter separator
        ctx.stroke(
            druid::piet::Line::new(
                Point::new(gutter_width - 4.0, content_y),
                Point::new(gutter_width - 4.0, size.height),
            ),
            &env.get(BORDER),
            1.0,
        );
    }
    
    /// Basic syntax highlighting (returns original line for now)
    fn highlight_line(&self, line: &str, _language: &TargetLanguage) -> String {
        // TODO: Implement proper syntax highlighting
        // For now, return the line as-is
        line.to_string()
    }
}

/// Create a code preview widget
pub fn code_preview() -> CodePreview {
    CodePreview::new()
}
