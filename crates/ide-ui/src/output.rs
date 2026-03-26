//! Output panel widget for 6IDE7
//!
//! Displays code output, errors, and console messages.

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};
use crate::theme::*;

/// Output types
#[derive(Data, Clone, PartialEq)]
pub enum OutputType {
    Standard,
    Error,
    Warning,
    Info,
}

/// A single output line
#[derive(Data, Clone)]
pub struct OutputLine {
    pub text: String,
    pub output_type: OutputType,
}

impl OutputLine {
    pub fn new(text: impl Into<String>, output_type: OutputType) -> Self {
        Self {
            text: text.into(),
            output_type,
        }
    }
    
    pub fn stdout(text: impl Into<String>) -> Self {
        Self::new(text, OutputType::Standard)
    }
    
    pub fn error(text: impl Into<String>) -> Self {
        Self::new(text, OutputType::Error)
    }
    
    pub fn warning(text: impl Into<String>) -> Self {
        Self::new(text, OutputType::Warning)
    }
    
    pub fn info(text: impl Into<String>) -> Self {
        Self::new(text, OutputType::Info)
    }
}

/// Output panel state
#[derive(Data, Clone)]
pub struct OutputState {
    pub lines: im::Vector<OutputLine>,
    pub scroll_offset: f64,
    pub active_tab: OutputTab,
}

impl Default for OutputState {
    fn default() -> Self {
        let mut lines = im::Vector::new();
        lines.push_back(OutputLine::info("6IDE7 Output Console"));
        lines.push_back(OutputLine::info("Ready to run your program..."));
        
        Self {
            lines,
            scroll_offset: 0.0,
            active_tab: OutputTab::Output,
        }
    }
}

/// Available output tabs
#[derive(Data, Clone, Copy, PartialEq)]
pub enum OutputTab {
    Output,
    Problems,
    Terminal,
}

impl OutputTab {
    pub fn label(&self) -> &'static str {
        match self {
            OutputTab::Output => "Output",
            OutputTab::Problems => "Problems",
            OutputTab::Terminal => "Terminal",
        }
    }
}

/// The output panel widget
pub struct OutputPanel {
    tab_height: f64,
    line_height: f64,
}

impl OutputPanel {
    pub fn new() -> Self {
        Self {
            tab_height: 32.0,
            line_height: 20.0,
        }
    }
}

impl Widget<OutputState> for OutputPanel {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut OutputState, env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                // Check tab clicks
                let tab_labels = [OutputTab::Output, OutputTab::Problems, OutputTab::Terminal];
                let mut x = 12.0;
                
                for tab in tab_labels.iter() {
                    let tab_width = 80.0;
                    if mouse.pos.x >= x && mouse.pos.x < x + tab_width
                        && mouse.pos.y >= 0.0 && mouse.pos.y < self.tab_height {
                        data.active_tab = *tab;
                        ctx.request_paint();
                        ctx.set_handled();
                        break;
                    }
                    x += tab_width;
                }
            }
            Event::Wheel(wheel) => {
                let max_scroll = (data.lines.len() as f64 * self.line_height - ctx.size().height + self.tab_height).max(0.0);
                data.scroll_offset = (data.scroll_offset - wheel.wheel_delta.y).clamp(0.0, max_scroll);
                ctx.request_paint();
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &OutputState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &OutputState, _data: &OutputState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &OutputState, _env: &Env) -> Size {
        Size::new(bc.max().width, bc.max().height.min(250.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &OutputState, env: &Env) {
        let size = ctx.size();
        
        // Draw background
        ctx.fill(size.to_rect(), &env.get(SURFACE));
        
        // Draw tab bar background
        let tab_rect = Size::new(size.width, self.tab_height).to_rect();
        ctx.fill(tab_rect, &env.get(BACKGROUND));
        
        // Draw tabs
        let tab_labels = [OutputTab::Output, OutputTab::Problems, OutputTab::Terminal];
        let mut x = 12.0;
        
        for tab in tab_labels.iter() {
            let is_active = data.active_tab == *tab;
            
            // Tab background
            let tab_rect = Size::new(80.0, self.tab_height)
                .to_rect()
                .with_origin(Point::new(x, 0.0));
            
            if is_active {
                ctx.fill(tab_rect, &env.get(SURFACE));
            }
            
            // Tab label
            ctx.text()
                .new_text_layout(tab.label().to_string())
                .font(druid::FontFamily::SYSTEM_UI, 12.0)
                .text_color(if is_active {
                    env.get(TEXT)
                } else {
                    env.get(TEXT_SECONDARY)
                })
                .build()
                .map(|layout| {
                    let label_x = x + (80.0 - layout.size().width) / 2.0;
                    let label_y = (self.tab_height - layout.size().height) / 2.0;
                    ctx.draw_text(&layout, Point::new(label_x, label_y));
                })
                .ok();
            
            // Active tab indicator
            if is_active {
                ctx.stroke(
                    druid::piet::Line::new(
                        Point::new(x, self.tab_height - 2.0),
                        Point::new(x + 80.0, self.tab_height - 2.0),
                    ),
                    &env.get(ACCENT),
                    2.0,
                );
            }
            
            x += 80.0;
        }
        
        // Draw top border
        ctx.stroke(
            druid::piet::Line::new(
                Point::new(0.0, 0.0),
                Point::new(size.width, 0.0),
            ),
            &env.get(BORDER),
            1.0,
        );
        
        // Draw output lines
        let content_y = self.tab_height;
        let content_height = size.height - self.tab_height;
        
        // Clip to content area
        ctx.clip(Size::new(size.width, content_height).to_rect().with_origin(Point::new(0.0, content_y)));
        
        let start_line = (data.scroll_offset / self.line_height) as usize;
        let y_offset = content_y - (data.scroll_offset % self.line_height);
        
        for (i, line) in data.lines.iter().skip(start_line).enumerate() {
            let y = y_offset + (i as f64) * self.line_height;
            
            if y > size.height {
                break;
            }
            
            // Line background for errors/warnings
            if line.output_type == OutputType::Error {
                let bg_rect = Size::new(size.width, self.line_height)
                    .to_rect()
                    .with_origin(Point::new(0.0, y));
                ctx.fill(bg_rect, &env.get(ERROR).with_alpha(0.1));
            } else if line.output_type == OutputType::Warning {
                let bg_rect = Size::new(size.width, self.line_height)
                    .to_rect()
                    .with_origin(Point::new(0.0, y));
                ctx.fill(bg_rect, &env.get(WARNING).with_alpha(0.1));
            }
            
            // Line text
            let text_color = match line.output_type {
                OutputType::Standard => env.get(TEXT),
                OutputType::Error => env.get(ERROR),
                OutputType::Warning => env.get(WARNING),
                OutputType::Info => env.get(INFO),
            };
            
            // Prefix based on type
            let prefix = match line.output_type {
                OutputType::Standard => "  ",
                OutputType::Error => "✗ ",
                OutputType::Warning => "⚠ ",
                OutputType::Info => "ℹ ",
            };
            
            let display_text = format!("{}{}", prefix, line.text);
            
            ctx.text()
                .new_text_layout(display_text)
                .font(druid::FontFamily::MONOSPACE, 12.0)
                .text_color(text_color)
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(12.0, y + 2.0));
                })
                .ok();
        }
    }
}

/// Create an output panel widget
pub fn output_panel() -> OutputPanel {
    OutputPanel::new()
}

impl OutputState {
    /// Add a standard output line
    pub fn print(&mut self, text: impl Into<String>) {
        self.lines.push_back(OutputLine::stdout(text));
    }
    
    /// Add an error line
    pub fn error(&mut self, text: impl Into<String>) {
        self.lines.push_back(OutputLine::error(text));
    }
    
    /// Add a warning line
    pub fn warning(&mut self, text: impl Into<String>) {
        self.lines.push_back(OutputLine::warning(text));
    }
    
    /// Add an info line
    pub fn info(&mut self, text: impl Into<String>) {
        self.lines.push_back(OutputLine::info(text));
    }
    
    /// Clear all output
    pub fn clear(&mut self) {
        self.lines.clear();
        self.scroll_offset = 0.0;
    }
}
