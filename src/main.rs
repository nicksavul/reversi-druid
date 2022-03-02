#![windows_subsystem = "windows"]

use druid::{AppLauncher, BoxConstraints, Color, Env, Event, EventCtx, LayoutCtx, LifeCycle, LifeCycleCtx, PaintCtx, Size, UpdateCtx, Widget, WindowDesc, RenderContext, Point, AppDelegate, WindowId, DelegateCtx, ImageBuf, TimerToken, FontFamily};
use druid::Data;
use druid::widget::{Align, Flex, Label, Button, FlexParams, CrossAxisAlignment, Either};
use crate::Cell::{Free, Black, White};
use druid::kurbo::{Line, Rect};
use std::mem::transmute_copy;
use std::ops::Neg;
use rand::{random, Rng, thread_rng};
use rand::rngs::ThreadRng;
use druid::image::{SubImage, GenericImageView, DynamicImage};
use druid::piet::{InterpolationMode};
use druid::piet::d2d::Bitmap;
use druid::piet::Image;
use druid::image::imageops::crop;
use druid::image::io::Reader as ICanRead;
use std::io::Cursor;
use std::borrow::Borrow;
use std::time::Duration;
use druid::piet::{Text, TextLayoutBuilder, TextLayout};
use druid::theme::WINDOW_BACKGROUND_COLOR;

pub const REVERSI_FIELD_WIDTH: usize = 8;
pub const REVERSI_FIELD_HEIGHT: usize = 8;
pub const REVERSI_FIELD_SIZE: usize = REVERSI_FIELD_WIDTH * REVERSI_FIELD_HEIGHT;
pub type Field = [Cell ; REVERSI_FIELD_SIZE];


pub const WINDOW_WIDTH: f64 = 800_f64;
pub const WINDOW_HEIGHT: f64 = 600_f64;




fn main() {

    let wnd = WindowDesc::<Reversi>::new(root)
        .window_size((WINDOW_WIDTH, WINDOW_HEIGHT))
        .title("REVERSI")
        .resizable(false);

    AppLauncher::with_window(wnd)
        .configure_env(|env, rev| {
            env.set(WINDOW_BACKGROUND_COLOR, Color::rgba8(0,155,119, 255))
        })
        .launch(Reversi::new())
        .expect("failed to launch window");


}

fn root() -> impl Widget<Reversi> {

    Flex::<Reversi>::row()
        .with_child(
            Either::<Reversi>::new(|rev, env| rev.is_game,
                                   Grid::new(),
                                   VictoryScreen::new(),
            ))
        .with_flex_child(
            Align::centered(
            Flex::column()
                .with_child(
                    Button::<Reversi>::new("Restart").on_click(
                        |ctx, rev, env| {
                            *rev = Reversi::new();
                        }
                    )
                )
                .with_child(Button::<Reversi>::new("mode: PvP")
                    .on_click(|ctx, rev, env| {
                        *rev = Reversi::new();
                        rev.mode = GameMode::PvP;

                    }))
                .with_child(Button::<Reversi>::new("mode: PvE (1)")
                    .on_click(|ctx, rev, env| {
                        *rev = Reversi::new();
                        rev.mode = GameMode::PvE(0.2);

                    }))
                .with_child(Button::<Reversi>::new("mode: PvE (2)")
                    .on_click(|ctx, rev, env| {
                        *rev = Reversi::new();
                        rev.mode = GameMode::PvE(0.4);

                    }))
                .with_child(Button::<Reversi>::new("mode: PvE (3)")
                    .on_click(|ctx, rev, env| {
                        *rev = Reversi::new();
                        rev.mode = GameMode::PvE(0.6);

                    }))),
            FlexParams::new(0.25, None)
        )
}

struct Grid {
    hot: Option<(usize, usize)>,
    ver_offset: f64,
    hor_offset: f64,
    cell_size: f64,
    invalid_cell: ImageBuf,
    black_cell: ImageBuf,
    white_cell: ImageBuf,
    swap_cells: [ImageBuf;3],
    timer_code: TimerToken,
    gaf: u32,

}

impl Widget<Reversi> for Grid {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Reversi, env: &Env) {

        match event {
            Event::MouseMove(mouse_event) => {
                if let Point{x, y} = mouse_event.pos {
                    let affected_x: usize = ((x - self.hor_offset) / self.cell_size).floor() as usize;
                    let affected_y: usize = ((y - self.ver_offset) / self.cell_size).floor() as usize;

                    self.hot = Some((affected_x, affected_y));

                    ctx.request_paint();
                }
            },

            Event::MouseUp(mouse_event) => {
                if let Point {x, y} = mouse_event.pos {
                    let affected_x: usize = ((x - self.hor_offset) / self.cell_size).floor() as usize;
                    let affected_y: usize = ((y - self.ver_offset) / self.cell_size).floor() as usize;
                    data.clicked(affected_x, affected_y);
                }

                self.timer_code = ctx.request_timer(
                Duration::from_millis(80)
                );
            },
            Event::Timer(tkn) => {
                if *tkn == self.timer_code {
                    let mut have_animated_cell = false;

                    for _cell in data.field.iter_mut() {
                        if let (Cell::Black(f) | Cell::White(f)) = _cell {
                            *f = (*f + 1) * (*f > 0 && *f < Self::SWP_LEN - 1) as u8 as usize;
                        }
                    }

                    ctx.request_paint();

                    self.timer_code = if self.gaf != 0 {
                        self.gaf -= 1;
                        ctx.request_timer(
                            Duration::from_millis(80)
                        )
                    } else {
                        self.gaf = Self::MAX_GAF;
                        TimerToken::INVALID
                    };
                }
            }
            _ => {},
        }
    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Reversi, env: &Env) {

    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Reversi, data: &Reversi, env: &Env) {
        ctx.request_paint();
    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Reversi, env: &Env) -> Size {
        Size::new(600.,600.)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Reversi, env: &Env) {



        if let Size {width, height} = ctx.size() {

            let mut hor_offset: f64 = width / (REVERSI_FIELD_WIDTH + 2) as f64;
            let mut ver_offset: f64 = height / (REVERSI_FIELD_HEIGHT + 2) as f64;

            self.cell_size = hor_offset.max(ver_offset);

            hor_offset = (width % (self.cell_size * REVERSI_FIELD_WIDTH as f64)) / 2_f64;
            ver_offset = (height % (self.cell_size * REVERSI_FIELD_HEIGHT as f64)) / 2_f64;
            self.ver_offset = ver_offset;
            self.hor_offset = hor_offset;

            /// draw cells
            for (idx, cell) in data.field.iter().enumerate() {

                match *cell {

                    _cell @ (Cell::White(mut f) | Cell::Black(mut f)) => {

                        let (x, y): (f64, f64) = ((idx % REVERSI_FIELD_WIDTH) as f64, (idx / REVERSI_FIELD_WIDTH) as f64);


                        let im = if _cell == Cell::Black(0) {
                            let swp_len = self.swap_cells.len();
                            if f > 0 {
                                &self.swap_cells[swp_len - f]
                            } else {&self.black_cell}
                        } else {
                            if f > 0 {
                                &self.swap_cells[f - 1]
                            } else {&self.white_cell}

                        }.to_image(
                            ctx.render_ctx
                        );





                        ctx.draw_image(
                            &im,
                            Rect::new(
                                        x * self.cell_size + self.hor_offset,
                                        y * self.cell_size + self.ver_offset,
                                        (x+1_f64) * self.cell_size + self.hor_offset,
                                        (y+1_f64) * self.cell_size + self.ver_offset
                                    ),
                            InterpolationMode::Bilinear
                        );

                    },

                    Cell::Free => {},
                }
            }

            /// draw grid
            for line_idx in 0..REVERSI_FIELD_HEIGHT + 1 {
                ctx.stroke(
                    Line::new((self.hor_offset, line_idx as f64 * self.cell_size + self.ver_offset), (width - self.hor_offset, line_idx as f64 * self.cell_size + self.ver_offset)),
                    &Color::SILVER,
                    1.5
                );
            }


            for line_idx in 0..REVERSI_FIELD_WIDTH + 1 {
                ctx.stroke(
                    Line::new((line_idx as f64 * self.cell_size + self.hor_offset, self.ver_offset), (line_idx as f64 * self.cell_size + self.hor_offset, height - self.ver_offset)),
                    &Color::SILVER,
                    1.5
                );
            }

            if let Some((affected_x, affected_y)) = self.hot {


                if affected_x >= 0 && affected_x <= REVERSI_FIELD_WIDTH - 1 && affected_y >= 0 && affected_y <= REVERSI_FIELD_HEIGHT - 1 {

                    let mut im = match data.player_turn {
                        PlayerTurn::Black => {
                            self.black_cell.to_image(
                                ctx.render_ctx
                            )
                        }
                        PlayerTurn::White => {
                            self.white_cell.to_image(
                                ctx.render_ctx
                            )
                        }
                    };

                    if !data.is_valid_cell(
                        affected_x, affected_y
                    ) {
                        im = self.invalid_cell.to_image(
                            ctx.render_ctx
                        );
                    }

                    ctx.draw_image(
                        &im,
                        Rect::new(
                                    affected_x as f64 * self.cell_size + self.hor_offset,
                                    affected_y as f64 * self.cell_size + self.ver_offset,
                                    (affected_x as f64 + 1_f64) * self.cell_size + self.hor_offset,
                                    (affected_y as f64 + 1_f64) * self.cell_size + self.ver_offset,
                                ),
                        InterpolationMode::Bilinear
                    );
                }

                self.hot = None;
            }

            let mut score_b_string = data.black_score.to_string();
            score_b_string.insert_str(0, "Black: ");

            ctx.text().new_text_layout(
                score_b_string
            ).font(FontFamily::MONOSPACE, 20.)
                .text_color(Color::BLACK)
                .build()
                .unwrap()
                .draw(
                    Point::new(20.,10.),
                        ctx.render_ctx
                );

            let mut score_b_string = data.white_score.to_string();
            score_b_string.insert_str(0, "White: ");

            ctx.text().new_text_layout(
                score_b_string
            ).font(FontFamily::MONOSPACE, 20.)
                .text_color(Color::WHITE)
                .build()
                .unwrap()
                .draw(
                    Point::new(20.,30.),
                    ctx.render_ctx
                );

        }

    }
}

impl Grid {
    const MAX_GAF: u32 = 3;

    const WHITE: [u8; 438] = *include_bytes!("../res/white.png");
    const BLACK: [u8; 454] = *include_bytes!("../res/black.png");
    const INVALID: [u8;777] = *include_bytes!("../res/invalid.png");

    const SWP1: [u8; 438] = *include_bytes!("../res/swp1.png");
    const SWP2: [u8; 607] = *include_bytes!("../res/swp2.png");
    const SWP3: [u8; 452] = *include_bytes!("../res/swp3.png");

    const SWP_LEN: usize = 3;

    const ERROR_COLOR: Color = Color::rgba8(255, 0, 0, 255/2);


    pub fn new() -> Self {

        Self {
            hot: None,
            ver_offset: 0.0,
            hor_offset: 0.0,
            cell_size: 0.0,

            invalid_cell: ImageBuf::from_dynamic_image(
                ICanRead::new(Cursor::new(&Self::INVALID)).with_guessed_format().unwrap().decode().unwrap()
            ),

            black_cell: ImageBuf::from_dynamic_image(
                ICanRead::new(Cursor::new(&Self::BLACK)).with_guessed_format().unwrap().decode().unwrap()
            ),

            white_cell: ImageBuf::from_dynamic_image(
                ICanRead::new(Cursor::new(&Self::WHITE)).with_guessed_format().unwrap().decode().unwrap()
            ),

            swap_cells: [
                ImageBuf::from_dynamic_image(
                    ICanRead::new(Cursor::new(&Self::SWP1)).with_guessed_format().unwrap().decode().unwrap()
                ),
                ImageBuf::from_dynamic_image(
                    ICanRead::new(Cursor::new(&Self::SWP2)).with_guessed_format().unwrap().decode().unwrap()
                ),
                ImageBuf::from_dynamic_image(
                    ICanRead::new(Cursor::new(&Self::SWP3)).with_guessed_format().unwrap().decode().unwrap()
                )
            ],
            timer_code: TimerToken::INVALID,
            gaf: Self::MAX_GAF,
        }
    }

}


#[derive(Data, Clone)]
pub struct Reversi {
    pub mode: GameMode,
    pub player_turn: PlayerTurn,
    #[data(ignore)]
    pub field: Field,
    #[data(ignore)]
    pub rng: ThreadRng,
    pub is_game: bool,
    pub victorious: String,
    pub black_score: u32,
    pub white_score: u32,

}

impl Reversi {
    pub fn new() -> Self {

        let mut initial_field: Field = [Cell::Free ; REVERSI_FIELD_SIZE];
        let top_left_center: (usize, usize) = ((REVERSI_FIELD_WIDTH / 2) - 1, (REVERSI_FIELD_HEIGHT / 2) - 1);

        initial_field[top_left_center.1 * REVERSI_FIELD_WIDTH + top_left_center.0] = Cell::White(0);
        initial_field[(top_left_center.1 + 1) * REVERSI_FIELD_WIDTH + top_left_center.0] = Cell::Black(0);
        initial_field[top_left_center.1 * REVERSI_FIELD_WIDTH + top_left_center.0 + 1] = Cell::Black(0);
        initial_field[(top_left_center.1 + 1) * REVERSI_FIELD_WIDTH + top_left_center.0 + 1] = Cell::White(0);


        Self {
            mode: GameMode::PvP,
            field: initial_field,
            player_turn: PlayerTurn::Black,
            rng: thread_rng(),
            is_game: true,
            victorious: "".to_string(),
            black_score: 2,
            white_score: 2,
        }

    }

    pub fn switch_turn(&mut self) {


        if let GameMode::PvE(error_chance) = self.mode {

            if self.player_turn == PlayerTurn::Black {
                self.player_turn = PlayerTurn::White;

                //

                let mut moves: Vec<(usize, usize, usize)> = Vec::new();

                for (idx, cell) in self.field.iter().enumerate() {
                    let (x, y): (usize, usize) = (idx % REVERSI_FIELD_WIDTH, idx / REVERSI_FIELD_WIDTH);

                    let mut cost: usize = 0;

                    if *cell == Cell::Free && self.is_valid_cell_cost(x, y, &mut cost) {
                        moves.push((x, y, cost));
                    }
                }
                if moves.len() > 0 {
                    moves.sort_by_key(|(x, y, cost)| *cost);

                    if self.rng.gen::<f64>() < f64::MAX * error_chance {
                        let random_move = moves[self.rng.gen_range(0..moves.len())];
                        self.clicked(random_move.0, random_move.1);
                    } else {
                        let best_move = moves[0];
                        self.clicked(best_move.0, best_move.1);
                    }

                    self.switch_turn();
                } else {
                    let current_mode = self.mode.clone();
                    *self = Self::new();
                    self.mode = current_mode;
                    self.is_game = false;
                }
                //

                self.player_turn = PlayerTurn::Black;

                let mut has_to_reset: bool = true;
                for (idx, cell) in self.field.iter().enumerate() {
                    let (x, y): (usize, usize) = (idx % REVERSI_FIELD_WIDTH, idx / REVERSI_FIELD_WIDTH);


                    if *cell == Cell::Free && self.is_valid_cell(x, y) {
                        has_to_reset = false;
                    }
                }

                if has_to_reset {
                    let current_mode = self.mode.clone();
                    *self = Self::new();
                    self.mode = current_mode;
                    self.is_game = false;
                }
            }

        } else {

            self.player_turn = if self.player_turn == PlayerTurn::Black {
                PlayerTurn::White
            } else {
                PlayerTurn::Black
            };

        }

        let mut black_score = 0;
        let mut white_score = 0;

        for cell in self.field {
            if let Cell::White(_) = cell {
                white_score += 1;
            }

            if let Cell::Black(_) = cell {
                black_score += 1;
            }
        }

        self.white_score = white_score;
        self.black_score = black_score;

        self.victorious = if (white_score > black_score) { "Black" } else { "White" }.parse().unwrap();

    }

    pub fn is_valid_cell(&self, x: usize, y: usize) -> bool {
        if self.field[y * REVERSI_FIELD_WIDTH + x] != Cell::Free { return false ;}

        let (x, y): (isize, isize) = (x as isize, y as isize);



        for (x_coef, y_coef) in [(1_isize, 0_isize),
            (0_isize, 1_isize),
            (-1_isize, 0_isize),
            (0_isize, -1_isize),
            (1_isize, 1_isize),
            (1_isize, -1_isize),
            (-1_isize, 1_isize),
            (-1_isize, -1_isize)] {

            let mut factor = 1_isize;



            while (0..REVERSI_FIELD_HEIGHT as isize).contains(&(y + y_coef * factor))
                && (0..REVERSI_FIELD_WIDTH as isize).contains(&(x + x_coef * factor))
                && self.player_turn.is_reverse_of(&self.field[(y + y_coef * factor) as usize * REVERSI_FIELD_WIDTH + (x + x_coef * factor) as usize]) {

                factor += 1;

            }



            if factor > 1 && (0..REVERSI_FIELD_HEIGHT as isize).contains(&(y + y_coef * factor))
                && (0..REVERSI_FIELD_WIDTH as isize).contains(&(x + x_coef * factor))
                && self.player_turn == self.field[(y + y_coef * factor) as usize * REVERSI_FIELD_WIDTH + (x + x_coef * factor) as usize] {

                return true;
            }
        }
        return false;
    }


    pub fn is_valid_cell_cost(&self, x: usize, y: usize, cost: &mut usize) -> bool {
        if !self.is_valid_cell(x, y) {
            return false;
        }

        let mut n_inverse: usize = 0;

        for (x_coef, y_coef) in [(1_isize, 0_isize),
            (0_isize, 1_isize),
            (-1_isize, 0_isize),
            (0_isize, -1_isize),
            (1_isize, 1_isize),
            (1_isize, -1_isize),
            (-1_isize, 1_isize),
            (-1_isize, -1_isize)] {

            let mut factor = 1_isize;



            while (0..REVERSI_FIELD_HEIGHT as isize).contains(&(y as isize + y_coef * factor))
                && (0..REVERSI_FIELD_WIDTH as isize).contains(&(x as isize + x_coef * factor))
                && self.player_turn.is_reverse_of(&self.field[(y as isize + y_coef * factor) as usize * REVERSI_FIELD_WIDTH + (x as isize + x_coef * factor) as usize]) {
                factor += 1;

            }



            if factor > 1 && (0..REVERSI_FIELD_HEIGHT as isize).contains(&(y as isize + y_coef * factor))
                && (0..REVERSI_FIELD_WIDTH as isize).contains(&(x as isize + x_coef * factor))
                && self.player_turn == self.field[(y as isize + y_coef * factor) as usize * REVERSI_FIELD_WIDTH + (x as isize + x_coef * factor) as usize] {

                n_inverse += factor as usize;

            }
        }

        *cost = n_inverse;
        true
    }


    pub fn clicked(&mut self, x: usize, y: usize) {

        if self.is_valid_cell(x, y) {
            self.field[y * REVERSI_FIELD_WIDTH + x] = self.player_turn.produce();


            let mut inverse: Vec<usize> = Vec::new();

            for (x_coef, y_coef) in [(1_isize, 0_isize),
                (0_isize, 1_isize),
                (-1_isize, 0_isize),
                (0_isize, -1_isize),
                (1_isize, 1_isize),
                (1_isize, -1_isize),
                (-1_isize, 1_isize),
                (-1_isize, -1_isize)] {

                let mut factor = 1_isize;



                while (0..REVERSI_FIELD_HEIGHT as isize).contains(&(y as isize + y_coef * factor))
                    && (0..REVERSI_FIELD_WIDTH as isize).contains(&(x as isize + x_coef * factor))
                    && self.player_turn.is_reverse_of(&self.field[(y as isize + y_coef * factor) as usize * REVERSI_FIELD_WIDTH + (x as isize + x_coef * factor) as usize]) {

                    inverse.push((y as isize + y_coef * factor) as usize * REVERSI_FIELD_WIDTH + (x as isize + x_coef * factor) as usize);
                    factor += 1;

                }



                if factor > 1 && (0..REVERSI_FIELD_HEIGHT as isize).contains(&(y as isize + y_coef * factor))
                    && (0..REVERSI_FIELD_WIDTH as isize).contains(&(x as isize + x_coef * factor))
                    && self.player_turn == self.field[(y as isize + y_coef * factor) as usize * REVERSI_FIELD_WIDTH + (x as isize + x_coef * factor) as usize] {

                    for idx in &inverse {
                        self.field[*idx].inverse();
                        if let (White(f) | Black(f)) = self.field.get_mut(*idx).unwrap() {
                            *f = 1;
                        }
                    }

                } else {
                    inverse.clear();
                }
            }

            self.switch_turn();
        }



        }


}



#[derive(Clone, Copy, Debug)]
pub enum Cell {
    Black(usize),
    White(usize),
    Free
}

impl PartialEq<Cell> for Cell {
    fn eq(&self, other: &Cell) -> bool {

        if let (Cell::White(_), Cell::White(_)) = (*self, *other) {
            return true;
        }

        if let (Cell::Black(_), Cell::Black(_)) = (*self, *other) {
            return true;
        }

        if let (Cell::Free, Cell::Free) = (*self, *other) {
            return true;
        }

        false
    }
}

impl Cell {
    pub fn clr(&self) -> &Color {
        match self {
            Cell::Black(_) => &Color::BLACK,
            Cell::White(_) => &Color::WHITE,
            Cell::Free => &Color::GRAY,
        }
    }

    pub fn inverse(&mut self) {
        *self = match self {
            Black(f) => White(*f),
            White(f) => Black(*f),
            Free => Free,
        }
    }
}

impl PartialEq<PlayerTurn> for Cell {
    fn eq(&self, other: &PlayerTurn) -> bool {
        (*self == Cell::White(0) && *other == PlayerTurn::White) || (*self == Cell::Black(0) && *other == PlayerTurn::Black)
    }
}




#[repr(u8)]
#[derive(PartialEq, Clone, Data, Copy, Debug)]
pub enum PlayerTurn {
    Black,
    White,
}

impl PartialEq<Cell> for PlayerTurn {
    fn eq(&self, other: &Cell) -> bool {
        (*self == PlayerTurn::White && *other == Cell::White(0)) || (*self == PlayerTurn::Black && *other == Cell::Black(0))
    }
}


impl PlayerTurn {
    const BLACK_NAME: &'static str = "Black";
    const WHITE_NAME: &'static str = "White";

    const SELECT_BLACK: Color = Color::rgba8(0, 0, 0, 255/5);
    const SELECT_WHITE: Color = Color::rgba8(255, 255, 255, 255/5);

    const SELECT_VALID_BLACK: Color = Color::rgba8(0, 0, 0, 255/2);
    const SELECT_VALID_WHITE: Color = Color::rgba8(255,255,255,255/2);

    pub fn name(&self) -> &'static str {
        match self {
            PlayerTurn::Black => Self::BLACK_NAME,
            PlayerTurn::White => Self::WHITE_NAME,
        }

    }

    pub fn clr(&self) -> &Color {
        match self {
            PlayerTurn::Black => &Self::SELECT_BLACK,
            PlayerTurn::White => &Self::SELECT_WHITE,
        }
    }

    pub fn strong_clr(&self) -> &Color {
        match self {
            PlayerTurn::Black => &Self::SELECT_VALID_BLACK,
            PlayerTurn::White => &Self::SELECT_VALID_WHITE,
        }
    }

    pub fn produce(&self) -> Cell {
        match self {
            PlayerTurn::Black => {
                Cell::Black(0)
            }
            PlayerTurn::White => {
                Cell::White(0)
            }
        }
    }

    pub fn is_reverse_of(&self, cell: &Cell) -> bool {
        (*self == PlayerTurn::White && *cell == Cell::Black(0)) ||
            (*self == PlayerTurn::Black && *cell == Cell::White(0))
    }
}

#[derive(Data, Clone, PartialEq)]
pub enum GameMode {
    PvP,
    PvE(f64),
}


struct VictoryScreen {
    crown: ImageBuf,
}

impl Widget<Reversi> for VictoryScreen {

    fn event(&mut self, ctx: &mut EventCtx, event: &Event, data: &mut Reversi, env: &Env) {

    }

    fn lifecycle(&mut self, ctx: &mut LifeCycleCtx, event: &LifeCycle, data: &Reversi, env: &Env) {

    }

    fn update(&mut self, ctx: &mut UpdateCtx, old_data: &Reversi, data: &Reversi, env: &Env) {

    }

    fn layout(&mut self, ctx: &mut LayoutCtx, bc: &BoxConstraints, data: &Reversi, env: &Env) -> Size {
        Size::new(600.,600.)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &Reversi, env: &Env) {
        let bbox = ctx.region().bounding_box();

        ctx.fill(
            bbox,
            if &data.victorious == "Black" { &Color::BLACK } else {&Color::WHITE}
        );

        let mut winner_string = data.victorious.clone();
        winner_string.push_str(" wins!");

        let text = ctx.text().new_text_layout(
            winner_string
        ).text_color(Color::SILVER)
            .font(FontFamily::SYSTEM_UI, 20.)
            .build()
            .unwrap();

        let text_sz = text.size();

        ctx.draw_text(
            &text, Point::new(bbox.width() / 2. - (text_sz.width / 2.), bbox.height() / 2. - (text_sz.height / 2.))
        );

        let bmp = self.crown.to_image(
            ctx.render_ctx
        );


        ctx.draw_image(
            &bmp,
            Rect::new(
                bbox.width() / 2.0 - self.crown.width() as f64 / 2.0,
                bbox.height() / 2.0 + text_sz.height / 2.0 + 60.,
                bbox.width() / 2.0 + self.crown.width() as f64 / 2.0,
                bbox.height() / 2.0 + text_sz.height / 2.0 + 60.0 + self.crown.height() as f64
            ),
            InterpolationMode::Bilinear
        )

    }
}

impl VictoryScreen {

    const CROWN: [u8;8714] = *include_bytes!("../res/crown.png");

    fn new() -> Self {
        Self { crown: ImageBuf::from_dynamic_image(
            ICanRead::new(Cursor::new(&Self::CROWN)).with_guessed_format().unwrap().decode().unwrap()
        ) }
    }
}






