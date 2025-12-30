/* window.rs
 *
 * Copyright 2025 Furqan Ali Shah
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::Duration;

use adw::prelude::*;
use adw::subclass::prelude::*;
use ashpd::desktop::screenshot::Screenshot;
use gtk::{gdk, gio, glib};
use gdk_pixbuf::Pixbuf;

use crate::editor::{self, Annotation, EditorState, Point, Rect, Tool};

mod imp {
    use super::*;

    #[derive(Debug, Default)]
    pub struct GreatshotWindow {}

    #[glib::object_subclass]
    impl ObjectSubclass for GreatshotWindow {
        const NAME: &'static str = "GreatshotWindow";
        type Type = super::GreatshotWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for GreatshotWindow {}
    impl WidgetImpl for GreatshotWindow {}
    impl WindowImpl for GreatshotWindow {}
    impl ApplicationWindowImpl for GreatshotWindow {}
    impl AdwApplicationWindowImpl for GreatshotWindow {}
}

glib::wrapper! {
    pub struct GreatshotWindow(ObjectSubclass<imp::GreatshotWindow>)
        @extends gtk::Widget, gtk::Window, gtk::ApplicationWindow, adw::ApplicationWindow;
}

impl GreatshotWindow {
    pub fn new<P: IsA<adw::Application>>(app: &P) -> Self {
        let window: Self = glib::Object::builder()
            .property("application", app)
            .property("default-width", 1400)
            .property("default-height", 900)
            .property("title", "GreatShot")
            .build();

        build_ui_for_window(&window);
        window
    }
}

fn build_ui_for_window(window: &GreatshotWindow) {
    let runtime = Arc::new(
        tokio::runtime::Runtime::new().expect("Failed to start async runtime"),
    );

    let state = Rc::new(RefCell::new(EditorState::new()));
    {
        let mut state = state.borrow_mut();
        state.color = gdk::RGBA::new(1.0, 0.30, 0.30, 1.0);
        state.fit_to_window = true;
        state.zoom = 1.0;
    }

    if let Some(display) = gdk::Display::default() {
        let css = gtk::CssProvider::new();
        gtk::style_context_add_provider_for_display(
            &display,
            &css,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
        let style_manager = adw::StyleManager::default();
        let css_provider = css.clone();
        let apply_theme_css = move |is_dark: bool| {
            if is_dark {
                css_provider.load_from_string(
                    ".tool-palette { background: rgba(18, 18, 18, 0.78); border-radius: 16px; padding: 10px; border: 1px solid rgba(255,255,255,0.06); box-shadow: 0 12px 30px rgba(0,0,0,0.35); }
                     .tool-button { min-width: 38px; min-height: 38px; border-radius: 10px; }
                     .tool-button.toggle:hover { background: rgba(255, 255, 255, 0.08); }
                     .tool-button.toggle:checked { background: rgba(255, 255, 255, 0.18); box-shadow: inset 0 0 0 2px rgba(255,255,255,0.55); }
                     .color-palette { background: rgba(18, 18, 18, 0.72); border-radius: 12px; padding: 8px; border: 1px solid rgba(255,255,255,0.06); }
                     .color-swatch { min-width: 20px; min-height: 20px; border-radius: 999px; border: 2px solid rgba(255,255,255,0.18); }
                     .color-swatch.toggle:checked { border: 2px solid rgba(255,255,255,0.9); }
                     .color-custom { min-width: 20px; min-height: 20px; border-radius: 999px; border: 2px solid rgba(255,255,255,0.25); background: rgba(255,255,255,0.08); }
                     .color-black { background: #1b1b1b; }
                     .color-white { background: #f5f5f5; }
                     .color-red { background: #ff4d4d; }
                     .color-orange { background: #ff9f1a; }
                     .color-yellow { background: #ffd93d; }
                     .color-green { background: #3ddc84; }
                     .color-blue { background: #3b82f6; }
                     .color-purple { background: #8b5cf6; }
                     .editor-canvas { background: #1e1e1e; }
                     .editor-status { color: #c9c9c9; font-size: 11px; }",
                );
            } else {
                css_provider.load_from_string(
                    ".tool-palette { background: rgba(250, 250, 250, 0.92); border-radius: 16px; padding: 10px; border: 1px solid rgba(0,0,0,0.08); box-shadow: 0 12px 30px rgba(0,0,0,0.12); }
                     .tool-button { min-width: 38px; min-height: 38px; border-radius: 10px; }
                     .tool-button.toggle:hover { background: rgba(0, 0, 0, 0.06); }
                     .tool-button.toggle:checked { background: rgba(0, 0, 0, 0.08); box-shadow: inset 0 0 0 2px rgba(0,0,0,0.35); }
                     .color-palette { background: rgba(250, 250, 250, 0.92); border-radius: 12px; padding: 8px; border: 1px solid rgba(0,0,0,0.08); }
                     .color-swatch { min-width: 20px; min-height: 20px; border-radius: 999px; border: 2px solid rgba(0,0,0,0.2); }
                     .color-swatch.toggle:checked { border: 2px solid rgba(0,0,0,0.8); }
                     .color-custom { min-width: 20px; min-height: 20px; border-radius: 999px; border: 2px solid rgba(0,0,0,0.25); background: rgba(0,0,0,0.04); }
                     .color-black { background: #1b1b1b; }
                     .color-white { background: #f5f5f5; }
                     .color-red { background: #ff4d4d; }
                     .color-orange { background: #ff9f1a; }
                     .color-yellow { background: #ffd93d; }
                     .color-green { background: #3ddc84; }
                     .color-blue { background: #3b82f6; }
                     .color-purple { background: #8b5cf6; }
                     .editor-canvas { background: #f4f4f4; }
                     .editor-status { color: #5c5c5c; font-size: 11px; }",
                );
            }
        };
        let initial_dark = style_manager.is_dark();
        apply_theme_css(initial_dark);
        let style_manager_for_notify = style_manager.clone();
        style_manager.connect_dark_notify(move |_| {
            apply_theme_css(style_manager_for_notify.is_dark());
        });
    }

    let header = adw::HeaderBar::builder()
        .title_widget(&adw::WindowTitle::new("GreatShot", ""))
        .build();

    let capture_button = gtk::Button::builder()
        .icon_name("camera-symbolic")
        .tooltip_text("Capture screenshot")
        .build();
    header.pack_start(&capture_button);

    let open_button = gtk::Button::builder()
        .icon_name("folder-open-symbolic")
        .tooltip_text("Open image")
        .build();
    let paste_button = gtk::Button::builder()
        .icon_name("clipboard-symbolic")
        .tooltip_text("Paste from clipboard")
        .build();
    header.pack_start(&open_button);
    header.pack_start(&paste_button);

    let delay_adjustment = gtk::Adjustment::new(0.0, 0.0, 10.0, 0.5, 1.0, 0.0);
    let delay_spin = gtk::SpinButton::builder()
        .adjustment(&delay_adjustment)
        .digits(1)
        .numeric(true)
        .width_chars(3)
        .tooltip_text("Capture delay (seconds)")
        .build();
    let interactive_toggle = gtk::Switch::builder()
        .tooltip_text("Interactive capture")
        .active(true)
        .build();
    let settings_button = gtk::MenuButton::builder()
        .icon_name("settings-symbolic")
        .tooltip_text("Capture settings")
        .build();
    let settings_box = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(10)
        .margin_top(10)
        .margin_bottom(10)
        .margin_start(10)
        .margin_end(10)
        .build();
    let size_adjustment = gtk::Adjustment::new(4.0, 1.0, 32.0, 1.0, 2.0, 0.0);
    let size_spin = gtk::SpinButton::builder()
        .adjustment(&size_adjustment)
        .climb_rate(1.0)
        .digits(0)
        .numeric(true)
        .width_chars(2)
        .tooltip_text("Stroke size")
        .build();
    let zoom_adjustment = gtk::Adjustment::new(1.0, 0.25, 3.0, 0.05, 0.1, 0.0);
    let zoom_scale = gtk::Scale::builder()
        .orientation(gtk::Orientation::Horizontal)
        .adjustment(&zoom_adjustment)
        .digits(0)
        .draw_value(false)
        .width_request(120)
        .tooltip_text("Zoom")
        .build();
    let fit_toggle = gtk::ToggleButton::with_label("Fit");
    fit_toggle.set_active(true);
    let zoom_reset = gtk::Button::with_label("100%");
    let size_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    let size_label = gtk::Label::new(Some("Stroke"));
    size_label.set_xalign(0.0);
    size_label.set_hexpand(true);
    size_row.append(&size_label);
    size_row.append(&size_spin);
    let size_icon = gtk::Image::from_icon_name("pencil-symbolic");
    let size_group = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    size_group.append(&size_icon);
    size_group.append(&size_row);
    let delay_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    let delay_label = gtk::Label::new(Some("Delay (s)"));
    delay_label.set_xalign(0.0);
    delay_label.set_hexpand(true);
    delay_row.append(&delay_label);
    delay_row.append(&delay_spin);
    let delay_icon = gtk::Image::from_icon_name("clock-symbolic");
    let delay_group = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    delay_group.append(&delay_icon);
    delay_group.append(&delay_row);
    let interactive_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    let interactive_label = gtk::Label::new(Some("Interactive"));
    interactive_label.set_xalign(0.0);
    interactive_label.set_hexpand(true);
    interactive_row.append(&interactive_label);
    interactive_row.append(&interactive_toggle);
    let interactive_icon = gtk::Image::from_icon_name("pointer-symbolic");
    let interactive_group = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    interactive_group.append(&interactive_icon);
    interactive_group.append(&interactive_row);
    let zoom_row = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    let zoom_label = gtk::Label::new(Some("Zoom"));
    zoom_label.set_xalign(0.0);
    zoom_label.set_hexpand(true);
    zoom_row.append(&zoom_label);
    zoom_row.append(&zoom_scale);
    let zoom_icon = gtk::Image::from_icon_name("zoom-in-symbolic");
    let zoom_group = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    zoom_group.append(&zoom_icon);
    zoom_group.append(&zoom_row);
    let zoom_actions = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(8)
        .build();
    zoom_actions.append(&fit_toggle);
    zoom_actions.append(&zoom_reset);
    let divider1 = gtk::Separator::new(gtk::Orientation::Horizontal);
    let divider2 = gtk::Separator::new(gtk::Orientation::Horizontal);
    let divider3 = gtk::Separator::new(gtk::Orientation::Horizontal);
    settings_box.append(&size_group);
    settings_box.append(&divider1);
    settings_box.append(&zoom_group);
    settings_box.append(&divider2);
    settings_box.append(&delay_group);
    settings_box.append(&interactive_group);
    settings_box.append(&divider3);
    settings_box.append(&zoom_actions);
    let settings_popover = gtk::Popover::new();
    settings_popover.set_child(Some(&settings_box));
    settings_button.set_popover(Some(&settings_popover));

    header.pack_end(&settings_button);

    let undo_button = gtk::Button::builder()
        .icon_name("arrow-back-up-symbolic")
        .tooltip_text("Undo")
        .build();
    let redo_button = gtk::Button::builder()
        .icon_name("arrow-forward-up-symbolic")
        .tooltip_text("Redo")
        .build();
    let copy_button = gtk::Button::builder()
        .icon_name("copy-symbolic")
        .tooltip_text("Copy to clipboard")
        .build();
    let save_button = gtk::Button::builder()
        .icon_name("device-floppy-symbolic")
        .tooltip_text("Save as PNG")
        .build();
    header.pack_end(&copy_button);
    header.pack_end(&save_button);
    header.pack_end(&redo_button);
    header.pack_end(&undo_button);

    let status = gtk::Label::builder().label("").xalign(0.0).build();
    status.add_css_class("editor-status");
    status.add_css_class("dim-label");
    status.set_visible(false);

    let set_status = Rc::new({
        let status = status.clone();
        move |msg: &str| {
            status.set_text(msg);
            status.set_visible(!msg.is_empty());
        }
    });

    let drawing_area = gtk::DrawingArea::builder()
        .content_width(900)
        .content_height(600)
        .build();
    drawing_area.set_hexpand(true);
    drawing_area.set_vexpand(true);
    drawing_area.add_css_class("editor-canvas");

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .vscrollbar_policy(gtk::PolicyType::Automatic)
        .child(&drawing_area)
        .build();

    let content = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(12)
        .margin_top(18)
        .margin_bottom(18)
        .margin_start(18)
        .margin_end(18)
        .build();

    content.append(&status);
    content.append(&scroller);

    let overlay = gtk::Overlay::new();
    overlay.set_child(Some(&content));

    let palette = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .spacing(6)
        .halign(gtk::Align::End)
        .valign(gtk::Align::End)
        .margin_end(16)
        .margin_bottom(16)
        .build();
    palette.add_css_class("tool-palette");

    let color_palette = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .spacing(6)
        .halign(gtk::Align::Start)
        .valign(gtk::Align::End)
        .margin_start(16)
        .margin_bottom(16)
        .build();
    color_palette.add_css_class("color-palette");

    let color_dialog = Rc::new(gtk::ColorDialog::new());
    let color_button = gtk::Button::builder()
        .icon_name("palette-symbolic")
        .tooltip_text("Custom color")
        .build();
    color_button.add_css_class("color-custom");

    let make_tool_button = |icon_name: &str, tooltip: &str| {
        let button = gtk::ToggleButton::builder()
            .icon_name(icon_name)
            .build();
        button.add_css_class("tool-button");
        button.set_tooltip_text(Some(tooltip));
        button
    };

    let tool_buttons: Vec<(Tool, gtk::ToggleButton)> = vec![
        (Tool::Select, make_tool_button("select-symbolic", "Select")),
        (Tool::Crop, make_tool_button("crop-symbolic", "Crop")),
        (Tool::Pen, make_tool_button("pencil-symbolic", "Pen")),
        (Tool::Rect, make_tool_button("square-symbolic", "Rectangle")),
        (Tool::Line, make_tool_button("minus-symbolic", "Line")),
        (Tool::Arrow, make_tool_button("arrow-right-symbolic", "Arrow")),
        (Tool::Text, make_tool_button("text-size-symbolic", "Text")),
        (Tool::Blur, make_tool_button("blur-symbolic", "Blur")),
    ];

    for (_, button) in tool_buttons.iter() {
        palette.append(button);
    }

    overlay.add_overlay(&palette);
    overlay.add_overlay(&color_palette);

    let toolbar_view = adw::ToolbarView::builder().content(&overlay).build();
    toolbar_view.add_top_bar(&header);

    window.set_content(Some(&toolbar_view));
    window.maximize();

    let zoom_updating = Rc::new(Cell::new(false));
    let fit_updating = Rc::new(Cell::new(false));

    let apply_background = {
        let drawing_area = drawing_area.clone();
        let state = state.clone();
        let zoom_adjustment = zoom_adjustment.clone();
        let fit_toggle = fit_toggle.clone();
        let zoom_updating = zoom_updating.clone();
        let fit_updating = fit_updating.clone();
        Rc::new(move |pixbuf: Pixbuf| {
            let width = pixbuf.width();
            let height = pixbuf.height();
            drawing_area.set_content_width(width);
            drawing_area.set_content_height(height);
            {
                let mut state = state.borrow_mut();
                state.set_background(pixbuf);
                state.fit_to_window = true;
                state.zoom = 1.0;
            }
            fit_updating.set(true);
            fit_toggle.set_active(true);
            fit_updating.set(false);
            zoom_updating.set(true);
            zoom_adjustment.set_value(1.0);
            zoom_updating.set(false);
            drawing_area.queue_draw();
        })
    };

    let (sender, receiver) = mpsc::channel::<Result<String, String>>();

    let set_status_for_timer = set_status.clone();
    let button_for_timer = capture_button.clone();
    let apply_background_for_timer = apply_background.clone();
    let window_for_timer = window.clone();

    glib::timeout_add_local(Duration::from_millis(100), move || {
        while let Ok(result) = receiver.try_recv() {
            button_for_timer.set_sensitive(true);
            match result {
                Ok(uri) => {
                    let msg = format!("Captured: {uri}");
                    set_status_for_timer(&msg);
                    let file = gio::File::for_uri(&uri);
                    match file.path() {
                        Some(path) => match Pixbuf::from_file(path) {
                            Ok(pixbuf) => {
                                apply_background_for_timer(pixbuf);
                            }
                            Err(err) => {
                                let msg = format!("Failed to load image: {err}");
                                set_status_for_timer(&msg);
                            }
                        },
                        None => {
                            set_status_for_timer("Failed to resolve capture path.");
                        }
                    }
                }
                Err(err) => {
                    let msg = format!("Capture failed: {err}");
                    set_status_for_timer(&msg);
                }
            }
            window_for_timer.present();
        }
        glib::ControlFlow::Continue
    });

    let runtime = runtime.clone();
    let set_status_for_capture = set_status.clone();
    let button = capture_button.clone();
    let delay_spin = delay_spin.clone();
    let interactive_toggle = interactive_toggle.clone();
    let window_for_capture = window.clone();

    capture_button.connect_clicked(move |_| {
        button.set_sensitive(false);
        set_status_for_capture("Capturing via portal...");
        window_for_capture.minimize();
        window_for_capture.set_visible(false);

        let runtime = runtime.clone();
        let sender = sender.clone();
        let delay = delay_spin.value();
        let interactive = interactive_toggle.is_active();

        runtime.spawn(async move {
            let hide_delay = std::time::Duration::from_millis(200);
            tokio::time::sleep(hide_delay).await;
            if delay > 0.0 {
                tokio::time::sleep(std::time::Duration::from_secs_f64(delay)).await;
            }
            let result = Screenshot::request()
                .interactive(interactive)
                .modal(true)
                .send()
                .await
                .and_then(|request| request.response())
                .map(|response| response.uri().to_string())
                .map_err(|err| err.to_string());

            let _ = sender.send(result);
        });
    });

    let state_for_draw = state.clone();
    let draw_area_for_draw = drawing_area.clone();
    drawing_area.set_draw_func(move |_, ctx, width, height| {
        {
            let mut state = state_for_draw.borrow_mut();
            state.viewport_width = width;
            state.viewport_height = height;
            if let Some(background) = state.background.as_ref() {
                let (scale, _, _) = editor::view_transform(&state);
                let scaled_w = (background.width() as f64 * scale).round() as i32;
                let scaled_h = (background.height() as f64 * scale).round() as i32;
                draw_area_for_draw.set_content_width(scaled_w.max(1));
                draw_area_for_draw.set_content_height(scaled_h.max(1));
            }
        }
        let state = state_for_draw.borrow();
        editor::draw(&state, ctx);
    });

    let drag = gtk::GestureDrag::new();
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        drag.connect_drag_begin(move |_, x, y| {
            let mut state = state.borrow_mut();
            let point_view = Point { x, y };
            let point = editor::map_to_image(&state, x, y);
            state.drag_start_view = Some(point_view);
            match state.tool {
                Tool::Select => {
                    state.selected = editor::hit_test(&state.annotations, point);
                    if let Some(index) = state.selected {
                        state.draft = None;
                        state.crop_rect = None;
                        state.selected_original = Some(state.annotations[index].clone());
                    }
                }
                Tool::Crop => {
                    state.selected = None;
                    state.draft = None;
                    state.crop_rect = Some(Rect {
                        x1: point.x,
                        y1: point.y,
                        x2: point.x,
                        y2: point.y,
                    });
                }
                Tool::Pen => {
                    state.draft = Some(Annotation::Pen {
                        points: vec![point],
                        color: state.color,
                        width: state.stroke_width,
                    });
                }
                Tool::Rect => {
                    state.draft = Some(Annotation::Rect {
                        rect: Rect {
                            x1: point.x,
                            y1: point.y,
                            x2: point.x,
                            y2: point.y,
                        },
                        color: state.color,
                        width: state.stroke_width,
                    });
                }
                Tool::Line | Tool::Arrow => {
                    state.draft = Some(Annotation::Line {
                        start: point,
                        end: point,
                        color: state.color,
                        width: state.stroke_width,
                        arrow: matches!(state.tool, Tool::Arrow),
                    });
                }
                Tool::Blur => {
                    state.draft = Some(Annotation::Blur {
                        rect: Rect {
                            x1: point.x,
                            y1: point.y,
                            x2: point.x,
                            y2: point.y,
                        },
                        pixel_size: 10,
                    });
                }
                Tool::Text => {
                    state.draft = None;
                }
            }
            drawing_area.queue_draw();
        });
    }
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        drag.connect_drag_update(move |_, offset_x, offset_y| {
            let mut state = state.borrow_mut();
            let Some(start) = state.drag_start_view else { return; };
            let current_view = Point {
                x: start.x + offset_x,
                y: start.y + offset_y,
            };
            let current = editor::map_to_image(&state, current_view.x, current_view.y);
            match state.tool {
                Tool::Select => {
                    if let Some(index) = state.selected {
                        if let Some(original) = state.selected_original.as_ref() {
                            let start_img = editor::map_to_image(&state, start.x, start.y);
                            let dx = current.x - start_img.x;
                            let dy = current.y - start_img.y;
                            let mut moved = original.clone();
                            editor::move_annotation(&mut moved, dx, dy);
                            state.annotations[index] = moved;
                        }
                    }
                }
                Tool::Crop => {
                    if let Some(rect) = state.crop_rect.as_mut() {
                        rect.x2 = current.x;
                        rect.y2 = current.y;
                    }
                }
                _ => match state.draft.as_mut() {
                    Some(Annotation::Pen { points, .. }) => {
                        points.push(current);
                    }
                    Some(Annotation::Rect { rect, .. }) => {
                        rect.x2 = current.x;
                        rect.y2 = current.y;
                    }
                    Some(Annotation::Line { end, .. }) => {
                        *end = current;
                    }
                    Some(Annotation::Blur { rect, .. }) => {
                        rect.x2 = current.x;
                        rect.y2 = current.y;
                    }
                    _ => {}
                },
            }
            drawing_area.queue_draw();
        });
    }
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let fit_toggle = fit_toggle.clone();
        let fit_updating = fit_updating.clone();
        let zoom_adjustment = zoom_adjustment.clone();
        let zoom_updating = zoom_updating.clone();
        drag.connect_drag_end(move |_, offset_x, offset_y| {
            let mut did_crop = false;
            let mut new_size = None;
            {
                let mut state = state.borrow_mut();
                if let Some(start) = state.drag_start_view.take() {
                    let end_view = Point {
                        x: start.x + offset_x,
                        y: start.y + offset_y,
                    };
                    let end = editor::map_to_image(&state, end_view.x, end_view.y);
                    match state.tool {
                        Tool::Select => {
                            state.selected_original = None;
                        }
                        Tool::Crop => {
                            if let Some(rect) = state.crop_rect {
                                if editor::apply_crop(&mut state, rect) {
                                    state.fit_to_window = true;
                                    state.zoom = 1.0;
                                    did_crop = true;
                                    new_size = state
                                        .background
                                        .as_ref()
                                        .map(|p| (p.width(), p.height()));
                                }
                            }
                        }
                        _ => {
                            if let Some(mut draft) = state.draft.take() {
                                match &mut draft {
                                    Annotation::Line { end: line_end, .. } => *line_end = end,
                                    Annotation::Rect { rect, .. } => {
                                        rect.x2 = end.x;
                                        rect.y2 = end.y;
                                    }
                                    Annotation::Blur { rect, .. } => {
                                        rect.x2 = end.x;
                                        rect.y2 = end.y;
                                    }
                                    Annotation::Pen { points, .. } => {
                                        points.push(end);
                                    }
                                    _ => {}
                                }
                                state.push_annotation(draft);
                            }
                        }
                    }
                }
            }
            if did_crop {
                if let Some((width, height)) = new_size {
                    drawing_area.set_content_width(width);
                    drawing_area.set_content_height(height);
                }
                fit_updating.set(true);
                fit_toggle.set_active(true);
                fit_updating.set(false);
                zoom_updating.set(true);
                zoom_adjustment.set_value(1.0);
                zoom_updating.set(false);
            }
            drawing_area.queue_draw();
        });
    }
    drawing_area.add_controller(drag);

    let click = gtk::GestureClick::new();
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        click.connect_pressed(move |_, _, x, y| {
            let pos = {
                let state = state.borrow();
                editor::map_to_image(&state, x, y)
            };
            let mut state = state.borrow_mut();
            match state.tool {
                Tool::Text => {
                    let color = state.color;
                    let size = state.text_size;
                    state.push_annotation(Annotation::Text {
                        pos,
                        text: "Text".to_string(),
                        color,
                        size,
                    });
                    drawing_area.queue_draw();
                }
                Tool::Select => {
                    state.selected = editor::hit_test(&state.annotations, pos);
                    state.selected_original = state
                        .selected
                        .and_then(|index| state.annotations.get(index).cloned());
                    drawing_area.queue_draw();
                }
                _ => {}
            }
        });
    }
    drawing_area.add_controller(click);

    {
        let buttons = Rc::new(tool_buttons);
        let state = state.clone();

        for (tool, button) in buttons.iter() {
            let tool = *tool;
            let buttons = buttons.clone();
            let state = state.clone();
            button.connect_toggled(move |active_button| {
                if !active_button.is_active() {
                    return;
                }
                for (_, other) in buttons.iter() {
                    if other != active_button {
                        other.set_active(false);
                    }
                }
                let mut state = state.borrow_mut();
                state.tool = tool;
                state.draft = None;
                state.crop_rect = None;
                state.selected = None;
                state.selected_original = None;
            });
        }

        for (tool, button) in buttons.iter() {
            if *tool == Tool::Pen {
                button.set_active(true);
                break;
            }
        }
    }

    {
        let colors: Vec<(&str, gdk::RGBA)> = vec![
            ("color-black", gdk::RGBA::new(0.11, 0.11, 0.11, 1.0)),
            ("color-white", gdk::RGBA::new(0.96, 0.96, 0.96, 1.0)),
            ("color-red", gdk::RGBA::new(1.0, 0.30, 0.30, 1.0)),
            ("color-orange", gdk::RGBA::new(1.0, 0.62, 0.10, 1.0)),
            ("color-yellow", gdk::RGBA::new(1.0, 0.85, 0.24, 1.0)),
            ("color-green", gdk::RGBA::new(0.24, 0.86, 0.52, 1.0)),
            ("color-blue", gdk::RGBA::new(0.23, 0.51, 0.96, 1.0)),
            ("color-purple", gdk::RGBA::new(0.55, 0.36, 0.96, 1.0)),
        ];

        let buttons: Vec<(gdk::RGBA, gtk::ToggleButton)> = colors
            .iter()
            .map(|(class, color)| {
                let button = gtk::ToggleButton::builder().build();
                button.add_css_class("color-swatch");
                button.add_css_class(class);
                button.set_tooltip_text(Some(&class["color-".len()..]));
                (*color, button)
            })
            .collect();

        for (_, button) in buttons.iter() {
            color_palette.append(button);
        }
        color_palette.append(&color_button);

        let buttons = Rc::new(buttons);
        let state = state.clone();

        for (color, button) in buttons.iter() {
            let color = *color;
            let buttons = buttons.clone();
            let state = state.clone();
            button.connect_toggled(move |active_button| {
                if !active_button.is_active() {
                    return;
                }
                for (_, other) in buttons.iter() {
                    if other != active_button {
                        other.set_active(false);
                    }
                }
                state.borrow_mut().color = color;
            });
        }

        for (color, button) in buttons.iter() {
            if (color.red() - 1.0).abs() < 0.001 && (color.green() - 0.30).abs() < 0.001 {
                button.set_active(true);
                break;
            }
        }
    }
    {
        let state = state.clone();
        let window = window.clone();
        let dialog = color_dialog.clone();
        color_button.connect_clicked(move |_| {
            let current = state.borrow().color;
            dialog.choose_rgba(Some(&window), Some(&current), None::<&gio::Cancellable>, {
                let state = state.clone();
                move |result| {
                    if let Ok(color) = result {
                        state.borrow_mut().color = color;
                    }
                }
            });
        });
    }
    {
        let state = state.clone();
        size_spin.connect_value_changed(move |spin| {
            state.borrow_mut().stroke_width = spin.value();
        });
    }
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        undo_button.connect_clicked(move |_| {
            state.borrow_mut().undo();
            drawing_area.queue_draw();
        });
    }
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        redo_button.connect_clicked(move |_| {
            state.borrow_mut().redo();
            drawing_area.queue_draw();
        });
    }

    {
        let state = state.clone();
        let set_status = set_status.clone();
        copy_button.connect_clicked(move |_| {
            let state = state.borrow();
            let Some(pixbuf) = editor::render_to_pixbuf(&state) else {
                set_status("Nothing to copy yet.");
                return;
            };
            let texture = gdk::Texture::for_pixbuf(&pixbuf);
            if let Some(display) = gdk::Display::default() {
                display.clipboard().set_texture(&texture);
                set_status("Copied to clipboard.");
            } else {
                set_status("Clipboard unavailable.");
            }
        });
    }

    {
        let window = window.clone();
        let set_status = set_status.clone();
        let apply_background = apply_background.clone();
        let file_dialog = gtk::FileDialog::new();
        file_dialog.set_title("Open Image");
        open_button.connect_clicked(move |_| {
            let apply_background = apply_background.clone();
            let set_status = set_status.clone();
            file_dialog.open(Some(&window), None::<&gio::Cancellable>, move |res| {
                match res {
                    Ok(file) => match file.path() {
                        Some(path) => match Pixbuf::from_file(path) {
                            Ok(pixbuf) => {
                                apply_background(pixbuf);
                                set_status("Opened image.");
                            }
                            Err(err) => {
                                let msg = format!("Failed to open image: {err}");
                                set_status(&msg);
                            }
                        },
                        None => set_status("Failed to resolve file path."),
                    },
                    Err(err) => {
                        let msg = format!("Open canceled: {err}");
                        set_status(&msg);
                    }
                }
            });
        });
    }

    {
        let set_status = set_status.clone();
        let apply_background = apply_background.clone();
        paste_button.connect_clicked(move |_| {
            let Some(display) = gdk::Display::default() else {
                set_status("Clipboard unavailable.");
                return;
            };
            let clipboard = display.clipboard();
            clipboard.read_texture_async(None::<&gio::Cancellable>, {
                let set_status = set_status.clone();
                let apply_background = apply_background.clone();
                move |res| match res {
                    Ok(Some(texture)) => {
                        #[allow(deprecated)]
                        if let Some(pixbuf) = gdk::pixbuf_get_from_texture(&texture) {
                            apply_background(pixbuf);
                            set_status("Pasted from clipboard.");
                        } else {
                            set_status("Clipboard image unavailable.");
                        }
                    }
                    Ok(None) => set_status("Clipboard has no image."),
                    Err(err) => {
                        let msg = format!("Paste failed: {err}");
                        set_status(&msg);
                    }
                }
            });
        });
    }

    {
        let window = window.clone();
        let state = state.clone();
        let set_status = set_status.clone();
        let file_dialog = gtk::FileDialog::new();
        file_dialog.set_title("Save PNG");
        save_button.connect_clicked(move |_| {
            let Some(pixbuf) = editor::render_to_pixbuf(&state.borrow()) else {
                set_status("Nothing to save yet.");
                return;
            };
            let texture = gdk::Texture::for_pixbuf(&pixbuf);
            let set_status = set_status.clone();
            file_dialog.save(Some(&window), None::<&gio::Cancellable>, move |res| {
                match res {
                    Ok(file) => match file.path() {
                        Some(mut path) => {
                            if path.extension().is_none() {
                                path.set_extension("png");
                            }
                            match texture.save_to_png(&path) {
                                Ok(()) => set_status("Saved PNG."),
                                Err(err) => {
                                    let msg = format!("Save failed: {err}");
                                    set_status(&msg);
                                }
                            }
                        }
                        None => set_status("Failed to resolve save path."),
                    },
                    Err(err) => {
                        let msg = format!("Save canceled: {err}");
                        set_status(&msg);
                    }
                }
            });
        });
    }

    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let fit_toggle = fit_toggle.clone();
        let zoom_updating = zoom_updating.clone();
        let fit_updating = fit_updating.clone();
        zoom_adjustment.connect_value_changed(move |adj| {
            if zoom_updating.get() {
                return;
            }
            let was_fit = state.borrow().fit_to_window;
            if was_fit {
                fit_updating.set(true);
                fit_toggle.set_active(false);
                fit_updating.set(false);
            }
            let mut state = state.borrow_mut();
            state.fit_to_window = false;
            state.zoom = adj.value();
            drawing_area.queue_draw();
        });
    }
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let zoom_adjustment = zoom_adjustment.clone();
        let zoom_updating = zoom_updating.clone();
        let fit_updating = fit_updating.clone();
        fit_toggle.connect_toggled(move |toggle| {
            if fit_updating.get() {
                return;
            }
            let mut state = state.borrow_mut();
            state.fit_to_window = toggle.is_active();
            if state.fit_to_window {
                let (scale, _, _) = editor::view_transform(&state);
                zoom_updating.set(true);
                zoom_adjustment.set_value(scale);
                zoom_updating.set(false);
            }
            drawing_area.queue_draw();
        });
    }
    {
        let state = state.clone();
        let drawing_area = drawing_area.clone();
        let zoom_adjustment = zoom_adjustment.clone();
        let zoom_updating = zoom_updating.clone();
        zoom_reset.connect_clicked(move |_| {
            let mut state = state.borrow_mut();
            state.fit_to_window = false;
            state.zoom = 1.0;
            zoom_updating.set(true);
            zoom_adjustment.set_value(1.0);
            zoom_updating.set(false);
            drawing_area.queue_draw();
        });
    }
    {
        let state = state.clone();
        let drawing_area_for_scroll = drawing_area.clone();
        let zoom_adjustment = zoom_adjustment.clone();
        let zoom_updating = zoom_updating.clone();
        let fit_updating = fit_updating.clone();
        let fit_toggle = fit_toggle.clone();
        let scroll = gtk::EventControllerScroll::new(gtk::EventControllerScrollFlags::VERTICAL);
        scroll.connect_scroll(move |controller, _, dy| {
            if !controller
                .current_event_state()
                .contains(gdk::ModifierType::CONTROL_MASK)
            {
                return glib::Propagation::Proceed;
            }
            let mut state = state.borrow_mut();
            state.fit_to_window = false;
            fit_updating.set(true);
            fit_toggle.set_active(false);
            fit_updating.set(false);
            let factor = if dy < 0.0 { 1.1 } else { 0.9 };
            state.zoom = (state.zoom * factor).clamp(0.25, 3.0);
            zoom_updating.set(true);
            zoom_adjustment.set_value(state.zoom);
            zoom_updating.set(false);
            drawing_area_for_scroll.queue_draw();
            glib::Propagation::Stop
        });
        drawing_area.add_controller(scroll);
    }
}
