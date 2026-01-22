use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::SmtpTransport;
use lettre::transport::Transport;
use lettre::{Address, Message};
use std::sync::LazyLock;
use tauri::Manager;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

use crate::backend::config::{create_empty_config, empty_config, Config};
use crate::backend::mail_list_utils::{create_empty_mail_list, empty_mail_list, MailList};
use crate::backend::mail_sender::MailSenderError;
use crate::AppState;

//---------------------------
pub fn error_pick_file(app: tauri::AppHandle) {
    let error_message: String = format!("Došlo k chybě při výběru souboru.");

    let _ = send_error_mail(error_message);

    show_file_pick_user_error_and_continue(app);
}

pub fn error_sending_mail(app: tauri::AppHandle, error: MailSenderError) {
    let error_message: String = format!("Došlo k chybě při odesílání mailu. \n\n {error}");

    //if there's connection Error, just show notification to user
    if let MailSenderError::ErrorOpeningSMTP(error) = error {
        if error.to_string().contains("Connection error") {
            show_connection_user_error_and_continue(app);
            return;
        }
    }

    let _ = send_error_mail(error_message);

    show_sending_user_error_and_continue(app);
}

pub fn error_load_person(app: tauri::AppHandle, original_id: usize) -> String {
    let error_message: String =
        format!("Neúspěšný pokus o načtení osoby z databáze. ID této osoby: {original_id}");

    let _ = send_error_mail(error_message);

    show_unexpected_user_error_and_quit(app);

    "".to_string()
}

pub fn error_id_parse(app: tauri::AppHandle, original_id: String) -> usize {
    let error_message: String =
        format!("Neúspěšný pokus o ID_parse. originální String: {original_id}");

    let _ = send_error_mail(error_message);

    show_unexpected_user_error_and_quit(app);

    0
}

pub fn error_parsing_mail_address(app: tauri::AppHandle, original_mail: String) -> Address {
    let error_message: String =
        format!("Neúspěšný pokus o parse E-mailové adresy. originální String: {original_mail}");

    show_unexpected_user_error_and_quit(app);

    let _ = send_error_mail(error_message);

    //error@error is valid Addres, so else block is unreachable
    Address::new("error".to_string(), "error".to_string()).unwrap_or_else(|_| unreachable!())
}

pub fn error_saving_config(app: tauri::AppHandle) {
    //Depending on the platform, this function may fail if the full directory path does not exist.
    //this should't happen as file path is just name of the file.
    let app_state = app.state::<AppState>();

    let error_message: String = format!(
        "Nepodařilo se uložit config. \n\n config:\n{:?}",
        app_state.config
    );

    let _ = send_error_mail(error_message);

    show_error_saving_config_and_continue(app);
}

pub fn error_saving_mail_list(app: tauri::AppHandle) {
    //Depending on the platform, this function may fail if the full directory path does not exist.
    //this should't happen as file path is just name of the file.
    let app_state = app.state::<AppState>();

    let error_message: String = format!(
        "Nepodařilo se uložit mail list. \n\n mail_list:\n{:?}",
        app_state.mail_list
    );

    let _ = send_error_mail(error_message);

    show_error_saving_mail_list_and_continue(app);
}

pub fn error_loading_config(app: tauri::AppHandle) -> String {
    let error_message: String = format!("Nepodařilo se načíst config.");

    let _ = send_error_mail(error_message);

    show_error_loading_config_and_continue(app.clone());

    create_empty_config(app)
}

pub fn error_loading_mail_list(app: tauri::AppHandle) -> String {
    let error_message: String = format!("Nepodařilo se načíst mail_list.");

    let _ = send_error_mail(error_message);

    show_error_loading_mail_list_and_continue(app.clone());

    create_empty_mail_list(app)
}

pub fn error_parsing_config_to_string(app: tauri::AppHandle) -> String {
    let app_state = app.state::<AppState>();

    let error_message: String = format!(
        "Nepodařilo se naparsovat config. \n\n config:\n{:?}",
        app_state.config
    );

    let _ = send_error_mail(error_message);

    show_error_saving_config_and_continue(app);

    "".to_string()
}

pub fn error_parsing_mail_list_to_string(app: tauri::AppHandle) -> String {
    let app_state = app.state::<AppState>();

    let error_message: String = format!(
        "Nepodařilo se uložit mail list. \n\n mail_list:\n{:?}",
        app_state.mail_list
    );

    let _ = send_error_mail(error_message);

    show_error_saving_mail_list_and_continue(app);

    "".to_string()
}

pub fn error_decoding_config_from_string(app: tauri::AppHandle, raw_config: &str) -> Config {
    let error_message: String = format!("Nepodařilo se dekodovat config.\nConfig:\n{}", raw_config);

    let _ = send_error_mail(error_message);

    show_error_loading_config_and_continue(app.clone());

    create_empty_config(app);
    empty_config()
}

pub fn error_decoding_mail_list_from_string(
    app: tauri::AppHandle,
    raw_mail_list: &str,
) -> MailList {
    let error_message: String = format!(
        "Nepodařilo se dekodovat mail_list.\nMail_list:\n{}",
        raw_mail_list
    );

    let _ = send_error_mail(error_message);

    show_error_loading_mail_list_and_continue(app.clone());

    create_empty_mail_list(app);
    empty_mail_list()
}

pub fn fail_back_system_error(app: tauri::AppHandle) {
    let error_message: String =
        "Nepodařilo se uložit prázdný config/mail_list v rámci fail_back systému.".to_string();

    let _ = send_error_mail(error_message);

    show_unexpected_user_error_and_quit(app)
}

pub fn show_error_saving_config_and_continue(app: tauri::AppHandle) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při ukládání configuračního souboru";
    static ERROR_MESSAGE_TEXT: &str = "Nebylo možné uložit config.\n\nAutor aplikace byl informován.\n\nInformujte prosím vedoucího.";

    app.dialog()
        .message(ERROR_MESSAGE_TEXT.to_string())
        .kind(MessageDialogKind::Info)
        .title(ERROR_MESSAGE_TITLE.to_string())
        .buttons(MessageDialogButtons::OkCustom("OK".to_string()))
        .show(|result| match result {
            true => (),
            false => (),
        });
}

pub fn show_error_saving_mail_list_and_continue(app: tauri::AppHandle) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při ukládání seznamu osob";
    static ERROR_MESSAGE_TEXT: &str = "Nebylo možné uložit seznam osob.\n\nAutor aplikace byl informován.\n\nInformujte prosím vedoucího.";

    app.dialog()
        .message(ERROR_MESSAGE_TEXT.to_string())
        .kind(MessageDialogKind::Info)
        .title(ERROR_MESSAGE_TITLE.to_string())
        .buttons(MessageDialogButtons::OkCustom("OK".to_string()))
        .show(|result| match result {
            true => (),
            false => (),
        });
}

pub fn show_error_loading_config_and_continue(app: tauri::AppHandle) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při načítání configuračního souboru";
    static ERROR_MESSAGE_TEXT: &str = "Nebylo možné načíst config.\n\nAutor aplikace byl informován.\n\nInformujte prosím vedoucího.";

    app.dialog()
        .message(ERROR_MESSAGE_TEXT.to_string())
        .kind(MessageDialogKind::Info)
        .title(ERROR_MESSAGE_TITLE.to_string())
        .buttons(MessageDialogButtons::OkCustom("OK".to_string()))
        .show(|result| match result {
            true => (),
            false => (),
        });
}

pub fn show_error_loading_mail_list_and_continue(app: tauri::AppHandle) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při načítání seznamu osob";
    static ERROR_MESSAGE_TEXT: &str = "Nebylo možné načíst seznam osob.\n\nAutor aplikace byl informován.\n\nInformujte prosím vedoucího.";

    app.dialog()
        .message(ERROR_MESSAGE_TEXT.to_string())
        .kind(MessageDialogKind::Info)
        .title(ERROR_MESSAGE_TITLE.to_string())
        .buttons(MessageDialogButtons::OkCustom("OK".to_string()))
        .show(|result| match result {
            true => (),
            false => (),
        });
}

pub fn show_file_pick_user_error_and_continue(app: tauri::AppHandle) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při výběru souboru";
    static ERROR_MESSAGE_TEXT: &str = "Nebylo možné vybrat soubor.\n\nAutor aplikace byl informován.\n\nInformujte prosím vedoucího.";

    app.dialog()
        .message(ERROR_MESSAGE_TEXT.to_string())
        .kind(MessageDialogKind::Info)
        .title(ERROR_MESSAGE_TITLE.to_string())
        .buttons(MessageDialogButtons::OkCustom("OK".to_string()))
        .show(|result| match result {
            true => (),
            false => (),
        });
}

pub fn show_connection_user_error_and_continue(app: tauri::AppHandle) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při odesílání E-mailu";
    static ERROR_MESSAGE_TEXT: &str =
        "Aplikace pravděpodobně nemá přístup k internetu.\n\nInformujte prosím vedoucího.";

    app.dialog()
        .message(ERROR_MESSAGE_TEXT.to_string())
        .kind(MessageDialogKind::Info)
        .title(ERROR_MESSAGE_TITLE.to_string())
        .buttons(MessageDialogButtons::OkCustom("OK".to_string()))
        .show(|result| match result {
            true => (),
            false => (),
        });
}

pub fn show_sending_user_error_and_continue(app: tauri::AppHandle) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při odesílání E-mailu";
    static ERROR_MESSAGE_TEXT: &str = "Nebylo možné odeslat E-mail.\n\nAutor aplikace byl informován.\n\nInformujte prosím vedoucího.";

    app.dialog()
        .message(ERROR_MESSAGE_TEXT.to_string())
        .kind(MessageDialogKind::Info)
        .title(ERROR_MESSAGE_TITLE.to_string())
        .buttons(MessageDialogButtons::OkCustom("OK".to_string()))
        .show(|result| match result {
            true => (),
            false => (),
        });
}

pub fn show_unexpected_user_error_and_quit(app: tauri::AppHandle) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při běhu aplikace";
    static ERROR_MESSAGE_TEXT: &str = "Při běhu aplikace došlo k neočekávané chybě.\n\nAutorovi aplikace byl odeslán E-mail.\n\nInformujte prosím vedoucího.";

    app.dialog()
        .message(ERROR_MESSAGE_TEXT.to_string())
        .kind(MessageDialogKind::Info)
        .title(ERROR_MESSAGE_TITLE.to_string())
        .buttons(MessageDialogButtons::OkCustom("OK".to_string()))
        .show(|result| match result {
            true => end_app(app),
            false => end_app(app),
        });
}

pub fn end_app(app: tauri::AppHandle) {
    app.exit(0);
}

pub fn send_error_mail(text: String) -> Result<(), MailSenderError> {
    //must be error proof, so config will be hard wired
    use crate::backend::hard_coded_credentials::*;

    static CREDENTIALS: LazyLock<Credentials> =
        LazyLock::new(|| Credentials::new(SENDER_MAIL.to_string(), SENDER_PASSWORD.to_string()));

    let mut message_builder = Message::builder();

    //sender
    message_builder = message_builder.from(Mailbox::new(
        Some(SENDER_NAME.to_string()),
        SENDER_MAIL
            .parse()
            .map_err(|_| MailSenderError::ErrorSendingFeedbackMail)?,
    ));

    //recepient
    message_builder = message_builder.to(Mailbox::new(
        Some(RECEPIENT_NAME.to_string()),
        RECEPIENT_MAIL
            .parse()
            .map_err(|_| MailSenderError::ErrorSendingFeedbackMail)?,
    ));

    //subject
    message_builder = message_builder.subject(TITLE);

    //body
    let message = message_builder.body(text);

    // open a remote connection to gmail
    let mailer = SmtpTransport::relay(SMTP_TRANSPORT)
        .map_err(|_| MailSenderError::NoRemoteConnection)?
        .credentials(CREDENTIALS.clone())
        .build();

    //send the email
    mailer
        .send(&message.map_err(|_| MailSenderError::InvalidMessage)?)
        .map_err(|_| MailSenderError::ErrorSendingFeedbackMail)?;

    Ok(())
}
