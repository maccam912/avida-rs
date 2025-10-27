// Import from the library instead of declaring modules
use avida_rs::{debug, ui::AvidaApp};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

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
#[wasm_bindgen]
pub async fn start() -> Result<(), wasm_bindgen::JsValue> {
    // Set up panic hook for better error messages in the browser
    console_error_panic_hook::set_once();

    // Initialize debug system
    debug::init();

    let web_options = eframe::WebOptions::default();

    eframe::WebRunner::new()
        .start(
            "avida-canvas",
            web_options,
            Box::new(|cc| Ok(Box::new(AvidaApp::new(cc)))),
        )
        .await?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
fn main() {
    // WASM entry point is handled by start() function
}
