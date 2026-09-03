use iced::{
    Center, Element, Fill,
    widget::{button, column, container, row, space, text},
};

use crate::puzzle::Difficulty;

#[derive(Debug)]
pub struct NewGame {
    difficulty: Difficulty,
}

#[derive(Debug, Clone, Copy)]
pub enum Message {
    DifficultySelected(Difficulty),
    CancelRequested,
    StartRequested,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Close,
    Start(Difficulty),
}

impl NewGame {
    pub fn new(difficulty: Difficulty) -> Self {
        Self { difficulty }
    }

    pub fn update(&mut self, message: Message) -> Option<Action> {
        match message {
            Message::DifficultySelected(difficulty) => {
                self.difficulty = difficulty;
            }
            Message::CancelRequested => {
                return Some(Action::Close);
            }
            Message::StartRequested => {
                return Some(Action::Start(self.difficulty));
            }
        }

        None
    }

    pub fn view(&self) -> Element<'_, Message> {
        let difficulties = column![
            row![
                difficulty_button(Difficulty::Simple, self.difficulty),
                difficulty_button(Difficulty::Easy, self.difficulty),
            ]
            .spacing(10),
            row![
                difficulty_button(Difficulty::Intermediate, self.difficulty),
                difficulty_button(Difficulty::Expert, self.difficulty),
            ]
            .spacing(10),
        ]
        .spacing(10);

        let actions = row![
            button("Back")
                .on_press(Message::CancelRequested)
                .style(button::text)
                .padding(12),
            space::horizontal(),
            button("Start game")
                .on_press(Message::StartRequested)
                .style(button::primary)
                .padding([12, 18]),
        ]
        .align_y(Center);

        let content = column![
            text("New game").size(24),
            text("Difficulty").size(14),
            difficulties,
            actions,
        ]
        .spacing(14);

        container(content)
            .padding(20)
            .width(Fill)
            .style(container::bordered_box)
            .into()
    }
}

fn difficulty_button(difficulty: Difficulty, selected: Difficulty) -> Element<'static, Message> {
    button(text(difficulty.to_string()).size(15))
        .on_press(Message::DifficultySelected(difficulty))
        .style(if difficulty == selected {
            button::primary
        } else {
            button::secondary
        })
        .padding([11, 12])
        .width(Fill)
        .into()
}
