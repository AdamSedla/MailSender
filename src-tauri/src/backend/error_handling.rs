use lettre::{Address, Message};
use lettre::transport::smtp::SmtpTransport;
use lettre::transport::Transport;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use std::sync::LazyLock;

use crate::backend::mail_sender::MailSenderError;

//---------------------------
pub fn error_pick_file(app: tauri::AppHandle){
    let error_message: String = format!("Došlo k chybě při výběru souboru.");

    let _ = send_error_mail(error_message);

    show_file_pick_user_error_and_continue(app);
}

pub fn error_sending_mail(app: tauri::AppHandle, error: MailSenderError) {
    let error_message: String = format!("Došlo k chybě při odesílání mailu. \n\n {error}");

    //if there's connection Error, just show notification to user
    if let MailSenderError::ErrorOpeningSMTP(error ) = error {
        if error.to_string().contains("Connection error") {
            show_connection_user_error_and_continue(app);
            return;
        }
    }

    let _ = send_error_mail(error_message);

    show_sending_user_error_and_continue(app);
}

pub fn error_load_person(app: tauri::AppHandle, original_id: usize) -> String{
    let error_message: String = format!("Neúspěšný pokus o načtení osoby z databáze. ID této osoby: {original_id}");

    let _ = send_error_mail(error_message);

    show_unexpected_user_error_and_quit(app);

    "".to_string()
}

pub fn error_id_parse(app: tauri::AppHandle, original_id: String) -> usize {
    let error_message: String = format!("Neúspěšný pokus o ID_parse. originální String: {original_id}");

    let _ = send_error_mail(error_message);

    show_unexpected_user_error_and_quit(app);

    0
}

pub fn error_parsing_mail_address(app: tauri::AppHandle, original_mail: String) -> Address {
    let error_message: String = format!("Neúspěšný pokus o parse E-mailové adresy. originální String: {original_mail}");
    
    show_unexpected_user_error_and_quit(app);

    let _ = send_error_mail(error_message);

    //error@error is valid Addres, so else block is unreachable
    Address::new("error".to_string(), "error".to_string()).unwrap_or_else(|_| unreachable!())
}

pub fn show_file_pick_user_error_and_continue(app: tauri::AppHandle){
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

pub fn show_connection_user_error_and_continue(app: tauri::AppHandle){
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při odesílání E-mailu";
    static ERROR_MESSAGE_TEXT: &str = "Aplikace pravděpodobně nemá přístup k internetu.\n\nInformujte prosím vedoucího.";

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

pub fn show_sending_user_error_and_continue(app: tauri::AppHandle){
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

pub fn show_unexpected_user_error_and_quit(app: tauri::AppHandle){
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

pub fn end_app(app: tauri::AppHandle){
    app.exit(0);
}

pub fn send_error_mail(text: String) -> Result<(), MailSenderError> {
    //must be error proof, so config will be hard wired
    use crate::backend::hard_coded_credentials::credentials::*;

    static CREDENTIALS: LazyLock<Credentials> = LazyLock::new(|| Credentials::new(SENDER_MAIL.to_string(), SENDER_PASSWORD.to_string()));

    let mut message_builder = Message::builder();

    //sender
    message_builder = message_builder.from(Mailbox::new(
        Some(SENDER_NAME.to_string()),
        SENDER_MAIL.parse().map_err(|_| MailSenderError::ErrorSendingFeedbackMail)?,
    ));

    //recepient
    message_builder = message_builder.to(Mailbox::new(
        Some(RECEPIENT_NAME.to_string()),
        RECEPIENT_MAIL.parse().map_err(|_| MailSenderError::ErrorSendingFeedbackMail)?,
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
    mailer.send(&message.map_err(|_| MailSenderError::InvalidMessage)?)
        .map_err(|_| MailSenderError::ErrorSendingFeedbackMail)?;

    Ok(())
}
