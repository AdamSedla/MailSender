use maud::{html, Markup};
use tauri::Manager;

use crate::backend::error_handling::error_id_parse;
use crate::AppState;
//---------------------------

#[tauri::command]
pub fn open_other(app: tauri::AppHandle) -> String {
    let app_state = app.state::<AppState>();

    let markup: Markup = html! {
        div #overlay-other .overlay .most-top
        {
            div.overlay-window
            {
                button.close-button
                hx-post="command:close_other"
                hx-trigger="click"
                hx-target="#overlay-other"
                hx-swap="outerHTML"
                {("X")}
                h1.overlay-title{("zadejte prosím E-mailové adresy")}
                div.other-mail-buttons #other-mail-buttons
                {(app_state.other_mail_list.lock().render_input_fields())}
                div.bottom-button-row{
                    button.add-button
                    hx-post="command:add_other_mail_row"
                    hx-trigger="click"
                    hx-target="#other-mail-list-placeholder"
                    hx-swap="outerHTML"
                    {("přidat další E-mail")}
                }
            }
        }
    };
    markup.into_string()
}

#[tauri::command]
pub fn add_other_mail_row(app: tauri::AppHandle) -> String {
    let app_state = app.state::<AppState>();

    let index = app_state.other_mail_list.lock().size();

    let markup: Markup = html! {
        div.other-mail-button-row{
            input.other-mail-input-field
            type="text"
            hx-post="command:edit_mail"
            name="text"
            hx-trigger="change"
            placeholder="Zadejte prosím E-mail"
            hx-vals={(format!(r#""index": {index}"#))}
            {}
            button.remove-button
            hx-post="command:remove_other_row"
            hx-trigger="click"
            hx-target="#other-mail-buttons"
            hx-swap="innerHTML"
            hx-vals={(format!(r#""index": {index}"#))}
            {("odstranit")}
        }

        div #other-mail-list-placeholder {}
    };

    app_state.other_mail_list.lock().increment_size();
    app_state.other_mail_list.lock().add_person();

    markup.into_string()
}

#[tauri::command]
pub fn edit_mail(app: tauri::AppHandle, index: String, text: String) {
    let app_state = app.state::<AppState>();

    let index: usize = index
        .parse()
        .unwrap_or_else(|_| error_id_parse(app.clone(), index));

    app_state.other_mail_list.lock().edit_person(&text, index);
}

#[tauri::command]
pub fn remove_other_row(app: tauri::AppHandle, index: String) -> String {
    let app_state: tauri::State<'_, AppState> = app.state::<AppState>();
    let index: usize = index
        .parse()
        .unwrap_or_else(|_| error_id_parse(app.clone(), index));

    app_state.other_mail_list.lock().remove_person(index);

    let markup: Markup = app_state.other_mail_list.lock().render_input_fields();

    markup.into_string()
}

#[tauri::command]
pub fn close_other(app: tauri::AppHandle) -> String {
    let app_state = app.state::<AppState>();

    app_state.other_mail_list.lock().remove_empty_persons();

    let has_other_mails = !app_state.other_mail_list.lock().is_empty();

    let markup: Markup = html! {
        @if has_other_mails {
            div #overlay-other-placeholder {}
            div
            hx-post="command:mark_other"
            hx-trigger="load delay:1ms"
            hx-target="#other-mails-button"
            hx-swap="outerHTML"
            {}
        }
        @else {
            div #overlay-other-placeholder {}
            div
            hx-post="command:unmark_other"
            hx-trigger="load delay:1ms"
            hx-target="#other-mails-button"
            hx-swap="outerHTML"
            {}
        }
    };

    markup.into_string()
}

#[tauri::command]
pub fn mark_other() -> String {
    let markup: Markup = html! {
        button.middle-button .clicked #other-mails-button
        hx-post="command:open_other"
        hx-trigger="click"
        hx-target="#overlay-other-placeholder"
        hx-swap="outerHTML"
        {("ostatní...")}
    };

    markup.into_string()
}

#[tauri::command]
pub fn unmark_other() -> String {
    let markup: Markup = html! {
        button.middle-button #other-mails-button
        hx-post="command:open_other"
        hx-trigger="click"
        hx-target="#overlay-other-placeholder"
        hx-swap="outerHTML"
        {("ostatní...")}
    };

    markup.into_string()
}
