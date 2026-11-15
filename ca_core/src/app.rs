/// Start a demo: a native window under `cargo run`, or the page's canvas under `trunk serve`.
///
/// Call it from the demo's `main`. On the web the page must hold a
/// `<canvas id="the_canvas_id">`, and an element `id="loading_text"` if it shows one.
pub fn run<A>(title: &'static str, make_app: impl FnOnce(&eframe::CreationContext<'_>) -> A + 'static)
where
    A: eframe::App + 'static,
{
    #[cfg(not(target_arch = "wasm32"))]
    {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_title(title)
                .with_inner_size([1200.0, 850.0]),
            ..Default::default()
        };
        eframe::run_native(title, options, Box::new(|cc| Ok(Box::new(make_app(cc)))))
            .expect("failed to start the native window");
    }

    #[cfg(target_arch = "wasm32")]
    {
        use eframe::wasm_bindgen::JsCast as _;

        let _ = title; // the page's <title> names the tab
        eframe::WebLogger::init(log::LevelFilter::Debug).ok();

        wasm_bindgen_futures::spawn_local(async move {
            let document = eframe::web_sys::window()
                .expect("no window")
                .document()
                .expect("no document");
            let canvas = document
                .get_element_by_id("the_canvas_id")
                .expect("the page has no element with id the_canvas_id")
                .dyn_into::<eframe::web_sys::HtmlCanvasElement>()
                .expect("the_canvas_id is not a <canvas>");

            let result = eframe::WebRunner::new()
                .start(canvas, eframe::WebOptions::default(), Box::new(|cc| Ok(Box::new(make_app(cc)))))
                .await;

            let loading = document.get_element_by_id("loading_text");
            match result {
                Ok(()) => {
                    if let Some(loading) = loading {
                        loading.remove();
                    }
                }
                Err(e) => {
                    if let Some(loading) = loading {
                        loading.set_inner_html("<p>The app crashed. See the developer console.</p>");
                    }
                    panic!("failed to start eframe: {e:?}");
                }
            }
        });
    }
}
