//! Sidebar widget for 6IDE7
//!
//! Displays block library and project structure.

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};
use crate::theme::*;
use crate::blocks::{BlockData, BlockLibrary};

/// Sidebar state
#[derive(Data, Clone)]
pub struct SidebarState {
    pub active_panel: SidebarPanel,
    pub expanded_categories: im::HashSet<String>,
    pub search_query: String,
    pub blocks: im::Vector<BlockData>,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            active_panel: SidebarPanel::Blocks,
            expanded_categories: im::HashSet::new(),
            search_query: String::new(),
            blocks: BlockLibrary::get_builtin_blocks(),
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

/// Block categories
const CATEGORIES: &[(&str, &str)] = &[
    ("io", "Input/Output"),
    ("data", "Data"),
    ("control", "Control Flow"),
    ("function", "Functions"),
    ("math", "Math"),
    ("string", "Strings"),
];

/// The sidebar widget
pub struct Sidebar {
    header_height: f64,
    item_height: f64,
    scroll_offset: f64,
}

impl Sidebar {
    pub fn new() -> Self {
        Self {
            header_height: 40.0,
            item_height: 28.0,
            scroll_offset: 0.0,
        }
    }
}

impl Widget<SidebarState> for Sidebar {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut SidebarState, env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                let size = ctx.size();
                
                // Check panel tabs
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
                
                // Check category clicks
                if data.active_panel == SidebarPanel::Blocks {
                    let mut y = self.header_height - self.scroll_offset;
                    
                    for (cat_id, cat_name) in CATEGORIES {
                        // Category header
                        if mouse.pos.y >= y && mouse.pos.y < y + self.item_height {
                            if data.expanded_categories.contains(*cat_id) {
                                data.expanded_categories.remove(*cat_id);
                            } else {
                                data.expanded_categories.insert(cat_id.to_string());
                            }
                            ctx.request_paint();
                            ctx.set_handled();
                            return;
                        }
                        y += self.item_height;
                        
                        // Category items (if expanded)
                        if data.expanded_categories.contains(*cat_id) {
                            let cat_blocks: Vec<_> = data.blocks
                                .iter()
                                .filter(|b| b.category == *cat_id)
                                .collect();
                            
                            for _ in cat_blocks {
                                if mouse.pos.y >= y && mouse.pos.y < y + self.item_height {
                                    // Block clicked - could trigger drag
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
            Event::Wheel(wheel) => {
                let content_height = self.calculate_content_height(data);
                let max_scroll = (content_height - ctx.size().height + self.header_height).max(0.0);
                self.scroll_offset = (self.scroll_offset - wheel.wheel_delta.y).clamp(0.0, max_scroll);
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
        
        let mut y = content_y - self.scroll_offset;
        
        for (cat_id, cat_name) in CATEGORIES {
            // Category header
            let is_expanded = data.expanded_categories.contains(*cat_id);
            let indicator = if is_expanded { "▼" } else { "▶" };
            
            // Category background
            let cat_rect = Size::new(size.width, self.item_height)
                .to_rect()
                .with_origin(Point::new(0.0, y));
            ctx.fill(cat_rect, &env.get(BACKGROUND).with_alpha(0.5));
            
            // Category text
            let cat_text = format!("{}  {}", indicator, cat_name);
            ctx.text()
                .new_text_layout(cat_text)
                .font(druid::FontFamily::SYSTEM_UI, 12.0)
                .text_color(env.get(TEXT_SECONDARY))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(12.0, y + (self.item_height - layout.size().height) / 2.0));
                })
                .ok();
            
            y += self.item_height;
            
            // Draw blocks in category if expanded
            if is_expanded {
                let cat_blocks: Vec<_> = data.blocks
                    .iter()
                    .filter(|b| b.category == *cat_id)
                    .collect();
                
                for block in cat_blocks {
                    // Block item background
                    let block_rect = Size::new(size.width, self.item_height)
                        .to_rect()
                        .with_origin(Point::new(0.0, y));
                    
                    // Block color indicator
                    let block_color = match block.category.as_str() {
                        "io" => env.get(BLOCK_IO),
                        "data" => env.get(BLOCK_DATA),
                        "control" => env.get(BLOCK_CONTROL),
                        "function" => env.get(BLOCK_FUNCTION),
                        "math" => env.get(BLOCK_MATH),
                        "string" => env.get(BLOCK_STRING),
                        _ => env.get(ACCENT),
                    };
                    
                    let indicator_rect = Size::new(3.0, self.item_height - 4.0)
                        .to_rect()
                        .with_origin(Point::new(8.0, y + 2.0));
                    ctx.fill(indicator_rect, &block_color);
                    
                    // Block name
                    ctx.text()
                        .new_text_layout(block.name.clone())
                        .font(druid::FontFamily::SYSTEM_UI, 11.0)
                        .text_color(env.get(TEXT))
                        .build()
                        .map(|layout| {
                            ctx.draw_text(&layout, Point::new(20.0, y + (self.item_height - layout.size().height) / 2.0));
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
        
        // Project structure placeholder
        let items = [
            ("📁", "src", true),
            ("  📄", "main.py", false),
            ("  📄", "utils.py", false),
            ("📁", "tests", true),
            ("  📄", "test_main.py", false),
            ("📄", "requirements.txt", false),
            ("📄", "README.md", false),
        ];
        
        let mut y = content_y - self.scroll_offset;
        
        for (icon, name, _is_folder) in items.iter() {
            let text = format!("{} {}", icon, name);
            ctx.text()
                .new_text_layout(text)
                .font(druid::FontFamily::SYSTEM_UI, 12.0)
                .text_color(env.get(TEXT))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(12.0, y + 6.0));
                })
                .ok();
            
            y += self.item_height;
        }
    }
    
    fn calculate_content_height(&self, data: &SidebarState) -> f64 {
        let mut height = 0.0;
        
        for (cat_id, _) in CATEGORIES {
            height += self.item_height; // Category header
            
            if data.expanded_categories.contains(*cat_id) {
                let count = data.blocks.iter().filter(|b| b.category == *cat_id).count();
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
