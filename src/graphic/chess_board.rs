use iced_graphics::{
    Backend, Defaults, Font, HorizontalAlignment, Primitive, Renderer, VerticalAlignment,
};
use iced_native::{
    event::{Event, Status},
    layout,
    widget::svg::Handle,
    Background, Clipboard, Color, Element, Hasher, Layout, Length, Point, Rectangle, Size, Vector,
    Widget,
    {
        mouse,
        mouse::{Button as MouseButton, Event as MouseEvent},
    },
};
use pleco::core::{sq::SQ, Piece, Player};
use pleco::Board;

use std::collections::HashMap;
use std::fs;
use std::rc::Rc;

struct DragAndDropState {
    active: bool,
    start_cell: Option<[u8; 2]>,
    end_cell: Option<[u8; 2]>,
}

pub struct ChessBoard {
    board: Board,
    cells_size: f32,
    piece_assets: Rc<HashMap<String, Handle>>,
    reversed: bool,
    dnd_state: DragAndDropState,
}

impl ChessBoard {
    pub fn new(cells_size: f32, reversed: bool) -> Self {
        let piece_assets = ChessBoard::load_assets();
        Self {
            cells_size,
            piece_assets,
            reversed,
            board: Board::start_pos(),
            dnd_state: DragAndDropState {
                active: false,
                start_cell: None,
                end_cell: None,
            },
        }
    }

    fn load_assets() -> Rc<HashMap<String, Handle>> {
        let assets_dir = format!(
            "{}/src/graphic/resources/merida",
            env!("CARGO_MANIFEST_DIR")
        );
        let files = fs::read_dir(assets_dir.clone())
            .unwrap_or_else(|e| panic!("Couldn't read directory {}: {}", assets_dir, e))
            .map(|f| f.unwrap())
            .filter(|f| f.metadata().unwrap().is_file());
        let mut res: HashMap<String, Handle> = HashMap::new();
        for file in files {
            let svg = Handle::from_path(file.path());
            res.insert(
                file.path()
                    .file_stem()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
                svg,
            );
        }
        Rc::new(res)
    }

    fn asset_name_for_piece(piece: &Piece) -> String {
        match *piece {
            Piece::WhitePawn => String::from("wP"),
            Piece::WhiteKnight => String::from("wN"),
            Piece::WhiteBishop => String::from("wB"),
            Piece::WhiteRook => String::from("wR"),
            Piece::WhiteQueen => String::from("wQ"),
            Piece::WhiteKing => String::from("wK"),
            Piece::BlackPawn => String::from("bP"),
            Piece::BlackKnight => String::from("bN"),
            Piece::BlackBishop => String::from("bB"),
            Piece::BlackRook => String::from("bR"),
            Piece::BlackQueen => String::from("bQ"),
            Piece::BlackKing => String::from("bK"),
            _ => String::default(),
        }
    }

    fn get_background_primitive(&self, layout: &Layout<'_>) -> Primitive {
        Primitive::Quad {
            bounds: layout.bounds(),
            background: Background::Color(Color::from_rgb8(214, 59, 96)),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }

    fn get_cells_primitives(&self, layout: &Layout<'_>) -> Vec<Primitive> {
        let mut res: Vec<Primitive> = Vec::new();

        let mut start_coordinates: Option<[u8; 2]> = None;
        let mut end_coordinates: Option<[u8; 2]> = None;
        if self.dnd_state.active {
            if let Some(start_cell) = self.dnd_state.start_cell {
                start_coordinates = Some(start_cell);
            }
            if let Some(end_cell) = self.dnd_state.end_cell {
                end_coordinates = Some(end_cell);
            }
        }

        for row in 0..8 {
            let rank = if self.reversed { row } else { 7 - row };
            for col in 0..8 {
                let file = if self.reversed { 7 - col } else { col };
                let is_white_cell = (row + col) % 2 == 0;
                let std_background = Background::Color(if is_white_cell {
                    Color::from_rgb8(255, 206, 158)
                } else {
                    Color::from_rgb8(209, 139, 71)
                });
                let mut background = std_background;

                if let Some([end_file , end_rank ]) = end_coordinates {
                    if file == end_file || rank == end_rank {
                        background = Background::Color(Color::from_rgb8(178, 46, 230));
                    }
                }
                if Some([file as u8, rank as u8]) == start_coordinates {
                    background = Background::Color(Color::from_rgb8(214, 59, 96));
                }
                if Some([file as u8, rank as u8]) == end_coordinates {
                    background = Background::Color(Color::from_rgb8(112, 209, 35));
                }

                let x = self.cells_size * ((col as f32) + 0.5);
                let y = self.cells_size * ((row as f32) + 0.5);
                let position = layout.bounds().position() + Vector::new(x, y);
                let size = Size::new(self.cells_size, self.cells_size);
                let bounds = Rectangle::new(position, size);

                res.push(Primitive::Quad {
                    bounds,
                    background,
                    border_radius: 0.0,
                    border_width: 0.0,
                    border_color: Color::TRANSPARENT,
                })
            }
        }

        res
    }

    fn get_cells_coordinates_primitives(&self, layout: &Layout<'_>) -> Vec<Primitive> {
        let mut res: Vec<Primitive> = Vec::new();

        let upper_a_ordinal = 65u8;
        let digit_1_ordinal = 49u8;
        let size = self.cells_size * 0.45;
        let font = Font::Default;
        let horizontal_alignment = HorizontalAlignment::Center;
        let vertical_alignment = VerticalAlignment::Center;
        let color = Color::from_rgb8(255, 199, 0);

        for col in 0..8 {
            let file = if self.reversed { 7 - col } else { col };
            let content = format!("{}", (upper_a_ordinal + file) as char);
            let x = self.cells_size * (1.0 + (col as f32));
            let y1 = self.cells_size * 0.25f32;
            let y2 = self.cells_size * 8.75f32;
            let position_1 = layout.bounds().position() + Vector::new(x, y1);
            let position_2 = layout.bounds().position() + Vector::new(x, y2);
            let board_size = Size::new(self.cells_size, self.cells_size);
            let bounds_1 = Rectangle::new(position_1, board_size);
            let bounds_2 = Rectangle::new(position_2, board_size);

            res.push(Primitive::Text {
                content: content.clone(),
                color,
                size,
                font,
                horizontal_alignment,
                vertical_alignment,
                bounds: bounds_1,
            });

            res.push(Primitive::Text {
                content,
                color,
                size,
                font,
                horizontal_alignment,
                vertical_alignment,
                bounds: bounds_2,
            });
        }

        for row in 0..8 {
            let rank = if self.reversed { row } else { 7 - row };
            let content = format!("{}", (digit_1_ordinal + rank) as char);
            let y = self.cells_size * (1.0 + (row as f32));
            let x1 = self.cells_size * 0.25f32;
            let x2 = self.cells_size * 8.75f32;
            let position_1 = layout.bounds().position() + Vector::new(x1, y);
            let position_2 = layout.bounds().position() + Vector::new(x2, y);
            let board_size = Size::new(self.cells_size, self.cells_size);
            let bounds_1 = Rectangle::new(position_1, board_size);
            let bounds_2 = Rectangle::new(position_2, board_size);

            res.push(Primitive::Text {
                content: content.clone(),
                color,
                size,
                font,
                horizontal_alignment,
                vertical_alignment,
                bounds: bounds_1,
            });

            res.push(Primitive::Text {
                content,
                color,
                size,
                font,
                horizontal_alignment,
                vertical_alignment,
                bounds: bounds_2,
            });
        }

        res
    }

    fn get_pieces_primitives(&self, layout: &Layout<'_>) -> Vec<Primitive> {
        let mut res: Vec<Primitive> = Vec::new();

        for row in 0..8 {
            let rank = if self.reversed { row } else { 7 - row };
            for col in 0..8 {
                let file = if self.reversed { 7 - col } else { col };

                if self.dnd_state.active {
                    if let Some(start_cell) = self.dnd_state.start_cell {
                        if file == start_cell[0] && rank == start_cell[1] {
                            continue;
                        }
                    }
                }

                let square = SQ(file + 8 * rank);
                let piece = self.board.piece_at_sq(square);
                if piece != Piece::None {
                    let asset_name = ChessBoard::asset_name_for_piece(&piece);
                    let handle = self.piece_assets.clone()[asset_name.as_str()].clone();

                    let x = self.cells_size * ((col as f32) + 0.5);
                    let y = self.cells_size * ((row as f32) + 0.5);
                    let position = layout.bounds().position() + Vector::new(x, y);
                    let size = Size::new(self.cells_size, self.cells_size);
                    let bounds = Rectangle::new(position, size);

                    res.push(Primitive::Svg { bounds, handle })
                }
            }
        }

        res
    }

    fn get_player_turn_primitive(&self, layout: &Layout<'_>) -> Primitive {
        let x = self.cells_size * 8.55;
        let y = self.cells_size * 8.55;
        let border_radius = self.cells_size * 0.4;
        let position = layout.bounds().position() + Vector::new(x, y);
        let size = Size::new(border_radius, border_radius);
        let bounds = Rectangle::new(position, size);

        let white_turn = self.board.turn() == Player::White;
        let background = Background::Color(if white_turn {
            Color::WHITE
        } else {
            Color::BLACK
        });

        Primitive::Quad {
            bounds,
            background,
            border_radius,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
        }
    }
}

impl<Message, B> Widget<Message, Renderer<B>> for ChessBoard
where
    B: Backend,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, _renderer: &Renderer<B>, _limits: &layout::Limits) -> layout::Node {
        let total_size = self.cells_size * 9.0;
        layout::Node::new(Size::new(total_size, total_size))
    }

    fn hash_layout(&self, state: &mut Hasher) {
        use std::hash::Hash;

        self.cells_size.to_bits().hash(state);
    }

    fn draw(
        &self,
        _renderer: &mut Renderer<B>,
        _defaults: &Defaults,
        layout: Layout<'_>,
        _cursor_position: Point,
        _viewport: &Rectangle,
    ) -> (Primitive, mouse::Interaction) {
        let mut res: Vec<Primitive> = Vec::new();

        res.push(self.get_background_primitive(&layout));

        for primitive in self.get_cells_primitives(&layout) {
            res.push(primitive);
        }

        for primitive in self.get_cells_coordinates_primitives(&layout) {
            res.push(primitive);
        }

        for primitive in self.get_pieces_primitives(&layout) {
            res.push(primitive);
        }

        res.push(self.get_player_turn_primitive(&layout));

        (
            Primitive::Group { primitives: res },
            mouse::Interaction::default(),
        )
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        _cursor_position: Point,
        _messages: &mut Vec<Message>,
        _renderer: &Renderer<B>,
        _clipboard: Option<&dyn Clipboard>,
    ) -> Status {
        match event {
            Event::Mouse(MouseEvent::ButtonPressed(MouseButton::Left)) => {
                self.dnd_state.active = true;
                Status::Captured
            }
            Event::Mouse(MouseEvent::ButtonReleased(MouseButton::Left)) => {
                self.dnd_state.active = false;
                self.dnd_state.start_cell = None;
                Status::Captured
            }
            Event::Mouse(MouseEvent::CursorMoved { x, y }) => {
                let self_bounds = layout.bounds();
                let local_x = x - self_bounds.x;
                let local_y = y - self_bounds.y;
                let col = ((local_x - self.cells_size * 0.5) / self.cells_size).floor() as i32;
                let row = ((local_y - self.cells_size * 0.5) / self.cells_size).floor() as i32;
                let file = if self.reversed { 7 - col } else { col };
                let rank = if self.reversed { row } else { 7 - row };

                let out_of_bounds = col < 0 || col > 7 || row < 0 || row > 7;
                if self.dnd_state.active {
                    if self.dnd_state.start_cell.is_none() {
                        self.dnd_state.start_cell = Some([file as u8, rank as u8]);
                    }
                }
                if self.dnd_state.active  {
                    self.dnd_state.end_cell = if out_of_bounds {None} else {Some([file as u8, rank as u8])};
                }
                Status::Captured
            }
            _ => Status::Ignored,
        }
    }
}

impl<'a, Message, B> Into<Element<'a, Message, Renderer<B>>> for ChessBoard
where
    B: Backend,
{
    fn into(self) -> Element<'a, Message, Renderer<B>> {
        Element::new(self)
    }
}
