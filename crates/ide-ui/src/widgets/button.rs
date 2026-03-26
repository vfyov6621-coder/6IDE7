//! Styled button widget for 6IDE7

use druid::{
    widget::{Controller, Label},
    BoxConstraints, Color, Data, Env, Event, EventCtx, LayoutCtx, LifeCycle,
    LifeCycleCtx, PaintCtx, Point, RenderContext, Size, TimerToken, UpdateCtx,
    Widget, WidgetExt,
};
use std::time::{Duration, Instant};

use crate::theme::*;

/// A styled button with hover and press effects
pub struct StyledButton {
    label: Label<String>,
    is_hovered: bool,
    is_pressed: bool,
    animation_start: Option<Instant>,
    animation_duration: Duration,
}

impl StyledButton {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            label: Label::new(text.into()),
            is_hovered: false,
            is_pressed: false,
            animation_start: None,
            animation_duration: Duration::from_millis(150),
        }
    }
    
    /// Create an icon button with a symbol
    pub fn icon(symbol: &str) -> Self {
        Self::new(symbol.to_string())
    }
    
    /// Calculate animation progress (0.0 to 1.0)
    fn animation_progress(&self) -> f64 {
        self.animation_start
            .map(|start| {
                let elapsed = start.elapsed().as_millis() as f64;
                let duration = self.animation_duration.as_millis() as f64;
                (elapsed / duration).min(1.0)
            })
            .unwrap_or(1.0)
    }
}

impl Widget<String> for StyledButton {
    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut String, env: &Env) {
        match event {
            Event::MouseMove(_) => {
                if ctx.is_hot() && !self.is_hovered {
                    self.is_hovered = true;
                    self.animation_start = Some(Instant::now());
                    ctx.request_anim_frame();
                    ctx.request_paint();
                } else if !ctx.is_hot() && self.is_hovered {
                    self.is_hovered = false;
                    self.is_pressed = false;
                    self.animation_start = Some(Instant::now());
                    ctx.request_anim_frame();
                    ctx.request_paint();
                }
            }
            Event::MouseDown(_) => {
                if ctx.is_hot() {
                    self.is_pressed = true;
                    ctx.request_paint();
                }
            }
            Event::MouseUp(_) => {
                if self.is_pressed && ctx.is_hot() {
                    ctx.submit_notification(druid::Notification::new("button_click"));
                }
                self.is_pressed = false;
                ctx.request_paint();
            }
            Event::AnimFrame(_) => {
                if self.animation_progress() < 1.0 {
                    ctx.request_anim_frame();
                    ctx.request_paint();
                }
            }
            _ => {}
        }
        self.label.event(ctx, event, data, env);
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &String, env: &Env) {
        self.label.lifecycle(ctx, event, data, env);
    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &String, data: &String, env: &Env) {
        self.label.update(ctx, old_data, data, env);
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &String, env: &Env) -> Size {
        let label_size = self.label.layout(ctx, bc, data, env);
        let padding = 16.0;
        Size::new(
            label_size.width + padding * 2.0,
            label_size.height + padding * 0.75,
        )
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &String, env: &Env) {
        let size = ctx.size();
        let rect = size.to_rect();
        let radius = env.get(BORDER_RADIUS);
        
        // Background color based on state
        let bg_color = if self.is_pressed {
            env.get(ACCENT).with_alpha(0.8)
        } else if self.is_hovered {
            env.get(SURFACE_HOVER)
        } else {
            env.get(SURFACE)
        };
        
        // Draw rounded rectangle background
        ctx.fill(rounded_rect(rect, radius), &bg_color);
        
        // Draw border
        let border_color = if self.is_hovered || self.is_pressed {
            env.get(ACCENT)
        } else {
            env.get(BORDER)
        };
        ctx.stroke(rounded_rect(rect, radius), &border_color, 1.0);
        
        // Paint label centered
        self.label.paint(ctx, data, env);
    }
}

/// Create a rounded rect path
fn rounded_rect(rect: druid::Rect, radius: f64) -> druid::piet::RoundedRectShape {
    druid::piet::RoundedRectShape::from_rect(rect, radius)
}

/// Controller for button click handling
pub struct ButtonController<F> {
    callback: F,
}

impl<F> ButtonController<F> {
    pub fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F, T: Data> Controller<String, StyledButton> for ButtonController<F>
where
    F: Fn(&mut EventCtx, &mut String, &Env),
{
    fn event(
        &mut self,
        child: &mut StyledButton,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut String,
        env: &Env,
    ) {
        if let Event::Notification(notification) = event {
            if notification.is("button_click") {
                (self.callback)(ctx, data, env);
                ctx.set_handled();
                return;
            }
        }
        child.event(ctx, event, data, env);
    }
}

/// Create a styled button with click handler
pub fn styled_button(text: impl Into<String>) -> StyledButton {
    StyledButton::new(text)
}

/// Create an icon button
pub fn icon_button(icon: &str) -> StyledButton {
    StyledButton::icon(icon)
}
