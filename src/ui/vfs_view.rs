//! Virtual File System Tree View for HFS-RS.

use std::collections::HashSet;
use std::sync::Arc;

use gpui::{
    AnyElement, ColorExt, Context, ElementId, FontWeight, IntoElement, MouseButton, SharedString,
    div, prelude::*, px,
};

use crate::i18n::{self, Msg};
use crate::server::AppState;
use crate::ui::components::CustomButton;
use crate::ui::theme::Theme;
use crate::vfs::{NodeId, NodeKind};

pub struct VfsViewProps<'a> {
    pub theme: &'a Theme,
    pub state: &'a Arc<AppState>,
    pub selected: Option<NodeId>,
    pub expanded: &'a HashSet<u64>,
    pub expert_mode: bool,
    pub context_menu_open: bool,
    pub add_menu_open: bool,
}

pub fn render_vfs_pane<V: 'static>(
    props: VfsViewProps<'_>,
    on_select: impl Fn(&mut V, NodeId, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_toggle_expand: impl Fn(&mut V, NodeId, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_toggle_add_menu: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_add_files: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_add_folder: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_new_virtual_folder: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_new_link: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_rename: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    _on_remove: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_properties: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_open_context_menu: impl Fn(&mut V, Option<NodeId>, &mut gpui::Window, &mut Context<V>)
    + 'static
    + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = props.theme;
    let has_selection = props.selected.is_some();

    // Flatten tree
    let visible_nodes = {
        let vfs = props.state.vfs.read();
        let mut list = Vec::new();
        fn walk(
            vfs: &crate::vfs::Vfs,
            id: NodeId,
            depth: usize,
            expanded: &HashSet<u64>,
            out: &mut Vec<(usize, NodeId, bool, String, NodeKind)>,
        ) {
            if let Some(node) = vfs.get(id) {
                let has_children = node.is_folder() && !node.children.is_empty();
                let name = if node.kind == NodeKind::Root {
                    i18n::tr(Msg::KindRoot).to_string()
                } else {
                    node.display_name().to_string()
                };
                out.push((depth, id, has_children, name, node.kind));
                if has_children && expanded.contains(&id.0) {
                    for cid in &node.children {
                        walk(vfs, *cid, depth + 1, expanded, out);
                    }
                }
            }
        }
        walk(&vfs, vfs.root_id(), 0, props.expanded, &mut list);
        list
    };

    let mut tree_rows = div().flex().flex_col().w_full().gap(px(1.0));

    if visible_nodes.is_empty() {
        tree_rows = tree_rows.child(
            div()
                .p(px(16.0))
                .text_size(px(12.5))
                .text_color(t.text_muted)
                .child(i18n::tr(Msg::VfsEmpty)),
        );
    } else {
        for (depth, id, has_children, name, kind) in visible_nodes {
            let is_selected = props.selected == Some(id);
            let is_expanded = props.expanded.contains(&id.0);

            let (icon, icon_color) = match kind {
                NodeKind::Root => ("🏠", t.accent),
                NodeKind::VirtualFolder => ("📂", t.warning),
                NodeKind::RealFolder => ("📁", t.accent),
                NodeKind::File => ("📄", t.text_secondary),
                NodeKind::Link => ("🔗", t.success),
            };

            let kind_tag = match kind {
                NodeKind::Root => format!("[{}]", i18n::tr(Msg::KindRoot)),
                NodeKind::VirtualFolder => format!("[{}]", i18n::tr(Msg::KindVirtual)),
                NodeKind::RealFolder => format!("[{}]", i18n::tr(Msg::KindRealFolder)),
                NodeKind::File => format!("[{}]", i18n::tr(Msg::KindFile)),
                NodeKind::Link => format!("[{}]", i18n::tr(Msg::KindLink)),
            };

            let indent_px = 6.0 + (depth as f32) * 16.0;

            let sel_cb = on_select.clone();
            let exp_cb = on_toggle_expand.clone();
            let ctx_cb = on_open_context_menu.clone();

            let row_bg = if is_selected {
                t.selection
            } else {
                crate::ui::theme::TRANSPARENT
            };

            let mut row = div()
                .id(ElementId::Name(format!("vfs-node-{}", id.0).into()))
                .flex()
                .items_center()
                .h(px(26.0))
                .w_full()
                .pl(px(indent_px))
                .pr(px(8.0))
                .rounded(px(5.0))
                .bg(row_bg)
                .border_1()
                .border_color(if is_selected {
                    t.accent.opacity(0.6)
                } else {
                    crate::ui::theme::TRANSPARENT
                })
                .cursor_pointer()
                .hover(|h| {
                    if !is_selected {
                        h.bg(t.hover_overlay)
                    } else {
                        h
                    }
                })
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _, w, cx| {
                        sel_cb(this, id, w, cx);
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, _, w, cx| {
                        ctx_cb(this, Some(id), w, cx);
                    }),
                );

            // Twist button
            if has_children {
                let twist_char = if is_expanded { "▾" } else { "▸" };
                row = row.child(
                    div()
                        .w(px(14.0))
                        .h(px(14.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(3.0))
                        .hover(|h| h.bg(t.active_overlay))
                        .text_size(px(11.0))
                        .font_weight(FontWeight::BOLD)
                        .text_color(t.text_secondary)
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, w, cx| {
                                cx.stop_propagation();
                                exp_cb(this, id, w, cx);
                            }),
                        )
                        .child(twist_char),
                );
            } else {
                row = row.child(div().w(px(14.0)));
            }

            // Icon + Name
            row = row
                .child(
                    div()
                        .mx(px(4.0))
                        .text_size(px(13.0))
                        .text_color(icon_color)
                        .child(icon),
                )
                .child(
                    div()
                        .flex_1()
                        .text_size(px(12.5))
                        .font_weight(if is_selected {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_color(if is_selected {
                            t.text_primary
                        } else {
                            t.text_secondary
                        })
                        .overflow_x_hidden()
                        .text_ellipsis()
                        .child(name),
                );

            if props.expert_mode && kind != NodeKind::Root {
                row = row.child(
                    div()
                        .text_size(px(10.0))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(t.text_muted)
                        .child(kind_tag),
                );
            }

            tree_rows = tree_rows.child(row);
        }
    }

    let bg_ctx_cb = on_open_context_menu.clone();

    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(t.panel_bg)
        .border_r_1()
        .border_color(t.card_border)
        // Top action toolbar for VFS
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
                .gap(px(4.0))
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::BOLD)
                        .text_color(t.text_primary)
                        .child(i18n::tr(Msg::VirtualFileSystem)),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(3.0))
                        // Consolidated Add (+) Dropdown Button
                        .child(
                            div()
                                .relative()
                                .child(
                                    CustomButton::new("vfs-btn-add", "+ ▾")
                                        .xs()
                                        .primary()
                                        .render(t, on_toggle_add_menu.clone(), cx),
                                )
                                .child({
                                    if props.add_menu_open {
                                        gpui::deferred(
                                            div().absolute().top_full().left_0().mt(px(4.0)).child(
                                                render_vfs_add_menu(
                                                    t,
                                                    on_add_files,
                                                    on_add_folder,
                                                    on_new_virtual_folder,
                                                    on_new_link,
                                                    on_toggle_add_menu,
                                                    cx,
                                                ),
                                            ),
                                        )
                                        .with_priority(2)
                                        .into_any_element()
                                    } else {
                                        div().into_any_element()
                                    }
                                }),
                        )
                        .child(
                            CustomButton::new("vfs-ren", i18n::tr(Msg::Rename))
                                .xs()
                                .outline()
                                .disabled(!has_selection)
                                .render(t, on_rename.clone(), cx),
                        )
                        .child(
                            CustomButton::new("vfs-prop", i18n::tr(Msg::Properties))
                                .xs()
                                .outline()
                                .disabled(!has_selection)
                                .render(t, on_properties.clone(), cx),
                        ),
                ),
        )
        // Scrollable tree list
        .child(
            div()
                .id("vfs-tree-scroll-container")
                .flex_1()
                .min_h(px(0.0))
                .p(px(6.0))
                .overflow_y_scroll()
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(move |this, _, w, cx| {
                        bg_ctx_cb(this, None, w, cx);
                    }),
                )
                .child(tree_rows),
        )
}

fn render_vfs_add_menu<V: 'static>(
    theme: &Theme,
    on_add_files: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_add_folder: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_new_virtual_folder: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_new_link: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_close: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = theme;
    let add_f_cb = on_add_files.clone();
    let add_d_cb = on_add_folder.clone();
    let new_v_cb = on_new_virtual_folder.clone();
    let new_l_cb = on_new_link.clone();
    let close_cb1 = on_close.clone();
    let close_cb2 = on_close.clone();
    let close_cb3 = on_close.clone();
    let close_cb4 = on_close.clone();

    div()
        .id("vfs-add-dropdown-menu")
        .w(px(180.0))
        .p(px(4.0))
        .rounded(px(8.0))
        .bg(t.card_bg)
        .border_1()
        .border_color(t.card_border)
        .shadow_lg()
        .flex()
        .flex_col()
        .gap(px(2.0))
        .on_mouse_down(MouseButton::Left, |_, _, cx| {
            cx.stop_propagation();
        })
        .child(render_menu_item(
            "vfs-add-f-item",
            i18n::tr(Msg::AddFiles),
            "📄",
            true,
            t,
            move |this, w, cx| {
                close_cb1(this, w, cx);
                add_f_cb(this, w, cx);
            },
            cx,
        ))
        .child(render_menu_item(
            "vfs-add-d-item",
            i18n::tr(Msg::AddFolder),
            "📁",
            true,
            t,
            move |this, w, cx| {
                close_cb2(this, w, cx);
                add_d_cb(this, w, cx);
            },
            cx,
        ))
        .child(render_menu_item(
            "vfs-new-v-item",
            i18n::tr(Msg::NewEmptyFolder),
            "📂",
            true,
            t,
            move |this, w, cx| {
                close_cb3(this, w, cx);
                new_v_cb(this, w, cx);
            },
            cx,
        ))
        .child(render_menu_item(
            "vfs-new-l-item",
            i18n::tr(Msg::NewLink),
            "🔗",
            true,
            t,
            move |this, w, cx| {
                close_cb4(this, w, cx);
                new_l_cb(this, w, cx);
            },
            cx,
        ))
}

pub fn render_vfs_context_menu<V: 'static>(
    theme: &Theme,
    selected: Option<NodeId>,
    on_close: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_add_files: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_add_folder: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_new_virtual_folder: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_new_link: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_rename: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_remove: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_properties: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_browse_it: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_copy_url: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    on_open_local: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> AnyElement {
    let t = theme;
    let has_selection = selected.is_some();
    let close_cb = on_close.clone();

    div()
        .absolute()
        .inset_0()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(move |this, _, w, cx| {
                close_cb(this, w, cx);
            }),
        )
        .child(
            div()
                .absolute()
                .top(px(60.0))
                .left(px(30.0))
                .w(px(220.0))
                .rounded(px(8.0))
                .bg(t.card_bg)
                .border_1()
                .border_color(t.card_border)
                .shadow_lg()
                .p(px(4.0))
                .flex()
                .flex_col()
                .gap(px(2.0))
                .on_mouse_down(MouseButton::Left, |_, _, cx| {
                    cx.stop_propagation();
                })
                .child(render_menu_item(
                    "ctx-browse",
                    i18n::tr(Msg::BrowseIt),
                    "🌐",
                    true,
                    t,
                    on_browse_it,
                    cx,
                ))
                .child(render_menu_item(
                    "ctx-copy",
                    i18n::tr(Msg::CopyUrl),
                    "📋",
                    true,
                    t,
                    on_copy_url,
                    cx,
                ))
                .child(render_menu_item(
                    "ctx-local",
                    i18n::tr(Msg::OpenItem),
                    "📂",
                    has_selection,
                    t,
                    on_open_local,
                    cx,
                ))
                .child(render_menu_divider(t))
                .child(render_menu_item(
                    "ctx-add-f",
                    i18n::tr(Msg::AddFiles),
                    "📄",
                    true,
                    t,
                    on_add_files,
                    cx,
                ))
                .child(render_menu_item(
                    "ctx-add-d",
                    i18n::tr(Msg::AddFolder),
                    "📁",
                    true,
                    t,
                    on_add_folder,
                    cx,
                ))
                .child(render_menu_item(
                    "ctx-new-v",
                    i18n::tr(Msg::NewEmptyFolder),
                    "📂",
                    true,
                    t,
                    on_new_virtual_folder,
                    cx,
                ))
                .child(render_menu_item(
                    "ctx-new-l",
                    i18n::tr(Msg::NewLink),
                    "🔗",
                    true,
                    t,
                    on_new_link,
                    cx,
                ))
                .child(render_menu_divider(t))
                .child(render_menu_item(
                    "ctx-ren",
                    i18n::tr(Msg::Rename),
                    "✏️",
                    has_selection,
                    t,
                    on_rename,
                    cx,
                ))
                .child(render_menu_item(
                    "ctx-del",
                    i18n::tr(Msg::Remove),
                    "🗑️",
                    has_selection,
                    t,
                    on_remove,
                    cx,
                ))
                .child(render_menu_divider(t))
                .child(render_menu_item(
                    "ctx-prop",
                    i18n::tr(Msg::Properties),
                    "ℹ️",
                    has_selection,
                    t,
                    on_properties,
                    cx,
                )),
        )
        .into_any_element()
}

fn render_menu_item<V: 'static>(
    id: &str,
    label: impl Into<SharedString>,
    icon: &str,
    enabled: bool,
    theme: &Theme,
    on_click: impl Fn(&mut V, &mut gpui::Window, &mut Context<V>) + 'static + Clone,
    cx: &mut Context<V>,
) -> impl IntoElement {
    let t = theme;
    let label_str = label.into();
    let mut item = div()
        .id(ElementId::Name(id.to_string().into()))
        .px(px(8.0))
        .py(px(5.0))
        .rounded(px(5.0))
        .flex()
        .items_center()
        .gap(px(8.0))
        .text_size(px(12.0))
        .text_color(if enabled {
            t.text_primary
        } else {
            t.text_muted
        });

    if enabled {
        item = item
            .cursor_pointer()
            .hover(|h| h.bg(t.hover_overlay))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _, w, cx| {
                    on_click(this, w, cx);
                }),
            );
    } else {
        item = item.opacity(0.5);
    }

    item.child(div().w(px(16.0)).child(icon.to_string()))
        .child(div().flex_1().child(label_str))
}

fn render_menu_divider(theme: &Theme) -> impl IntoElement {
    div().my(px(2.0)).h(px(1.0)).w_full().bg(theme.card_border)
}
