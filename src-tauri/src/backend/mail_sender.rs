use anyhow::Result;

use lettre::message::Mailbox;
use lettre::message::{header::ContentType, Attachment, Body, MultiPart};
use lettre::{Address, Message, SmtpTransport, Transport};

use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use tauri_plugin_dialog::FilePath;

use thiserror::Error;

use crate::backend::config::Config;
use crate::backend::error_handling::error_parsing_mail_address;
use crate::backend::mail_list_utils;
use crate::backend::mail_list_utils::Person;

//---------------------------

#[derive(Error, Debug)]
pub enum MailSenderError {
    #[error("invalid file path")]
    InvalidFilePath,

    #[error("no recipients")]
    NoRecipients,

    #[error("no file")]
    NoFile,

    #[error("couldn't send email: {0}")]
    CouldntSendEmail(#[from] lettre::error::Error),

    #[error("Error opening SMTP: {0}")]
    ErrorOpeningSMTP(#[from] lettre::transport::smtp::Error),

    #[error("Couldn't send built email")]
    InvalidMessage,

    #[error("Couldn't open a remote connection to gmail")]
    NoRemoteConnection,

    #[error("Couldn't send feedback mail")]
    ErrorSendingFeedbackMail,

    #[error("Invalid attachment content")]
    AttachmentContentError,

    #[error("Couldn't parse sender mail")]
    InvalidSenderMail,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Recipient {
    pub name: String,
    pub mail: Address,
}

#[derive(Default, Debug)]
pub struct MailSender {
    people: Vec<Recipient>,
    files: Option<Vec<PathBuf>>,
}

impl MailSender {
    pub fn add_person(&mut self, person: Person, app: tauri::AppHandle) -> &mut Self {
        let person_parsed = Recipient {
            name: person.name,
            mail: person
                .mail
                .parse()
                .unwrap_or_else(|_| error_parsing_mail_address(app, person.mail)),
        };

        self.people.push(person_parsed);

        self
    }

    pub fn remove_person(&mut self, person: Person, app: tauri::AppHandle) -> &mut Self {
        let person_parsed = Recipient {
            name: person.name,
            mail: person
                .mail
                .parse()
                .unwrap_or_else(|_| error_parsing_mail_address(app, person.mail)),
        };

        self.people.retain(|x| *x != person_parsed);

        self
    }

    pub fn add_file(&mut self, vec_path: Vec<FilePath>) -> Result<(), MailSenderError> {
        let mut file_paths: Vec<PathBuf> = vec![];

        for file in vec_path {
            let path = file
                .into_path()
                .map_err(|_| MailSenderError::InvalidFilePath)?;

            if !path.is_file() {
                return Err(MailSenderError::InvalidFilePath);
            }

            file_paths.push(path);
        }

        self.files = Some(file_paths);

        Ok(())
    }

    pub fn send(
        &mut self,
        other_mail_list: Vec<mail_list_utils::Person>,
        config: Config,
        app: tauri::AppHandle,
    ) -> Result<(), MailSenderError> {
        let mut mail = MailSender {
            people: self.people.clone(),
            files: self.files.clone(),
        };

        other_mail_list.iter().for_each(|person| {
            mail.add_person(person.clone(), app.clone());
        });

        if mail.people.is_empty() {
            return Err(MailSenderError::NoRecipients.into());
        }
        if mail.files.is_none() {
            return Err(MailSenderError::NoFile.into());
        }

        let mut message_builder = Message::builder();

        //sender
        message_builder = message_builder.from(Mailbox::new(
            Some(config.sender_name().to_string()),
            config
                .sender_mail()
                .parse()
                .map_err(|_| MailSenderError::InvalidSenderMail)?,
        ));

        //recipient
        message_builder = mail
            .people
            .iter()
            .fold(message_builder, |message_builder, recipient| {
                message_builder.to(Mailbox::new(
                    Some(recipient.name.clone()),
                    recipient.mail.clone(),
                ))
            });

        //subject
        message_builder = message_builder.subject(config.title());

        //attachments
        let mut attachment_multipart = MultiPart::mixed().build();

        if let Some(mail_files) = mail.files {
            for file_path in mail_files {
                let file = fs::read(&file_path).map_err(|_| MailSenderError::InvalidFilePath)?;

                if let Some(mime_type) = mime_guess::from_path(&file_path).first() {
                    let file_name = file_path
                        .file_name()
                        .unwrap_or(OsStr::new("soubor"))
                        .to_str()
                        .unwrap_or("soubor")
                        .to_string();

                    attachment_multipart = attachment_multipart.clone().singlepart(
                        Attachment::new(file_name).body(
                            Body::new(file),
                            ContentType::parse(mime_type.essence_str())
                                .map_err(|_| MailSenderError::AttachmentContentError)?,
                        ),
                    );
                } else {
                    return Err(MailSenderError::InvalidFilePath);
                }
            }
        } else {
            return Err(MailSenderError::InvalidFilePath);
        }

        let message = message_builder.multipart(attachment_multipart);

        //get credentials
        let creds = config.credentials();

        // open a remote connection to gmail
        let mailer = SmtpTransport::relay(config.smtp_transport())
            .map_err(|_| MailSenderError::NoRemoteConnection)?
            .credentials(creds)
            .build();

        //send the email
        mailer
            .send(&message.map_err(|e| MailSenderError::CouldntSendEmail(e))?)
            .map_err(|e| MailSenderError::ErrorOpeningSMTP(e))?;

        Ok(())
    }

    pub fn file_is_valid(&self) -> bool {
        self.files.is_some()
    }

    pub fn person_list_is_valid(&self) -> bool {
        !self.people.is_empty()
    }

    pub fn send_feedback(text: String, config: Config) -> Result<()> {
        let mut message_builder = Message::builder();

        //sender
        message_builder = message_builder.from(Mailbox::new(
            Some(config.sender_name().to_string()),
            config.sender_mail().parse()?,
        ));

        //recepient
        message_builder = message_builder.to(Mailbox::new(
            Some(config.feedback_recepient().to_string()),
            config.feedback_mail().parse()?,
        ));

        //subject
        message_builder = message_builder.subject(config.feedback_subject());

        //body
        let message = message_builder.body(text);

        //get credentials
        let creds = config.credentials();

        // open a remote connection to gmail
        let mailer = SmtpTransport::relay(config.smtp_transport())
            .map_err(|_| MailSenderError::NoRemoteConnection)?
            .credentials(creds)
            .build();

        //send the email
        mailer
            .send(&message.map_err(|e| MailSenderError::CouldntSendEmail(e))?)
            .map_err(|e| MailSenderError::ErrorOpeningSMTP(e))?;

        Ok(())
    }

    pub fn clear(&mut self) {
        self.files = None;
        self.people.clear();
    }
}
