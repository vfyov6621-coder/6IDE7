//! Toolbar widget for 6IDE7
//!
//! Top toolbar with project actions and tools.

use druid::{
    widget::{Flex, Label, SizedBox},
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget, WidgetExt,
};
use crate::theme::*;
use crate::widgets::*;

/// Toolbar state
#[derive(Data, Clone)]
pub struct ToolbarState {
    pub project_name: String,
    pub is_modified: bool,
    pub current_tool: Tool,
}

impl Default for ToolbarState {
    fn default() -> Self {
        Self {
            project_name: "Untitled Project".to_string(),
            is_modified: false,
            current_tool: Tool::Select,
        }
    }
}

/// Available tools
#[derive(Data, Clone, Copy, PartialEq)]
pub enum Tool {
    Select,
    Pan,
    Connect,
    Delete,
}

/// The toolbar widget
pub struct Toolbar {
    buttons: Vec<ToolButton>,
}

struct ToolButton {
    tool: Tool,
    icon: &'static str,
    tooltip: &'static str,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            buttons: vec![
                ToolButton { tool: Tool::Select, icon: "⟷", tooltip: "Select (V)" },
                ToolButton { tool: Tool::Pan, icon: "✋", tooltip: "Pan (H)" },
                ToolButton { tool: Tool::Connect, icon: "⤫", tooltip: "Connect (C)" },
                ToolButton { tool: Tool::Delete, icon: "✕", tooltip: "Delete (Del)" },
            ],
        }
    }
}

impl Widget<ToolbarState> for Toolbar {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut ToolbarState, env: &Env) {
        // Handle button clicks
        match event {
            Event::MouseDown(mouse) => {
                // Check which button was clicked
                let button_width = 36.0;
                let start_x = 200.0; // After project name
                
                for (i, btn) in self.buttons.iter().enumerate() {
                    let btn_x = start_x + (i as f64) * button_width;
                    if mouse.pos.x >= btn_x && mouse.pos.x < btn_x + button_width
                        && mouse.pos.y >= 8.0 && mouse.pos.y < 36.0 {
                        data.current_tool = btn.tool;
                        ctx.request_paint();
                        ctx.set_handled();
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &ToolbarState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &ToolbarState, _data: &ToolbarState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &ToolbarState, _env: &Env) -> Size {
        Size::new(bc.max().width, 44.0)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &ToolbarState, env: &Env) {
        let size = ctx.size();
        
        // Draw background
        ctx.fill(size.to_rect(), &env.get(SURFACE));
        
        // Draw bottom border
        ctx.stroke(
            druid::piet::Line::new(
                Point::new(0.0, size.height),
                Point::new(size.width, size.height),
            ),
            &env.get(BORDER),
            1.0,
        );
        
        // Draw project name with modified indicator
        let project_text = if data.is_modified {
            format!("● {}", data.project_name)
        } else {
            data.project_name.clone()
        };
        
        ctx.text()
            .new_text_layout(project_text)
            .font(druid::FontFamily::SYSTEM_UI, 14.0)
            .text_color(env.get(TEXT))
            .build()
            .map(|layout| {
                ctx.draw_text(&layout, Point::new(16.0, (size.height - layout.size().height) / 2.0));
            })
            .ok();
        
        // Draw tool buttons
        let button_width = 36.0;
        let start_x = 200.0;
        
        for (i, btn) in self.buttons.iter().enumerate() {
            let btn_x = start_x + (i as f64) * button_width;
            let btn_rect = Size::new(28.0, 28.0)
                .to_rect()
                .with_origin(Point::new(btn_x + 4.0, 8.0));
            
            // Highlight selected tool
            if data.current_tool == btn.tool {
                ctx.fill(rounded_rect(btn_rect, 4.0), &env.get(ACCENT).with_alpha(0.2));
            }
            
            // Draw icon
            ctx.text()
                .new_text_layout(btn.icon.to_string())
                .font(druid::FontFamily::SYSTEM_UI, 14.0)
                .text_color(if data.current_tool == btn.tool {
                    env.get(ACCENT)
                } else {
                    env.get(TEXT_SECONDARY)
                })
                .build()
                .map(|layout| {
                    let icon_x = btn_x + 4.0 + (28.0 - layout.size().width) / 2.0;
                    let icon_y = 8.0 + (28.0 - layout.size().height) / 2.0;
                    ctx.draw_text(&layout, Point::new(icon_x, icon_y));
                })
                .ok();
        }
        
        // Draw action buttons on the right
        let right_buttons = ["▶ Run", "⚙ Settings", "💾 Save"];
        let mut x = size.width - 16.0;
        
        for label in right_buttons.iter().rev() {
            ctx.text()
                .new_text_layout(label.to_string())
                .font(druid::FontFamily::SYSTEM_UI, 12.0)
                .text_color(env.get(TEXT_SECONDARY))
                .build()
                .map(|layout| {
                    x -= layout.size().width + 24.0;
                    ctx.draw_text(&layout, Point::new(x, (size.height - layout.size().height) / 2.0));
                })
                .ok();
        }
    }
}

fn rounded_rect(rect: druid::Rect, radius: f64) -> druid::piet::RoundedRectShape {
    druid::piet::RoundedRectShape::from_rect(rect, radius)
}

/// Create a toolbar widget
pub fn toolbar() -> Toolbar {
    Toolbar::new()
}
