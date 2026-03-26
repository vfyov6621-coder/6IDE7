//! Settings panel for 6IDE7
//!
//! Configuration and preferences dialog.

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget,
};
use crate::theme::*;

/// Application settings
#[derive(Data, Clone)]
pub struct AppSettings {
    pub editor: EditorSettings,
    pub code_gen: CodeGenSettings,
    pub appearance: AppearanceSettings,
    pub show_settings: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            editor: EditorSettings::default(),
            code_gen: CodeGenSettings::default(),
            appearance: AppearanceSettings::default(),
            show_settings: false,
        }
    }
}

/// Editor settings
#[derive(Data, Clone)]
pub struct EditorSettings {
    pub grid_visible: bool,
    pub snap_to_grid: bool,
    pub grid_size: f64,
    pub auto_save: bool,
    pub auto_save_interval: u64,
}

impl Default for EditorSettings {
    fn default() -> Self {
        Self {
            grid_visible: true,
            snap_to_grid: true,
            grid_size: 20.0,
            auto_save: true,
            auto_save_interval: 60,
        }
    }
}

/// Code generation settings
#[derive(Data, Clone)]
pub struct CodeGenSettings {
    pub target_language: String,
    pub format_on_generate: bool,
    pub include_comments: bool,
    pub output_directory: String,
}

impl Default for CodeGenSettings {
    fn default() -> Self {
        Self {
            target_language: "Python".to_string(),
            format_on_generate: true,
            include_comments: true,
            output_directory: "./output".to_string(),
        }
    }
}

/// Appearance settings
#[derive(Data, Clone)]
pub struct AppearanceSettings {
    pub theme: String,
    pub font_size: f64,
    pub font_family: String,
    pub show_minimap: bool,
    pub sidebar_width: f64,
}

impl Default for AppearanceSettings {
    fn default() -> Self {
        Self {
            theme: "Dark".to_string(),
            font_size: 12.0,
            font_family: "JetBrains Mono".to_string(),
            show_minimap: true,
            sidebar_width: 260.0,
        }
    }
}

/// Settings section
#[derive(Data, Clone, Copy, PartialEq)]
pub enum SettingsSection {
    Editor,
    CodeGeneration,
    Appearance,
    Keybindings,
}

impl SettingsSection {
    pub fn label(&self) -> &'static str {
        match self {
            SettingsSection::Editor => "Editor",
            SettingsSection::CodeGeneration => "Code Generation",
            SettingsSection::Appearance => "Appearance",
            SettingsSection::Keybindings => "Keybindings",
        }
    }
    
    pub fn icon(&self) -> &'static str {
        match self {
            SettingsSection::Editor => "✎",
            SettingsSection::CodeGeneration => "⚡",
            SettingsSection::Appearance => "🎨",
            SettingsSection::Keybindings => "⌨",
        }
    }
}

/// Settings dialog state
#[derive(Data, Clone)]
pub struct SettingsState {
    pub settings: AppSettings,
    pub active_section: SettingsSection,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            settings: AppSettings::default(),
            active_section: SettingsSection::Editor,
        }
    }
}

/// The settings dialog widget
pub struct SettingsDialog {
    section_width: f64,
    item_height: f64,
}

impl SettingsDialog {
    pub fn new() -> Self {
        Self {
            section_width: 180.0,
            item_height: 36.0,
        }
    }
}

impl Widget<SettingsState> for SettingsDialog {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut SettingsState, env: &Env) {
        match event {
            Event::MouseDown(mouse) => {
                // Check section clicks
                let sections = [
                    SettingsSection::Editor,
                    SettingsSection::CodeGeneration,
                    SettingsSection::Appearance,
                    SettingsSection::Keybindings,
                ];
                
                for (i, section) in sections.iter().enumerate() {
                    let y = 60.0 + (i as f64) * self.item_height;
                    if mouse.pos.x < self.section_width
                        && mouse.pos.y >= y && mouse.pos.y < y + self.item_height {
                        data.active_section = *section;
                        ctx.request_paint();
                        ctx.set_handled();
                        return;
                    }
                }
                
                // Close button
                let size = ctx.size();
                if mouse.pos.x >= size.width - 40.0 && mouse.pos.y < 40.0 {
                    data.settings.show_settings = false;
                    ctx.request_paint();
                    ctx.set_handled();
                }
            }
            _ => {}
        }
    }

    fn lifecycle(&mut self, _ctx: &mut LifeCycleCtx, _event: &LifeCycle, _data: &SettingsState, _env: &Env) {}

    fn update(&mut self, _ctx: &mut UpdateCtx, _old_data: &SettingsState, _data: &SettingsState, _env: &Env) {}

    fn layout(&mut self, _ctx: &mut LayoutCtx, bc: &BoxConstraints, _data: &SettingsState, _env: &Env) -> Size {
        Size::new(bc.max().width.min(700.0), bc.max().height.min(500.0))
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &SettingsState, env: &Env) {
        let size = ctx.size();
        
        // Dialog background
        ctx.fill(size.to_rect(), &env.get(SURFACE));
        
        // Draw border and shadow
        ctx.stroke(size.to_rect(), &env.get(BORDER), 1.0);
        
        // Header
        let header_rect = Size::new(size.width, 48.0).to_rect();
        ctx.fill(header_rect, &env.get(BACKGROUND));
        
        // Title
        ctx.text()
            .new_text_layout("Settings")
            .font(druid::FontFamily::SYSTEM_UI, 16.0)
            .text_color(env.get(TEXT))
            .build()
            .map(|layout| {
                ctx.draw_text(&layout, Point::new(20.0, (48.0 - layout.size().height) / 2.0));
            })
            .ok();
        
        // Close button
        ctx.text()
            .new_text_layout("✕")
            .font(druid::FontFamily::SYSTEM_UI, 16.0)
            .text_color(env.get(TEXT_SECONDARY))
            .build()
            .map(|layout| {
                ctx.draw_text(&layout, Point::new(size.width - 28.0, 16.0));
            })
            .ok();
        
        // Section sidebar
        let sidebar_rect = Size::new(self.section_width, size.height - 48.0)
            .to_rect()
            .with_origin(Point::new(0.0, 48.0));
        ctx.fill(sidebar_rect, &env.get(BACKGROUND));
        
        // Draw sections
        let sections = [
            SettingsSection::Editor,
            SettingsSection::CodeGeneration,
            SettingsSection::Appearance,
            SettingsSection::Keybindings,
        ];
        
        for (i, section) in sections.iter().enumerate() {
            let y = 60.0 + (i as f64) * self.item_height;
            let is_active = data.active_section == *section;
            
            // Section background
            if is_active {
                let section_rect = Size::new(self.section_width, self.item_height)
                    .to_rect()
                    .with_origin(Point::new(0.0, y));
                ctx.fill(section_rect, &env.get(ACCENT).with_alpha(0.1));
                
                // Active indicator
                ctx.stroke(
                    druid::piet::Line::new(
                        Point::new(0.0, y),
                        Point::new(0.0, y + self.item_height),
                    ),
                    &env.get(ACCENT),
                    3.0,
                );
            }
            
            // Section icon and label
            let text = format!("{}  {}", section.icon(), section.label());
            ctx.text()
                .new_text_layout(text)
                .font(druid::FontFamily::SYSTEM_UI, 13.0)
                .text_color(if is_active {
                    env.get(ACCENT)
                } else {
                    env.get(TEXT_SECONDARY)
                })
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(20.0, y + (self.item_height - layout.size().height) / 2.0));
                })
                .ok();
        }
        
        // Draw content area
        self.draw_content(ctx, data, env);
    }
}

impl SettingsDialog {
    fn draw_content(&self, ctx: &mut PaintCtx, data: &SettingsState, env: &Env) {
        let content_x = self.section_width + 24.0;
        let content_y = 60.0;
        let size = ctx.size();
        
        // Content title
        ctx.text()
            .new_text_layout(data.active_section.label().to_string())
            .font(druid::FontFamily::SYSTEM_UI, 18.0)
            .text_color(env.get(TEXT))
            .build()
            .map(|layout| {
                ctx.draw_text(&layout, Point::new(content_x, content_y));
            })
            .ok();
        
        // Draw settings items based on active section
        match data.active_section {
            SettingsSection::Editor => {
                self.draw_editor_settings(ctx, &data.settings.editor, content_x, content_y + 40.0, env);
            }
            SettingsSection::CodeGeneration => {
                self.draw_codegen_settings(ctx, &data.settings.code_gen, content_x, content_y + 40.0, env);
            }
            SettingsSection::Appearance => {
                self.draw_appearance_settings(ctx, &data.settings.appearance, content_x, content_y + 40.0, env);
            }
            SettingsSection::Keybindings => {
                self.draw_keybindings(ctx, content_x, content_y + 40.0, env);
            }
        }
    }
    
    fn draw_editor_settings(&self, ctx: &mut PaintCtx, settings: &EditorSettings, x: f64, y: f64, env: &Env) {
        let items = [
            ("Show Grid", settings.grid_visible),
            ("Snap to Grid", settings.snap_to_grid),
            ("Auto Save", settings.auto_save),
        ];
        
        for (i, (label, _value)) in items.iter().enumerate() {
            let item_y = y + (i as f64) * 36.0;
            
            // Label
            ctx.text()
                .new_text_layout(label.to_string())
                .font(druid::FontFamily::SYSTEM_UI, 13.0)
                .text_color(env.get(TEXT))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(x, item_y));
                })
                .ok();
            
            // Toggle (placeholder)
            let toggle_x = x + 300.0;
            let toggle_rect = Size::new(40.0, 20.0)
                .to_rect()
                .with_origin(Point::new(toggle_x, item_y));
            ctx.fill(rounded_rect(toggle_rect, 10.0), &env.get(BORDER));
        }
    }
    
    fn draw_codegen_settings(&self, ctx: &mut PaintCtx, settings: &CodeGenSettings, x: f64, y: f64, env: &Env) {
        let items = [
            ("Target Language", &settings.target_language),
            ("Format on Generate", ""),
            ("Include Comments", ""),
        ];
        
        for (i, (label, value)) in items.iter().enumerate() {
            let item_y = y + (i as f64) * 36.0;
            
            ctx.text()
                .new_text_layout(label.to_string())
                .font(druid::FontFamily::SYSTEM_UI, 13.0)
                .text_color(env.get(TEXT))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(x, item_y));
                })
                .ok();
            
            if !value.is_empty() {
                ctx.text()
                    .new_text_layout(value.to_string())
                    .font(druid::FontFamily::SYSTEM_UI, 13.0)
                    .text_color(env.get(ACCENT))
                    .build()
                    .map(|layout| {
                        ctx.draw_text(&layout, Point::new(x + 300.0, item_y));
                    })
                    .ok();
            }
        }
    }
    
    fn draw_appearance_settings(&self, ctx: &mut PaintCtx, settings: &AppearanceSettings, x: f64, y: f64, env: &Env) {
        let items = [
            ("Theme", &settings.theme),
            ("Font Size", &format!("{}px", settings.font_size)),
            ("Font Family", &settings.font_family),
        ];
        
        for (i, (label, value)) in items.iter().enumerate() {
            let item_y = y + (i as f64) * 36.0;
            
            ctx.text()
                .new_text_layout(label.to_string())
                .font(druid::FontFamily::SYSTEM_UI, 13.0)
                .text_color(env.get(TEXT))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(x, item_y));
                })
                .ok();
            
            ctx.text()
                .new_text_layout(value.to_string())
                .font(druid::FontFamily::SYSTEM_UI, 13.0)
                .text_color(env.get(ACCENT))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(x + 300.0, item_y));
                })
                .ok();
        }
    }
    
    fn draw_keybindings(&self, ctx: &mut PaintCtx, x: f64, y: f64, env: &Env) {
        let bindings = [
            ("Save", "Ctrl+S"),
            ("Run", "F5"),
            ("Undo", "Ctrl+Z"),
            ("Redo", "Ctrl+Y"),
            ("Delete", "Del"),
            ("Select All", "Ctrl+A"),
        ];
        
        for (i, (action, shortcut)) in bindings.iter().enumerate() {
            let item_y = y + (i as f64) * 32.0;
            
            ctx.text()
                .new_text_layout(action.to_string())
                .font(druid::FontFamily::SYSTEM_UI, 13.0)
                .text_color(env.get(TEXT))
                .build()
                .map(|layout| {
                    ctx.draw_text(&layout, Point::new(x, item_y));
                })
                .ok();
            
            // Shortcut key
            let key_rect = Size::new(60.0, 24.0)
                .to_rect()
                .with_origin(Point::new(x + 300.0, item_y - 2.0));
            ctx.fill(rounded_rect(key_rect, 4.0), &env.get(BORDER).with_alpha(0.5));
            
            ctx.text()
                .new_text_layout(shortcut.to_string())
                .font(druid::FontFamily::MONOSPACE, 11.0)
                .text_color(env.get(TEXT_SECONDARY))
                .build()
                .map(|layout| {
                    let text_x = x + 300.0 + (60.0 - layout.size().width) / 2.0;
                    ctx.draw_text(&layout, Point::new(text_x, item_y + 2.0));
                })
                .ok();
        }
    }
}

fn rounded_rect(rect: druid::Rect, radius: f64) -> druid::piet::RoundedRectShape {
    druid::piet::RoundedRectShape::from_rect(rect, radius)
}

/// Create a settings dialog widget
pub fn settings_dialog() -> SettingsDialog {
    SettingsDialog::new()
}
