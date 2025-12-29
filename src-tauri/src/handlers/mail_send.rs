use maud::{html, Markup};
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

use crate::AppState;
use crate::backend::error_handling::{error_id_parse, error_load_person};
//---------------------------

#[tauri::command]
pub fn send(app: tauri::AppHandle) -> String {
    let app_state = app.state::<AppState>();
    let mut mail = app_state.mail.lock();
    let mut other_mail_list = app_state.other_mail_list.lock();
    let config = app_state.config.lock().clone();

    let file_valid = mail.file_is_valid();
    let basic_list_valid = mail.person_list_is_valid();
    let other_mail_list_valid = other_mail_list.is_valid() && !(other_mail_list.is_empty());

    let valid_request = file_valid && (basic_list_valid || other_mail_list_valid);

    if !valid_request {
        println!("not send");
        return html!{
            input.truck
            type="image"
            src="src/assets/send_truck.svg"
            alt="truck-icon"
            hx-trigger="click"
            hx-post="command:send"
            {}
        }.into_string();
    }

    mail.send(other_mail_list.export_other_mail_list(), config, app.clone()).unwrap();

    html!{
        input.truck.drive-animation
        type="image"
        src="src/assets/send_truck.svg"
        alt="truck-icon"
        hx-trigger="click"
        hx-post="command:send"
        hx-swap="outerHTML"
        {}
    }.into_string()
}

#[tauri::command]
pub fn load_mechanics(app: tauri::AppHandle) -> String {
    let app_state = app.state::<AppState>();

    let mail_list = app_state.mail_list.lock();

    let markup: Markup = html! {
        @for i in 0..24 {
            @if let Some(mechanic) = mail_list.load_person(i){
                button.middle-button
                hx-trigger="click"
                hx-post="command:add_person"
                hx-swap="outerHTML"
                hx-vals={(format!(r#""id": {i}"#))}
                {(mechanic.name)}
            }
            @else{
                button.middle-button.placeholder{}
            }
        }
    };

    markup.into_string()
}

#[tauri::command]
pub fn load_technics(app: tauri::AppHandle) -> String {
    let app_state = app.state::<AppState>();

    let mail_list = app_state.mail_list.lock();

    let markup: Markup = html! {
        @for i in 24..29 {
            @if let Some(technic) = mail_list.load_person(i){
                button.middle-button
                hx-trigger="click"
                hx-post="command:add_person"
                hx-swap="outerHTML"
                hx-vals={(format!(r#""id": {i}"#))}
                {(technic.name)}
            }
            @else{
                button.middle-button.placeholder{}
            }
        }
        button.middle-button
        hx-post="command:open_other"
        hx-trigger="click"
        hx-target="#overlay-other-placeholder"
        hx-swap="outerHTML"
        {("ostatní...")}
    };

    markup.into_string()
}

#[tauri::command]
pub fn add_person(id: String, app: tauri::AppHandle) -> String {
    let id: usize = id.parse().unwrap_or_else(|_| error_id_parse(app.clone(), id));
    let app_state = app.state::<AppState>();

    if let Some(person) = app_state.mail_list.lock().load_person(id){
        app_state.mail.lock().add_person(person.clone(), app.clone());

        let markup: Markup = html! {
            button.middle-button.clicked
                hx-trigger="click"
                hx-post="command:remove_person"
                hx-swap="outerHTML"
                hx-vals={(format!(r#""id": {id}"#))}
            {(person.name)}
        };

        return markup.into_string()
    }

    error_load_person(app, id)
}

#[tauri::command]
pub fn remove_person(id: String, app: tauri::AppHandle) -> String {
    let id: usize = id.parse().unwrap_or_else(|_| error_id_parse(app.clone(), id));
    let app_state = app.state::<AppState>();

    if let Some(person) = app_state.mail_list.lock().load_person(id){
        app_state.mail.lock().remove_person(person.clone(), app.clone());

        let markup: Markup = html! {
            button.middle-button
                hx-trigger="click"
                hx-post="command:add_person"
                hx-swap="outerHTML"
                hx-vals={(format!(r#""id": {id}"#))}
            {(person.name)}
        };

        return markup.into_string()
    }

    error_load_person(app, id)
}


#[tauri::command]
pub fn pick_file(app: tauri::AppHandle) -> String {
    app.dialog().file().pick_files(move |file_path| {
        let app_state = app.state::<AppState>();

        if let Some(path) = file_path {
            app_state.mail.lock().add_file(path).unwrap();
        }
    });

    let markup: Markup = html! {
        button.choosen-file-picker
        hx-trigger="click"
        hx-post="command:pick_file"
        hx-swap="outerHTML"
        {"soubor(y) vybrán(y)"}
    };

    markup.into_string()
}

