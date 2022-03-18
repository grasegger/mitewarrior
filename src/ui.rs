

use std::{collections::HashMap, hash::Hash};

use chrono::NaiveDate;
use cursive::{
    theme::{PaletteColor, Style},
    views::{
        Dialog, DummyView, EditView, LinearLayout, OnEventView, ScrollView, SelectView,
        TextContent, TextView, ViewRef,
    },
    Cursive, CursiveRunnable,
};

use cursive::traits::*;
use serde::Serialize;

use crate::{mite::Mite, taskwarrior::Task, timew_input::TimewInput, tw_entry::TWEntry};

pub struct UI {
    interface: CursiveRunnable,
}

#[derive(Debug, Clone)]
struct UIData {
    mite: Mite,
    input: TimewInput,
    current_client: i64,
    current_project: i64,
    current_service: i64,
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
        self.service_id == other.service_id && self.project_id == other.project_id && self.note == other.note && self.date_at == other.date_at
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

        for answer in data.anwers  {
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
        let current_client = mite.customers.first().unwrap().customer.id;
        let current_project = mite.projects.first().unwrap().project.id;
        let current_service = mite.services.first().unwrap().service.id;
        interface.set_user_data(UIData {
            mite,
            input,
            current_client,
            current_project,
            current_service,
            anwers: vec![],
            current_entry: None,
        });
        Self { interface }
    }

    fn get_entry_page(
        data: &UIData,
        entry: &TWEntry,
        client: i64,
        project: i64,
        service: i64,
    ) -> LinearLayout {
        let duration = entry.get_duration();
        let task = Task::from_twentry(entry);
        let client = data.mite.find_client_name(client);
        let project = data.mite.find_project_name(project);
        let service = data.mite.find_service_name(service);

        let mut note = task.description;

        if let Some(url) = task.trellourl {
            note = format!("{} {}", note, url);
        }

        if let Some(url) = task.githuburl {
            note = format!("{} {}", note, url);
        }

        let item_wrapper = EditView::new()
            .content(note)
            .on_submit(|c, _| Self::callback_next(c))
            .with_name("current_note");

        let when = TextView::new_with_content(TextContent::new(format!(
            "+ {} min \n@ {}",
            duration,
            entry.start.unwrap()
        )));

        let taskw_project =
            TextView::new_with_content(TextContent::new(format!("# {}", task.project)));
        let tags = if let Some(_tags) = task.tags {
            "+ <no tags>"
        } else {
            "+ <no tags>"
        };

        let taskw_tags = TextView::new_with_content(TextContent::new(tags));

        let mut labels = LinearLayout::vertical();

        for label in ["client", "project", "service"] {
            let content = TextContent::new(format!("{}: ", label));
            let mut view =
                TextView::new_with_content(content).h_align(cursive::align::HAlign::Right);
            view.set_style(Style::from(PaletteColor::Secondary));
            labels.add_child(view);
        }

        let assignments = LinearLayout::vertical()
            .child(TextView::new_with_content(TextContent::new(client)).with_name("current_client"))
            .child(
                TextView::new_with_content(TextContent::new(project)).with_name("current_project"),
            )
            .child(
                TextView::new_with_content(TextContent::new(service)).with_name("current_service"),
            );

        let details = LinearLayout::horizontal().child(labels).child(assignments);

        LinearLayout::vertical()
            .child(item_wrapper)
            .child(when)
            .child(taskw_project)
            .child(taskw_tags)
            .child(DummyView)
            .child(details)
            .child(DummyView)
    }

    fn callback_client_selected(c: &mut Cursive, id: &i64) {
        let mut data: UIData = c.take_user_data().unwrap();
        data.current_client = *id;

        let client_name = data.mite.find_client_name(*id);
        let mut current_client = c.find_name::<TextView>("current_client").unwrap();
        current_client.set_content(client_name);
        c.set_user_data(data);
        c.pop_layer();

        Self::callback_project(c);
    }

    fn callback_client(c: &mut Cursive) {
        let data: UIData = c.take_user_data().unwrap();
        let clients = &data.mite.customers;

        let mut select = SelectView::new().autojump();

        for client in clients {
            select.add_item(&client.customer.name, client.customer.id);
        }

        select.set_on_submit(Self::callback_client_selected);

        c.set_user_data(data);
        c.add_layer(Dialog::around(ScrollView::new(select)).title("Client"));
    }

    fn callback_service_selected(c: &mut Cursive, id: &i64) {
        let mut data: UIData = c.take_user_data().unwrap();
        data.current_service = *id;

        let service_name = data.mite.find_service_name(*id);
        let mut current_service = c.find_name::<TextView>("current_service").unwrap();
        current_service.set_content(service_name);

        c.set_user_data(data);
        c.pop_layer();
    }

    fn callback_service(c: &mut Cursive) {
        let data: UIData = c.take_user_data().unwrap();
        let services = &data.mite.services;

        let mut select = SelectView::new().autojump();

        for service in services {
            select.add_item(&service.service.name, service.service.id);
        }

        select.set_on_submit(Self::callback_service_selected);

        c.set_user_data(data);
        c.add_layer(Dialog::around(ScrollView::new(select)).title("Service"));
    }

    fn callback_project_selected(c: &mut Cursive, id: &i64) {
        let mut data: UIData = c.take_user_data().unwrap();
        data.current_project = *id;

        let project_name = data.mite.find_project_name(*id);
        let mut current_project = c.find_name::<TextView>("current_project").unwrap();
        current_project.set_content(project_name);

        c.set_user_data(data);
        c.pop_layer();
    }

    fn callback_project(c: &mut Cursive) {
        let data: UIData = c.take_user_data().unwrap();
        let projects = &data.mite.projects;

        let mut select = SelectView::new().autojump();

        for project in projects {
            if let Some(id) = project.project.customer_id {
                if id == data.current_client {
                    select.add_item(&project.project.name, project.project.id);
                }
            }
        }

        select.set_on_submit(Self::callback_project_selected);

        c.set_user_data(data);
        c.add_layer(Dialog::around(ScrollView::new(select)).title("Project"));
    }

    fn callback_next(c: &mut Cursive) {
        let mut data: UIData = c.take_user_data().unwrap();
        let entry = data.current_entry.clone().unwrap();
        let note: ViewRef<EditView> = c.find_name::<EditView>("current_note").unwrap();
        let note_content = note.get_content();

        let answer = Answer {
            project_id: data.current_project,
            service_id: data.current_service,
            minutes: entry.get_duration(),
            note: note_content.as_str().to_string(),
            date_at: entry.start.unwrap().date(),
        };
        data.anwers.push(answer);

        let new_layer = Self::get_next_page(&mut data);

        c.set_user_data(data);
        c.pop_layer();
        c.add_layer(new_layer);
    }

    fn get_next_page(data: &mut UIData) -> OnEventView<Dialog> {
        if let Some(entry) = data.get_next_entry() {
            data.current_entry = Some(entry.clone());
            let page = Self::get_entry_page(
                &data,
                &entry,
                data.current_client,
                data.current_project,
                data.current_service,
            );
            OnEventView::new(
                Dialog::around(page)
                    .title("mitewarrior")
                    .button("Cancel", |c| c.quit())
                    .button("Client", Self::callback_client)
                    .button("Project", Self::callback_project)
                    .button("Service", Self::callback_service)
                    .button("Next", Self::callback_next),
            )
            .on_event('n', Self::callback_next)
            .on_event('c', Self::callback_client)
            .on_event('p', Self::callback_project)
            .on_event('s', Self::callback_service)
        } else {
            OnEventView::new(
                Dialog::around(TextView::new("You did it, no more data to assign!"))
                    .title("mitewarrior")
                    .button("Quit", |s| s.quit()),
            )
            .on_event('q', |c| c.quit())
        }
    }

    pub fn boot(&mut self) {
        let mut data: UIData = self.interface.take_user_data().unwrap();
        let page = Self::get_next_page(&mut data);
        self.interface.set_user_data(data);
        self.interface.add_layer(page);
        self.interface.run();
    }
}
