//! Log view pane for HFS-RS.

use gpui::{
    Context, ElementId, Entity, FontWeight, IntoElement, SharedString, div, prelude::*, px,
};

use crate::i18n::{self, Msg};
use crate::server::LogLevel;
use crate::ui::components::CustomButton;
use crate::ui::text_input::TextInput;
use crate::ui::theme::Theme;

#[derive(Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub time: SharedString,
    pub text: SharedString,
}

pub struct LogViewProps<'a> {
    pub theme: &'a Theme,
    pub search_input: &'a Entity<TextInput>,
    pub logs: &'a [LogEntry],
}

pub fn render_log_pane<V: 'static>(
    props: LogViewProps<'_>,
    on_clear: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_copy_all: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = props.theme;
    let query = props.search_input.read(cx).text().trim().to_lowercase();

    let filtered_logs: Vec<&LogEntry> = if query.is_empty() {
        props.logs.iter().collect()
    } else {
        props
            .logs
            .iter()
            .filter(|l| l.text.to_lowercase().contains(&query))
            .collect()
    };

    let mut list = div().flex().flex_col().w_full().gap(px(1.0));

    if filtered_logs.is_empty() {
        list = list.child(
            div()
                .p(px(16.0))
                .text_size(px(12.0))
                .text_color(t.text_muted)
                .child(i18n::tr(Msg::NoLogs)),
        );
    } else {
        for (idx, entry) in filtered_logs.iter().enumerate() {
            let (badge_bg, badge_border, badge_color, badge_text) = match entry.level {
                LogLevel::Info => (t.accent_subtle, t.accent, t.accent, "INFO"),
                LogLevel::Http => (t.success_subtle, t.success, t.success, "HTTP"),
                LogLevel::Warn => (t.warning_subtle, t.warning, t.warning, "WARN"),
                LogLevel::Error => (t.danger_subtle, t.danger, t.danger, "ERR"),
            };

            list = list.child(
                div()
                    .id(ElementId::Name(format!("log-entry-{}", idx).into()))
                    .px(px(8.0))
                    .py(px(3.0))
                    .rounded(px(4.0))
                    .hover(|h| h.bg(t.hover_overlay))
                    .flex()
                    .items_start()
                    .gap(px(8.0))
                    .text_size(px(12.0))
                    .child(
                        div()
                            .text_size(px(11.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(t.text_muted)
                            .child(entry.time.clone()),
                    )
                    .child(
                        div()
                            .px(px(4.0))
                            .py(px(0.5))
                            .rounded(px(3.0))
                            .bg(badge_bg)
                            .border_1()
                            .border_color(badge_border)
                            .text_size(px(9.5))
                            .font_weight(FontWeight::BOLD)
                            .text_color(badge_color)
                            .child(badge_text),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(t.text_primary)
                            .child(entry.text.clone()),
                    ),
            );
        }
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(t.panel_bg)
        // Log header
        .child(
            div()
                .px(px(8.0))
                .py(px(6.0))
                .bg(t.header_bg)
                .border_b_1()
                .border_color(t.card_border)
                .flex()
                .items_center()
                .justify_between()
                .gap(px(8.0))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.0))
                        .child(
                            div()
                                .text_size(px(12.5))
                                .font_weight(FontWeight::BOLD)
                                .text_color(t.text_primary)
                                .child(i18n::tr(Msg::Log)),
                        )
                        .child(
                            div()
                                .px(px(5.0))
                                .py(px(1.0))
                                .rounded(px(4.0))
                                .bg(t.card_bg)
                                .border_1()
                                .border_color(t.card_border)
                                .text_size(px(10.0))
                                .text_color(t.text_muted)
                                .child(format!("{}", props.logs.len())),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(4.0))
                        // Search bar
                        .child({
                            let search_entity = props.search_input.clone();
                            let search_key = props.search_input.clone();
                            let search_focus_handle =
                                props.search_input.read(cx).focus_handle.clone();

                            div()
                                .id("log-search-box")
                                .track_focus(&search_focus_handle)
                                .w(px(160.0))
                                .h(px(26.0))
                                .px(px(6.0))
                                .rounded(px(4.0))
                                .bg(t.input_bg)
                                .border_1()
                                .border_color(t.input_border)
                                .cursor_text()
                                .flex()
                                .items_center()
                                .on_mouse_down(
                                    gpui::MouseButton::Left,
                                    cx.listener(
                                        move |_this, event: &gpui::MouseDownEvent, window, cx| {
                                            search_entity.update(cx, |inp, cx| {
                                                inp.focus_handle.focus(window, cx);
                                                inp.start_blink(cx);
                                                inp.on_mouse_down(event.position, cx);
                                            });
                                        },
                                    ),
                                )
                                .on_key_down(cx.listener(
                                    move |_this, event: &gpui::KeyDownEvent, window, cx| {
                                        search_key.update(cx, |inp, cx| {
                                            inp.handle_key_down(event, window, cx);
                                        });
                                    },
                                ))
                                .child(props.search_input.clone())
                        })
                        .child(
                            CustomButton::new("btn-log-copy", i18n::tr(Msg::CopyAll))
                                .xs()
                                .outline()
                                .render(t, on_copy_all, cx),
                        )
                        .child(
                            CustomButton::new("btn-log-clear", i18n::tr(Msg::ClearLog))
                                .xs()
                                .outline()
                                .render(t, on_clear, cx),
                        ),
                ),
        )
        // Log lines scroll container
        .child(
            div()
                .id("log-scroll-container")
                .flex_1()
                .min_h(px(0.0))
                .p(px(6.0))
                .overflow_y_scroll()
                .child(list),
        )
}
