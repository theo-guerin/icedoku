use iced::{
    Element, Length, Rectangle, Size, Vector,
    advanced::{
        self, Clipboard, Shell,
        layout::{Layout, Limits, Node},
        mouse, overlay, renderer,
        widget::{self, Operation, Tree, Widget},
    },
    event::Event,
};

pub fn aspect_ratio<'a, Message, Theme, Renderer>(
    ratio: f32,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> AspectRatio<'a, Message, Theme, Renderer> {
    AspectRatio {
        content: content.into(),
        ratio,
    }
}

#[allow(missing_debug_implementations)]
pub struct AspectRatio<'a, Message, Theme, Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    ratio: f32,
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for AspectRatio<'_, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn tag(&self) -> widget::tree::Tag {
        self.content.as_widget().tag()
    }

    fn state(&self) -> widget::tree::State {
        self.content.as_widget().state()
    }

    fn children(&self) -> Vec<Tree> {
        self.content.as_widget().children()
    }

    fn diff(&self, tree: &mut Tree) {
        self.content.as_widget().diff(tree);
    }

    fn size(&self) -> Size<Length> {
        Size::new(Length::Fill, Length::Fill)
    }

    fn layout(&mut self, tree: &mut Tree, renderer: &Renderer, limits: &Limits) -> Node {
        let max = limits.max();
        let max_ratio = max.width / max.height;

        let size = if max_ratio > self.ratio {
            Size::new(max.height * self.ratio, max.height)
        } else {
            Size::new(max.width, max.width / self.ratio)
        };

        let child_limits = Limits::new(Size::ZERO, size);
        let child = self
            .content
            .as_widget_mut()
            .layout(tree, renderer, &child_limits);

        Node::with_children(size, vec![child])
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        #[allow(clippy::expect_used)]
        let child_layout = layout
            .children()
            .next()
            .expect("aspect-ratio widget always has one child layout");
        operation.traverse(&mut |operation| {
            self.content
                .as_widget_mut()
                .operate(tree, child_layout, renderer, operation);
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        #[allow(clippy::expect_used)]
        let child_layout = layout
            .children()
            .next()
            .expect("aspect-ratio widget always has one child layout");
        self.content.as_widget_mut().update(
            tree,
            event,
            child_layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &widget::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        #[allow(clippy::expect_used)]
        let child_layout = layout
            .children()
            .next()
            .expect("aspect-ratio widget always has one child layout");
        self.content
            .as_widget()
            .mouse_interaction(tree, child_layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        #[allow(clippy::expect_used)]
        let child_layout = layout
            .children()
            .next()
            .expect("aspect-ratio widget always has one child layout");
        self.content
            .as_widget()
            .draw(tree, renderer, theme, style, child_layout, cursor, viewport);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut widget::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        #[allow(clippy::expect_used)]
        let child_layout = layout
            .children()
            .next()
            .expect("aspect-ratio widget always has one child layout");
        self.content
            .as_widget_mut()
            .overlay(tree, child_layout, renderer, viewport, translation)
    }
}

impl<'a, Message, Theme, Renderer> From<AspectRatio<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(widget: AspectRatio<'a, Message, Theme, Renderer>) -> Self {
        Self::new(widget)
    }
}
