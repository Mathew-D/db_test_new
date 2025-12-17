/*
By: <Your Name Here>
Date: 2025-12-17
Program Details: Turso Database Test - Basic CRUD operations
*/

mod modules;

use crate::modules::database::{create_database_client, create_messages_table, DatabaseTable};
use crate::modules::label::Label;
use crate::modules::listview::ListView;
use crate::modules::text_button::TextButton;
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
    let mut status = String::from("Initializing...");
    let mut should_fetch = false;

    // Create UI elements
    let mut lbl_title = Label::new("=== Turso Database Test ===", 10.0, 30.0, 28);
    lbl_title.with_colors(WHITE, None);

    let mut lbl_status = Label::new(&status, 10.0, 70.0, 20);
    lbl_status.with_colors(YELLOW, None);

    let mut lbl_table = Label::new(&format!("Table: {}", table_name), 10.0, 95.0, 18);
    lbl_table.with_colors(GRAY, None);

    let mut lbl_records_header = Label::new("Records:", 10.0, 130.0, 20);
    lbl_records_header.with_colors(GRAY, None);

    let mut lbl_empty = Label::new("  (no records - click Insert)", 15.0, 155.0, 18);
    lbl_empty.with_colors(GRAY, None);

    // Create ListView for displaying records
    let mut list_view = ListView::new(&Vec::<String>::new(), 15.0, 165.0, 18);
    list_view.with_colors(WHITE, None, Some(DARKGRAY)).with_max_visible_items(20);

    let btn_insert = TextButton::new(10.0, screen_height() - 120.0, 180.0, 50.0, "Insert", DARKBLUE, BLUE, 18);
    let btn_clear = TextButton::new(200.0, screen_height() - 120.0, 180.0, 50.0, "Clear", DARKPURPLE, PURPLE, 18);
    let btn_refresh = TextButton::new(390.0, screen_height() - 120.0, 180.0, 50.0, "Refresh", DARKGREEN, GREEN, 18);
    let btn_exit = TextButton::new(580.0, screen_height() - 120.0, 180.0, 50.0, "Exit", MAROON, RED, 18);

    let mut lbl_instructions = Label::new("Click buttons to interact with database", 10.0, screen_height() - 50.0, 16);
    lbl_instructions.with_colors(GRAY, None);

    // Create table on startup
    match create_messages_table(table_name).await {
        Ok(_) => {
            status = format!("Table '{}' created/verified", table_name);
            should_fetch = true;
        }
        Err(e) => {
            status = format!("Error creating table: {}", e);
        }
    }

    loop {
        clear_background(BLACK);
        if should_fetch {
            println!("Fetching records...");
            match client.fetch_table(table_name).await {
                Ok(records) => {
                    messages = records;
                    update_listview(&mut list_view, &messages);
                }
                Err(e) => {
                    status = format!("Fetch error: {}", e);
                }
            }
            should_fetch = false;
        }

        // Handle button clicks
        if btn_insert.click() {
            // Insert a test record
            let new_record = DatabaseTable {
                id: 0,
                text: format!("Message {}", (messages.len() + 1)),
            };

            match client.insert_record(table_name, &new_record).await {
                Ok(id) => {
                    status = format!("Inserted with ID: {}", id);
                    messages.push(DatabaseTable {
                        id: id as i32,
                        text: new_record.text,
                    });
                    update_listview(&mut list_view, &messages);
                }
                Err(e) => {
                    status = format!("Insert error: {}", e);
                }
            }
        }

        if btn_refresh.click() {
            // Refresh records
            should_fetch = true;
        }

        if btn_clear.click() {
            // Clear all records
            match client.clear_table(table_name).await {
                Ok(count) => {
                    status = format!("Cleared {} records", count);
                    messages.clear();
                    list_view.clear();
                }
                Err(e) => {
                    status = format!("Clear error: {}", e);
                }
            }
        }

        if btn_exit.click() {
            break;
        }

        // Draw all labels at the end
        lbl_title.draw();
        lbl_status.set_text(&status);
        lbl_status.draw();
        lbl_table.draw();
        lbl_records_header.draw();

        list_view.draw();

        lbl_instructions.draw();

        next_frame().await;
    }
}
