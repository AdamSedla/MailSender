use lettre::Address;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

use crate::backend::error_handling::*;

//---------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Person {
    pub name: String,
    pub mail: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MailList {
    list: Vec<Option<Person>>, //0-23 - mechanic | 24-28 - technique
}

impl MailList {
    pub fn save_list(&mut self, app: AppHandle) -> Result<(), Vec<String>> {
        //will change every empty names into None
        self.list
            .iter_mut()
            .filter(|person| person.as_ref().is_some_and(|person| person.name.is_empty()))
            .for_each(|person| *person = None);

        //will create list of invalid mails
        let wrong_mail_list: Vec<String> = self
            .list
            .iter()
            .filter_map(|person| {
                person.as_ref().and_then(|p| {
                    if p.mail.parse::<Address>().is_err() {
                        Some(p.name.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();

        if !wrong_mail_list.is_empty() {
            return Err(wrong_mail_list);
        }

        let ron_string = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())
            .unwrap_or_else(|_| error_parsing_mail_list_to_string(app.clone()));

        std::fs::write("mail_list.ron", ron_string).unwrap_or_else(|_| error_saving_mail_list(app));

        Ok(())
    }

    pub fn load_list(app: AppHandle) -> MailList {
        let ron_string = std::fs::read_to_string("mail_list.ron")
            .unwrap_or_else(|_| error_loading_mail_list(app.clone()));

        let mut new_mail_list = ron::de::from_str(&ron_string)
            .unwrap_or_else(|_| error_decoding_mail_list_from_string(app.clone(), &ron_string));

        if new_mail_list.list.len() < 29 {
            new_mail_list = error_mail_list_id_overflow(app);
        }

        new_mail_list
    }

    pub fn load_person(&self, id: usize) -> Option<Person> {
        self.list[id].clone()
    }

    pub fn save_person_name(&mut self, id: usize, name: String) {
        let mut person = match self.load_person(id) {
            Some(person) => person,
            None => Person {
                name: "".to_string(),
                mail: "".to_string(),
            },
        };

        person.name = name;

        self.list[id] = Some(person);
    }

    pub fn save_person_mail(&mut self, id: usize, mail: String) {
        let mut person = match self.load_person(id) {
            Some(person) => person,
            None => Person {
                name: "".to_string(),
                mail: "".to_string(),
            },
        };

        person.mail = mail;

        self.list[id] = Some(person);
    }
}

pub fn create_empty_mail_list(app: AppHandle) -> String {
    static EMPTY_MAIL_LIST: &str = "
(
    list: [
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ],
)
";

    std::fs::write("mail_list.ron", EMPTY_MAIL_LIST)
        .unwrap_or_else(|_| error_of_fail_back_system(app));

    EMPTY_MAIL_LIST.to_string()
}

pub fn empty_mail_list() -> MailList {
    MailList {
        list: vec![None; 30],
    }
}
