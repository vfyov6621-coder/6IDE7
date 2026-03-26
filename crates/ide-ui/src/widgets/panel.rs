//! Panel widget for IDE sections

use druid::{
    BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, RenderContext, Size, UpdateCtx, Widget, WidgetPod,
};
use crate::theme::*;

/// A panel with a title bar and content area
pub struct Panel<T: Data> {
    title: String,
    content: WidgetPod<T, Box<dyn Widget<T>>>,
    is_collapsed: bool,
    header_height: f64,
}

impl<T: Data> Panel<T> {
    pub fn new(title: impl Into<String>, content: impl Widget<T> + 'static) -> Self {
        Self {
            title: title.into(),
            content: WidgetPod::new(Box::new(content)),
            is_collapsed: false,
            header_height: 32.0,
        }
    }
    
    pub fn collapsed(mut self) -> Self {
        self.is_collapsed = true;
        self
    }
    
    pub fn header_height(mut self, height: f64) -> Self {
        self.header_height = height;
        self
    }
}

impl<T: Data> Widget<T> for Panel<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        // Handle double-click on header to collapse/expand
        if let Event::MouseDown(mouse) = event {
            if mouse.pos.y <= self.header_height && mouse.count == 2 {
                self.is_collapsed = !self.is_collapsed;
                ctx.request_layout();
                ctx.set_handled();
                return;
            }
        }
        
        if !self.is_collapsed {
            self.content.event(ctx, event, data, env);
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.content.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.content.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let width = bc.max().width;
        
        if self.is_collapsed {
            Size::new(width, self.header_height)
        } else {
            let content_bc = BoxConstraints::new(
                Size::new(0.0, 0.0),
                Size::new(width, bc.max().height - self.header_height),
            );
            let content_size = self.content.layout(ctx, &content_bc, data, env);
            self.content.set_origin(ctx, Point::new(0.0, self.header_height));
            
            Size::new(
                width,
                self.header_height + content_size.height,
            )
        }
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let size = ctx.size();
        let radius = env.get(BORDER_RADIUS);
        
        // Draw header background
        let header_rect = Size::new(size.width, self.header_height).to_rect();
        ctx.fill(header_rect, &env.get(SURFACE));
        
        // Draw header text
        let text_color = env.get(TEXT);
        ctx.text()
            .new_text_layout(self.title.clone())
            .font(druid::FontFamily::SYSTEM_UI, 13.0)
            .text_color(text_color)
            .build()
            .map(|layout| {
                let y_offset = (self.header_height - layout.size().height) / 2.0;
                ctx.draw_text(&layout, Point::new(12.0, y_offset));
            })
            .ok();
        
        // Draw collapse indicator
        let indicator = if self.is_collapsed { "▶" } else { "▼" };
        ctx.text()
            .new_text_layout(indicator.to_string())
            .font(druid::FontFamily::SYSTEM_UI, 10.0)
            .text_color(env.get(TEXT_SECONDARY))
            .build()
            .map(|layout| {
                let x_offset = size.width - 20.0;
                let y_offset = (self.header_height - layout.size().height) / 2.0;
                ctx.draw_text(&layout, Point::new(x_offset, y_offset));
            })
            .ok();
        
        // Draw border
        ctx.stroke(size.to_rect(), &env.get(BORDER), 1.0);
        
        // Paint content if not collapsed
        if !self.is_collapsed {
            self.content.paint(ctx, data, env);
        }
    }
}

use druid::Point;

/// A simple container panel with padding
pub struct Container<T: Data> {
    padding: f64,
    child: WidgetPod<T, Box<dyn Widget<T>>>,
}

impl<T: Data> Container<T> {
    pub fn new(child: impl Widget<T> + 'static) -> Self {
        Self {
            padding: 8.0,
            child: WidgetPod::new(Box::new(child)),
        }
    }
    
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = padding;
        self
    }
}

impl<T: Data> Widget<T> for Container<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        self.child.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.child.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let child_bc = bc.shrink((self.padding * 2.0, self.padding * 2.0));
        let child_size = self.child.layout(ctx, &child_bc, data, env);
        self.child.set_origin(ctx, Point::new(self.padding, self.padding));
        
        Size::new(
            child_size.width + self.padding * 2.0,
            child_size.height + self.padding * 2.0,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        self.child.paint(ctx, data, env);
    }
}

/// Create a panel with title
pub fn panel<T: Data>(title: impl Into<String>, content: impl Widget<T> + 'static) -> Panel<T> {
    Panel::new(title, content)
}

/// Create a container with padding
pub fn container<T: Data>(child: impl Widget<T> + 'static) -> Container<T> {
    Container::new(child)
}
