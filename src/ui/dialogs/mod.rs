//! Modal Dialogs for HFS-RS: New Folder, Rename, New Link, Properties, Options, Accounts.

use std::sync::Arc;

use gpui::{
    AnyElement, ColorExt, Context, ElementId, Entity, FontWeight, MouseButton, SharedString, div,
    prelude::*, px,
};

use crate::i18n::{self, Msg};
use crate::server::AppState;
use crate::ui::components::{CustomButton, render_switch};
use crate::ui::text_input::TextInput;
use crate::ui::theme::Theme;
use crate::vfs::{NodeId, NodeKind};

pub enum DialogKind {
    None,
    NewFolder {
        parent: NodeId,
        input: Entity<TextInput>,
    },
    Rename {
        id: NodeId,
        input: Entity<TextInput>,
    },
    NewLink {
        parent: NodeId,
        name_input: Entity<TextInput>,
        url_input: Entity<TextInput>,
    },
    Properties {
        id: NodeId,
    },
    Options {
        port_input: Entity<TextInput>,
        max_conn_input: Entity<TextInput>,
        upload_max_input: Entity<TextInput>,
    },
    Accounts {
        user_input: Entity<TextInput>,
        pass_input: Entity<TextInput>,
    },
}

impl DialogKind {
    pub fn is_open(&self) -> bool {
        !matches!(self, DialogKind::None)
    }
}

pub fn render_dialog_overlay<V: 'static>(
    dialog: &DialogKind,
    theme: &Theme,
    state: &Arc<AppState>,
    _selected_id: Option<NodeId>,
    on_close: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_confirm_new_folder: impl Fn(&mut V, NodeId, String, &mut gpui::Window, &mut Context<V>)
    + 'static
    + Clone,
    on_confirm_rename: impl Fn(&mut V, NodeId, String, &mut gpui::Window, &mut Context<V>)
    + 'static
    + Clone,
    on_confirm_new_link: impl Fn(&mut V, NodeId, String, String, &mut gpui::Window, &mut Context<V>)
    + 'static
    + Clone,
    on_save_options: impl Fn(
        &mut V,
        u16,
        Option<usize>,
        u64,
        bool,
        bool,
        bool,
        bool,
        bool,
        &mut gpui::Window,
        &mut Context<V>,
    )
    + 'static
    + Clone,
    on_add_account: impl Fn(&mut V, String, String, &mut gpui::Window, &mut Context<V>)
    + 'static
    + Clone,
    on_remove_account: impl Fn(&mut V, usize, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_toggle_account: impl Fn(&mut V, usize, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> AnyElement {
    if !dialog.is_open() {
        return div().into_any_element();
    }

    let t = theme;
    let close_bg = on_close.clone();

    let mut modal_content = div()
        .w(px(460.0))
        .max_w(px(600.0))
        .rounded(px(10.0))
        .bg(t.panel_bg)
        .border_1()
        .border_color(t.card_border)
        .shadow_lg()
        .flex()
        .flex_col();

    match dialog {
        DialogKind::None => return div().into_any_element(),

        DialogKind::NewFolder { parent, input } => {
            let parent_id = *parent;
            let input_entity = input.clone();
            let confirm_cb = on_confirm_new_folder.clone();
            let close_cb = on_close.clone();

            modal_content = modal_content
                .child(render_modal_header(
                    i18n::tr(Msg::NewEmptyFolder),
                    t,
                    close_cb.clone(),
                    cx,
                ))
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(t.text_secondary)
                                .child(i18n::tr(Msg::RenamePrompt)),
                        )
                        .child(render_input_box(
                            "dlg-new-folder-input",
                            &input_entity,
                            t,
                            cx,
                        )),
                )
                .child(
                    render_modal_footer(t)
                        .child(
                            CustomButton::new("btn-cancel", i18n::tr(Msg::Cancel))
                                .outline()
                                .render(t, move |v, w, cx| close_cb(v, w, cx), cx),
                        )
                        .child(
                            CustomButton::new("btn-ok", i18n::tr(Msg::ConfirmRename))
                                .primary()
                                .render(
                                    t,
                                    move |v, w, cx| {
                                        let text = input_entity.read(cx).text().trim().to_string();
                                        if !text.is_empty() {
                                            confirm_cb(v, parent_id, text, w, cx);
                                        }
                                    },
                                    cx,
                                ),
                        ),
                );
        }

        DialogKind::Rename { id, input } => {
            let target_id = *id;
            let input_entity = input.clone();
            let confirm_cb = on_confirm_rename.clone();
            let close_cb = on_close.clone();

            modal_content = modal_content
                .child(render_modal_header(
                    i18n::tr(Msg::Rename),
                    t,
                    close_cb.clone(),
                    cx,
                ))
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(
                            div()
                                .text_size(px(12.5))
                                .text_color(t.text_secondary)
                                .child(i18n::tr(Msg::RenamePrompt)),
                        )
                        .child(render_input_box("dlg-rename-input", &input_entity, t, cx)),
                )
                .child(
                    render_modal_footer(t)
                        .child(
                            CustomButton::new("btn-cancel", i18n::tr(Msg::Cancel))
                                .outline()
                                .render(t, move |v, w, cx| close_cb(v, w, cx), cx),
                        )
                        .child(
                            CustomButton::new("btn-ok", i18n::tr(Msg::ConfirmRename))
                                .primary()
                                .render(
                                    t,
                                    move |v, w, cx| {
                                        let text = input_entity.read(cx).text().trim().to_string();
                                        if !text.is_empty() {
                                            confirm_cb(v, target_id, text, w, cx);
                                        }
                                    },
                                    cx,
                                ),
                        ),
                );
        }

        DialogKind::NewLink {
            parent,
            name_input,
            url_input,
        } => {
            let parent_id = *parent;
            let name_entity = name_input.clone();
            let url_entity = url_input.clone();
            let confirm_cb = on_confirm_new_link.clone();
            let close_cb = on_close.clone();

            modal_content = modal_content
                .child(render_modal_header(
                    i18n::tr(Msg::NewLink),
                    t,
                    close_cb.clone(),
                    cx,
                ))
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(t.text_secondary)
                                        .child(i18n::tr(Msg::RenamePrompt)),
                                )
                                .child(render_input_box(
                                    "dlg-link-name-input",
                                    &name_entity,
                                    t,
                                    cx,
                                )),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.0))
                                .child(
                                    div()
                                        .text_size(px(12.0))
                                        .text_color(t.text_secondary)
                                        .child(i18n::tr(Msg::LinkPrompt)),
                                )
                                .child(render_input_box("dlg-link-url-input", &url_entity, t, cx)),
                        ),
                )
                .child(
                    render_modal_footer(t)
                        .child(
                            CustomButton::new("btn-cancel", i18n::tr(Msg::Cancel))
                                .outline()
                                .render(t, move |v, w, cx| close_cb(v, w, cx), cx),
                        )
                        .child(
                            CustomButton::new("btn-ok", i18n::tr(Msg::ConfirmLink))
                                .primary()
                                .render(
                                    t,
                                    move |v, w, cx| {
                                        let name = name_entity.read(cx).text().trim().to_string();
                                        let url = url_entity.read(cx).text().trim().to_string();
                                        if !url.is_empty() {
                                            confirm_cb(v, parent_id, name, url, w, cx);
                                        }
                                    },
                                    cx,
                                ),
                        ),
                );
        }

        DialogKind::Properties { id } => {
            let close_cb = on_close.clone();
            let vfs = state.vfs.read();
            let node_opt = vfs.get(*id);

            let (name, kind_str, path_str, comment_str, url_str) = if let Some(node) = node_opt {
                let kind_label = match node.kind {
                    NodeKind::Root => i18n::tr(Msg::KindRoot),
                    NodeKind::VirtualFolder => i18n::tr(Msg::KindVirtual),
                    NodeKind::RealFolder => i18n::tr(Msg::KindRealFolder),
                    NodeKind::File => i18n::tr(Msg::KindFile),
                    NodeKind::Link => i18n::tr(Msg::KindLink),
                };
                let res = node
                    .resource
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| "-".into());
                let comm = if node.comment.is_empty() {
                    "-".into()
                } else {
                    node.comment.clone()
                };
                let url = vfs.url_path(*id);
                (node.display_name().to_string(), kind_label, res, comm, url)
            } else {
                ("-".into(), "-", "-".into(), "-".into(), "-".into())
            };

            modal_content = modal_content
                .child(render_modal_header(
                    i18n::tr(Msg::PropertiesTitle),
                    t,
                    close_cb.clone(),
                    cx,
                ))
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(10.0))
                        .child(render_prop_row(i18n::tr(Msg::File), name, t))
                        .child(render_prop_row(i18n::tr(Msg::Status), kind_str, t))
                        .child(render_prop_row("URL", url_str, t))
                        .child(render_prop_row(i18n::tr(Msg::Resource), path_str, t))
                        .child(render_prop_row(i18n::tr(Msg::Comment), comment_str, t)),
                )
                .child(
                    render_modal_footer(t).child(
                        CustomButton::new("btn-close", i18n::tr(Msg::Close))
                            .primary()
                            .render(t, move |v, w, cx| close_cb(v, w, cx), cx),
                    ),
                );
        }

        DialogKind::Options {
            port_input,
            max_conn_input,
            upload_max_input,
        } => {
            let close_cb = on_close.clone();
            let save_cb = on_save_options.clone();
            let port_ent = port_input.clone();
            let max_conn_ent = max_conn_input.clone();
            let upload_max_ent = upload_max_input.clone();

            let cfg = state.config.read().clone();

            modal_content = modal_content
                .w(px(520.0))
                .child(render_modal_header(
                    i18n::tr(Msg::OtherOptions),
                    t,
                    close_cb.clone(),
                    cx,
                ))
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(12.0))
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(t.text_primary)
                                        .child(i18n::tr(Msg::Port)),
                                )
                                .child(div().w(px(120.0)).child(render_input_box(
                                    "dlg-opt-port-input",
                                    &port_ent,
                                    t,
                                    cx,
                                ))),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(12.0))
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(t.text_primary)
                                        .child("Max Connections"),
                                )
                                .child(div().w(px(120.0)).child(render_input_box(
                                    "dlg-opt-conn-input",
                                    &max_conn_ent,
                                    t,
                                    cx,
                                ))),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .gap(px(12.0))
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(t.text_primary)
                                        .child("Max Upload Size (MB)"),
                                )
                                .child(div().w(px(120.0)).child(render_input_box(
                                    "dlg-opt-upload-input",
                                    &upload_max_ent,
                                    t,
                                    cx,
                                ))),
                        )
                        .child(div().my(px(4.0)).h(px(1.0)).w_full().bg(t.card_border))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(t.text_primary)
                                        .child(i18n::tr(Msg::AutoCopyUrl)),
                                )
                                .child(render_switch(t, cfg.auto_copy_url_on_add)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(t.text_primary)
                                        .child(i18n::tr(Msg::SendHfsId)),
                                )
                                .child(render_switch(t, cfg.send_server_header)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(t.text_primary)
                                        .child(i18n::tr(Msg::BrowseLocalhost)),
                                )
                                .child(render_switch(t, cfg.open_in_browser_use_localhost)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(t.text_primary)
                                        .child(i18n::tr(Msg::Upload)),
                                )
                                .child(render_switch(t, cfg.allow_upload)),
                        )
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_between()
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(t.text_primary)
                                        .child(i18n::tr(Msg::ProtectUploads)),
                                )
                                .child(render_switch(t, cfg.protect_uploads)),
                        ),
                )
                .child(
                    render_modal_footer(t)
                        .child(
                            CustomButton::new("btn-opt-cancel", i18n::tr(Msg::Cancel))
                                .outline()
                                .render(t, move |v, w, cx| close_cb(v, w, cx), cx),
                        )
                        .child(
                            CustomButton::new("btn-opt-save", i18n::tr(Msg::Save))
                                .primary()
                                .render(
                                    t,
                                    move |v, w, cx| {
                                        let port_val = port_ent
                                            .read(cx)
                                            .text()
                                            .trim()
                                            .parse::<u16>()
                                            .unwrap_or(cfg.port);
                                        let max_conn = max_conn_ent
                                            .read(cx)
                                            .text()
                                            .trim()
                                            .parse::<usize>()
                                            .ok();
                                        let max_upload = upload_max_ent
                                            .read(cx)
                                            .text()
                                            .trim()
                                            .parse::<u64>()
                                            .unwrap_or(cfg.upload_max_mb);

                                        save_cb(
                                            v,
                                            port_val,
                                            max_conn,
                                            max_upload,
                                            cfg.auto_copy_url_on_add,
                                            cfg.send_server_header,
                                            cfg.open_in_browser_use_localhost,
                                            cfg.allow_upload,
                                            cfg.protect_uploads,
                                            w,
                                            cx,
                                        );
                                    },
                                    cx,
                                ),
                        ),
                );
        }

        DialogKind::Accounts {
            user_input,
            pass_input,
        } => {
            let close_cb = on_close.clone();
            let add_cb = on_add_account.clone();
            let remove_cb = on_remove_account.clone();
            let toggle_cb = on_toggle_account.clone();
            let user_ent = user_input.clone();
            let pass_ent = pass_input.clone();

            let accounts = state.config.read().accounts.clone();

            let mut accounts_list = div()
                .id("accounts-scroll-list")
                .flex()
                .flex_col()
                .gap(px(6.0))
                .max_h(px(180.0))
                .overflow_y_scroll();

            if accounts.is_empty() {
                accounts_list = accounts_list.child(
                    div()
                        .p(px(8.0))
                        .rounded(px(6.0))
                        .bg(t.card_bg)
                        .text_size(px(12.0))
                        .text_color(t.text_muted)
                        .child(i18n::tr(Msg::NoAccounts)),
                );
            } else {
                for (idx, acc) in accounts.iter().enumerate() {
                    let tog = toggle_cb.clone();
                    let rem = remove_cb.clone();
                    accounts_list = accounts_list.child(
                        div()
                            .flex()
                            .items_center()
                            .justify_between()
                            .p(px(8.0))
                            .rounded(px(6.0))
                            .bg(t.card_bg)
                            .border_1()
                            .border_color(t.card_border)
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
                                                    tog(this, idx, w, cx);
                                                }),
                                            )
                                            .child(render_switch(t, acc.enabled)),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(13.0))
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(t.text_primary)
                                            .child(acc.name.clone()),
                                    ),
                            )
                            .child(
                                CustomButton::new(
                                    SharedString::from(format!("del-acc-{}", idx)),
                                    i18n::tr(Msg::Remove),
                                )
                                .danger()
                                .xs()
                                .render(
                                    t,
                                    move |v, w, cx| rem(v, idx, w, cx),
                                    cx,
                                ),
                            ),
                    );
                }
            }

            modal_content = modal_content
                .w(px(480.0))
                .child(render_modal_header(
                    i18n::tr(Msg::UserAccounts),
                    t,
                    close_cb.clone(),
                    cx,
                ))
                .child(
                    div()
                        .p(px(16.0))
                        .flex()
                        .flex_col()
                        .gap(px(12.0))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(t.text_secondary)
                                .child("Accounts List"),
                        )
                        .child(accounts_list)
                        .child(div().my(px(4.0)).h(px(1.0)).w_full().bg(t.card_border))
                        .child(
                            div()
                                .text_size(px(12.0))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(t.text_secondary)
                                .child("Add New Account"),
                        )
                        .child(
                            div()
                                .flex()
                                .gap(px(8.0))
                                .items_center()
                                .child(div().flex_1().child(render_input_box(
                                    "dlg-acc-user-input",
                                    &user_ent,
                                    t,
                                    cx,
                                )))
                                .child(div().flex_1().child(render_input_box(
                                    "dlg-acc-pass-input",
                                    &pass_ent,
                                    t,
                                    cx,
                                )))
                                .child(CustomButton::new("btn-add-acc", "Add").primary().render(
                                    t,
                                    move |v, w, cx| {
                                        let user = user_ent.read(cx).text().trim().to_string();
                                        let pass = pass_ent.read(cx).text().trim().to_string();
                                        if !user.is_empty() {
                                            add_cb(v, user, pass, w, cx);
                                        }
                                    },
                                    cx,
                                )),
                        ),
                )
                .child(
                    render_modal_footer(t).child(
                        CustomButton::new("btn-acc-close", i18n::tr(Msg::Close))
                            .primary()
                            .render(t, move |v, w, cx| close_cb(v, w, cx), cx),
                    ),
                );
        }
    }

    // Modal backdrop
    div()
        .absolute()
        .inset_0()
        .bg(gpui::black().opacity(0.45))
        .flex()
        .items_center()
        .justify_center()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, w, cx| {
                close_bg(this, w, cx);
            }),
        )
        .child(
            div()
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .child(modal_content),
        )
        .into_any_element()
}

fn render_modal_header<V: 'static>(
    title: impl Into<SharedString>,
    theme: &Theme,
    on_close: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = theme;
    let title_str = title.into();
    div()
        .px(px(16.0))
        .py(px(12.0))
        .border_b_1()
        .border_color(t.card_border)
        .bg(t.header_bg)
        .flex()
        .items_center()
        .justify_between()
        .child(
            div()
                .text_size(px(14.0))
                .font_weight(FontWeight::BOLD)
                .text_color(t.text_primary)
                .child(title_str),
        )
        .child(
            div()
                .id("modal-btn-x")
                .cursor_pointer()
                .px(px(6.0))
                .py(px(2.0))
                .rounded(px(4.0))
                .hover(|h| h.bg(t.hover_overlay))
                .text_size(px(14.0))
                .text_color(t.text_muted)
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, w, cx| {
                        on_close(this, w, cx);
                    }),
                )
                .child("✕"),
        )
}

fn render_modal_footer(theme: &Theme) -> gpui::Div {
    div()
        .px(px(16.0))
        .py(px(12.0))
        .border_t_1()
        .border_color(theme.card_border)
        .bg(theme.header_bg)
        .flex()
        .items_center()
        .justify_end()
        .gap(px(8.0))
}

fn render_input_box<V: 'static>(
    id_str: impl Into<SharedString>,
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

fn render_prop_row(
    label: impl Into<SharedString>,
    value: impl Into<SharedString>,
    theme: &Theme,
) -> impl IntoElement {
    div()
        .flex()
        .items_start()
        .justify_between()
        .gap(px(12.0))
        .text_size(px(12.5))
        .child(
            div()
                .w(px(90.0))
                .font_weight(FontWeight::MEDIUM)
                .text_color(theme.text_secondary)
                .child(label.into()),
        )
        .child(
            div()
                .flex_1()
                .font_weight(FontWeight::NORMAL)
                .text_color(theme.text_primary)
                .child(value.into()),
        )
}
