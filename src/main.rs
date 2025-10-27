// Import from the library instead of declaring modules
use avida_rs::{debug, ui::AvidaApp};

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), eframe::Error> {
    // Initialize debug system
    debug::init();
    println!("[AVIDA-RS] Starting with debug logging enabled");
    println!("[AVIDA-RS] Use Ctrl+C to exit and see final statistics");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("Avida-RS - Digital Evolution Simulation"),
        ..Default::default()
    };

    let result = eframe::run_native(
        "Avida-RS",
        options,
        Box::new(|cc| Ok(Box::new(AvidaApp::new(cc)))),
    );

    // Print final statistics before exit
    println!("\n[AVIDA-RS] Shutting down...");
    debug::print_stats();

    result
}

#[cfg(target_arch = "wasm32")]
fn main() {
    use wasm_bindgen::JsCast;

    // Set up panic hook for better error messages in the browser
    console_error_panic_hook::set_once();

    // Initialize debug system
    debug::init();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("avida-canvas")
            .expect("Failed to find avida-canvas")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("avida-canvas was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(AvidaApp::new(cc)))),
            )
            .await;

        match start_result {
            Ok(_) => {}
            Err(e) => {
                panic!("Failed to start eframe: {e:?}");
            }
        }
    });
}
