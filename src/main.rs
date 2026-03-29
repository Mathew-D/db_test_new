/*
By: <Your Name Here>
Date: 2025-12-17
Program Details: Turso Database Test - Basic CRUD operations
*/

mod modules;

use crate::modules::database::{create_database_client, create_table_from_struct, DatabaseTable};
use crate::modules::label::Label;
use crate::modules::listview::ListView;
use crate::modules::text_button::TextButton;
use crate::modules::text_input::TextInput;
use macroquad::prelude::*;

/// Set up window settings before the app runs
fn window_conf() -> Conf {
    Conf {
        window_title: "db_test".to_string(),
        window_width: 1024,
        window_height: 768,
        fullscreen: false,
        high_dpi: true,
        window_resizable: true,
        sample_count: 4,
        ..Default::default()
    }
}

/// Update the ListView with current messages
fn update_listview(list_view: &mut ListView, messages: &Vec<DatabaseTable>) {
    list_view.clear();
    let mut items: Vec<String> = Vec::new();
    for (i, msg) in messages.iter().enumerate() {
        items.push(format!("  {}: ID={}, Text={}", i + 1, msg.id, msg.text));
    }
    list_view.add_items(&items);
}

#[macroquad::main(window_conf)]
async fn main() {
    let client = create_database_client();
    let table_name = "messages";
    let mut messages: Vec<DatabaseTable> = Vec::new();
    let mut status = String::from("");

    // Text input for user to set text
    let mut txt_input = TextInput::new(15.0, 20.0, 350.0, 36.0, 22.0);
    txt_input.set_prompt("Enter message text...");

    // Text input for user to set ID to delete
    let mut txt_delete_id = TextInput::new(650.0, 20.0, 100.0, 36.0, 22.0);
    txt_delete_id.set_prompt("ID");

    // UI: Only Add and Delete buttons
    let btn_add = TextButton::new(380.0, 20.0, 120.0, 36.0, "Add", DARKBLUE, BLUE, 22);
    let btn_delete = TextButton::new(510.0, 20.0, 120.0, 36.0, "Delete", MAROON, RED, 22);
    let btn_update = TextButton::new(640.0, 60.0, 120.0, 36.0, "Update", DARKGREEN, GREEN, 22);
    let mut list_view = ListView::new(&Vec::<String>::new(), 15.0, 70.0, 22);
    list_view.with_colors(WHITE, None, Some(DARKGRAY)).with_max_visible_items(20);
    let mut lbl_status = Label::new(&status, 15.0, 60.0 + 22.0 * 20.0 + 10.0, 20);
    lbl_status.with_colors(YELLOW, None);

    // Create table on startup
    let _ = create_table_from_struct(table_name).await;

    // Initial fetch
    if let Ok(records) = client.fetch_table(table_name).await {
        update_listview(&mut list_view, &records);
    } else {
        status = "Fetch error on startup".to_string();
    }

    loop {
        clear_background(BLACK);

        // Handle Add button
        if btn_add.click() {
            let text = txt_input.get_text();
            if !text.trim().is_empty() {
                let new_record = DatabaseTable { id: 0, text: text.clone() };
                if let Ok(_id) = client.insert_record(table_name, &new_record).await {
                    if let Ok(records) = client.fetch_table(table_name).await {
                        update_listview(&mut list_view, &records);
                        status = "Added record".to_string();
                        txt_input.set_text("");
                    } else {
                        status = "Fetch error after add".to_string();
                    }
                } else {
                    status = "Insert error".to_string();
                }
            } else {
                status = "Enter text first".to_string();
            }
        }

        // Handle Delete button (delete by ID from input)
        if btn_delete.click() {
            let id_text = txt_delete_id.get_text();
            if let Ok(id) = id_text.trim().parse::<i64>() {
                if let Ok(_count) = client.delete_record_by_id(table_name, id).await {
                    if let Ok(records) = client.fetch_table(table_name).await {
                        update_listview(&mut list_view, &records);
                        status = format!("Deleted record with ID {}", id);
                        txt_delete_id.set_text("");
                    } else {
                        status = "Fetch error after delete".to_string();
                    }
                } else {
                    status = format!("Delete error for ID {}", id);
                }
            } else {
                status = "Enter a valid ID to delete".to_string();
            }
        }

        // Handle Update button (update by ID from input)
        if btn_update.click() {
            let id_text = txt_delete_id.get_text();
            let new_text = txt_input.get_text();
            if let Ok(id) = id_text.trim().parse::<i64>() {
                if !new_text.trim().is_empty() {
                    if let Ok(_count) = client.update_record_by_id(table_name, id, "text", &new_text).await {
                        if let Ok(records) = client.fetch_table(table_name).await {
                            update_listview(&mut list_view, &records);
                            status = format!("Updated record with ID {}", id);
                        } else {
                            status = "Fetch error after update".to_string();
                        }
                    } else {
                        status = format!("Update error for ID {}", id);
                    }
                } else {
                    status = "Enter new text to update".to_string();
                }
            } else {
                status = "Enter a valid ID to update".to_string();
            }
        }

        // Draw UI

        lbl_status.set_text(&status);
        txt_input.draw();
        txt_delete_id.draw();

        lbl_status.set_text(&status);
        lbl_status.draw();
        list_view.draw();

        next_frame().await;
    }
}
