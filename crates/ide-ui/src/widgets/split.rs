//! Split panel widget for resizable sections

use druid::{
    BoxConstraints, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, UpdateCtx, Widget, WidgetPod,
};
use crate::theme::*;

/// A horizontal or vertical split between two panels
pub struct Split<T: Data> {
    first: WidgetPod<T, Box<dyn Widget<T>>>,
    second: WidgetPod<T, Box<dyn Widget<T>>>,
    split_position: f64,
    min_first: f64,
    min_second: f64,
    direction: SplitDirection,
    dragging: bool,
    handle_size: f64,
}

#[derive(Clone, Copy, PartialEq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

impl<T: Data> Split<T> {
    pub fn horizontal(first: impl Widget<T> + 'static, second: impl Widget<T> + 'static) -> Self {
        Self {
            first: WidgetPod::new(Box::new(first)),
            second: WidgetPod::new(Box::new(second)),
            split_position: 0.5,
            min_first: 100.0,
            min_second: 100.0,
            direction: SplitDirection::Horizontal,
            dragging: false,
            handle_size: 4.0,
        }
    }
    
    pub fn vertical(first: impl Widget<T> + 'static, second: impl Widget<T> + 'static) -> Self {
        Self {
            first: WidgetPod::new(Box::new(first)),
            second: WidgetPod::new(Box::new(second)),
            split_position: 0.5,
            min_first: 100.0,
            min_second: 100.0,
            direction: SplitDirection::Vertical,
            dragging: false,
            handle_size: 4.0,
        }
    }
    
    pub fn split_position(mut self, position: f64) -> Self {
        self.split_position = position.clamp(0.1, 0.9);
        self
    }
    
    pub fn min_sizes(mut self, first: f64, second: f64) -> Self {
        self.min_first = first;
        self.min_second = second;
        self
    }

    fn is_on_handle(&self, pos: Point, size: Size) -> bool {
        match self.direction {
            SplitDirection::Horizontal => {
                let handle_x = size.width * self.split_position;
                pos.x >= handle_x - 4.0 && pos.x <= handle_x + self.handle_size + 4.0
            }
            SplitDirection::Vertical => {
                let handle_y = size.height * self.split_position;
                pos.y >= handle_y - 4.0 && pos.y <= handle_y + self.handle_size + 4.0
            }
        }
    }
}

impl<T: Data> Widget<T> for Split<T> {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut T, env: &Env) {
        let size = ctx.size();
        
        match event {
            Event::MouseDown(mouse) => {
                if self.is_on_handle(mouse.pos, size) {
                    self.dragging = true;
                    ctx.set_active(true);
                    ctx.set_handled();
                }
            }
            Event::MouseUp(_) => {
                if self.dragging {
                    self.dragging = false;
                    ctx.set_active(false);
                }
            }
            Event::MouseMove(mouse) => {
                if self.dragging {
                    match self.direction {
                        SplitDirection::Horizontal => {
                            let new_pos = mouse.pos.x / size.width;
                            let min_ratio = self.min_first / size.width;
                            let max_ratio = 1.0 - self.min_second / size.width;
                            self.split_position = new_pos.clamp(min_ratio, max_ratio);
                        }
                        SplitDirection::Vertical => {
                            let new_pos = mouse.pos.y / size.height;
                            let min_ratio = self.min_first / size.height;
                            let max_ratio = 1.0 - self.min_second / size.height;
                            self.split_position = new_pos.clamp(min_ratio, max_ratio);
                        }
                    }
                    ctx.request_layout();
                    ctx.set_handled();
                } else {
                    // Update cursor
                    if self.is_on_handle(mouse.pos, size) {
                        ctx.set_cursor(match self.direction {
                            SplitDirection::Horizontal => druid::Cursor::ResizeLeftRight,
                            SplitDirection::Vertical => druid::Cursor::ResizeUpDown,
                        });
                    } else {
                        ctx.clear_cursor();
                    }
                }
            }
            _ => {}
        }
        
        // Forward events to children based on position
        let first_size = match self.direction {
            SplitDirection::Horizontal => Size::new(size.width * self.split_position - self.handle_size / 2.0, size.height),
            SplitDirection::Vertical => Size::new(size.width, size.height * self.split_position - self.handle_size / 2.0),
        };
        
        self.first.event(ctx, event, data, env);
        self.second.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &T, env: &Env) {
        self.first.lifecycle(ctx, event, data, env);
        self.second.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, _old_data: &T, data: &T, env: &Env) {
        self.first.update(ctx, data, env);
        self.second.update(ctx, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &T, env: &Env) -> Size {
        let size = bc.max();
        
        match self.direction {
            SplitDirection::Horizontal => {
                let first_width = size.width * self.split_position - self.handle_size / 2.0;
                let second_width = size.width * (1.0 - self.split_position) - self.handle_size / 2.0;
                
                let first_bc = BoxConstraints::new(
                    Size::new(self.min_first, 0.0),
                    Size::new(first_width, size.height),
                );
                let first_size = self.first.layout(ctx, &first_bc, data, env);
                self.first.set_origin(ctx, Point::ORIGIN);
                
                let second_bc = BoxConstraints::new(
                    Size::new(self.min_second, 0.0),
                    Size::new(second_width, size.height),
                );
                let second_size = self.second.layout(ctx, &second_bc, data, env);
                self.second.set_origin(ctx, Point::new(size.width * self.split_position + self.handle_size / 2.0, 0.0));
            }
            SplitDirection::Vertical => {
                let first_height = size.height * self.split_position - self.handle_size / 2.0;
                let second_height = size.height * (1.0 - self.split_position) - self.handle_size / 2.0;
                
                let first_bc = BoxConstraints::new(
                    Size::new(0.0, self.min_first),
                    Size::new(size.width, first_height),
                );
                let first_size = self.first.layout(ctx, &first_bc, data, env);
                self.first.set_origin(ctx, Point::ORIGIN);
                
                let second_bc = BoxConstraints::new(
                    Size::new(0.0, self.min_second),
                    Size::new(size.width, second_height),
                );
                let second_size = self.second.layout(ctx, &second_bc, data, env);
                self.second.set_origin(ctx, Point::new(0.0, size.height * self.split_position + self.handle_size / 2.0));
            }
        }
        
        size
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &T, env: &Env) {
        let size = ctx.size();
        
        // Paint children
        self.first.paint(ctx, data, env);
        self.second.paint(ctx, data, env);
        
        // Paint handle
        let handle_rect = match self.direction {
            SplitDirection::Horizontal => {
                Size::new(self.handle_size, size.height)
                    .to_rect()
                    .with_origin(Point::new(size.width * self.split_position - self.handle_size / 2.0, 0.0))
            }
            SplitDirection::Vertical => {
                Size::new(size.width, self.handle_size)
                    .to_rect()
                    .with_origin(Point::new(0.0, size.height * self.split_position - self.handle_size / 2.0))
            }
        };
        
        ctx.fill(handle_rect, &env.get(BORDER));
    }
}

/// Create a horizontal split
pub fn h_split<T: Data>(
    first: impl Widget<T> + 'static,
    second: impl Widget<T> + 'static,
) -> Split<T> {
    Split::horizontal(first, second)
}

/// Create a vertical split
pub fn v_split<T: Data>(
    first: impl Widget<T> + 'static,
    second: impl Widget<T> + 'static,
) -> Split<T> {
    Split::vertical(first, second)
}
