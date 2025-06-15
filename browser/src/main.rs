use anyhow::Result;
use std::{
    fs::create_dir_all,
    os::raw::c_void,
    path::PathBuf,
    process::exit,
    sync::{Arc, Mutex},
};

use cef_ui::{
    AccessibilityHandler, App, AppCallbacks, Browser, BrowserHost, BrowserSettings, Client,
    ClientCallbacks, CommandLine, Context, ContextMenuHandler, ContextMenuHandlerCallbacks,
    ContextMenuParams, DictionaryValue, DragData, DragOperations, EventFlags, Frame,
    HorizontalAlignment, KeyboardHandler, LifeSpanHandler, LifeSpanHandlerCallbacks, LogSeverity,
    MainArgs, MenuCommandId, MenuModel, PaintElementType, Point, PopupFeatures,
    QuickMenuEditStateFlags, Range, Rect, RenderHandler, RenderHandlerCallbacks,
    RunContextMenuCallback, RunQuickMenuCallback, ScreenInfo, Settings, Size, TextInputMode,
    TouchHandleState, WindowInfo, WindowOpenDisposition,
};

use gpui::{
    actions, div, img, linear_color_stop, linear_gradient, point, prelude::*, px, rgb, rgba, size,
    svg, App as GpuiApp, Application, AssetSource, Bounds, Global, Image, ImageSource, KeyBinding,
    SharedString, Window, WindowBounds, WindowOptions,
};

// Asset loader for SVG files
struct Assets {
    base: PathBuf,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<std::borrow::Cow<'static, [u8]>>> {
        let full_path = self.base.join(path);
        match std::fs::read(&full_path) {
            Ok(data) => Ok(Some(std::borrow::Cow::Owned(data))),
            Err(err) => {
                println!("Failed to load asset: {:?} - Error: {}", full_path, err);
                Err(err.into())
            }
        }
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        let full_path = self.base.join(path);
        match std::fs::read_dir(&full_path) {
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

// Browser state that will be managed by GPUI
struct BrowserState {
    browser: Option<Browser>,
    context: Option<Context>,
    image: Option<Image>,
}

impl Global for BrowserState {}

// SVG button component
fn svg_button(
    svg_path: &str,
    size: f32,
    color: impl Into<gpui::Hsla>,
    _on_click: impl Fn(&mut Window, &mut GpuiApp) + 'static,
) -> impl IntoElement {
    let svg_path = svg_path.to_string();
    let color = color.into();

    div()
        .flex()
        .items_center()
        .justify_center()
        .size(px(size))
        .rounded_md()
        .cursor_pointer()
        .hover(|this| this.bg(rgba(0x00000010)))
        .child(svg().path(svg_path).size(px(size)).text_color(color))
}

struct WindowDemo {}

impl Render for WindowDemo {
    fn render(
        &mut self,
        _window: &mut Window,
        cx: &mut gpui::Context<'_, WindowDemo>,
    ) -> impl IntoElement {
        let state = cx.global::<BrowserState>();

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
                                // Refresh button
                                svg_button("rotate-cw.svg", 12.0, rgb(0xf2f2f2), |_, _| {
                                    println!("Refresh clicked!")
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
                                                linear_color_stop(rgba(0x2e2e2e1c), 0.1),
                                                linear_color_stop(rgba(0x6161621c), 0.8),
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
                                        linear_color_stop(rgba(0x2e2e2e1c), 0.05),
                                        linear_color_stop(rgba(0x6161621c), 0.85),
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
            // Render the browser content
            .child(
                div()
                    .flex()
                    .flex_1()
                    .bg(rgb(0xffffff))
                    .items_center()
                    .justify_center()
                    .child(if let Some(image) = &state.image {
                        div()
                            .size_full()
                            .child(img(ImageSource::from(Arc::new(image.clone()))))
                    } else {
                        div().child("Loading...")
                    }),
            )
    }
}

actions!(window, [Quit]);

// CEF Handlers
pub struct MyContextMenuHandler;

impl ContextMenuHandlerCallbacks for MyContextMenuHandler {
    fn on_before_context_menu(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _params: ContextMenuParams,
        model: MenuModel,
    ) {
        // Prevent context menus from spawning
        if let Err(e) = model.clear() {
            eprintln!("Error clearing context menu: {}", e);
        }
    }

    fn run_context_menu(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _params: ContextMenuParams,
        _model: MenuModel,
        _callback: RunContextMenuCallback,
    ) -> bool {
        false
    }

    fn on_context_menu_command(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _params: ContextMenuParams,
        _command_id: MenuCommandId,
        _event_flags: EventFlags,
    ) -> bool {
        false
    }

    fn on_context_menu_dismissed(&mut self, _browser: Browser, _frame: Frame) {}

    fn run_quick_menu(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _location: &Point,
        _size: &Size,
        _edit_state_flags: QuickMenuEditStateFlags,
        _callback: RunQuickMenuCallback,
    ) -> bool {
        false
    }

    fn on_quick_menu_command(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _command_id: MenuCommandId,
        _event_flags: EventFlags,
    ) -> bool {
        false
    }

    fn on_quick_menu_dismissed(&mut self, _browser: Browser, _frame: Frame) {}
}

pub struct MyLifeSpanHandlerCallbacks;

impl LifeSpanHandlerCallbacks for MyLifeSpanHandlerCallbacks {
    unsafe fn on_before_popup(
        &mut self,
        _browser: Browser,
        _frame: Frame,
        _target_url: Option<String>,
        _target_frame_name: Option<String>,
        _target_disposition: WindowOpenDisposition,
        _user_gesture: bool,
        _popup_features: PopupFeatures,
        _window_info: &mut WindowInfo,
        _client: &mut Option<Client>,
        _settings: &mut BrowserSettings,
        _extra_info: &mut Option<DictionaryValue>,
        _no_javascript_access: &mut bool,
    ) -> bool {
        true // Block popups
    }

    fn on_before_dev_tools_popup(
        &mut self,
        _browser: Browser,
        _window_info: &mut WindowInfo,
        _client: &mut Option<Client>,
        _settings: &mut BrowserSettings,
        _extra_info: &mut Option<DictionaryValue>,
        _use_default_window: &mut bool,
    ) {
    }

    fn on_after_created(&mut self, _browser: Browser) {}

    fn do_close(&mut self, _browser: Browser) -> bool {
        false
    }

    fn on_before_close(&mut self, _browser: Browser) {
        // Quit CEF when browser closes
        unsafe {
            cef_ui_sys::cef_quit_message_loop();
        }
    }
}

pub struct MyClientCallbacks;

impl ClientCallbacks for MyClientCallbacks {
    fn get_context_menu_handler(&mut self) -> Option<ContextMenuHandler> {
        Some(ContextMenuHandler::new(MyContextMenuHandler {}))
    }

    fn get_keyboard_handler(&mut self) -> Option<KeyboardHandler> {
        None
    }

    fn get_life_span_handler(&mut self) -> Option<LifeSpanHandler> {
        Some(LifeSpanHandler::new(MyLifeSpanHandlerCallbacks {}))
    }

    fn get_render_handler(&mut self) -> Option<RenderHandler> {
        Some(RenderHandler::new(MyRenderHandler::new()))
    }
}

/// Render handler for windowless rendering
pub struct MyRenderHandler {
    view_size: Arc<Mutex<Size>>,
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl MyRenderHandler {
    fn new() -> Self {
        Self {
            view_size: Arc::new(Mutex::new(Size {
                width: 1024,
                height: 768,
            })),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl RenderHandlerCallbacks for MyRenderHandler {
    fn get_view_rect(&mut self, browser: Browser) -> Rect {
        let size = *self.view_size.lock().unwrap();
        Rect {
            x: 0,
            y: 0,
            width: size.width,
            height: size.height,
        }
    }

    fn get_screen_point(&mut self, browser: Browser, view: &Point) -> Option<Point> {
        // For windowless rendering, we can just return the same point
        Some(*view)
    }

    fn get_screen_info(&mut self, browser: Browser) -> Option<ScreenInfo> {
        let rect = self.get_view_rect(browser);
        let mut info = ScreenInfo {
            device_scale_factor: 1.0,
            depth: 32,
            depth_per_component: 8,
            is_monochrome: false,
            rect,
            available_rect: rect,
        };
        Some(info)
    }

    fn on_paint(
        &mut self,
        browser: Browser,
        element_type: PaintElementType,
        dirty_rects: &[Rect],
        buffer: &[u8],
        width: usize,
        height: usize,
    ) {
        // Print first few items in the buffer
        println!(
            "Paint event - Element type: {:?}, Width: {}, Height: {}",
            element_type, width, height
        );
        println!(
            "First 10 bytes of buffer: {:?}",
            &buffer[..std::cmp::min(10, buffer.len())]
        );

        // Store the buffer data
        let mut current_buffer = self.buffer.lock().unwrap();
        current_buffer.clear();
        current_buffer.extend_from_slice(buffer);

        // Update view size if needed
        let mut current_size = self.view_size.lock().unwrap();
        if current_size.width != width as i32 || current_size.height != height as i32 {
            *current_size = Size {
                width: width as i32,
                height: height as i32,
            };
        }
    }

    fn get_accessibility_handler(&mut self) -> Option<AccessibilityHandler> {
        None
    }

    fn get_root_screen_rect(&mut self, browser: Browser) -> Option<Rect> {
        Some(self.get_view_rect(browser))
    }

    fn on_popup_show(&mut self, browser: Browser, show: bool) {
        // Handle popup show/hide
    }

    fn on_popup_size(&mut self, browser: Browser, rect: &Rect) {
        // Handle popup size changes
    }

    fn on_accelerated_paint(
        &mut self,
        browser: Browser,
        element_type: PaintElementType,
        dirty_rects: &[Rect],
        shared_handle: *mut c_void,
    ) {
        // Handle accelerated painting if needed
    }

    fn get_touch_handle_size(
        &mut self,
        browser: Browser,
        orientation: HorizontalAlignment,
    ) -> Size {
        Size {
            width: 0,
            height: 0,
        }
    }

    fn on_touch_handle_state_changed(&mut self, browser: Browser, state: &TouchHandleState) {
        // Handle touch handle state changes
    }

    fn start_dragging(
        &mut self,
        browser: Browser,
        drag_data: DragData,
        allowed_ops: DragOperations,
        point: &Point,
    ) -> bool {
        false
    }

    fn update_drag_cursor(&mut self, browser: Browser, operation: DragOperations) {
        // Update drag cursor
    }

    fn on_scroll_offset_changed(&mut self, browser: Browser, x: f64, y: f64) {
        // Handle scroll offset changes
    }

    fn on_ime_composition_range_changed(
        &mut self,
        browser: Browser,
        selected_range: &Range,
        character_bounds: &[Rect],
    ) {
        // Handle IME composition range changes
    }

    fn on_text_selection_changed(
        &mut self,
        browser: Browser,
        selected_text: Option<String>,
        selected_range: &Range,
    ) {
        // Handle text selection changes
    }

    fn on_virtual_keyboard_requested(&mut self, browser: Browser, input_mode: TextInputMode) {
        // Handle virtual keyboard requests
    }
}

pub struct MyAppCallbacks;

impl AppCallbacks for MyAppCallbacks {
    fn on_before_command_line_processing(
        &mut self,
        _process_type: Option<&str>,
        _command_line: Option<CommandLine>,
    ) {
    }

    fn get_browser_process_handler(&mut self) -> Option<cef_ui::BrowserProcessHandler> {
        None
    }
}

pub fn get_root_cache_dir() -> Result<PathBuf> {
    let path = PathBuf::from("/tmp/browser");
    create_dir_all(&path)?;
    Ok(path)
}

fn initialize_cef() -> Result<Context, Box<dyn std::error::Error>> {
    let root_cache_dir = get_root_cache_dir()?;
    let main_args = MainArgs::new()?;

    let settings = Settings::new()
        .log_severity(LogSeverity::Info)
        .root_cache_path(&root_cache_dir)?
        .windowless_rendering_enabled(true)
        .no_sandbox(false);

    let app = App::new(MyAppCallbacks {});
    let context = Context::new(main_args, settings, Some(app));

    // Check if this is a CEF subprocess
    if let Some(code) = context.is_cef_subprocess() {
        exit(code);
    }

    // Initialize CEF
    context.initialize()?;

    Ok(context)
}

fn create_browser() -> Result<Browser, Box<dyn std::error::Error>> {
    let window_info = WindowInfo::new()
        .window_name(&String::from("browser"))
        .windowless_rendering_enabled(true);

    let browser_settings = BrowserSettings::new();

    let client = Client::new(MyClientCallbacks);

    // BrowserHost::create_browser_sync returns Browser directly, not Result
    let browser = BrowserHost::create_browser_sync(
        &window_info,
        client,
        "https://www.google.com",
        &browser_settings,
        None,
        None,
    );

    Ok(browser)
}

fn initialize_browser_in_context(cx: &mut GpuiApp) -> Result<(), Box<dyn std::error::Error>> {
    let context = initialize_cef()?;
    let browser = create_browser()?;

    let state = cx.global_mut::<BrowserState>();
    state.context = Some(context);
    state.browser = Some(browser);

    Ok(())
}

fn try_main() -> Result<()> {
    Application::new()
        .with_assets(Assets {
            base: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets"),
        })
        .run(|cx: &mut GpuiApp| {
            // Initialize browser state in GPUI context
            cx.set_global(BrowserState {
                browser: None,
                context: None,
                image: None,
            });

            // Initialize CEF and browser
            if let Err(e) = initialize_browser_in_context(cx) {
                eprintln!("Failed to initialize browser: {:?}", e);
                return;
            }

            let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_background: gpui::WindowBackgroundAppearance::Blurred,
                    titlebar: Some(gpui::TitlebarOptions {
                        appears_transparent: true,
                        traffic_light_position: Some(point(px(16.0), px(18.0))),
                        title: Some(SharedString::from("CEF Browser Demo")),
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

            // Run the message loop
            if let Some(context) = cx.global::<BrowserState>().context.as_ref() {
                context.run_message_loop();
            }

            cx.on_action(|_: &Quit, cx| {
                // Cleanup using GPUI's global state
                let state = cx.global_mut::<BrowserState>();
                if let Some(context) = state.context.take() {
                    context.shutdown();
                }
                cx.quit();
            });
            cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
        });

    Ok(())
}

fn main() {
    if let Err(e) = try_main() {
        eprintln!("Error: {}", e);
        exit(1);
    }
}
