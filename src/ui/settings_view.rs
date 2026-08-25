//! Independent Settings View with Flyclip-style Cards and Dropdowns.

use std::sync::Arc;

use gpui::{
    AnyElement, Context, ElementId, Entity, FontWeight, IntoElement, MouseButton, SharedString,
    Window, div, prelude::*, px,
};

use crate::i18n::{self, Locale, Msg};
use crate::server::AppState;
use crate::ui::components::{CustomButton, render_switch};
use crate::ui::text_input::TextInput;
use crate::ui::theme::{Theme, ThemeMode};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    General,
    Server,
    Security,
    About,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsDropdownKind {
    Language,
    Theme,
    Mode,
    UploadPolicy,
    GraphDisplay,
}

pub struct SettingsViewProps<'a> {
    pub theme: &'a Theme,
    pub theme_mode: ThemeMode,
    pub locale: Locale,
    pub expert_mode: bool,
    pub show_graph: bool,
    pub state: &'a Arc<AppState>,
    pub active_tab: SettingsTab,
    pub open_dropdown: Option<SettingsDropdownKind>,
    pub port_input: &'a Entity<TextInput>,
    pub max_conn_input: &'a Entity<TextInput>,
    pub upload_max_input: &'a Entity<TextInput>,
    pub new_user_input: &'a Entity<TextInput>,
    pub new_pass_input: &'a Entity<TextInput>,
}

pub fn render_settings_overlay<V: 'static>(
    props: SettingsViewProps<'_>,
    on_close: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_switch_tab: impl Fn(&mut V, SettingsTab, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_dropdown: impl Fn(&mut V, Option<SettingsDropdownKind>, &mut Window, &mut Context<V>)
    + 'static
    + Clone,
    on_set_locale: impl Fn(&mut V, Locale, &mut Window, &mut Context<V>) + 'static + Clone,
    on_set_theme: impl Fn(&mut V, ThemeMode, &mut Window, &mut Context<V>) + 'static + Clone,
    on_set_expert_mode: impl Fn(&mut V, bool, &mut Window, &mut Context<V>) + 'static + Clone,
    on_set_show_graph: impl Fn(&mut V, bool, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_auto_copy: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_send_id: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_browse_local: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_set_upload_policy: impl Fn(&mut V, (bool, bool), &mut Window, &mut Context<V>) + 'static + Clone,
    on_save_server_cfg: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_add_account: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_remove_account: impl Fn(&mut V, usize, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_account: impl Fn(&mut V, usize, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> AnyElement {
    let t = props.theme;
    let close_for_backdrop = on_close.clone();

    let close_cb = on_close.clone();

    div()
        .id("settings-modal-backdrop")
        .absolute()
        .inset_0()
        .bg(gpui::rgba(0x00000088))
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, w, cx| {
                close_for_backdrop(this, w, cx);
            }),
        )
        .child(
            div()
                .id("settings-modal-card")
                .w(px(720.0))
                .h(px(540.0))
                .bg(t.panel_bg)
                .border_1()
                .border_color(t.card_border)
                .rounded(px(12.0))
                .shadow_xl()
                .flex()
                .flex_col()
                .overflow_x_hidden()
                .overflow_y_hidden()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                // Header
                .child(
                    div()
                        .px(px(20.0))
                        .py(px(14.0))
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
                                .gap(px(10.0))
                                .child(
                                    div()
                                        .size(px(32.0))
                                        .rounded(px(8.0))
                                        .bg(t.accent_subtle)
                                        .border_1()
                                        .border_color(t.accent)
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(px(16.0))
                                        .child("⚙️"),
                                )
                                .child(
                                    div()
                                        .text_size(px(16.0))
                                        .font_weight(FontWeight::BOLD)
                                        .text_color(t.text_primary)
                                        .child(i18n::tr(Msg::Options)),
                                ),
                        )
                        .child(
                            div()
                                .id("btn-settings-close")
                                .cursor_pointer()
                                .px(px(8.0))
                                .py(px(4.0))
                                .rounded(px(6.0))
                                .hover(|h| h.bg(t.hover_overlay))
                                .text_size(px(14.0))
                                .text_color(t.text_secondary)
                                .on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, w, cx| {
                                        close_cb(this, w, cx);
                                    }),
                                )
                                .child("✕"),
                        ),
                )
                // Body: Sidebar Tabs + Content Area
                .child(
                    div()
                        .flex_1()
                        .min_h(px(0.0))
                        .flex()
                        // Left Tabs
                        .child(
                            div()
                                .w(px(180.0))
                                .h_full()
                                .bg(t.header_bg)
                                .border_r_1()
                                .border_color(t.card_border)
                                .p(px(8.0))
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(render_tab_button(
                                    SettingsTab::General,
                                    i18n::tr(Msg::TabGeneral),
                                    "⚙️",
                                    props.active_tab == SettingsTab::General,
                                    t,
                                    on_switch_tab.clone(),
                                    cx,
                                ))
                                .child(render_tab_button(
                                    SettingsTab::Server,
                                    i18n::tr(Msg::TabServer),
                                    "🌐",
                                    props.active_tab == SettingsTab::Server,
                                    t,
                                    on_switch_tab.clone(),
                                    cx,
                                ))
                                .child(render_tab_button(
                                    SettingsTab::Security,
                                    i18n::tr(Msg::TabSecurity),
                                    "🔒",
                                    props.active_tab == SettingsTab::Security,
                                    t,
                                    on_switch_tab.clone(),
                                    cx,
                                ))
                                .child(render_tab_button(
                                    SettingsTab::About,
                                    i18n::tr(Msg::TabAbout),
                                    "ℹ️",
                                    props.active_tab == SettingsTab::About,
                                    t,
                                    on_switch_tab.clone(),
                                    cx,
                                )),
                        )
                        // Right Tab Content
                        .child(
                            div()
                                .id("settings-tab-content-scroll")
                                .flex_1()
                                .min_w(px(0.0))
                                .h_full()
                                .p(px(20.0))
                                .overflow_y_scroll()
                                .child(match props.active_tab {
                                    SettingsTab::General => render_general_tab(
                                        &props,
                                        on_toggle_dropdown.clone(),
                                        on_set_locale.clone(),
                                        on_set_theme.clone(),
                                        on_set_expert_mode.clone(),
                                        on_toggle_auto_copy.clone(),
                                        on_toggle_send_id.clone(),
                                        on_toggle_browse_local.clone(),
                                        cx,
                                    )
                                    .into_any_element(),
                                    SettingsTab::Server => render_server_tab(
                                        &props,
                                        on_toggle_dropdown.clone(),
                                        on_set_show_graph.clone(),
                                        on_save_server_cfg.clone(),
                                        cx,
                                    )
                                    .into_any_element(),
                                    SettingsTab::Security => render_security_tab(
                                        &props,
                                        on_toggle_dropdown.clone(),
                                        on_set_upload_policy.clone(),
                                        on_add_account.clone(),
                                        on_remove_account.clone(),
                                        on_toggle_account.clone(),
                                        cx,
                                    )
                                    .into_any_element(),
                                    SettingsTab::About => render_about_tab(t).into_any_element(),
                                }),
                        ),
                )
                // Footer
                .child(
                    div()
                        .w_full()
                        .px(px(20.0))
                        .py(px(12.0))
                        .bg(t.header_bg)
                        .border_t_1()
                        .border_color(t.card_border)
                        .flex()
                        .items_center()
                        .justify_end()
                        .child(
                            CustomButton::new("btn-settings-done", i18n::tr(Msg::Done))
                                .primary()
                                .render(t, on_close, cx),
                        ),
                ),
        )
        .into_any_element()
}

fn render_tab_button<V: 'static>(
    tab: SettingsTab,
    label: &str,
    icon: &str,
    is_active: bool,
    theme: &Theme,
    on_switch: impl Fn(&mut V, SettingsTab, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = theme;
    let bg = if is_active {
        t.accent_subtle
    } else {
        crate::ui::theme::TRANSPARENT
    };
    let border = if is_active {
        t.accent
    } else {
        crate::ui::theme::TRANSPARENT
    };
    let text_color = if is_active {
        t.accent
    } else {
        t.text_secondary
    };

    div()
        .id(ElementId::Name(format!("tab-btn-{:?}", tab).into()))
        .px(px(10.0))
        .py(px(8.0))
        .rounded(px(6.0))
        .bg(bg)
        .border_1()
        .border_color(border)
        .cursor_pointer()
        .hover(|h| if !is_active { h.bg(t.hover_overlay) } else { h })
        .flex()
        .items_center()
        .gap(px(8.0))
        .text_size(px(13.0))
        .font_weight(if is_active {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        })
        .text_color(text_color)
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, w, cx| {
                on_switch(this, tab, w, cx);
            }),
        )
        .child(div().text_size(px(14.0)).child(icon.to_string()))
        .child(div().child(label.to_string()))
}

fn render_card_container(theme: &Theme, children: Vec<AnyElement>) -> impl IntoElement {
    let t = theme;
    let mut col = div()
        .flex()
        .flex_col()
        .p(px(14.0))
        .rounded(px(10.0))
        .bg(t.card_bg)
        .border_1()
        .border_color(t.card_border);

    for (idx, row) in children.into_iter().enumerate() {
        if idx > 0 {
            col = col.child(div().my(px(10.0)).h(px(1.0)).w_full().bg(t.card_border));
        }
        col = col.child(row);
    }

    col
}

fn render_dropdown_selector<V: 'static>(
    id_str: &'static str,
    is_open: bool,
    current_icon: &'static str,
    current_label: impl Into<SharedString>,
    options: Vec<(&'static str, String, &'static str)>,
    theme: &Theme,
    on_toggle: impl Fn(&mut V, Option<SettingsDropdownKind>, &mut Window, &mut Context<V>)
    + 'static
    + Clone,
    kind: SettingsDropdownKind,
    on_select: impl Fn(&mut V, &'static str, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = theme;
    let label_str = current_label.into();
    let tog_cb = on_toggle.clone();

    let trigger = div()
        .id(ElementId::Name(format!("dd-trig-{}", id_str).into()))
        .h(px(32.0))
        .w(px(210.0))
        .px(px(10.0))
        .rounded(px(6.0))
        .bg(t.input_bg)
        .border_1()
        .border_color(if is_open { t.accent } else { t.input_border })
        .hover(|h| h.border_color(t.card_border_hover))
        .cursor_pointer()
        .flex()
        .items_center()
        .justify_between()
        .gap(px(8.0))
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, w, cx| {
                cx.stop_propagation();
                tog_cb(this, if is_open { None } else { Some(kind) }, w, cx);
            }),
        )
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.0))
                .child(div().text_size(px(13.0)).child(current_icon.to_string()))
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(t.text_primary)
                        .child(label_str),
                ),
        )
        .child(
            div()
                .text_size(px(10.0))
                .text_color(t.text_muted)
                .child(if is_open { "▲" } else { "▼" }),
        );

    let mut container = div()
        .id(ElementId::Name(format!("dd-cnt-{}", id_str).into()))
        .relative()
        .child(trigger);

    if is_open {
        let mut menu = div()
            .id(ElementId::Name(format!("dd-menu-{}", id_str).into()))
            .w(px(210.0))
            .p(px(4.0))
            .rounded(px(8.0))
            .bg(t.card_bg)
            .border_1()
            .border_color(t.card_border)
            .shadow_lg()
            .flex()
            .flex_col()
            .gap(px(2.0));

        for (opt_key, opt_label, opt_icon) in options {
            let sel_cb = on_select.clone();
            menu = menu.child(
                div()
                    .id(ElementId::Name(
                        format!("dd-opt-{}-{}", id_str, opt_key).into(),
                    ))
                    .px(px(8.0))
                    .py(px(6.0))
                    .rounded(px(5.0))
                    .hover(|h| h.bg(t.hover_overlay))
                    .cursor_pointer()
                    .flex()
                    .items_center()
                    .justify_between()
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(move |this, _, w, cx| {
                            cx.stop_propagation();
                            sel_cb(this, opt_key, w, cx);
                        }),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.0))
                            .child(div().text_size(px(13.0)).child(opt_icon.to_string()))
                            .child(
                                div()
                                    .text_size(px(12.0))
                                    .text_color(t.text_primary)
                                    .child(opt_label),
                            ),
                    ),
            );
        }

        container = container.child(
            gpui::deferred(
                div()
                    .absolute()
                    .top_full()
                    .left_0()
                    .mt(px(4.0))
                    .w_full()
                    .child(menu),
            )
            .with_priority(1),
        );
    }

    container
}

fn render_general_tab<V: 'static>(
    props: &SettingsViewProps<'_>,
    on_toggle_dropdown: impl Fn(&mut V, Option<SettingsDropdownKind>, &mut Window, &mut Context<V>)
    + 'static
    + Clone,
    on_set_locale: impl Fn(&mut V, Locale, &mut Window, &mut Context<V>) + 'static + Clone,
    on_set_theme: impl Fn(&mut V, ThemeMode, &mut Window, &mut Context<V>) + 'static + Clone,
    on_set_expert_mode: impl Fn(&mut V, bool, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_auto_copy: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_send_id: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_browse_local: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = props.theme;
    let cfg = props.state.config.read().clone();

    let (lang_icon, lang_label) = match props.locale {
        Locale::System => ("🖥️", i18n::tr(Msg::LangSystem)),
        Locale::Zh => ("🇨🇳", i18n::tr(Msg::LangZh)),
        Locale::En => ("🇺🇸", i18n::tr(Msg::LangEn)),
    };

    let (theme_icon, theme_label) = match props.theme_mode {
        ThemeMode::System => ("🖥️", i18n::tr(Msg::ThemeSystem)),
        ThemeMode::Dark => ("🌙", i18n::tr(Msg::ThemeDark)),
        ThemeMode::Light => ("☀️", i18n::tr(Msg::ThemeLight)),
    };

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        // Appearance & Localization Card
        .child(render_card_container(
            t,
            vec![
                // Language Dropdown Row
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(i18n::tr(Msg::Language)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.text_muted)
                                    .child(i18n::tr(Msg::SelectLanguageDesc)),
                            ),
                    )
                    .child({
                        let lang_cb = on_set_locale.clone();
                        render_dropdown_selector(
                            "lang",
                            props.open_dropdown == Some(SettingsDropdownKind::Language),
                            lang_icon,
                            lang_label,
                            vec![
                                ("system", i18n::tr(Msg::LangSystem).to_string(), "🖥️"),
                                ("zh", i18n::tr(Msg::LangZh).to_string(), "🇨🇳"),
                                ("en", i18n::tr(Msg::LangEn).to_string(), "🇺🇸"),
                            ],
                            t,
                            on_toggle_dropdown.clone(),
                            SettingsDropdownKind::Language,
                            move |this, key, w, cx| {
                                let loc = match key {
                                    "system" => Locale::System,
                                    "zh" => Locale::Zh,
                                    _ => Locale::En,
                                };
                                lang_cb(this, loc, w, cx);
                            },
                            cx,
                        )
                    })
                    .into_any_element(),
                // Theme Dropdown Row
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(i18n::tr(Msg::ThemeAppearance)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.text_muted)
                                    .child(i18n::tr(Msg::ThemeAppearanceDesc)),
                            ),
                    )
                    .child({
                        let theme_cb = on_set_theme.clone();
                        render_dropdown_selector(
                            "theme",
                            props.open_dropdown == Some(SettingsDropdownKind::Theme),
                            theme_icon,
                            theme_label,
                            vec![
                                ("system", i18n::tr(Msg::ThemeSystem).to_string(), "🖥️"),
                                ("dark", i18n::tr(Msg::ThemeDark).to_string(), "🌙"),
                                ("light", i18n::tr(Msg::ThemeLight).to_string(), "☀️"),
                            ],
                            t,
                            on_toggle_dropdown.clone(),
                            SettingsDropdownKind::Theme,
                            move |this, key, w, cx| {
                                let mode = match key {
                                    "system" => ThemeMode::System,
                                    "light" => ThemeMode::Light,
                                    _ => ThemeMode::Dark,
                                };
                                theme_cb(this, mode, w, cx);
                            },
                            cx,
                        )
                    })
                    .into_any_element(),
                // Mode Dropdown Row
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(i18n::tr(Msg::Mode)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.text_muted)
                                    .child(i18n::tr(Msg::ModeDesc)),
                            ),
                    )
                    .child({
                        let exp_cb = on_set_expert_mode.clone();
                        render_dropdown_selector(
                            "mode",
                            props.open_dropdown == Some(SettingsDropdownKind::Mode),
                            if props.expert_mode { "⚡" } else { "🟢" },
                            if props.expert_mode {
                                i18n::tr(Msg::ExpertMode)
                            } else {
                                i18n::tr(Msg::EasyMode)
                            },
                            vec![
                                ("easy", i18n::tr(Msg::EasyMode).to_string(), "🟢"),
                                ("expert", i18n::tr(Msg::ExpertMode).to_string(), "⚡"),
                            ],
                            t,
                            on_toggle_dropdown.clone(),
                            SettingsDropdownKind::Mode,
                            move |this, key, w, cx| {
                                exp_cb(this, key == "expert", w, cx);
                            },
                            cx,
                        )
                    })
                    .into_any_element(),
            ],
        ))
        // Convenience Switches Card
        .child(render_card_container(
            t,
            vec![
                // Auto Copy
                render_switch_row(
                    i18n::tr(Msg::AutoCopyUrl),
                    i18n::tr(Msg::AutoCopyUrlDesc),
                    cfg.auto_copy_url_on_add,
                    t,
                    on_toggle_auto_copy,
                    cx,
                ),
                // Send HFS Identifier
                render_switch_row(
                    i18n::tr(Msg::SendHfsId),
                    i18n::tr(Msg::SendHfsIdDesc),
                    cfg.send_server_header,
                    t,
                    on_toggle_send_id,
                    cx,
                ),
                // Browse using Localhost
                render_switch_row(
                    i18n::tr(Msg::BrowseLocalhost),
                    i18n::tr(Msg::BrowseLocalhostDesc),
                    cfg.open_in_browser_use_localhost,
                    t,
                    on_toggle_browse_local,
                    cx,
                ),
            ],
        ))
}

fn render_server_tab<V: 'static>(
    props: &SettingsViewProps<'_>,
    on_toggle_dropdown: impl Fn(&mut V, Option<SettingsDropdownKind>, &mut Window, &mut Context<V>)
    + 'static
    + Clone,
    on_set_show_graph: impl Fn(&mut V, bool, &mut Window, &mut Context<V>) + 'static + Clone,
    on_save: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = props.theme;

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        // Server Network Card
        .child(render_card_container(
            t,
            vec![
                // Port input row
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(i18n::tr(Msg::Port)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.text_muted)
                                    .child(i18n::tr(Msg::PortDesc)),
                            ),
                    )
                    .child(div().w(px(140.0)).child(render_input_field(
                        "settings-port-in",
                        props.port_input,
                        t,
                        cx,
                    )))
                    .into_any_element(),
                // Max connections row
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(i18n::tr(Msg::MaxConnections)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.text_muted)
                                    .child(i18n::tr(Msg::MaxConnectionsDesc)),
                            ),
                    )
                    .child(div().w(px(140.0)).child(render_input_field(
                        "settings-conn-in",
                        props.max_conn_input,
                        t,
                        cx,
                    )))
                    .into_any_element(),
                // Max upload size row
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(i18n::tr(Msg::MaxUploadSize)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.text_muted)
                                    .child(i18n::tr(Msg::MaxUploadSizeDesc)),
                            ),
                    )
                    .child(div().w(px(140.0)).child(render_input_field(
                        "settings-upload-in",
                        props.upload_max_input,
                        t,
                        cx,
                    )))
                    .into_any_element(),
            ],
        ))
        // Bandwidth Graph Dropdown Card
        .child(render_card_container(
            t,
            vec![
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(i18n::tr(Msg::BandwidthGraph)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.text_muted)
                                    .child(i18n::tr(Msg::BandwidthGraphDesc)),
                            ),
                    )
                    .child({
                        let g_cb = on_set_show_graph.clone();
                        render_dropdown_selector(
                            "graph-disp",
                            props.open_dropdown == Some(SettingsDropdownKind::GraphDisplay),
                            "📊",
                            if props.show_graph {
                                i18n::tr(Msg::ShowBandwidthGraph)
                            } else {
                                i18n::tr(Msg::HideBandwidthGraph)
                            },
                            vec![
                                ("show", i18n::tr(Msg::ShowBandwidthGraph).to_string(), "📊"),
                                ("hide", i18n::tr(Msg::HideBandwidthGraph).to_string(), "🙈"),
                            ],
                            t,
                            on_toggle_dropdown.clone(),
                            SettingsDropdownKind::GraphDisplay,
                            move |this, key, w, cx| {
                                g_cb(this, key == "show", w, cx);
                            },
                            cx,
                        )
                    })
                    .into_any_element(),
            ],
        ))
        .child(
            div().flex().justify_end().child(
                CustomButton::new("btn-save-server-cfg", i18n::tr(Msg::ApplyServerSettings))
                    .primary()
                    .render(t, on_save, cx),
            ),
        )
}

fn render_security_tab<V: 'static>(
    props: &SettingsViewProps<'_>,
    on_toggle_dropdown: impl Fn(&mut V, Option<SettingsDropdownKind>, &mut Window, &mut Context<V>)
    + 'static
    + Clone,
    on_set_upload_policy: impl Fn(&mut V, (bool, bool), &mut Window, &mut Context<V>) + 'static + Clone,
    on_add_account: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    on_remove_account: impl Fn(&mut V, usize, &mut Window, &mut Context<V>) + 'static + Clone,
    on_toggle_account: impl Fn(&mut V, usize, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = props.theme;
    let cfg = props.state.config.read().clone();

    let (current_policy_label, current_policy_icon) = if !cfg.allow_upload {
        (i18n::tr(Msg::UploadDisabled), "⛔")
    } else if cfg.protect_uploads {
        (i18n::tr(Msg::UploadProtected), "🔐")
    } else {
        (i18n::tr(Msg::UploadPublic), "🔓")
    };

    let accounts = cfg.accounts.clone();

    let mut accounts_rows = div().flex().flex_col().gap(px(4.0));

    if accounts.is_empty() {
        accounts_rows = accounts_rows.child(
            div()
                .p(px(8.0))
                .rounded(px(6.0))
                .bg(t.input_bg)
                .text_size(px(12.0))
                .text_color(t.text_muted)
                .child(i18n::tr(Msg::NoAccounts)),
        );
    } else {
        for (idx, acc) in accounts.iter().enumerate() {
            let tog_cb = on_toggle_account.clone();
            let rem_cb = on_remove_account.clone();

            accounts_rows = accounts_rows.child(
                div()
                    .id(ElementId::Name(format!("settings-acc-row-{}", idx).into()))
                    .px(px(8.0))
                    .py(px(5.0))
                    .rounded(px(6.0))
                    .bg(t.input_bg)
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .text_size(px(13.0))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(acc.name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(t.text_muted)
                                    .child("••••••••"),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.0))
                            .child(
                                div()
                                    .cursor_pointer()
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, w, cx| {
                                            tog_cb(this, idx, w, cx);
                                        }),
                                    )
                                    .child(render_switch(t, acc.enabled)),
                            )
                            .child(
                                div()
                                    .cursor_pointer()
                                    .px(px(6.0))
                                    .py(px(2.0))
                                    .rounded(px(4.0))
                                    .hover(|h| h.bg(t.danger_subtle))
                                    .text_size(px(12.0))
                                    .text_color(t.danger)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(move |this, _, w, cx| {
                                            rem_cb(this, idx, w, cx);
                                        }),
                                    )
                                    .child("🗑️"),
                            ),
                    ),
            );
        }
    }

    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        // Upload Policy Card
        .child(render_card_container(
            t,
            vec![
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.0))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(t.text_primary)
                                    .child(i18n::tr(Msg::UploadPolicy)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(t.text_muted)
                                    .child(i18n::tr(Msg::UploadPolicyDesc)),
                            ),
                    )
                    .child({
                        let pol_cb = on_set_upload_policy.clone();
                        render_dropdown_selector(
                            "upload-policy",
                            props.open_dropdown == Some(SettingsDropdownKind::UploadPolicy),
                            current_policy_icon,
                            current_policy_label,
                            vec![
                                ("disable", i18n::tr(Msg::UploadDisabled).to_string(), "⛔"),
                                ("public", i18n::tr(Msg::UploadPublic).to_string(), "🔓"),
                                ("protect", i18n::tr(Msg::UploadProtected).to_string(), "🔐"),
                            ],
                            t,
                            on_toggle_dropdown.clone(),
                            SettingsDropdownKind::UploadPolicy,
                            move |this, key, w, cx| {
                                let pol = match key {
                                    "public" => (true, false),
                                    "protect" => (true, true),
                                    _ => (false, false),
                                };
                                pol_cb(this, pol, w, cx);
                            },
                            cx,
                        )
                    })
                    .into_any_element(),
            ],
        ))
        // User Accounts Management Card
        .child(
            div()
                .flex()
                .flex_col()
                .p(px(14.0))
                .rounded(px(10.0))
                .bg(t.card_bg)
                .border_1()
                .border_color(t.card_border)
                .gap(px(10.0))
                .child(
                    div()
                        .text_size(px(13.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(t.text_primary)
                        .child(i18n::tr(Msg::AccountsList)),
                )
                .child(accounts_rows)
                .child(div().my(px(4.0)).h(px(1.0)).w_full().bg(t.card_border))
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(t.text_secondary)
                        .child(i18n::tr(Msg::AddNewAccount)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.0))
                        .child(div().flex_1().child(render_input_field(
                            "settings-acc-user",
                            props.new_user_input,
                            t,
                            cx,
                        )))
                        .child(div().flex_1().child(render_input_field(
                            "settings-acc-pass",
                            props.new_pass_input,
                            t,
                            cx,
                        )))
                        .child(
                            CustomButton::new("btn-add-acc-done", i18n::tr(Msg::Add))
                                .primary()
                                .render(t, on_add_account, cx),
                        ),
                ),
        )
}

fn render_about_tab(theme: &Theme) -> impl IntoElement {
    let t = theme;
    div()
        .flex()
        .flex_col()
        .gap(px(16.0))
        .child(
            div()
                .p(px(20.0))
                .rounded(px(10.0))
                .bg(t.card_bg)
                .border_1()
                .border_color(t.card_border)
                .flex()
                .flex_col()
                .items_center()
                .gap(px(8.0))
                .child(
                    div()
                        .size(px(48.0))
                        .rounded(px(12.0))
                        .bg(t.accent_subtle)
                        .border_1()
                        .border_color(t.accent)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(24.0))
                        .child("🌐"),
                )
                .child(
                    div()
                        .text_size(px(18.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(t.text_primary)
                        .child("HFS-RS"),
                )
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(t.text_secondary)
                        .child(i18n::tr(Msg::AboutDesc)),
                )
                .child(
                    div()
                        .px(px(8.0))
                        .py(px(2.0))
                        .rounded(px(4.0))
                        .bg(t.input_bg)
                        .border_1()
                        .border_color(t.card_border)
                        .text_size(px(11.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(t.text_muted)
                        .child(i18n::tr(Msg::VersionInfo)),
                ),
        )
        .child(render_card_container(
            t,
            vec![
                render_info_row(i18n::tr(Msg::CoreEngine), i18n::tr(Msg::CoreEngineDesc), t),
                render_info_row(
                    i18n::tr(Msg::UiFramework),
                    i18n::tr(Msg::UiFrameworkDesc),
                    t,
                ),
                render_info_row(
                    i18n::tr(Msg::Compatibility),
                    i18n::tr(Msg::CompatibilityDesc),
                    t,
                ),
                render_info_row(i18n::tr(Msg::License), i18n::tr(Msg::LicenseDesc), t),
            ],
        ))
}

fn render_info_row(title: &str, value: &str, theme: &Theme) -> AnyElement {
    let t = theme;
    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(12.5))
                .font_weight(FontWeight::MEDIUM)
                .text_color(t.text_secondary)
                .child(title.to_string()),
        )
        .child(
            div()
                .text_size(px(12.0))
                .text_color(t.text_primary)
                .child(value.to_string()),
        )
        .into_any_element()
}

fn render_switch_row<V: 'static>(
    title: impl Into<SharedString>,
    desc: &str,
    on: bool,
    theme: &Theme,
    on_toggle: impl Fn(&mut V, &mut Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> AnyElement {
    let t = theme;
    let title_str = title.into();
    let tog_cb = on_toggle.clone();

    div()
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(2.0))
                .child(
                    div()
                        .text_size(px(13.5))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(t.text_primary)
                        .child(title_str),
                )
                .child(
                    div()
                        .text_size(px(11.5))
                        .text_color(t.text_muted)
                        .child(desc.to_string()),
                ),
        )
        .child(
            div()
                .cursor_pointer()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, w, cx| {
                        tog_cb(this, w, cx);
                    }),
                )
                .child(render_switch(t, on)),
        )
        .into_any_element()
}

fn render_input_field<V: 'static>(
    id_str: &'static str,
    input: &Entity<TextInput>,
    theme: &Theme,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let focus_handle = input.read(cx).focus_handle.clone();
    let input_for_click = input.clone();
    let input_for_key = input.clone();

    div()
        .id(ElementId::Name(id_str.into()))
        .track_focus(&focus_handle)
        .w_full()
        .h(px(32.0))
        .px(px(8.0))
        .rounded(px(6.0))
        .bg(theme.input_bg)
        .border_1()
        .border_color(theme.input_border)
        .cursor_text()
        .flex()
        .items_center()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |_this, event: &gpui::MouseDownEvent, window, cx| {
                input_for_click.update(cx, |inp, cx| {
                    inp.focus_handle.focus(window, cx);
                    inp.start_blink(cx);
                    inp.on_mouse_down(event.position, cx);
                });
            }),
        )
        .on_key_down(
            cx.listener(move |_this, event: &gpui::KeyDownEvent, window, cx| {
                input_for_key.update(cx, |inp, cx| {
                    inp.handle_key_down(event, window, cx);
                });
            }),
        )
        .child(input.clone())
}
