mod app;
mod theme;
mod views;

use app::create;

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([960.0, 720.0])
            .with_min_inner_size([900.0, 600.0])
            .with_title("SECURE-PASSAGE"),
        ..Default::default()
    };

    eframe::run_native(
        "SECURE-PASSAGE",
        options,
        Box::new(|cc| Ok(Box::new(create(cc)))),
    )
}
