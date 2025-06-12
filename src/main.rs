use anyhow::Result;
use std::path::PathBuf;
use std::string;
use std::{fs, vec};

use gpui::{
    App, Application, AssetSource, Bounds, Context, KeyBinding, PromptButton, PromptLevel,
    SharedString, Timer, TitlebarOptions, Window, WindowBounds, WindowKind, WindowOptions, actions,
    div, linear_color_stop, linear_gradient, point, prelude::*, px, rgb, rgba, size, svg,
};

// Asset loader for SVG files
struct Assets {
    base: PathBuf,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        let full_path = self.base.join(path);

        match fs::read(&full_path) {
            Ok(data) => Ok(Some(std::borrow::Cow::Owned(data))),
            Err(err) => {
                println!("Failed to load asset: {:?} - Error: {}", full_path, err);
                Err(err.into())
            }
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let full_path = self.base.join(path);

        match fs::read_dir(&full_path) {
            Ok(entries) => {
                let files: Vec<SharedString> = entries
                    .filter_map(|entry| match entry {
                        Ok(entry) => {
                            let file_name = entry.file_name();
                            file_name.into_string().ok().map(SharedString::from)
                        }
                        Err(err) => {
                            println!("Error reading directory entry: {}", err);
                            None
                        }
                    })
                    .collect();

                println!("Listed {} files in directory: {:?}", files.len(), full_path);
                Ok(files)
            }
            Err(err) => {
                println!("Failed to list directory: {:?} - Error: {}", full_path, err);
                Err(err.into())
            }
        }
    }
}

struct SubWindow {
    custom_titlebar: bool,
}

fn button(text: &str, on_click: impl Fn(&mut Window, &mut App) + 'static) -> impl IntoElement {
    div()
        .id(SharedString::from(text.to_string()))
        .flex_none()
        .px_2()
        .bg(rgb(0xf7f7f7))
        .active(|this| this.opacity(0.85))
        .border_1()
        .border_color(rgb(0xe0e0e0))
        .rounded_sm()
        .cursor_pointer()
        .child(text.to_string())
        .on_click(move |_, window, cx| on_click(window, cx))
}

// SVG button component
fn svg_button(
    svg_path: &str,
    size: f32,
    color: impl Into<gpui::Hsla>,
    on_click: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let svg_path = svg_path.to_string(); // Clone the string to own it
    let color = color.into(); // Convert color to owned type

    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size)) // Add padding around SVG
        .rounded_md()
        .cursor_pointer()
        .hover(|this| this.bg(rgba(0x00000010))) // Light hover effect
        .child(
            svg()
                .path(svg_path) // Now using owned string
                .size(px(size))
                .text_color(color), // Now using owned color
        )
}

impl Render for SubWindow {
    fn render(&mut self, _window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .size_full()
            .gap_2()
            .when(self.custom_titlebar, |cx| {
                cx.child(
                    div()
                        .flex()
                        .h(px(32.))
                        .px_4()
                        .bg(gpui::blue())
                        .text_color(gpui::white())
                        .w_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .justify_center()
                                .size_full()
                                .child("Custom Titlebar"),
                        ),
                )
            })
            .child(
                div()
                    .p_8()
                    .gap_2()
                    .child("SubWindow")
                    .child(button("Close", |window, _| {
                        window.remove_window();
                    })),
            )
    }
}

struct WindowDemo {}

impl Render for WindowDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let window_bounds =
            WindowBounds::Windowed(Bounds::centered(None, size(px(300.0), px(300.0)), cx));

        div()
            .border_1()
            .border_color(rgba(0xd3d9d92b))
            .rounded_xl()
            .bg(rgba(0x0404055e))
            .size_full()
            .justify_start()
            .overflow_hidden()
            .content_start()
            .child(
                div()
                    .pl(px(84.)) // Left padding to clear traffic lights
                    .pt(px(10.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap_2()
                            .child(
                                // Back button
                                svg_button("back.svg", 14.0, rgb(0xf2f2f2), |_, _| {
                                    println!("Back clicked!")
                                }),
                            )
                            .child(
                                // Forward button
                                svg_button("forward.svg", 14.0, rgba(0xd3d9d92b), |_, _| {
                                    println!("Forward clicked!")
                                }),
                            )
                            .child(
                                // Plus button (new tab/add)
                                svg_button("rotate-cw.svg", 12.0, rgb(0xf2f2f2), |_, _| {
                                    println!("Plus clicked!")
                                }),
                            )
                            .child(
                                div()
                                    .flex()
                                    .border_1()
                                    .border_color(rgba(0xd3d9d92b))
                                    .rounded_md()
                                    .h_8()
                                    .w_64()
                                    .items_center()
                                    .child(
                                        div()
                                            .flex()
                                            .p_3()
                                            .h_full()
                                            .items_center()
                                            .justify_start()
                                            .bg(linear_gradient(
                                                150.,
                                                linear_color_stop(rgba(0x2e2e2e1c), 0.1), // transparent
                                                linear_color_stop(rgba(0x6161621c), 0.8), // Very dark/black
                                            ))
                                            .w_full()
                                            .rounded_md()
                                            .children(vec![
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .h_full()
                                                    .justify_center()
                                                    .pr_2p5()
                                                    .child(
                                                        svg()
                                                            .path("vercel.svg")
                                                            .size(px(10.0))
                                                            .text_color(rgb(0xfefefe)),
                                                    ),
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .h_full()
                                                    .justify_center()
                                                    .text_color(rgb(0xd1d1d1))
                                                    .text_xs()
                                                    .text_center()
                                                    .line_height(px(10.0))
                                                    .mt(px(1.0))
                                                    .child("vercel.com"),
                                                div()
                                                    .flex()
                                                    .items_center()
                                                    .h_full()
                                                    .justify_center()
                                                    .ml_auto()
                                                    .child(
                                                        svg()
                                                            .path("close.svg")
                                                            .size(px(10.0))
                                                            .text_color(rgba(0xffffffb3)),
                                                    ),
                                            ]),
                                    ),
                            )
                            .child(
                                div()
                                    .px_1()
                                    .py_1()
                                    .bg(linear_gradient(
                                        150.,
                                        linear_color_stop(rgba(0x2e2e2e1c), 0.05), // transparent
                                        linear_color_stop(rgba(0x6161621c), 0.85), // Very dark/black
                                    ))
                                    .border_1()
                                    .border_color(rgba(0xd3d9d92b))
                                    .rounded_md()
                                    .items_center()
                                    .justify_center()
                                    .child(
                                        svg()
                                            .path("plus.svg")
                                            .size(px(12.0))
                                            .text_color(rgb(0xf2f2f2)),
                                    ),
                            ),
                    ),
            )
    }
}

actions!(window, [Quit]);

fn main() {
    Application::new()
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        })
        .run(|cx: &mut App| {
            let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: gpui::WindowBackgroundAppearance::Blurred,
                    titlebar: Some(gpui::TitlebarOptions {
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(16.0), px(18.0))), // Custom position
                        title: Option::Some(SharedString::from("Window Demo")),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                |window, cx| {
                    cx.new(|cx| {
                        cx.observe_window_bounds(window, move |_, window, _| {
                            println!("Window bounds changed: {:?}", window.bounds());
                        })
                        .detach();

                        WindowDemo {}
                    })
                },
            )
            .unwrap();

            cx.activate(true);
            cx.on_action(|_: &Quit, cx| cx.quit());
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        });
}
