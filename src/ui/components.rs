//! Reusable, beautiful, pure GPUI components for HFS-RS.

use gpui::{
    AnyElement, Context, ElementId, FontWeight, IntoElement, MouseButton, SharedString, div,
    prelude::*, px,
};

use crate::ui::theme::Theme;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
    Danger,
    Success,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ButtonSize {
    Xs,
    Sm,
    Md,
}

pub struct CustomButton {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    variant: ButtonVariant,
    size: ButtonSize,
    disabled: bool,
    full_width: bool,
}

impl CustomButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            variant: ButtonVariant::Outline,
            size: ButtonSize::Sm,
            disabled: false,
            full_width: false,
        }
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn primary(mut self) -> Self {
        self.variant = ButtonVariant::Primary;
        self
    }

    pub fn secondary(mut self) -> Self {
        self.variant = ButtonVariant::Secondary;
        self
    }

    pub fn outline(mut self) -> Self {
        self.variant = ButtonVariant::Outline;
        self
    }

    pub fn ghost(mut self) -> Self {
        self.variant = ButtonVariant::Ghost;
        self
    }

    pub fn danger(mut self) -> Self {
        self.variant = ButtonVariant::Danger;
        self
    }

    pub fn success(mut self) -> Self {
        self.variant = ButtonVariant::Success;
        self
    }

    pub fn xs(mut self) -> Self {
        self.size = ButtonSize::Xs;
        self
    }

    pub fn sm(mut self) -> Self {
        self.size = ButtonSize::Sm;
        self
    }

    pub fn md(mut self) -> Self {
        self.size = ButtonSize::Md;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn full_width(mut self, full: bool) -> Self {
        self.full_width = full;
        self
    }

    pub fn render<V: 'static>(
        self,
        theme: &Theme,
        on_click: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
        cx: &mut Context<V>,
    ) -> AnyElement {
        let (bg, border, text_color, hover_bg, hover_border) = match self.variant {
            ButtonVariant::Primary => (
                theme.accent,
                theme.accent,
                crate::ui::theme::WHITE,
                theme.accent_hover,
                theme.accent_hover,
            ),
            ButtonVariant::Secondary => (
                theme.card_bg,
                theme.card_border,
                theme.text_primary,
                theme.card_hover,
                theme.card_border_hover,
            ),
            ButtonVariant::Outline => (
                crate::ui::theme::TRANSPARENT,
                theme.card_border,
                theme.text_primary,
                theme.hover_overlay,
                theme.card_border_hover,
            ),
            ButtonVariant::Ghost => (
                crate::ui::theme::TRANSPARENT,
                crate::ui::theme::TRANSPARENT,
                theme.text_primary,
                theme.hover_overlay,
                crate::ui::theme::TRANSPARENT,
            ),
            ButtonVariant::Danger => (
                theme.danger,
                theme.danger,
                crate::ui::theme::WHITE,
                theme.danger_subtle,
                theme.danger,
            ),
            ButtonVariant::Success => (
                theme.success,
                theme.success,
                crate::ui::theme::WHITE,
                theme.success_subtle,
                theme.success,
            ),
        };

        let (px_h, px_v, font_size, radius) = match self.size {
            ButtonSize::Xs => (px(6.0), px(2.0), px(11.0), px(4.0)),
            ButtonSize::Sm => (px(9.0), px(4.0), px(12.0), px(5.0)),
            ButtonSize::Md => (px(14.0), px(6.0), px(13.0), px(6.0)),
        };

        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .gap(px(5.0))
            .px(px_h)
            .py(px_v)
            .rounded(radius)
            .bg(bg)
            .border_1()
            .border_color(border)
            .text_size(font_size)
            .font_weight(FontWeight::MEDIUM)
            .text_color(if self.disabled {
                theme.text_muted
            } else {
                text_color
            });

        if self.full_width {
            el = el.w_full();
        }

        if self.disabled {
            el = el.opacity(0.5);
        } else {
            el = el
                .cursor_pointer()
                .hover(move |h| h.bg(hover_bg).border_color(hover_border))
                .active(move |a| a.bg(theme.active_overlay));

            let click_handler = on_click.clone();
            el = el.on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, window, cx| {
                    click_handler(this, window, cx);
                }),
            );
        }

        if let Some(icon) = self.icon {
            el = el.child(div().child(icon));
        }
        el = el.child(div().child(self.label));

        el.into_any_element()
    }
}

pub fn render_badge(
    text: impl Into<SharedString>,
    theme: &Theme,
    is_active: bool,
) -> impl IntoElement {
    let (bg, border, color) = if is_active {
        (theme.success_subtle, theme.success, theme.success)
    } else {
        (theme.card_bg, theme.card_border, theme.text_muted)
    };

    div()
        .flex()
        .items_center()
        .px(px(6.0))
        .py(px(1.5))
        .rounded(px(4.0))
        .bg(bg)
        .border_1()
        .border_color(border)
        .text_size(px(10.5))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(color)
        .child(text.into())
}

pub fn render_switch(theme: &Theme, on: bool) -> impl IntoElement {
    let track_bg = if on { theme.track_on } else { theme.track_off };

    div()
        .w(px(34.0))
        .h(px(18.0))
        .rounded(px(9.0))
        .bg(track_bg)
        .relative()
        .flex()
        .items_center()
        .p(px(2.0))
        .child(
            div()
                .size(px(14.0))
                .rounded(px(7.0))
                .bg(theme.thumb)
                .map(|s| if on { s.ml(px(16.0)) } else { s.ml(px(0.0)) }),
        )
}
