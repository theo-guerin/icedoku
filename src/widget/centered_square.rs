use iced::{
    Element, Length, Size,
    widget::{container, responsive},
};

pub fn centered_square<'a, Message: 'a, F>(content: F) -> Element<'a, Message>
where
    F: Fn() -> Element<'a, Message> + 'a,
{
    container(
        responsive(move |size: Size| {
            let side = size.width.min(size.height);
            container(content()).width(side).height(side).into()
        })
        .width(Length::Shrink)
        .height(Length::Shrink),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center(Length::Fill)
    .into()
}
