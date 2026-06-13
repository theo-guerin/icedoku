use iced::{Element, Theme, widget::text};

fn main() -> iced::Result {
    iced::application(IceDoku::new, IceDoku::update, IceDoku::view)
        .title(IceDoku::TITLE)
        .theme(IceDoku::THEME)
        .run()
}

#[derive(Debug)]
struct IceDoku {}

#[derive(Debug)]
enum Message {}

impl IceDoku {
    const TITLE: &'static str = "IceDoku";
    const THEME: Theme = Theme::Dark;

    fn new() -> Self {
        Self {}
    }

    fn update(&mut self, _message: Message) {}

    fn view(&self) -> Element<'_, Message> {
        text!("Hello, IceDoku!").into()
    }
}
