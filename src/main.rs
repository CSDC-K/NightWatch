#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use slint;
use std::error::Error;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>>{
    let ui = AppWindow::new()?;
    let app_theme = ui.global::<UAppTheme>();
    app_theme.set_color_scheme(UColorScheme::Dark);
    app_theme.set_scale_factor(1.5);


    ui.run()?;
    Ok(())

}
