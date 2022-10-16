use std::{collections::HashMap, hash::Hash};

use chrono::NaiveDate;
use cursive::{
    views::{
        Dialog, DummyView, LinearLayout, OnEventView, ScrollView, SelectView, TextContent,
        TextView, ViewRef,
    },
    Cursive, CursiveRunnable,
};

use cursive::traits::*;
use regex::Regex;
use serde::Serialize;

use crate::{
    mite::Mite, model::project_selection::ProjectSelection, timew_input::TimewInput,
    tw_entry::TWEntry,
};

pub struct UI {
    interface: CursiveRunnable,
}

#[derive(Debug, Clone)]
struct UIData {
    mite: Mite,
    input: TimewInput,
    current_entry: Option<TWEntry>,
    anwers: Vec<Answer>,
}

#[derive(Debug, Clone, Serialize, Eq)]
pub struct Answer {
    service_id: i64,
    project_id: i64,
    note: String,
    minutes: i64,
    date_at: NaiveDate,
}

impl PartialEq for Answer {
    fn eq(&self, other: &Self) -> bool {
        self.service_id == other.service_id
            && self.project_id == other.project_id
            && self.note == other.note
            && self.date_at == other.date_at
    }
}

impl Hash for Answer {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.service_id.hash(state);
        self.project_id.hash(state);
        self.note.hash(state);
        self.date_at.hash(state);
    }
}

impl UIData {
    pub fn get_next_entry(&mut self) -> Option<TWEntry> {
        self.input.data.pop()
    }
}

impl UI {
    pub fn get_answers(&mut self) -> Vec<Answer> {
        let data: UIData = self.interface.take_user_data().unwrap();

        let mut answers = HashMap::new();

        for answer in data.anwers {
            let minutes = answer.minutes;
            let hashentry = answers.entry(answer).or_insert(1);
            *hashentry += minutes;
        }

        let mut out = vec![];

        for (mut entry, duration) in answers {
            entry.minutes = duration;
            out.push(entry);
        }

        out
    }

    pub fn new(input: TimewInput, mite: Mite) -> Self {
        let mut interface = cursive::default();
        interface.set_theme(cursive::theme::load_toml(include_str!("theme.toml")).unwrap());
        interface.set_user_data(UIData {
            mite,
            input,
            anwers: vec![],
            current_entry: None,
        });
        Self { interface }
    }

    fn get_entry_page(data: &UIData, entry: &TWEntry) -> LinearLayout {
        let mut name = "".to_string();
        let rep = Regex::new(r"^p.*?:").unwrap();
        let ret = Regex::new(r"^t.*?:").unwrap();
        let res = Regex::new(r"^s:").unwrap();
        let reth = Regex::new(r"^http").unwrap();

        for tag in &entry.tags {
            if !rep.is_match(&tag)
                && !ret.is_match(&tag)
                && !reth.is_match(tag)
                && !res.is_match(tag)
            {
                name = if name == "" {
                    format!("{}", tag)
                } else {
                    format!("{} {}", name, tag)
                };
            }
        }

        for tag in &entry.tags {
            if reth.is_match(tag) {
                name = format!("{} {}", name, tag);
            }
        }

        let mut item_wrapper = LinearLayout::vertical()
            .child(TextView::new_with_content(TextContent::new(&name)).with_name("current_note"));

        for tag in &entry.tags {
            if rep.is_match(&tag) || ret.is_match(&tag) {
                item_wrapper.add_child(TextView::new_with_content(TextContent::new(tag)))
            }
        }

        let mut select = SelectView::new().autojump();

        for project in &data.mite.project_selection {
            select.add_item(&project.name, project.clone());
        }
        select.set_on_submit(Self::callback_next);

        let select_wrapper = ScrollView::new(select);

        LinearLayout::vertical()
            .child(item_wrapper)
            .child(DummyView)
            .child(select_wrapper)
    }

    fn callback_service_selected(c: &mut Cursive, id: &i64) {
        let mut data: UIData = c.take_user_data().unwrap();
        let entry = data.anwers.last_mut().unwrap();
        entry.service_id = *id;
        let new_layer = Self::get_next_page(&mut data);

        c.set_user_data(data);
        c.pop_layer();
        c.add_layer(new_layer);
    }

    fn callback_next(c: &mut Cursive, selection: &ProjectSelection) {
        let mut data: UIData = c.take_user_data().unwrap();
        let entry = data.current_entry.clone().unwrap();
        let note: ViewRef<TextView> = c.find_name::<TextView>("current_note").unwrap();
        let note_content = note.get_content();

        let mut answer = Answer {
            project_id: selection.id,
            service_id: 0,
            minutes: entry.get_duration(),
            note: note_content.source().clone().to_string(),
            date_at: entry.start.unwrap().date(),
        };

        let res = Regex::new(r"^s:").unwrap();

        for tag in &entry.tags {
            if res.is_match(tag) {
                let id = res.replace(tag, "").to_string();

                answer.service_id = id.trim().parse().unwrap();
            }
        }

        let new_layer = if answer.service_id == 0 {
            Self::get_service_page(&mut data)
        } else {
            Self::get_next_page(&mut data)
        };

        data.anwers.push(answer);

        c.set_user_data(data);
        c.pop_layer();
        c.add_layer(new_layer);
    }

    fn get_service_page(data: &mut UIData) -> OnEventView<Dialog> {
        let page = Self::get_service_selection(data);
        OnEventView::new(Dialog::around(page))
    }

    fn get_next_page(data: &mut UIData) -> OnEventView<Dialog> {
        if let Some(entry) = data.get_next_entry() {
            data.current_entry = Some(entry.clone());
            let page = Self::get_entry_page(&data, &entry);
            OnEventView::new(Dialog::around(page).title("mitewarrior"))
        } else {
            OnEventView::new(
                Dialog::around(TextView::new("You did it, no more data to assign!"))
                    .title("mitewarrior")
                    .button("Quit", |s| s.quit()),
            )
        }
    }

    pub fn boot(&mut self) {
        let mut data: UIData = self.interface.take_user_data().unwrap();
        let page = Self::get_next_page(&mut data);
        self.interface.set_user_data(data);
        self.interface.add_layer(page);
        self.interface.run();
    }

    fn get_service_selection(data: &mut UIData) -> LinearLayout {
        let entry = data.anwers.last().unwrap();
        let services = &data.mite.services;

        let mut select = SelectView::new().autojump();

        for service in services {
            select.add_item(&service.service.name, service.service.id);
        }

        select.set_on_submit(Self::callback_service_selected);

        let title = TextView::new_with_content(TextContent::new(&entry.note));

        let select_wrapper = ScrollView::new(select);

        LinearLayout::vertical()
            .child(title)
            .child(DummyView)
            .child(select_wrapper)
    }
}
