//! Real-time Bandwidth Graph and Speed Meters for HFS-RS.

use std::collections::VecDeque;

use gpui::{Context, ElementId, FontWeight, IntoElement, MouseButton, div, prelude::*, px};

use crate::i18n::{self, Msg};
use crate::ui::theme::Theme;
use crate::util::format_speed;

#[derive(Clone, Copy)]
pub struct GraphPoint {
    pub out_bps: f64,
    pub in_bps: f64,
}

pub struct GraphViewProps<'a> {
    pub theme: &'a Theme,
    pub show_graph: bool,
    pub out_bps: f64,
    pub in_bps: f64,
    pub history: &'a VecDeque<GraphPoint>,
}

pub fn render_graph_pane<V: 'static>(
    props: GraphViewProps<'_>,
    on_toggle: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = props.theme;

    if !props.show_graph {
        return div()
            .w_full()
            .h(px(24.0))
            .px(px(8.0))
            .bg(t.header_bg)
            .border_b_1()
            .border_color(t.card_border)
            .flex()
            .items_center()
            .justify_between()
            .text_size(px(11.5))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.0))
                    .child(
                        div()
                            .font_weight(FontWeight::BOLD)
                            .text_color(t.text_secondary)
                            .child(i18n::tr(Msg::BandwidthGraph)),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(t.graph_out)
                            .child(format!("OUT {}", format_speed(props.out_bps))),
                    )
                    .child(div().text_color(t.text_muted).child("|"))
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(t.graph_in)
                            .child(format!("IN {}", format_speed(props.in_bps))),
                    ),
            )
            .child(
                div()
                    .cursor_pointer()
                    .px(px(6.0))
                    .py(px(1.0))
                    .rounded(px(3.0))
                    .hover(|h| h.bg(t.hover_overlay))
                    .text_size(px(11.0))
                    .text_color(t.accent)
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, w, cx| {
                            on_toggle(this, w, cx);
                        }),
                    )
                    .child("▼ Expand Graph"),
            )
            .into_any_element();
    }

    let max_bps = props
        .history
        .iter()
        .map(|p| p.out_bps.max(p.in_bps))
        .fold(1024.0_f64, |a, b| a.max(b));

    let mut bars = div()
        .w_full()
        .h(px(46.0))
        .flex()
        .items_end()
        .gap(px(2.0))
        .px(px(8.0))
        .py(px(4.0));

    if props.history.is_empty() {
        bars = bars.child(
            div()
                .text_size(px(11.0))
                .text_color(t.text_muted)
                .child(i18n::tr(Msg::GraphHint)),
        );
    } else {
        for (i, p) in props.history.iter().enumerate() {
            let out_h = ((p.out_bps / max_bps) * 38.0).clamp(2.0, 38.0) as f32;
            let in_h = ((p.in_bps / max_bps) * 38.0).clamp(1.0, 38.0) as f32;

            bars = bars.child(
                div()
                    .id(ElementId::Name(format!("bar-pair-{}", i).into()))
                    .flex()
                    .items_end()
                    .gap(px(1.0))
                    .child(
                        div()
                            .w(px(4.0))
                            .h(px(out_h))
                            .rounded(px(1.0))
                            .bg(t.graph_out),
                    )
                    .child(div().w(px(4.0)).h(px(in_h)).rounded(px(1.0)).bg(t.graph_in)),
            );
        }
    }

    div()
        .w_full()
        .bg(t.panel_bg)
        .border_b_1()
        .border_color(t.card_border)
        .flex()
        .flex_col()
        // Top status bar for graph
        .child(
            div()
                .px(px(8.0))
                .py(px(3.0))
                .bg(t.header_bg)
                .flex()
                .items_center()
                .justify_between()
                .text_size(px(11.5))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(
                            div()
                                .font_weight(FontWeight::BOLD)
                                .text_color(t.text_primary)
                                .child(i18n::tr(Msg::BandwidthGraph)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .child(div().size(px(7.0)).rounded(px(1.0)).bg(t.graph_out))
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(t.graph_out)
                                        .child(format!("OUT: {}", format_speed(props.out_bps))),
                                ),
                        )
                        .child(div().text_color(t.text_muted).child("|"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(4.0))
                                .child(div().size(px(7.0)).rounded(px(1.0)).bg(t.graph_in))
                                .child(
                                    div()
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(t.graph_in)
                                        .child(format!("IN: {}", format_speed(props.in_bps))),
                                ),
                        ),
                )
                .child(
                    div()
                        .cursor_pointer()
                        .px(px(6.0))
                        .py(px(1.0))
                        .rounded(px(3.0))
                        .hover(|h| h.bg(t.hover_overlay))
                        .text_size(px(11.0))
                        .text_color(t.text_secondary)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, w, cx| {
                                on_toggle(this, w, cx);
                            }),
                        )
                        .child("▲ Collapse Graph"),
                ),
        )
        .child(bars)
        .into_any_element()
}
