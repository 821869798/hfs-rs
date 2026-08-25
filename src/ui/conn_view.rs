//! Active Connections View for HFS-RS.

use gpui::{Context, ElementId, FontWeight, IntoElement, div, prelude::*, px};

use crate::i18n::{self, Msg};
use crate::server::ConnInfo;
use crate::ui::theme::Theme;
use crate::util::{format_eta, format_speed};

pub struct ConnViewProps<'a> {
    pub theme: &'a Theme,
    pub connections: &'a [ConnInfo],
}

pub fn render_conn_pane<V: 'static>(
    props: ConnViewProps<'_>,
    _cx: &mut Context<V>,
) -> impl IntoElement {
    let t = props.theme;

    let mut table_rows = div().flex().flex_col().w_full().gap(px(1.0));

    if props.connections.is_empty() {
        table_rows = table_rows.child(
            div()
                .p(px(16.0))
                .text_size(px(12.0))
                .text_color(t.text_muted)
                .child(i18n::tr(Msg::ConnEmpty)),
        );
    } else {
        for (idx, conn) in props.connections.iter().enumerate() {
            let pct = (conn.progress * 100.0).clamp(0.0, 100.0) as u32;
            let eta = if conn.speed > 1.0 && conn.bytes_total > conn.bytes_sent {
                let remain = (conn.bytes_total - conn.bytes_sent) as f64;
                format_eta(Some(remain / conn.speed))
            } else {
                "-".to_string()
            };

            let (status_bg, status_color) = if conn.status.contains("Done") {
                (t.success_subtle, t.success)
            } else {
                (t.accent_subtle, t.accent)
            };

            table_rows = table_rows.child(
                div()
                    .id(ElementId::Name(format!("conn-row-{}", idx).into()))
                    .px(px(8.0))
                    .py(px(4.0))
                    .rounded(px(4.0))
                    .hover(|h| h.bg(t.hover_overlay))
                    .flex()
                    .items_center()
                    .text_size(px(12.0))
                    .gap(px(8.0))
                    .child(
                        div()
                            .w(px(120.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(t.text_primary)
                            .child(conn.peer.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(t.text_primary)
                            .overflow_x_hidden()
                            .text_ellipsis()
                            .child(conn.file.clone()),
                    )
                    .child(
                        div().w(px(85.0)).child(
                            div()
                                .px(px(5.0))
                                .py(px(1.0))
                                .rounded(px(3.0))
                                .bg(status_bg)
                                .text_size(px(10.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(status_color)
                                .child(conn.status.clone()),
                        ),
                    )
                    .child(
                        div()
                            .w(px(85.0))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(t.text_secondary)
                            .child(format_speed(conn.speed)),
                    )
                    .child(div().w(px(70.0)).text_color(t.text_muted).child(eta))
                    .child(
                        div()
                            .w(px(110.0))
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(
                                div()
                                    .flex_1()
                                    .h(px(6.0))
                                    .rounded(px(3.0))
                                    .bg(t.input_border)
                                    .overflow_x_hidden()
                                    .child(
                                        div()
                                            .h_full()
                                            .w(gpui::relative(conn.progress.clamp(0.0, 1.0)))
                                            .bg(t.accent),
                                    ),
                            )
                            .child(
                                div()
                                    .w(px(32.0))
                                    .text_size(px(11.0))
                                    .text_color(t.text_secondary)
                                    .child(format!("{}%", pct)),
                            ),
                    ),
            );
        }
    }

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(t.panel_bg)
        .border_t_1()
        .border_color(t.card_border)
        // Connections Header
        .child(
            div()
                .px(px(8.0))
                .py(px(5.0))
                .bg(t.header_bg)
                .border_b_1()
                .border_color(t.card_border)
                .flex()
                .items_center()
                .justify_between()
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
                                .child(i18n::tr(Msg::Connections)),
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
                                .child(format!("{}", props.connections.len())),
                        ),
                ),
        )
        // Table Column Titles
        .child(
            div()
                .px(px(8.0))
                .py(px(3.0))
                .bg(t.card_bg)
                .border_b_1()
                .border_color(t.card_border)
                .flex()
                .items_center()
                .text_size(px(11.0))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(t.text_muted)
                .gap(px(8.0))
                .child(div().w(px(120.0)).child(i18n::tr(Msg::IpAddress)))
                .child(div().flex_1().child(i18n::tr(Msg::File)))
                .child(div().w(px(85.0)).child(i18n::tr(Msg::Status)))
                .child(div().w(px(85.0)).child(i18n::tr(Msg::Speed)))
                .child(div().w(px(70.0)).child(i18n::tr(Msg::TimeLeft)))
                .child(div().w(px(110.0)).child(i18n::tr(Msg::Progress))),
        )
        // Table content
        .child(
            div()
                .id("conn-scroll-container")
                .flex_1()
                .min_h(px(0.0))
                .p(px(4.0))
                .overflow_y_scroll()
                .child(table_rows),
        )
}
