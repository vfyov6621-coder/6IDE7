//! 6IDE7 - Visual Block Programming IDE
//!
//! Main application entry point with code generation and execution.

use druid::{
    AppLauncher, Data, Env, Event, EventCtx, LocalizedString, Menu, MenuItem,
    PlatformError, Point, Size, Widget, WidgetExt, WindowDesc,
};
use ide_ui::{
    theme::apply_theme,
    canvas::{Canvas, CanvasState, DragState},
    toolbar::{Toolbar, ToolbarState, Tool},
    output::{OutputPanel, OutputState, OutputLine, OutputType},
    sidebar::{Sidebar, SidebarState},
    settings::{SettingsState},
    blocks::{BlockData, BlockLibrary, BlockDefinition},
    types::PortId,
    widgets::*,
    codegen::{generate_code, GeneratedCode, TargetLanguage},
    execution::{CodeExecutor, ExecutionResult, is_runtime_available},
    code_preview::{CodePreview, CodePreviewState},
};

/// Main application state
#[derive(Data, Clone)]
pub struct AppState {
    pub canvas: CanvasState,
    pub toolbar: ToolbarState,
    pub output: OutputState,
    pub sidebar: SidebarState,
    pub settings: SettingsState,
    pub code_preview: CodePreviewState,
    pub generated_code: Option<GeneratedCode>,
    pub is_executing: bool,
    pub window_title: String,
}

impl Default for AppState {
    fn default() -> Self {
        let mut canvas = CanvasState::default();
        
        // Add demo blocks to showcase
        canvas.blocks.push_back(
            BlockData::new("block_1", "Start", "control")
                .with_position(100.0, 100.0)
                .with_exec_output("exec", "Next")
        );
        
        canvas.blocks.push_back(
            BlockData::new("block_2", "Print", "io")
                .with_position(350.0, 80.0)
                .with_exec_input("exec", "Exec")
                .with_input("value", "Value", ide_ui::types::DataType::String)
                .with_exec_output("next", "Next")
        );
        
        canvas.blocks.push_back(
            BlockData::new("block_3", "Add", "math")
                .with_position(200.0, 250.0)
                .with_input("a", "A", ide_ui::types::DataType::Integer)
                .with_input("b", "B", ide_ui::types::DataType::Integer)
                .with_output("result", "Result", ide_ui::types::DataType::Integer)
        );
        
        // Expand some categories by default
        let mut sidebar_state = SidebarState::default();
        sidebar_state.expanded_categories.insert("io".to_string());
        sidebar_state.expanded_categories.insert("math".to_string());
        sidebar_state.expanded_categories.insert("control".to_string());
        
        Self {
            canvas,
            toolbar: ToolbarState::default(),
            output: OutputState::default(),
            sidebar: sidebar_state,
            settings: SettingsState::default(),
            code_preview: CodePreviewState::default(),
            generated_code: None,
            is_executing: false,
            window_title: "6IDE7 - Visual Programming IDE".to_string(),
        }
    }
}

impl AppState {
    /// Generate code from current blocks
    pub fn generate_code_from_blocks(&mut self) {
        let blocks: Vec<_> = self.canvas.blocks.iter().cloned().collect();
        let code = generate_code(
            self.code_preview.target_language,
            &blocks,
            &self.canvas.connections,
        );
        
        // Log warnings
        for warning in &code.warnings {
            self.output.warning(warning.clone());
        }
        
        self.generated_code = Some(code.clone());
        self.code_preview.code = Some(code);
    }
    
    /// Execute the generated code
    pub fn execute_code(&mut self) {
        // First generate code
        self.generate_code_from_blocks();
        
        if let Some(code) = &self.generated_code {
            let code = code.clone();
            self.is_executing = true;
            self.output.info(format!("Executing {} code...", code.language.display_name()));
            self.output.info("-".repeat(40));
            
            // Check if runtime is available
            if !is_runtime_available(code.language) {
                self.output.error(format!(
                    "{} runtime not found. Please install {}.",
                    code.language.display_name(),
                    match code.language {
                        TargetLanguage::Python => "Python 3",
                        TargetLanguage::JavaScript => "Node.js",
                        TargetLanguage::Rust => "Rust/Cargo",
                    }
                ));
                self.is_executing = false;
                return;
            }
            
            // Execute
            let executor = CodeExecutor::default();
            let result = executor.execute(&code);
            
            self.output.info(format!(
                "Execution time: {:.2?}",
                result.duration
            ));
            self.output.info("-".repeat(40));
            
            // Display output
            if !result.stdout.is_empty() {
                for line in result.stdout.lines() {
                    self.output.print(line);
                }
            }
            
            if !result.stderr.is_empty() {
                for line in result.stderr.lines() {
                    self.output.error(line);
                }
            }
            
            if result.success {
                self.output.info("Execution completed successfully.");
            } else if let Some(error) = &result.error {
                self.output.error(format!("Error: {}", error));
            }
            
            self.output.info("=".repeat(40));
            self.is_executing = false;
        } else {
            self.output.warning("No code to execute. Add some blocks first.");
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
                    data.output.print("Project saved successfully!");
                    ctx.request_paint();
                    ctx.set_handled();
                }
                (druid::KeyModifiers::CONTROL_SHIFT, druid::KbKey::Character(ref c)) if c == "G" => {
                    data.generate_code_from_blocks();
                    data.output.print(format!(
                        "Generated {} code ({} lines)",
                        data.code_preview.target_language.display_name(),
                        data.code_preview.code.as_ref().map(|c| c.line_count()).unwrap_or(0)
                    ));
                    ctx.request_paint();
                    ctx.set_handled();
                }
                (druid::KeyModifiers::NONE, druid::KbKey::F5) => {
                    data.execute_code();
                    ctx.request_paint();
                    ctx.set_handled();
                }
                (druid::KeyModifiers::CONTROL, druid::KbKey::Character(ref c)) if c == "g" => {
                    data.canvas.grid_visible = !data.canvas.grid_visible;
                    ctx.request_paint();
                    ctx.set_handled();
                }
                (druid::KeyModifiers::NONE, druid::KbKey::Delete) => {
                    if let Some(block_id) = &data.canvas.selected_block.clone() {
                        let block_name = data.canvas.get_block(block_id)
                            .map(|b| b.name.clone())
                            .unwrap_or_default();
                        data.canvas.remove_block(block_id);
                        data.output.print(format!("Deleted block: {}", block_name));
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
        
        // Handle sidebar block drag
        if data.sidebar.is_dragging_block {
            if let Some(block_type) = &data.sidebar.hovered_block_type.clone() {
                if let Some(definition) = BlockLibrary::get_definition(block_type) {
                    data.canvas.drag_state = DragState::NewBlock {
                        definition,
                        offset: Point::ZERO,
                    };
                }
            }
            data.sidebar.is_dragging_block = false;
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
    
    let code_preview = CodePreview::new()
        .lens(AppState::code_preview);
    
    let output = OutputPanel::new()
        .lens(AppState::output);
    
    let sidebar = Sidebar::new()
        .lens(AppState::sidebar);
    
    // Right panel: code preview + output
    let right_panel = v_split(
        code_preview,
        output,
    )
    .split_position(0.5)
    .min_sizes(150.0, 100.0);
    
    // Main content split: sidebar | canvas | right panel
    let main_content = h_split(
        sidebar,
        h_split(canvas, right_panel)
            .split_position(0.65)
            .min_sizes(400.0, 300.0),
    )
    .split_position(0.16)
    .min_sizes(180.0, 500.0);
    
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
        MenuItem::new(LocalizedString::new("Delete"))
    );
    edit_menu = edit_menu.separator();
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
    view_menu = view_menu.separator();
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
    menu = menu.entry(view_menu);
    
    // Run menu
    let mut run_menu = Menu::new(LocalizedString::new("Run"));
    run_menu = run_menu.entry(
        MenuItem::new(LocalizedString::new("Run Program"))
    );
    run_menu = run_menu.entry(
        MenuItem::new(LocalizedString::new("Generate Code"))
            .hotkey(druid::RawMods::CtrlShift, "G")
    );
    run_menu = run_menu.separator();
    run_menu = run_menu.entry(
        MenuItem::new(LocalizedString::new("Stop Execution"))
    );
    menu = menu.entry(run_menu);
    
    // Help menu
    let mut help_menu = Menu::new(LocalizedString::new("Help"));
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
    
    // Print runtime availability
    println!("6IDE7 - Visual Programming IDE");
    println!("Checking runtimes...");
    for lang in [TargetLanguage::Python, TargetLanguage::JavaScript, TargetLanguage::Rust] {
        let available = is_runtime_available(lang);
        println!("  {}: {}", lang.display_name(), if available { "✓ Available" } else { "✗ Not found" });
    }
    println!();
    
    // Create main window
    let main_window = WindowDesc::new(build_ui())
        .title("6IDE7 - Visual Programming IDE")
        .window_size(Size::new(1500.0, 900.0))
        .with_min_size(Size::new(1000.0, 700.0))
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
