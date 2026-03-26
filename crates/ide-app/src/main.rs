//! 6IDE7 - Visual Block Programming IDE
//!
//! Main application entry point.

use druid::{
    AppLauncher, Color, Data, Env, Event, EventCtx, LocalizedString, Menu, MenuItem,
    PlatformError, Point, Size, Widget, WidgetExt, WindowDesc, WindowState,
};
use ide_ui::{
    theme::apply_theme,
    canvas::{Canvas, CanvasState},
    toolbar::{Toolbar, ToolbarState, Tool},
    output::{OutputPanel, OutputState, OutputType},
    sidebar::{Sidebar, SidebarState, SidebarPanel},
    settings::{SettingsDialog, SettingsState, AppSettings},
    blocks::BlockData,
    widgets::*,
};

/// Main application state
#[derive(Data, Clone)]
pub struct AppState {
    pub canvas: CanvasState,
    pub toolbar: ToolbarState,
    pub output: OutputState,
    pub sidebar: SidebarState,
    pub settings: SettingsState,
    pub window_title: String,
}

impl Default for AppState {
    fn default() -> Self {
        let mut canvas = CanvasState::default();
        
        // Add some demo blocks
        canvas.blocks.push_back(
            BlockData::new("demo_print", "Print", "io")
                .with_position(100.0, 100.0)
                .with_input("value")
        );
        
        canvas.blocks.push_back(
            BlockData::new("demo_var", "Variable", "data")
                .with_position(100.0, 250.0)
                .with_input("value")
                .with_output("result")
        );
        
        canvas.blocks.push_back(
            BlockData::new("demo_add", "Add", "math")
                .with_position(350.0, 150.0)
                .with_input("a")
                .with_input("b")
                .with_output("result")
        );
        
        canvas.blocks.push_back(
            BlockData::new("demo_if", "If", "control")
                .with_position(350.0, 300.0)
                .with_input("condition")
                .with_output("then")
                .with_size(160.0, 140.0)
        );
        
        Self {
            canvas,
            toolbar: ToolbarState::default(),
            output: OutputState::default(),
            sidebar: SidebarState::default(),
            settings: SettingsState::default(),
            window_title: "6IDE7 - Visual Programming IDE".to_string(),
        }
    }
}

/// Main application widget
pub struct IdeApp;

impl IdeApp {
    pub fn new() -> Self {
        Self
    }
}

impl Widget<AppState> for IdeApp {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut AppState, env: &Env) {
        // Handle keyboard shortcuts
        if let Event::KeyDown(key) = event {
            match (key.mods, key.key) {
                (druid::KeyModifiers::CONTROL, druid::KbKey::Character(ref c)) if c == "s" => {
                    // Save
                    data.output.print("Project saved successfully!");
                    ctx.request_paint();
                    ctx.set_handled();
                }
                (druid::KeyModifiers::CONTROL, druid::KbKey::Character(ref c)) if c == "z" => {
                    // Undo
                    ctx.set_handled();
                }
                (druid::KeyModifiers::CONTROL, druid::KbKey::Character(ref c)) if c == "y" => {
                    // Redo
                    ctx.set_handled();
                }
                (_, druid::KbKey::Delete) => {
                    // Delete selected block
                    if let Some(idx) = data.canvas.selected_block {
                        data.canvas.blocks.remove(idx);
                        data.canvas.selected_block = None;
                        ctx.request_paint();
                    }
                    ctx.set_handled();
                }
                (_, druid::KbKey::Character(ref c)) if c == "v" || c == "V" => {
                    data.toolbar.current_tool = Tool::Select;
                    ctx.request_paint();
                }
                (_, druid::KbKey::Character(ref c)) if c == "h" || c == "H" => {
                    data.toolbar.current_tool = Tool::Pan;
                    ctx.request_paint();
                }
                (_, druid::KbKey::Character(ref c)) if c == "c" || c == "C" => {
                    data.toolbar.current_tool = Tool::Connect;
                    ctx.request_paint();
                }
                _ => {}
            }
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut druid::LifeCycleCtx,
        _event: &druid::LifeCycle,
        _data: &AppState,
        _env: &Env,
    ) {
    }

    fn update(&mut self, _ctx: &mut druid::UpdateCtx, _old_data: &AppState, _data: &AppState, _env: &Env) {
    }

    fn layout(
        &mut self,
        _ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        _data: &AppState,
        _env: &Env,
    ) -> Size {
        bc.max()
    }

    fn paint(&mut self, _ctx: &mut druid::PaintCtx, _data: &AppState, _env: &Env) {
    }
}

/// Build the main UI
fn build_ui() -> impl Widget<AppState> {
    let toolbar = Toolbar::new()
        .lens(AppState::toolbar);
    
    let canvas = Canvas::new()
        .lens(AppState::canvas);
    
    let output = OutputPanel::new()
        .lens(AppState::output);
    
    let sidebar = Sidebar::new()
        .lens(AppState::sidebar);
    
    // Main content split: sidebar | canvas+output
    let main_content = h_split(
        sidebar,
        v_split(canvas, output)
            .split_position(0.7)
            .min_sizes(200.0, 100.0),
    )
    .split_position(0.18)
    .min_sizes(180.0, 400.0);
    
    // Full layout: toolbar | main content
    v_split(toolbar, main_content)
        .split_position(0.0)
        .min_sizes(44.0, 200.0)
        .padding(0.0)
        .expand()
}

/// Build the application menu
fn build_menu<T: Data>() -> Menu<T> {
    let mut menu = Menu::empty();
    
    // File menu
    let mut file_menu = Menu::new(LocalizedString::new("File"));
    file_menu = file_menu.entry(
        MenuItem::new(LocalizedString::new("New Project"))
            .shortcut(druid::Command::new(druid::commands::NEW_FILE, ()))
            .hotkey(druid::RawMods::Ctrl, "n")
    );
    file_menu = file_menu.entry(
        MenuItem::new(LocalizedString::new("Open..."))
            .hotkey(druid::RawMods::Ctrl, "o")
    );
    file_menu = file_menu.entry(
        MenuItem::new(LocalizedString::new("Save"))
            .hotkey(druid::RawMods::Ctrl, "s")
    );
    file_menu = file_menu.entry(
        MenuItem::new(LocalizedString::new("Save As..."))
            .hotkey(druid::RawMods::CtrlShift, "s")
    );
    file_menu = file_menu.separator();
    file_menu = file_menu.entry(
        MenuItem::new(LocalizedString::new("Export Code..."))
    );
    file_menu = file_menu.separator();
    file_menu = file_menu.entry(
        MenuItem::new(LocalizedString::new("Exit"))
    );
    menu = menu.entry(file_menu);
    
    // Edit menu
    let mut edit_menu = Menu::new(LocalizedString::new("Edit"));
    edit_menu = edit_menu.entry(
        MenuItem::new(LocalizedString::new("Undo"))
            .hotkey(druid::RawMods::Ctrl, "z")
    );
    edit_menu = edit_menu.entry(
        MenuItem::new(LocalizedString::new("Redo"))
            .hotkey(druid::RawMods::Ctrl, "y")
    );
    edit_menu = edit_menu.separator();
    edit_menu = edit_menu.entry(
        MenuItem::new(LocalizedString::new("Cut"))
            .hotkey(druid::RawMods::Ctrl, "x")
    );
    edit_menu = edit_menu.entry(
        MenuItem::new(LocalizedString::new("Copy"))
            .hotkey(druid::RawMods::Ctrl, "c")
    );
    edit_menu = edit_menu.entry(
        MenuItem::new(LocalizedString::new("Paste"))
            .hotkey(druid::RawMods::Ctrl, "v")
    );
    edit_menu = edit_menu.separator();
    edit_menu = edit_menu.entry(
        MenuItem::new(LocalizedString::new("Delete"))
            .hotkey(druid::RawMods::None, druid::KbKey::Delete.into())
    );
    edit_menu = edit_menu.entry(
        MenuItem::new(LocalizedString::new("Select All"))
            .hotkey(druid::RawMods::Ctrl, "a")
    );
    menu = menu.entry(edit_menu);
    
    // View menu
    let mut view_menu = Menu::new(LocalizedString::new("View"));
    view_menu = view_menu.entry(
        MenuItem::new(LocalizedString::new("Toggle Grid"))
            .hotkey(druid::RawMods::Ctrl, "g")
    );
    view_menu = view_menu.entry(
        MenuItem::new(LocalizedString::new("Zoom In"))
            .hotkey(druid::RawMods::Ctrl, "+")
    );
    view_menu = view_menu.entry(
        MenuItem::new(LocalizedString::new("Zoom Out"))
            .hotkey(druid::RawMods::Ctrl, "-")
    );
    view_menu = view_menu.entry(
        MenuItem::new(LocalizedString::new("Reset Zoom"))
            .hotkey(druid::RawMods::Ctrl, "0")
    );
    view_menu = view_menu.separator();
    view_menu = view_menu.entry(
        MenuItem::new(LocalizedString::new("Toggle Sidebar"))
            .hotkey(druid::RawMods::Ctrl, "b")
    );
    view_menu = view_menu.entry(
        MenuItem::new(LocalizedString::new("Toggle Output Panel"))
            .hotkey(druid::RawMods::Ctrl, "`")
    );
    menu = menu.entry(view_menu);
    
    // Run menu
    let mut run_menu = Menu::new(LocalizedString::new("Run"));
    run_menu = run_menu.entry(
        MenuItem::new(LocalizedString::new("Run Program"))
            .hotkey(druid::RawMods::None, druid::KbKey::F5.into())
    );
    run_menu = run_menu.entry(
        MenuItem::new(LocalizedString::new("Debug Program"))
            .hotkey(druid::RawMods::Shift, druid::KbKey::F5.into())
    );
    run_menu = run_menu.separator();
    run_menu = run_menu.entry(
        MenuItem::new(LocalizedString::new("Stop Execution"))
            .hotkey(druid::RawMods::Shift, druid::KbKey::F5.into())
    );
    run_menu = run_menu.entry(
        MenuItem::new(LocalizedString::new("Generate Code"))
            .hotkey(druid::RawMods::CtrlShift, "g")
    );
    menu = menu.entry(run_menu);
    
    // Help menu
    let mut help_menu = Menu::new(LocalizedString::new("Help"));
    help_menu = help_menu.entry(
        MenuItem::new(LocalizedString::new("Documentation"))
            .hotkey(druid::RawMods::Ctrl, "d")
    );
    help_menu = help_menu.entry(
        MenuItem::new(LocalizedString::new("Keyboard Shortcuts"))
            .hotkey(druid::RawMods::Ctrl, "/")
    );
    help_menu = help_menu.separator();
    help_menu = help_menu.entry(
        MenuItem::new(LocalizedString::new("About 6IDE7"))
    );
    menu = menu.entry(help_menu);
    
    menu
}

fn main() -> Result<(), PlatformError> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create main window
    let main_window = WindowDesc::new(build_ui())
        .title("6IDE7 - Visual Programming IDE")
        .window_size(Size::new(1280.0, 800.0))
        .with_min_size(Size::new(800.0, 600.0))
        .menu(build_menu)
        .show_titlebar(true)
        .resizable(true);
    
    // Launch application
    AppLauncher::with_window(main_window)
        .configure_env(|env, _| {
            apply_theme(env);
        })
        .launch(AppState::default())?;
    
    Ok(())
}
