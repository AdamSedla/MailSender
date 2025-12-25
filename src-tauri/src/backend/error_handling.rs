use lettre::Message;
use lettre::transport::smtp::SmtpTransport;
use lettre::transport::Transport;
use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::Credentials;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use std::sync::LazyLock;

use crate::backend::mail_sender::MailSenderError;

//---------------------------

pub fn error_message_mutex_lock(app: tauri::AppHandle, mutex_type: &str) {
    static ERROR_MESSAGE_TITLE: &str = "Došlo k chybě při běhu aplikace";
    static ERROR_MESSAGE_TEXT: &str = "Při běhu aplikace došlo k neočekávané chybě.\n\nAutorovi aplikace byl odeslán E-mail.\n\nInformujte prosím vedoucího.";
    let error_message_mail: String = format!("Neúspěšný pokus o uzamčení MUTEX lock: {mutex_type}");

    //error upozornění autorovi aplikace
    let _ = send_error_mail(error_message_mail);
    
    //Error upozornění a ukončení aplikace
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
    static TITLE: &str = "Chyba v MailSender Postřižín";
    static SENDER_NAME: &str = "MAN Diag Postřižín";
    static SENDER_MAIL: &str = "man.diag.postrizin@gmail.com";
    static RECEPIENT_NAME: &str = "Adam Sedláček";
    static RECEPIENT_MAIL: &str = "adam.sedlacek.2003@gmail.com";
    static SMTP_TRANSPORT: &str = "smtp.gmail.com";
    static CREDENTIALS: LazyLock<Credentials> = LazyLock::new(|| Credentials::new("man.diag.postrizin@gmail.com".to_string(), "ptjk ybko urqu liiv".to_string()));

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
