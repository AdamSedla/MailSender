use directories::ProjectDirs;
use std::{fs, path::PathBuf};

use lettre::transport::smtp::authentication::Credentials;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::backend::error_handling::*;

//---------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    sender_name: String,
    sender_mail: String,
    sender_password: String,
    title: String,
    smtp_transport: String,
    feedback_mail: String,
    feedback_recepient: String,
    feedback_subject: String,
    settings_password: String,
    app_folder_path: String,
}

impl Config {
    pub fn save_config(&self, app: AppHandle) {
        let ron_string = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .unwrap_or_else(|_| error_parsing_config_to_string(app.clone()));

        let app_folder_path = create_app_folder_path(app.clone());
        let config_file_path = app_folder_path.join("config.ron");

        std::fs::write(config_file_path, ron_string).unwrap_or_else(|_| error_saving_config(app));
    }
    pub fn load_config(app: AppHandle) -> Config {
        let app_folder_path = create_app_folder_path(app.clone());
        let config_file_path = app_folder_path.join("config.ron");

        let ron_string: String = std::fs::read_to_string(config_file_path)
            .unwrap_or_else(|_| error_loading_config(app.clone()));

        let mut loaded_config: Config = ron::de::from_str(&ron_string)
            .unwrap_or_else(|_| error_decoding_config_from_string(app.clone(), &ron_string));

        //this is only for display purpose
        loaded_config.save_app_folder_path(app_folder_path.to_str().unwrap_or("").to_string());

        loaded_config
    }
    pub fn sender_name(&self) -> &str {
        &self.sender_name
    }
    pub fn save_sender_name(&mut self, text: String) {
        self.sender_name = text;
    }
    pub fn sender_mail(&self) -> &str {
        &self.sender_mail
    }
    pub fn save_sender_mail(&mut self, text: String) {
        self.sender_mail = text;
    }
    pub fn sender_password(&self) -> &str {
        &self.sender_password
    }
    pub fn save_sender_password(&mut self, text: String) {
        self.sender_password = text;
    }
    pub fn credentials(&self) -> Credentials {
        Credentials::new(self.sender_mail.clone(), self.sender_password.clone())
    }
    pub fn title(&self) -> &str {
        &self.title
    }
    pub fn save_title(&mut self, text: String) {
        self.title = text;
    }
    pub fn smtp_transport(&self) -> &str {
        &self.smtp_transport
    }
    pub fn save_smtp_transport(&mut self, text: String) {
        self.smtp_transport = text;
    }
    pub fn feedback_mail(&self) -> &str {
        &self.feedback_mail
    }
    pub fn save_feedback_mail(&mut self, text: String) {
        self.feedback_mail = text;
    }
    pub fn feedback_recepient(&self) -> &str {
        &self.feedback_recepient
    }
    pub fn save_feedback_recepient(&mut self, text: String) {
        self.feedback_recepient = text;
    }
    pub fn feedback_subject(&self) -> &str {
        &self.feedback_subject
    }
    pub fn save_feedback_subject(&mut self, text: String) {
        self.feedback_subject = text;
    }
    pub fn settings_password(&self) -> &str {
        &self.settings_password
    }
    pub fn save_settings_password(&mut self, text: String) {
        self.settings_password = text;
    }
    pub fn settings_password_check(&self, password: &str) -> bool {
        self.settings_password == password
    }
    pub fn app_folder_path(&self) -> &str {
        &self.app_folder_path
    }
    pub fn save_app_folder_path(&mut self, text: String) {
        self.app_folder_path = text;
    }
}

pub fn create_empty_config(app: AppHandle) -> PathBuf {
    let app_folder_path = create_app_folder_path(app.clone());
    let config_file_path = app_folder_path.join("config.ron");
    //only for display purpose
    let app_folder_path_string = app_folder_path.to_str().unwrap_or("");

    let empty_config: String = format!(
        "(
    sender_name: \"\",
    sender_mail: \"\",
    sender_password: \"\",
    title: \"\",
    smtp_transport: \"\",
    feedback_mail: \"\",
    feedback_recepient: \"\",
    feedback_subject: \"\",
    settings_password: \"\",
    app_folder_path: \"{app_folder_path_string}\"
    )"
    );

    std::fs::write(&config_file_path, empty_config.clone())
        .unwrap_or_else(|_| error_of_fail_back_system(app));

    config_file_path
}

pub fn empty_config() -> Config {
    Config {
        sender_name: "".to_string(),
        sender_mail: "".to_string(),
        sender_password: "".to_string(),
        title: "".to_string(),
        smtp_transport: "".to_string(),
        feedback_mail: "".to_string(),
        feedback_recepient: "".to_string(),
        feedback_subject: "".to_string(),
        settings_password: "".to_string(),
        app_folder_path: "".to_string(),
    }
}

pub fn create_app_folder_path(app: AppHandle) -> PathBuf {
    let app_directories = ProjectDirs::from("", "", "MailSender")
        .unwrap_or_else(|| error_creating_app_folder(app.clone()));

    let app_directory_path = app_directories.config_dir();

    fs::create_dir_all(app_directory_path).unwrap_or_else(|_| error_of_fail_back_system(app));

    app_directory_path.to_path_buf()
}
