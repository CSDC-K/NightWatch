#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use slint::{ModelRc, SharedString, VecModel};
use std::error::Error;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use chrono::Local;

slint::include_modules!();

#[derive(Serialize, Deserialize, Clone, Debug)]
struct WatchedItemData {
    id: i32,
    name: String,
    url: String,
    current_price: String,
    last_updated: String,
}

#[derive(Serialize, Deserialize, Default)]
struct AppData {
    items: Vec<WatchedItemData>,
    next_id: i32,
}

fn get_data_file_path() -> PathBuf {
    let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("NightWatch");
    fs::create_dir_all(&path).ok();
    path.push("watched_items.json");
    path
}

fn load_app_data() -> AppData {
    let path = get_data_file_path();
    if path.exists() {
        match fs::read_to_string(&path) {
            Ok(content) => {
                serde_json::from_str(&content).unwrap_or_default()
            }
            Err(_) => AppData::default(),
        }
    } else {
        AppData::default()
    }
}

fn save_app_data(data: &AppData) -> Result<(), Box<dyn Error>> {
    let path = get_data_file_path();
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;
    Ok(())
}

fn convert_to_slint_items(items: &[WatchedItemData]) -> ModelRc<WatchedItem> {
    let slint_items: Vec<WatchedItem> = items
        .iter()
        .map(|item| WatchedItem {
            id: item.id,
            name: SharedString::from(&item.name),
            url: SharedString::from(&item.url),
            current_price: SharedString::from(&item.current_price),
            last_updated: SharedString::from(&item.last_updated),
        })
        .collect();
    ModelRc::new(VecModel::from(slint_items))
}

fn main() -> Result<(), Box<dyn Error>> {
    let ui = AppWindow::new()?;
    


    // Load saved data
    let app_data = Arc::new(Mutex::new(load_app_data()));
    
    // Set initial items
    {
        let data = app_data.lock().unwrap();
        ui.set_watched_items(convert_to_slint_items(&data.items));
    }

    // Handle add item
    let ui_weak = ui.as_weak();
    let app_data_clone = Arc::clone(&app_data);
    ui.on_add_item(move |name, url| {
        let mut data = app_data_clone.lock().unwrap();
        let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
        
        let new_item = WatchedItemData {
            id: data.next_id,
            name: name.to_string(),
            url: url.to_string(),
            current_price: "Fetching...".to_string(),
            last_updated: now,
        };
        
        data.next_id += 1;
        data.items.push(new_item);
        
        if let Err(e) = save_app_data(&data) {
            eprintln!("Failed to save data: {}", e);
        }
        
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_watched_items(convert_to_slint_items(&data.items));
        }
    });

    // Handle remove item
    let ui_weak = ui.as_weak();
    let app_data_clone = Arc::clone(&app_data);
    ui.on_remove_item(move |id| {
        let mut data = app_data_clone.lock().unwrap();
        data.items.retain(|item| item.id != id);
        
        if let Err(e) = save_app_data(&data) {
            eprintln!("Failed to save data: {}", e);
        }
        
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_watched_items(convert_to_slint_items(&data.items));
        }
    });

    // Handle refresh item
    let ui_weak = ui.as_weak();
    let app_data_clone = Arc::clone(&app_data);
    ui.on_refresh_item(move |id| {
        let mut data = app_data_clone.lock().unwrap();
        let now = Local::now().format("%Y-%m-%d %H:%M").to_string();
        
        if let Some(item) = data.items.iter_mut().find(|i| i.id == id) {
            // For now, just update the timestamp
            // In a real implementation, you would fetch the price here
            item.last_updated = now;
            item.current_price = "Updated".to_string();
        }
        
        if let Err(e) = save_app_data(&data) {
            eprintln!("Failed to save data: {}", e);
        }
        
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_watched_items(convert_to_slint_items(&data.items));
        }
    });

    // Handle theme change
    let ui_weak = ui.as_weak();
    ui.on_theme_changed(move |is_dark| {
        if let Some(ui) = ui_weak.upgrade() {
            let app_theme = ui.global::<UAppTheme>();
            if is_dark {
                app_theme.set_color_scheme(UColorScheme::Dark);
            } else {
                app_theme.set_color_scheme(UColorScheme::Light);
            }
        }
    });

    // Handle load items (for initial load or refresh)
    let ui_weak = ui.as_weak();
    let app_data_clone = Arc::clone(&app_data);
    ui.on_load_items(move || {
        let data = app_data_clone.lock().unwrap();
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_watched_items(convert_to_slint_items(&data.items));
        }
    });

    ui.run()?;
    Ok(())
}
