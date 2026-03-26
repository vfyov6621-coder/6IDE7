//! Sidebar widget for 6IDE7
//!
//! Displays block library and project structure with drag support.

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};
use crate::theme::*;
use crate::blocks::{BlockData, BlockLibrary, BlockDefinition};
use crate::canvas::DragState;

/// Sidebar state
#[derive(Data, Clone)]
pub struct SidebarState {
    pub active_panel: SidebarPanel,
    pub expanded_categories: im::HashSet<String>,
    pub search_query: String,
    pub block_definitions: im::Vector<BlockDefinition>,
    pub scroll_offset: f64,
    pub is_dragging_block: bool,
    pub hovered_block_type: Option<String>,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            active_panel: SidebarPanel::Blocks,
            expanded_categories: im::HashSet::new(),
            search_query: String::new(),
            block_definitions: BlockLibrary::get_definitions().into_iter().collect(),
            scroll_offset: 0.0,
            is_dragging_block: false,
            hovered_block_type: None,
        }
    }
}

/// Available sidebar panels
#[derive(Data, Clone, Copy, PartialEq)]
pub enum SidebarPanel {
    Blocks,
    Project,
}

impl SidebarPanel {
    pub fn icon(&self) -> &'static str {
        match self {
            SidebarPanel::Blocks => "◨",
            SidebarPanel::Project => "📁",
        }
    }
    
    pub fn label(&self) -> &'static str {
        match self {
            SidebarPanel::Blocks => "Blocks",
            SidebarPanel::Project => "Project",
        }
    }
}

/// The sidebar widget
pub struct Sidebar {
    header_height: f64,
    item_height: f64,
    category_height: f64,
    drag_start_pos: Option<Point>,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            header_height: 40.0,
            item_height: 32.0,
            category_height: 28.0,
            drag_start_pos: None,
        }
    }
}

impl Widget<SidebarState> for Sidebar {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut SidebarState, env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                let size = ctx.size();
                
                // Check panel tab clicks
                let tab_width = size.width / 2.0;
                if mouse.pos.y < self.header_height {
                    if mouse.pos.x < tab_width {
                        data.active_panel = SidebarPanel::Blocks;
                    } else {
                        data.active_panel = SidebarPanel::Project;
                    }
                    ctx.request_paint();
                    ctx.set_handled();
                    return;
                }
                
                // Handle block panel
                if data.active_panel == SidebarPanel::Blocks {
                    let mut y = self.header_height - data.scroll_offset;
                    
                    for (cat_id, cat_name) in BlockLibrary::get_categories() {
                        let cat_id = cat_id.to_string();
                        
                        // Category header click
                        if mouse.pos.y >= y && mouse.pos.y < y + self.category_height {
                            if data.expanded_categories.contains(&cat_id) {
                                data.expanded_categories.remove(&cat_id);
                            } else {
                                data.expanded_categories.insert(cat_id.clone());
                            }
                            ctx.request_paint();
                            ctx.set_handled();
                            return;
                        }
                        y += self.category_height;
                        
                        // Category items (if expanded)
                        if data.expanded_categories.contains(&cat_id) {
                            let cat_blocks: Vec<_> = data.block_definitions
                                .iter()
                                .filter(|b| b.category == cat_id)
                                .collect();
                            
                            for block in cat_blocks {
                                if mouse.pos.y >= y && mouse.pos.y < y + self.item_height {
                                    // Start potential drag
                                    self.drag_start_pos = Some(mouse.pos);
                                    data.hovered_block_type = Some(block.block_type.clone());
                                    ctx.request_paint();
                                    ctx.set_handled();
                                    return;
                                }
                                y += self.item_height;
                            }
                        }
                    }
                }
            }
            
            Event::MouseUp(_) => {
                self.drag_start_pos = None;
                data.is_dragging_block = false;
                data.hovered_block_type = None;
            }
            
            Event::MouseMove(mouse) => {
                // Check if we should start dragging
                if let Some(start_pos) = self.drag_start_pos {
                    let distance = ((mouse.pos.x - start_pos.x).powi(2) + 
                                   (mouse.pos.y - start_pos.y).powi(2)).sqrt();
                    
                    if distance > 5.0 {
                        data.is_dragging_block = true;
                    }
                }
                
                // Update hovered item
                if !data.is_dragging_block {
                    let size = ctx.size();
                    if mouse.pos.y < self.header_height || mouse.pos.x > size.width {
                        data.hovered_block_type = None;
                        return;
                    }
                    
                    let mut y = self.header_height - data.scroll_offset;
                    
                    for (cat_id, _) in BlockLibrary::get_categories() {
                        let cat_id = cat_id.to_string();
                        y += self.category_height;
                        
                        if data.expanded_categories.contains(&cat_id) {
                            let cat_blocks: Vec<_> = data.block_definitions
                                .iter()
                                .filter(|b| b.category == cat_id)
                                .collect();
                            
                            for block in cat_blocks {
                                if mouse.pos.y >= y && mouse.pos.y < y + self.item_height {
                                    data.hovered_block_type = Some(block.block_type.clone());
                                    ctx.request_paint();
                                    return;
                                }
                                y += self.item_height;
                            }
                        }
                    }
                    
                    data.hovered_block_type = None;
                }
            }
            
            Event::Wheel(wheel) => {
                let content_height = self.calculate_content_height(data);
                let max_scroll = (content_height - ctx.size().height + self.header_height).max(0.0);
                data.scroll_offset = (data.scroll_offset - wheel.wheel_delta.y).clamp(0.0, max_scroll);
                ctx.request_paint();
                ctx.set_handled();
            }
            
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &SidebarState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &SidebarState, _data: &SidebarState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &SidebarState, _env: &Env) -> Size {
        Size::new(bc.max().width.min(260.0), bc.max().height)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &SidebarState, env: &Env) {
        let size = ctx.size();
        
        // Draw background
        ctx.fill(size.to_rect(), &env.get(SURFACE));
        
        // Draw header with tabs
        self.draw_header(ctx, data, env);
        
        // Draw content
        if data.active_panel == SidebarPanel::Blocks {
            self.draw_blocks_panel(ctx, data, env);
        } else {
            self.draw_project_panel(ctx, data, env);
        }
        
        // Draw right border
        ctx.stroke(
            druid::piet::Line::new(
                Point::new(size.width, 0.0),
                Point::new(size.width, size.height),
            ),
            &env.get(BORDER),
            1.0,
        );
        
        // Draw drag indicator if dragging
        if data.is_dragging_block {
            self.draw_drag_indicator(ctx, env);
        }
    }
}

impl Sidebar {
    fn draw_header(&self, ctx: &mut PaintCtx, data: &SidebarState, env: &Env) {
        let size = ctx.size();
        let tab_width = size.width / 2.0;
        
        // Header background
        ctx.fill(Size::new(size.width, self.header_height).to_rect(), &env.get(BACKGROUND));
        
        // Draw tabs
        for (i, panel) in [SidebarPanel::Blocks, SidebarPanel::Project].iter().enumerate() {
            let x = (i as f64) * tab_width;
            let is_active = data.active_panel == *panel;
            
            // Tab background
            let tab_rect = Size::new(tab_width, self.header_height)
                .to_rect()
                .with_origin(Point::new(x, 0.0));
            
            if is_active {
                ctx.fill(tab_rect, &env.get(SURFACE));
            }
            
            // Tab icon and label
            let text = format!("{} {}", panel.icon(), panel.label());
            ctx.text()
                .new_text_layout(text)
                .font(druid::FontFamily::SYSTEM_UI, 12.0)
                .text_color(if is_active {
                    env.get(TEXT)
                } else {
                    env.get(TEXT_SECONDARY)
                })
                .build()
                .map(|layout| {
                    let label_x = x + (tab_width - layout.size().width) / 2.0;
                    let label_y = (self.header_height - layout.size().height) / 2.0;
                    ctx.draw_text(&layout, Point::new(label_x, label_y));
                })
                .ok();
            
            // Active indicator
            if is_active {
                ctx.stroke(
                    druid::piet::Line::new(
                        Point::new(x, self.header_height - 2.0),
                        Point::new(x + tab_width, self.header_height - 2.0),
                    ),
                    &env.get(ACCENT),
                    2.0,
                );
            }
        }
    }
    
    fn draw_blocks_panel(&self, ctx: &mut PaintCtx, data: &SidebarState, env: &Env) {
        let size = ctx.size();
        let content_y = self.header_height;
        
        // Clip to content area
        ctx.clip(Size::new(size.width, size.height - content_y)
            .to_rect()
            .with_origin(Point::new(0.0, content_y)));
        
        let mut y = content_y - data.scroll_offset;
        
        for (cat_id, cat_name) in BlockLibrary::get_categories() {
            let cat_id = cat_id.to_string();
            let is_expanded = data.expanded_categories.contains(&cat_id);
            let indicator = if is_expanded { "▼" } else { "▶" };
            
            // Category header
            let cat_rect = Size::new(size.width, self.category_height)
                .to_rect()
                .with_origin(Point::new(0.0, y));
            
            // Category background
            ctx.fill(cat_rect, &env.get(BACKGROUND).with_alpha(0.5));
            
            // Category text
            let cat_text = format!("{}  {}", indicator, cat_name);
            ctx.text()
                .new_text_layout(cat_text)
                .font(druid::FontFamily::SYSTEM_UI, 11.0)
                .text_color(env.get(TEXT_SECONDARY))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(12.0, y + (self.category_height - layout.size().height) / 2.0));
                })
                .ok();
            
            // Category color indicator
            let cat_color = match cat_id.as_str() {
                "io" => env.get(BLOCK_IO),
                "data" => env.get(BLOCK_DATA),
                "control" => env.get(BLOCK_CONTROL),
                "function" => env.get(BLOCK_FUNCTION),
                "math" => env.get(BLOCK_MATH),
                "string" => env.get(BLOCK_STRING),
                _ => env.get(ACCENT),
            };
            
            let color_rect = Size::new(3.0, self.category_height - 6.0)
                .to_rect()
                .with_origin(Point::new(4.0, y + 3.0));
            ctx.fill(color_rect, &cat_color);
            
            y += self.category_height;
            
            // Draw blocks in category if expanded
            if is_expanded {
                let cat_blocks: Vec<_> = data.block_definitions
                    .iter()
                    .filter(|b| b.category == cat_id)
                    .collect();
                
                for block in cat_blocks {
                    let is_hovered = data.hovered_block_type.as_ref() == Some(&block.block_type);
                    
                    // Block item background
                    let block_rect = Size::new(size.width, self.item_height)
                        .to_rect()
                        .with_origin(Point::new(0.0, y));
                    
                    if is_hovered {
                        ctx.fill(block_rect, &env.get(SURFACE_HOVER));
                    }
                    
                    // Block color indicator
                    let indicator_rect = Size::new(3.0, self.item_height - 6.0)
                        .to_rect()
                        .with_origin(Point::new(8.0, y + 3.0));
                    ctx.fill(indicator_rect, &cat_color);
                    
                    // Block icon
                    let icon_x = 18.0;
                    ctx.text()
                        .new_text_layout(block.icon.clone())
                        .font(druid::FontFamily::SYSTEM_UI, 12.0)
                        .text_color(env.get(TEXT))
                        .build()
                        .map(|layout| {
                            ctx.draw_text(&layout, Point::new(icon_x, y + (self.item_height - layout.size().height) / 2.0));
                        })
                        .ok();
                    
                    // Block name
                    ctx.text()
                        .new_text_layout(block.name.clone())
                        .font(druid::FontFamily::SYSTEM_UI, 11.0)
                        .text_color(if is_hovered {
                            env.get(TEXT)
                        } else {
                            env.get(TEXT_SECONDARY)
                        })
                        .build()
                        .map(|layout| {
                            ctx.draw_text(&layout, Point::new(38.0, y + (self.item_height - layout.size().height) / 2.0));
                        })
                        .ok();
                    
                    // Port count indicator
                    let input_count = block.ports.iter().filter(|p| p.direction.is_input()).count();
                    let output_count = block.ports.iter().filter(|p| p.direction.is_output()).count();
                    let port_info = format!("{}→{}", input_count, output_count);
                    
                    ctx.text()
                        .new_text_layout(port_info)
                        .font(druid::FontFamily::SYSTEM_UI, 9.0)
                        .text_color(env.get(TEXT_MUTED))
                        .build()
                        .map(|layout| {
                            ctx.draw_text(&layout, Point::new(size.width - layout.size().width - 8.0, y + (self.item_height - layout.size().height) / 2.0));
                        })
                        .ok();
                    
                    y += self.item_height;
                }
            }
        }
    }
    
    fn draw_project_panel(&self, ctx: &mut PaintCtx, data: &SidebarState, env: &Env) {
        let size = ctx.size();
        let content_y = self.header_height;
        
        // Project structure (placeholder)
        let items = [
            ("📁", "src", true),
            ("  🖨", "main.py", false),
            ("  📦", "utils.py", false),
            ("📁", "tests", true),
            ("  🧪", "test_main.py", false),
            ("📄", "requirements.txt", false),
            ("📄", "README.md", false),
            ("📄", ".gitignore", false),
        ];
        
        let mut y = content_y - data.scroll_offset;
        
        for (icon, name, _is_folder) in items.iter() {
            // Item background on hover
            if data.hovered_block_type.is_some() {
                // No hover state for project items yet
            }
            
            let text = format!("{} {}", icon, name);
            ctx.text()
                .new_text_layout(text)
                .font(druid::FontFamily::SYSTEM_UI, 12.0)
                .text_color(env.get(TEXT))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(12.0, y + 8.0));
                })
                .ok();
            
            y += self.item_height;
        }
        
        // Project info footer
        let footer_y = size.height - 50.0;
        ctx.fill(
            Size::new(size.width, 50.0).to_rect().with_origin(Point::new(0.0, footer_y)),
            &env.get(BACKGROUND),
        );
        
        ctx.text()
            .new_text_layout("Project: Untitled".to_string())
            .font(druid::FontFamily::SYSTEM_UI, 10.0)
            .text_color(env.get(TEXT_MUTED))
            .build()
            .map(|layout| {
                ctx.draw_text(&layout, Point::new(12.0, footer_y + 8.0));
            })
            .ok();
        
        ctx.text()
            .new_text_layout("3 blocks • 2 connections".to_string())
            .font(druid::FontFamily::SYSTEM_UI, 10.0)
            .text_color(env.get(TEXT_MUTED))
            .build()
            .map(|layout| {
                ctx.draw_text(&layout, Point::new(12.0, footer_y + 24.0));
            })
            .ok();
    }
    
    fn draw_drag_indicator(&self, ctx: &mut PaintCtx, env: &Env) {
        let size = ctx.size();
        
        // Highlight border when dragging
        ctx.stroke(
            size.to_rect(),
            &env.get(ACCENT).with_alpha(0.5),
            2.0,
        );
        
        // "Drag to canvas" hint
        ctx.text()
            .new_text_layout("⬇ Drag to canvas".to_string())
            .font(druid::FontFamily::SYSTEM_UI, 10.0)
            .text_color(env.get(ACCENT))
            .build()
            .map(|layout| {
                let x = (size.width - layout.size().width) / 2.0;
                let y = size.height - 20.0;
                
                // Background
                let bg_rect = Size::new(layout.size().width + 16.0, layout.size().height + 8.0)
                    .to_rect()
                    .with_origin(Point::new(x - 8.0, y - 4.0));
                ctx.fill(
                    druid::piet::RoundedRectShape::from_rect(bg_rect, 4.0),
                    &env.get(SURFACE).with_alpha(0.95),
                );
                
                ctx.draw_text(&layout, Point::new(x, y));
            })
            .ok();
    }
    
    fn calculate_content_height(&self, data: &SidebarState) -> f64 {
        let mut height = 0.0;
        
        for (cat_id, _) in BlockLibrary::get_categories() {
            height += self.category_height;
            
            let cat_id = cat_id.to_string();
            if data.expanded_categories.contains(&cat_id) {
                let count = data.block_definitions.iter().filter(|b| b.category == cat_id).count();
                height += count as f64 * self.item_height;
            }
        }
        
        height
    }
}

/// Create a sidebar widget
pub fn sidebar() -> Sidebar {
    Sidebar::new()
}
