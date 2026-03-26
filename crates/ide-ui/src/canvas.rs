//! Canvas widget for block placement and connection
//!
//! The main workspace where users can place and connect code blocks.

use crate::theme::*;
use crate::types::{ConnectionValidation, DataType, PortDirection, PortId, PortSpec};
use crate::blocks::{BlockData, BlockLibrary, BlockDefinition};
use crate::graph::{Connection, ConnectionGraph, ConnectionType};
use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};

/// Drag state for block operations
#[derive(Data, Clone)]
pub enum DragState {
    /// Not dragging
    None,
    /// Dragging a new block from sidebar
    NewBlock {
        definition: BlockDefinition,
        offset: Point,
    },
    /// Dragging an existing block on canvas
    Block {
        block_id: String,
        start_pos: Point,
        offset: Point,
    },
    /// Creating a connection
    Connection {
        source_port: PortId,
        source_pos: Point,
        current_pos: Point,
        is_valid: bool,
    },
}

impl Default for DragState {
    fn default() -> Self {
        DragState::None
    }
}

/// Canvas state
#[derive(Data, Clone)]
pub struct CanvasState {
    /// Zoom level
    pub zoom: f64,
    /// Pan offset
    pub offset: Point,
    /// Whether grid is visible
    pub grid_visible: bool,
    /// Snap to grid
    pub snap_to_grid: bool,
    /// Grid size
    pub grid_size: f64,
    /// Blocks on canvas
    pub blocks: im::Vector<BlockData>,
    /// Connection graph
    pub connections: ConnectionGraph,
    /// Currently selected block
    pub selected_block: Option<String>,
    /// Current drag state
    pub drag_state: DragState,
    /// Hovered port (for highlighting)
    pub hovered_port: Option<PortId>,
    /// Next block ID counter
    pub next_block_id: u64,
    /// Validation error message
    pub validation_error: Option<String>,
}

impl Default for CanvasState {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            offset: Point::ZERO,
            grid_visible: true,
            snap_to_grid: true,
            grid_size: 20.0,
            blocks: im::Vector::new(),
            connections: ConnectionGraph::new(),
            selected_block: None,
            drag_state: DragState::None,
            hovered_port: None,
            next_block_id: 1,
            validation_error: None,
        }
    }
}

impl CanvasState {
    /// Generate a new unique block ID
    pub fn generate_block_id(&mut self) -> String {
        let id = format!("block_{}", self.next_block_id);
        self.next_block_id += 1;
        id
    }
    
    /// Add a new block to the canvas
    pub fn add_block(&mut self, block: BlockData) {
        self.blocks.push_back(block);
    }
    
    /// Get block by ID
    pub fn get_block(&self, id: &str) -> Option<&BlockData> {
        self.blocks.iter().find(|b| b.id == id)
    }
    
    /// Get mutable block by ID
    pub fn get_block_mut(&mut self, id: &str) -> Option<&mut BlockData> {
        self.blocks.iter_mut().find(|b| b.id == id)
    }
    
    /// Remove a block and its connections
    pub fn remove_block(&mut self, id: &str) {
        self.connections.remove_block_connections(id);
        self.blocks.retain(|b| b.id != id);
        if self.selected_block.as_deref() == Some(id) {
            self.selected_block = None;
        }
    }
    
    /// Snap a position to the grid
    pub fn snap_to_grid(&self, pos: Point) -> Point {
        if !self.snap_to_grid {
            return pos;
        }
        Point::new(
            (pos.x / self.grid_size).round() * self.grid_size,
            (pos.y / self.grid_size).round() * self.grid_size,
        )
    }
    
    /// Convert screen coordinates to canvas coordinates
    pub fn screen_to_canvas(&self, screen: Point) -> Point {
        Point::new(
            (screen.x - self.offset.x) / self.zoom,
            (screen.y - self.offset.y) / self.zoom,
        )
    }
    
    /// Convert canvas coordinates to screen coordinates
    pub fn canvas_to_screen(&self, canvas: Point) -> Point {
        Point::new(
            canvas.x * self.zoom + self.offset.x,
            canvas.y * self.zoom + self.offset.y,
        )
    }
    
    /// Find block at canvas position
    pub fn find_block_at(&self, pos: Point) -> Option<&BlockData> {
        self.blocks.iter().find(|b| b.contains_point(pos))
    }
    
    /// Find port at canvas position
    pub fn find_port_at(&self, pos: Point, radius: f64) -> Option<(&BlockData, &PortSpec)> {
        for block in &self.blocks {
            if let Some(port) = block.get_port_at(pos, radius) {
                return Some((block, port));
            }
        }
        None
    }
}

/// The canvas widget
pub struct Canvas {
    is_panning: bool,
    last_mouse_pos: Point,
    port_radius: f64,
}

impl Canvas {
    pub fn new() -> Self {
        Self {
            is_panning: false,
            last_mouse_pos: Point::ZERO,
            port_radius: 8.0,
        }
    }
}

impl Widget<CanvasState> for Canvas {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut CanvasState, env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                let canvas_pos = data.screen_to_canvas(mouse.pos);
                
                match mouse.button {
                    druid::MouseButton::Middle => {
                        // Start panning
                        self.is_panning = true;
                        self.last_mouse_pos = mouse.pos;
                        ctx.set_active(true);
                        ctx.set_handled();
                    }
                    druid::MouseButton::Left => {
                        // Check for port click first
                        let port_radius = self.port_radius / data.zoom;
                        if let Some((block, port)) = data.find_port_at(canvas_pos, port_radius) {
                            // Start connection drag
                            if port.direction.is_output() {
                                let port_pos = block.get_port_canvas_position(port);
                                data.drag_state = DragState::Connection {
                                    source_port: PortId::new(&block.id, &port.id),
                                    source_pos: port_pos,
                                    current_pos: canvas_pos,
                                    is_valid: false,
                                };
                                ctx.request_paint();
                            }
                        } else if let Some(block) = data.find_block_at(canvas_pos) {
                            // Select/deselect block
                            data.selected_block = Some(block.id.clone());
                            
                            // Start block drag
                            data.drag_state = DragState::Block {
                                block_id: block.id.clone(),
                                start_pos: block.position,
                                offset: Point::new(
                                    canvas_pos.x - block.position.x,
                                    canvas_pos.y - block.position.y,
                                ),
                            };
                            
                            // Mark block as dragging
                            if let Some(b) = data.get_block_mut(&block.id) {
                                b.is_dragging = true;
                            }
                            
                            ctx.request_paint();
                        } else {
                            // Click on empty space - deselect
                            data.selected_block = None;
                            ctx.request_paint();
                        }
                    }
                    druid::MouseButton::Right => {
                        // Context menu (future implementation)
                    }
                    _ => {}
                }
            }
            
            Event::MouseUp(mouse) => {
                match mouse.button {
                    druid::MouseButton::Middle => {
                        self.is_panning = false;
                        ctx.set_active(false);
                    }
                    druid::MouseButton::Left => {
                        match &data.drag_state {
                            DragState::None => {}
                            DragState::NewBlock { definition, .. } => {
                                let canvas_pos = data.screen_to_canvas(mouse.pos);
                                let snapped = data.snap_to_grid(canvas_pos);
                                let id = data.generate_block_id();
                                let block = definition.create_instance(id, snapped);
                                data.add_block(block);
                                data.selected_block = None;
                            }
                            DragState::Block { block_id, .. } => {
                                if let Some(block) = data.get_block_mut(block_id) {
                                    block.is_dragging = false;
                                }
                            }
                            DragState::Connection { source_port, current_pos, is_valid, .. } => {
                                let port_radius = self.port_radius / data.zoom;
                                if let Some((target_block, target_port)) = 
                                    data.find_port_at(*current_pos, port_radius) {
                                    
                                    if *is_valid {
                                        let target_port_id = PortId::new(&target_block.id, &target_port.id);
                                        
                                        // Determine connection type
                                        let conn_type = if target_port.direction.is_control() {
                                            ConnectionType::Control
                                        } else {
                                            ConnectionType::Data
                                        };
                                        
                                        let connection = Connection::new(
                                            source_port.clone(),
                                            target_port_id,
                                            conn_type,
                                        );
                                        
                                        data.connections.add_connection(connection);
                                    }
                                }
                                data.validation_error = None;
                            }
                        }
                        data.drag_state = DragState::None;
                        ctx.request_paint();
                    }
                    _ => {}
                }
            }
            
            Event::MouseMove(mouse) => {
                let canvas_pos = data.screen_to_canvas(mouse.pos);
                
                if self.is_panning {
                    let delta = mouse.pos - self.last_mouse_pos;
                    data.offset.x += delta.x;
                    data.offset.y += delta.y;
                    self.last_mouse_pos = mouse.pos;
                    ctx.request_paint();
                } else {
                    match &mut data.drag_state {
                        DragState::None => {
                            // Update hovered port
                            let port_radius = self.port_radius / data.zoom;
                            let new_hovered = data.find_port_at(canvas_pos, port_radius)
                                .map(|(b, p)| PortId::new(&b.id, &p.id));
                            
                            if new_hovered != data.hovered_port {
                                data.hovered_port = new_hovered;
                                ctx.request_paint();
                            }
                        }
                        DragState::NewBlock { offset, .. } => {
                            *offset = mouse.pos;
                            ctx.request_paint();
                        }
                        DragState::Block { block_id, offset, .. } => {
                            let new_pos = data.snap_to_grid(Point::new(
                                canvas_pos.x - offset.x,
                                canvas_pos.y - offset.y,
                            ));
                            
                            if let Some(block) = data.get_block_mut(block_id) {
                                block.position = new_pos;
                            }
                            ctx.request_paint();
                        }
                        DragState::Connection { 
                            source_port, 
                            source_pos, 
                            current_pos, 
                            is_valid 
                        } => {
                            *current_pos = canvas_pos;
                            
                            // Check if we're hovering over a valid target port
                            let port_radius = self.port_radius / data.zoom;
                            if let Some((target_block, target_port)) = 
                                data.find_port_at(canvas_pos, port_radius) {
                                
                                let target_port_id = PortId::new(&target_block.id, &target_port.id);
                                
                                // Get source block and port
                                if let Some(source_block) = data.get_block(&source_port.block_id) {
                                    if let Some(source_spec) = source_block.get_port(&source_port.port_id) {
                                        // Validate connection
                                        let validation = data.connections.validate_connection(
                                            source_port,
                                            source_spec,
                                            &target_port_id,
                                            target_port,
                                        );
                                        
                                        *is_valid = validation.is_valid();
                                        data.validation_error = validation.error_message();
                                    }
                                }
                            } else {
                                *is_valid = false;
                                data.validation_error = None;
                            }
                            
                            ctx.request_paint();
                        }
                    }
                }
            }
            
            Event::Wheel(wheel) => {
                // Zoom
                let old_zoom = data.zoom;
                let zoom_delta = if wheel.wheel_delta.y > 0.0 { 1.1 } else { 0.9 };
                data.zoom = (data.zoom * zoom_delta).clamp(0.25, 4.0);
                
                // Adjust offset to zoom towards mouse position
                let scale = data.zoom / old_zoom;
                data.offset.x = mouse.pos.x - (mouse.pos.x - data.offset.x) * scale;
                data.offset.y = mouse.pos.y - (mouse.pos.y - data.offset.y) * scale;
                
                ctx.request_paint();
                ctx.set_handled();
            }
            
            Event::KeyDown(key) => {
                match key.key {
                    druid::KbKey::Delete => {
                        if let Some(block_id) = &data.selected_block {
                            data.remove_block(block_id);
                            ctx.request_paint();
                        }
                    }
                    _ => {}
                }
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
        
        // Draw connections
        self.draw_connections(ctx, data, env);
        
        // Draw blocks
        for block in &data.blocks {
            let is_selected = data.selected_block.as_ref() == Some(&block.id);
            self.draw_block(ctx, block, data, is_selected, env);
        }
        
        // Draw pending connection
        if let DragState::Connection { source_pos, current_pos, is_valid, .. } = &data.drag_state {
            self.draw_pending_connection(ctx, data, *source_pos, *current_pos, *is_valid, env);
        }
        
        // Draw dragged new block preview
        if let DragState::NewBlock { definition, offset } = &data.drag_state {
            let preview_pos = data.screen_to_canvas(*offset);
            let preview = definition.create_instance("preview", preview_pos);
            self.draw_block_preview(ctx, &preview, data, env);
        }
        
        // Draw validation error
        if let Some(error) = &data.validation_error {
            self.draw_validation_error(ctx, error, env);
        }
    }
}

impl Canvas {
    fn draw_grid(&self, ctx: &mut PaintCtx, data: &CanvasState, env: &Env) {
        let size = ctx.size();
        let grid_size = data.grid_size * data.zoom;
        let grid_color = env.get(BORDER).with_alpha(0.2);
        
        // Only draw if grid is visible
        if grid_size < 5.0 {
            return;
        }
        
        let offset_x = (data.offset.x % grid_size + grid_size) % grid_size;
        let offset_y = (data.offset.y % grid_size + grid_size) % grid_size;
        
        // Vertical lines
        let mut x = offset_x;
        while x < size.width {
            let line = druid::piet::Line::new(Point::new(x, 0.0), Point::new(x, size.height));
            ctx.stroke(line, &grid_color, 1.0);
            x += grid_size;
        }
        
        // Horizontal lines
        let mut y = offset_y;
        while y < size.height {
            let line = druid::piet::Line::new(Point::new(0.0, y), Point::new(size.width, y));
            ctx.stroke(line, &grid_color, 1.0);
            y += grid_size;
        }
    }
    
    fn draw_block(
        &self,
        ctx: &mut PaintCtx,
        block: &BlockData,
        data: &CanvasState,
        is_selected: bool,
        env: &Env,
    ) {
        let screen_pos = data.canvas_to_screen(block.position);
        let screen_size = Size::new(
            block.size.width * data.zoom,
            block.size.height * data.zoom,
        );
        let rect = screen_size.to_rect().with_origin(screen_pos);
        let radius = env.get(BORDER_RADIUS) * data.zoom;
        
        // Block color based on category
        let block_color = self.get_block_color(&block.category, env);
        
        // Shadow (skip if dragging)
        if !block.is_dragging {
            let shadow_offset = 4.0 * data.zoom;
            let shadow_rect = rect.inflate(shadow_offset, shadow_offset);
            ctx.blurred_rect(shadow_rect, shadow_offset, &Color::BLACK.with_alpha(0.2));
        }
        
        // Background
        let bg_color = block_color.with_alpha(0.15);
        ctx.fill(self.rounded_rect(rect, radius), &bg_color);
        
        // Border
        let border_color = if is_selected {
            env.get(ACCENT)
        } else {
            block_color
        };
        let border_width = if is_selected { 2.0 } else { 1.0 } * data.zoom;
        ctx.stroke(self.rounded_rect(rect, radius), &border_color, border_width);
        
        // Header
        let header_height = 28.0 * data.zoom;
        let header_rect = Size::new(rect.width(), header_height)
            .to_rect()
            .with_origin(screen_pos);
        
        // Header gradient effect
        ctx.fill(
            self.rounded_rect_top(header_rect, radius),
            &block_color.with_alpha(0.4),
        );
        
        // Block name
        ctx.text()
            .new_text_layout(block.name.clone())
            .font(druid::FontFamily::SYSTEM_UI, (11.0 * data.zoom) as f64)
            .text_color(env.get(TEXT))
            .build()
            .map(|layout| {
                let text_y = screen_pos.y + (header_height - layout.size().height) / 2.0;
                ctx.draw_text(&layout, Point::new(screen_pos.x + 10.0 * data.zoom, text_y));
            })
            .ok();
        
        // Draw ports
        self.draw_ports(ctx, block, data, env);
    }
    
    fn draw_ports(&self, ctx: &mut PaintCtx, block: &BlockData, data: &CanvasState, env: &Env) {
        let zoom = data.zoom;
        let port_radius = self.port_radius * zoom;
        
        for port in &block.ports {
            let canvas_pos = block.get_port_canvas_position(port);
            let screen_pos = data.canvas_to_screen(canvas_pos);
            
            let is_hovered = data.hovered_port.as_ref() == Some(&PortId::new(&block.id, &port.id));
            let is_connected = if port.direction.is_input() {
                data.connections.get_connection_to(&PortId::new(&block.id, &port.id)).is_some()
            } else {
                !data.connections.get_connections_from(&PortId::new(&block.id, &port.id)).is_empty()
            };
            
            // Port color based on type
            let port_color = self.get_port_color(&port.data_type, env);
            
            // Port background
            let bg_radius = if is_hovered { port_radius + 2.0 } else { port_radius };
            
            // Outer glow for hovered
            if is_hovered {
                ctx.fill(
                    druid::piet::Circle::new(screen_pos, bg_radius + 4.0),
                    &port_color.with_alpha(0.3),
                );
            }
            
            // Port circle
            ctx.fill(
                druid::piet::Circle::new(screen_pos, bg_radius),
                &if is_connected {
                    port_color
                } else {
                    env.get(SURFACE)
                },
            );
            
            // Port border
            ctx.stroke(
                druid::piet::Circle::new(screen_pos, bg_radius),
                &port_color,
                2.0 * zoom,
            );
            
            // Port label
            let font_size = 9.0 * zoom;
            ctx.text()
                .new_text_layout(port.name.clone())
                .font(druid::FontFamily::SYSTEM_UI, font_size)
                .text_color(env.get(TEXT_SECONDARY))
                .build()
                .map(|layout| {
                    if port.direction.is_input() {
                        ctx.draw_text(
                            &layout,
                            Point::new(screen_pos.x + port_radius + 4.0 * zoom, screen_pos.y - layout.size().height / 2.0),
                        );
                    } else {
                        ctx.draw_text(
                            &layout,
                            Point::new(screen_pos.x - port_radius - layout.size().width - 4.0 * zoom, screen_pos.y - layout.size().height / 2.0),
                        );
                    }
                })
                .ok();
        }
    }
    
    fn draw_connections(&self, ctx: &mut PaintCtx, data: &CanvasState, env: &Env) {
        for connection in &data.connections.connections {
            // Find source and target positions
            let source_block = match data.get_block(&connection.source.block_id) {
                Some(b) => b,
                None => continue,
            };
            let target_block = match data.get_block(&connection.target.block_id) {
                Some(b) => b,
                None => continue,
            };
            
            let source_port = match source_block.get_port(&connection.source.port_id) {
                Some(p) => p,
                None => continue,
            };
            let target_port = match target_block.get_port(&connection.target.port_id) {
                Some(p) => p,
                None => continue,
            };
            
            let source_canvas = source_block.get_port_canvas_position(source_port);
            let target_canvas = target_block.get_port_canvas_position(target_port);
            
            let source_screen = data.canvas_to_screen(source_canvas);
            let target_screen = data.canvas_to_screen(target_canvas);
            
            // Connection color
            let color = match connection.connection_type {
                ConnectionType::Control => env.get(WARNING),
                ConnectionType::Data => self.get_port_color(&source_port.data_type, env),
            };
            
            self.draw_bezier_connection(ctx, source_screen, target_screen, &color, 2.0 * data.zoom, env);
        }
    }
    
    fn draw_pending_connection(
        &self,
        ctx: &mut PaintCtx,
        data: &CanvasState,
        source: Point,
        current: Point,
        is_valid: bool,
        env: &Env,
    ) {
        let source_screen = data.canvas_to_screen(source);
        let target_screen = data.canvas_to_screen(current);
        
        let color = if is_valid {
            env.get(SUCCESS)
        } else {
            env.get(ERROR).with_alpha(0.5)
        };
        
        self.draw_bezier_connection(ctx, source_screen, target_screen, &color, 2.0 * data.zoom, env);
    }
    
    fn draw_bezier_connection(
        &self,
        ctx: &mut PaintCtx,
        start: Point,
        end: Point,
        color: &Color,
        width: f64,
        env: &Env,
    ) {
        // Calculate control points for smooth bezier curve
        let dx = (end.x - start.x).abs();
        let control_offset = dx.max(50.0).min(150.0);
        
        let ctrl1 = Point::new(start.x + control_offset, start.y);
        let ctrl2 = Point::new(end.x - control_offset, end.y);
        
        // Draw bezier curve
        let path = druid::piet::kurbo::BezPath::new();
        path.move_to(druid::piet::kurbo::Point::new(start.x, start.y));
        path.curve_to(
            druid::piet::kurbo::Point::new(ctrl1.x, ctrl1.y),
            druid::piet::kurbo::Point::new(ctrl2.x, ctrl2.y),
            druid::piet::kurbo::Point::new(end.x, end.y),
        );
        
        ctx.stroke(&path, color, width);
        
        // Draw arrow at end
        let arrow_size = 6.0;
        let angle = (end.y - ctrl2.y).atan2(end.x - ctrl2.x);
        
        let arrow_color = *color;
        ctx.fill(
            &druid::piet::kurbo::BezPath::from(vec![
                druid::piet::kurbo::PathEl::MoveTo(druid::piet::kurbo::Point::new(end.x, end.y)),
                druid::piet::kurbo::PathEl::LineTo(druid::piet::kurbo::Point::new(
                    end.x - arrow_size * (angle + 0.5).cos(),
                    end.y - arrow_size * (angle + 0.5).sin(),
                )),
                druid::piet::kurbo::PathEl::LineTo(druid::piet::kurbo::Point::new(
                    end.x - arrow_size * (angle - 0.5).cos(),
                    end.y - arrow_size * (angle - 0.5).sin(),
                )),
                druid::piet::kurbo::PathEl::ClosePath,
            ]),
            &arrow_color,
        );
    }
    
    fn draw_block_preview(
        &self,
        ctx: &mut PaintCtx,
        block: &BlockData,
        data: &CanvasState,
        env: &Env,
    ) {
        let screen_pos = data.canvas_to_screen(block.position);
        let screen_size = Size::new(
            block.size.width * data.zoom,
            block.size.height * data.zoom,
        );
        let rect = screen_size.to_rect().with_origin(screen_pos);
        let radius = env.get(BORDER_RADIUS) * data.zoom;
        
        let block_color = self.get_block_color(&block.category, env);
        
        // Semi-transparent preview
        ctx.fill(
            self.rounded_rect(rect, radius),
            &block_color.with_alpha(0.3),
        );
        ctx.stroke(
            self.rounded_rect(rect, radius),
            &block_color.with_alpha(0.6),
            2.0 * data.zoom,
        );
    }
    
    fn draw_validation_error(&self, ctx: &mut PaintCtx, error: &str, env: &Env) {
        let size = ctx.size();
        let padding = 12.0;
        
        ctx.text()
            .new_text_layout(format!("⚠ {}", error))
            .font(druid::FontFamily::SYSTEM_UI, 12.0)
            .text_color(env.get(WARNING))
            .build()
            .map(|layout| {
                let x = padding;
                let y = size.height - layout.size().height - padding - 150.0;
                
                // Background
                let bg_rect = Size::new(layout.size().width + 16.0, layout.size().height + 8.0)
                    .to_rect()
                    .with_origin(Point::new(x - 8.0, y - 4.0));
                ctx.fill(self.rounded_rect(bg_rect, 4.0), &env.get(SURFACE).with_alpha(0.9));
                
                ctx.draw_text(&layout, Point::new(x, y));
            })
            .ok();
    }
    
    fn get_block_color(&self, category: &str, env: &Env) -> Color {
        match category {
            "io" => env.get(BLOCK_IO),
            "data" => env.get(BLOCK_DATA),
            "control" => env.get(BLOCK_CONTROL),
            "function" => env.get(BLOCK_FUNCTION),
            "math" => env.get(BLOCK_MATH),
            "string" => env.get(BLOCK_STRING),
            _ => env.get(ACCENT),
        }
    }
    
    fn get_port_color(&self, data_type: &DataType, env: &Env) -> Color {
        match data_type {
            DataType::Integer => Color::rgb8(0x4a, 0x9, 0xff),  // Blue
            DataType::Float => Color::rgb8(0x4a, 0x9, 0xff),    // Blue
            DataType::String => Color::rgb8(0x22, 0xc5, 0x5e),  // Green
            DataType::Boolean => Color::rgb8(0xf5, 0x9e, 0x0b), // Amber
            DataType::Array(_) => Color::rgb8(0xec, 0x48, 0x99), // Pink
            DataType::Function => Color::rgb8(0xa8, 0x55, 0xf7), // Purple
            DataType::ControlFlow => Color::rgb8(0xf5, 0x9e, 0x0b), // Amber
            DataType::Any => env.get(TEXT_SECONDARY),
            _ => env.get(TEXT_SECONDARY),
        }
    }
    
    fn rounded_rect(&self, rect: druid::Rect, radius: f64) -> druid::piet::RoundedRectShape {
        druid::piet::RoundedRectShape::from_rect(rect, radius)
    }
    
    fn rounded_rect_top(&self, rect: druid::Rect, radius: f64) -> druid::piet::RoundedRectShape {
        druid::piet::RoundedRectShape::from_rect(rect, radius)
    }
}

/// Create a canvas widget
pub fn canvas() -> Canvas {
    Canvas::new()
}
