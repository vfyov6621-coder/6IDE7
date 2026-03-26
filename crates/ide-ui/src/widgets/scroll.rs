//! Scrollable container widget

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget, WidgetPod,
};
use crate::theme::*;

/// A scrollable container
pub struct ScrollArea<T: Data> {
    child: WidgetPod<T, Box<dyn Widget<T>>>,
    scroll_offset: Point,
    content_size: Size,
    viewport_size: Size,
}

impl<T: Data> ScrollArea<T> {
    pub fn new(child: impl Widget<T> + 'static) -> Self {
        Self {
            child: WidgetPod::new(Box::new(child)),
            scroll_offset: Point::ZERO,
            content_size: Size::ZERO,
            viewport_size: Size::ZERO,
        }
    }
}

impl<T: Data> Widget<T> for ScrollArea<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        match event {
            Event::Wheel(wheel) => {
                // Scroll content
                let delta = wheel.wheel_delta;
                self.scroll_offset.y = (self.scroll_offset.y - delta.y)
                    .max(0.0)
                    .min((self.content_size.height - self.viewport_size.height).max(0.0));
                self.scroll_offset.x = (self.scroll_offset.x - delta.x)
                    .max(0.0)
                    .min((self.content_size.width - self.viewport_size.width).max(0.0));
                ctx.request_paint();
                ctx.set_handled();
            }
            _ => {}
        }
        
        // Transform mouse events for child
        self.child.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.child.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.child.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        self.viewport_size = bc.max();
        
        // Give child unbounded constraints
        let child_bc = BoxConstraints::new(
            Size::new(bc.min().width, 0.0),
            Size::new(bc.max().width, f64::INFINITY),
        );
        self.content_size = self.child.layout(ctx, &child_bc, data, env);
        self.child.set_origin(ctx, Point::new(-self.scroll_offset.x, -self.scroll_offset.y));
        
        self.viewport_size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        // Clip to viewport
        ctx.clip(self.viewport_size.to_rect());
        
        // Paint child
        self.child.paint(ctx, data, env);
        
        // Draw scrollbar if content is larger than viewport
        if self.content_size.height > self.viewport_size.height {
            let scrollbar_width = 8.0;
            let scrollbar_margin = 2.0;
            let track_height = self.viewport_size.height - scrollbar_margin * 2.0;
            
            // Calculate scrollbar position and size
            let content_ratio = self.viewport_size.height / self.content_size.height;
            let thumb_height = (track_height * content_ratio).max(20.0);
            let thumb_y = scrollbar_margin + (self.scroll_offset.y / self.content_size.height) * track_height;
            
            // Draw track
            let track_rect = Size::new(scrollbar_width, track_height)
                .to_rect()
                .with_origin(Point::new(
                    self.viewport_size.width - scrollbar_width - scrollbar_margin,
                    scrollbar_margin,
                ));
            ctx.fill(track_rect, &env.get(BORDER).with_alpha(0.3));
            
            // Draw thumb
            let thumb_rect = Size::new(scrollbar_width, thumb_height)
                .to_rect()
                .with_origin(Point::new(
                    self.viewport_size.width - scrollbar_width - scrollbar_margin,
                    thumb_y,
                ));
            ctx.fill(thumb_rect, &env.get(ACCENT).with_alpha(0.6));
        }
    }
}

/// Create a scroll area
pub fn scroll_area<T: Data>(child: impl Widget<T> + 'static) -> ScrollArea<T> {
    ScrollArea::new(child)
}
