//! Canvas widget for block placement
//!
//! The main workspace where users can place and connect code blocks.

use druid::{
    BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};
use crate::theme::*;
use crate::blocks::BlockData;

/// Canvas state
#[derive(Data, Clone)]
pub struct CanvasState {
    pub zoom: f64,
    pub offset: Point,
    pub grid_visible: bool,
    pub blocks: im::Vector<BlockData>,
    pub selected_block: Option<usize>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Point::ORIGIN,
            grid_visible: true,
            blocks: im::Vector::new(),
            selected_block: None,
        }
    }
}

/// The canvas widget for placing and manipulating blocks
pub struct Canvas {
    is_panning: bool,
    last_mouse_pos: Point,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            is_panning: false,
            last_mouse_pos: Point::ORIGIN,
        }
    }
}

impl Widget<CanvasState> for Canvas {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CanvasState, env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                // Middle mouse button or Space+Left for panning
                if mouse.button == druid::MouseButton::Middle {
                    self.is_panning = true;
                    self.last_mouse_pos = mouse.pos;
                    ctx.set_active(true);
                    ctx.set_handled();
                }
                
                // Left click - select or deselect blocks
                if mouse.button == druid::MouseButton::Left {
                    // Check if clicking on a block
                    let canvas_pos = screen_to_canvas(mouse.pos, data.offset, data.zoom);
                    
                    let mut clicked_block = None;
                    for (i, block) in data.blocks.iter().enumerate() {
                        if block.contains_point(canvas_pos) {
                            clicked_block = Some(i);
                            break;
                        }
                    }
                    
                    data.selected_block = clicked_block;
                    ctx.request_paint();
                }
            }
            Event::MouseUp(mouse) => {
                if mouse.button == druid::MouseButton::Middle {
                    self.is_panning = false;
                    ctx.set_active(false);
                }
            }
            Event::MouseMove(mouse) => {
                if self.is_panning {
                    let delta = mouse.pos - self.last_mouse_pos;
                    data.offset.x += delta.x;
                    data.offset.y += delta.y;
                    self.last_mouse_pos = mouse.pos;
                    ctx.request_paint();
                }
            }
            Event::Wheel(wheel) => {
                // Zoom with mouse wheel
                let zoom_delta = if wheel.wheel_delta.y > 0.0 { 1.1 } else { 0.9 };
                data.zoom = (data.zoom * zoom_delta).clamp(0.25, 4.0);
                ctx.request_paint();
                ctx.set_handled();
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &CanvasState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &CanvasState, _data: &CanvasState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &CanvasState, _env: &Env) -> Size {
        bc.max()
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &CanvasState, env: &Env) {
        let size = ctx.size();
        
        // Fill background
        ctx.fill(size.to_rect(), &env.get(BACKGROUND));
        
        // Draw grid if visible
        if data.grid_visible {
            self.draw_grid(ctx, data, env);
        }
        
        // Draw blocks
        for (i, block) in data.blocks.iter().enumerate() {
            let is_selected = data.selected_block == Some(i);
            self.draw_block(ctx, block, data.offset, data.zoom, is_selected, env);
        }
        
        // Draw connections between blocks
        self.draw_connections(ctx, data, env);
    }
}

impl Canvas {
    fn draw_grid(&self, ctx: &mut PaintCtx, data: &CanvasState, env: &Env) {
        let size = ctx.size();
        let grid_size = 20.0 * data.zoom;
        let grid_color = env.get(BORDER).with_alpha(0.3);
        
        // Calculate grid offset based on canvas offset
        let offset_x = (data.offset.x % grid_size + grid_size) % grid_size;
        let offset_y = (data.offset.y % grid_size + grid_size) % grid_size;
        
        // Draw vertical lines
        let mut x = offset_x;
        while x < size.width {
            let line = druid::piet::Line::new(
                Point::new(x, 0.0),
                Point::new(x, size.height),
            );
            ctx.stroke(line, &grid_color, 1.0);
            x += grid_size;
        }
        
        // Draw horizontal lines
        let mut y = offset_y;
        while y < size.height {
            let line = druid::piet::Line::new(
                Point::new(0.0, y),
                Point::new(size.width, y),
            );
            ctx.stroke(line, &grid_color, 1.0);
            y += grid_size;
        }
    }
    
    fn draw_block(
        &self,
        ctx: &mut PaintCtx,
        block: &BlockData,
        offset: Point,
        zoom: f64,
        is_selected: bool,
        env: &Env,
    ) {
        // Transform block position to screen coordinates
        let screen_pos = canvas_to_screen(block.position, offset, zoom);
        let screen_size = Size::new(block.size.width * zoom, block.size.height * zoom);
        let rect = screen_size.to_rect().with_origin(screen_pos);
        let radius = env.get(BORDER_RADIUS) * zoom;
        
        // Get block color based on category
        let block_color = match block.category.as_str() {
            "io" => env.get(BLOCK_IO),
            "data" => env.get(BLOCK_DATA),
            "control" => env.get(BLOCK_CONTROL),
            "function" => env.get(BLOCK_FUNCTION),
            "math" => env.get(BLOCK_MATH),
            "string" => env.get(BLOCK_STRING),
            _ => env.get(ACCENT),
        };
        
        // Draw block shadow
        let shadow_rect = rect.inflate(4.0, 4.0);
        ctx.blurred_rect(shadow_rect, 4.0, &Color::BLACK.with_alpha(0.3));
        
        // Draw block background with gradient
        let bg_color = block_color.with_alpha(0.2);
        ctx.fill(rounded_rect(rect, radius), &bg_color);
        
        // Draw block border
        let border_color = if is_selected {
            env.get(ACCENT)
        } else {
            block_color
        };
        ctx.stroke(rounded_rect(rect, radius), &border_color, if is_selected { 2.0 } else { 1.0 });
        
        // Draw block header
        let header_height = 28.0 * zoom;
        let header_rect = Size::new(rect.width(), header_height)
            .to_rect()
            .with_origin(screen_pos);
        
        // Rounded rect for header (top corners only)
        ctx.fill(
            rounded_rect_top(header_rect, radius),
            &block_color.with_alpha(0.8),
        );
        
        // Draw block name
        ctx.text()
            .new_text_layout(block.name.clone())
            .font(druid::FontFamily::SYSTEM_UI, (12.0 * zoom) as f64)
            .text_color(env.get(TEXT))
            .build()
            .map(|layout| {
                let text_y = screen_pos.y + (header_height - layout.size().height) / 2.0;
                ctx.draw_text(&layout, Point::new(screen_pos.x + 10.0 * zoom, text_y));
            })
            .ok();
        
        // Draw ports (input/output connectors)
        self.draw_ports(ctx, block, screen_pos, zoom, env);
    }
    
    fn draw_ports(
        &self,
        ctx: &mut PaintCtx,
        block: &BlockData,
        screen_pos: Point,
        zoom: f64,
        env: &Env,
    ) {
        let port_radius = 6.0 * zoom;
        let port_color = env.get(TEXT);
        
        // Draw input ports on the left
        for (i, port) in block.inputs.iter().enumerate() {
            let port_y = screen_pos.y + 40.0 * zoom + (i as f64) * 24.0 * zoom;
            let port_x = screen_pos.x - port_radius;
            
            ctx.fill(
                druid::piet::Circle::new(Point::new(port_x, port_y), port_radius),
                &port_color,
            );
            
            // Port label
            ctx.text()
                .new_text_layout(port.clone())
                .font(druid::FontFamily::SYSTEM_UI, 10.0 * zoom)
                .text_color(env.get(TEXT_SECONDARY))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(port_x + port_radius + 4.0, port_y - 5.0 * zoom));
                })
                .ok();
        }
        
        // Draw output ports on the right
        let block_width = block.size.width * zoom;
        for (i, port) in block.outputs.iter().enumerate() {
            let port_y = screen_pos.y + 40.0 * zoom + (i as f64) * 24.0 * zoom;
            let port_x = screen_pos.x + block_width + port_radius;
            
            ctx.fill(
                druid::piet::Circle::new(Point::new(port_x, port_y), port_radius),
                &port_color,
            );
            
            // Port label
            ctx.text()
                .new_text_layout(port.clone())
                .font(druid::FontFamily::SYSTEM_UI, 10.0 * zoom)
                .text_color(env.get(TEXT_SECONDARY))
                .build()
                .map(|layout| {
                    let label_width = layout.size().width;
                    ctx.draw_text(&layout, Point::new(port_x - label_width - port_radius - 4.0, port_y - 5.0 * zoom));
                })
                .ok();
        }
    }
    
    fn draw_connections(&self, ctx: &mut PaintCtx, data: &CanvasState, env: &Env) {
        // TODO: Implement connection drawing
        // For now, draw placeholder connection lines
    }
}

// Helper functions

fn screen_to_canvas(screen: Point, offset: Point, zoom: f64) -> Point {
    Point::new(
        (screen.x - offset.x) / zoom,
        (screen.y - offset.y) / zoom,
    )
}

fn canvas_to_screen(canvas: Point, offset: Point, zoom: f64) -> Point {
    Point::new(
        canvas.x * zoom + offset.x,
        canvas.y * zoom + offset.y,
    )
}

fn rounded_rect(rect: druid::Rect, radius: f64) -> druid::piet::RoundedRectShape {
    druid::piet::RoundedRectShape::from_rect(rect, radius)
}

fn rounded_rect_top(rect: druid::Rect, radius: f64) -> druid::piet::RoundedRectShape {
    druid::piet::RoundedRectShape::from_rect(rect, radius)
}

/// Create a canvas widget
pub fn canvas() -> Canvas {
    Canvas::new()
}
