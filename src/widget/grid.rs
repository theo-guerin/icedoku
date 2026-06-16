use iced::{
    Color, Element, Length, Rectangle, Size,
    advanced::{
        layout::{self, Layout},
        renderer,
        widget::{self, Widget},
    },
    mouse,
};

pub struct Grid {}

impl Grid {
    const CELL_LINE_WIDTH: f32 = 1.0;
    const BLOCK_LINE_WIDTH: f32 = 3.0;

    pub fn new() -> Self {
        Self {}
    }
}

pub fn grid() -> Grid {
    Grid::new()
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Grid
where
    Renderer: renderer::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fill,
            height: Length::Fill,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut widget::Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::Node::new(limits.max())
    }

    fn draw(
        &self,
        _tree: &widget::Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let cell_size = bounds.width / 9.0;

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                ..renderer::Quad::default()
            },
            Color::WHITE,
        );

        for line in 1..9 {
            let thickness = if line % 3 == 0 {
                Self::BLOCK_LINE_WIDTH
            } else {
                Self::CELL_LINE_WIDTH
            };

            let x = bounds.x + cell_size * line as f32;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: x - thickness / 2.0,
                        y: bounds.y,
                        width: thickness,
                        height: bounds.height,
                    },
                    snap: true,
                    ..renderer::Quad::default()
                },
                Color::BLACK,
            );

            let y = bounds.y + cell_size * line as f32;
            renderer.fill_quad(
                renderer::Quad {
                    bounds: Rectangle {
                        x: bounds.x,
                        y: y - thickness / 2.0,
                        width: bounds.width,
                        height: thickness,
                    },
                    snap: true,
                    ..renderer::Quad::default()
                },
                Color::BLACK,
            );
        }
    }
}

impl<Message, Theme, Renderer> From<Grid> for Element<'_, Message, Theme, Renderer>
where
    Renderer: renderer::Renderer,
{
    fn from(circle: Grid) -> Self {
        Self::new(circle)
    }
}
