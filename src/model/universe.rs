use anyhow::anyhow;

const DEFAULT_WIDTH: usize = 40;
const DEFAULT_HEIGHT: usize = 13;

fn fix_coords(width: usize, height: usize, coords: (i64, i64)) -> (usize, usize) {
    let mut coords = coords;

    if coords.0 < 0 {
        coords.0 = width as i64 - 1;
    }

    if coords.0 >= width as i64 {
        coords.0 = 0;
    }

    if coords.1 < 0 {
        coords.1 = height as i64 - 1;
    }

    if coords.1 >= height as i64 {
        coords.1 = 0;
    }

    (coords.0 as usize, coords.1 as usize)
}

#[derive(Clone, Copy)]
struct Neighbours {
    neighour_top: (usize, usize),
    neighour_bottom: (usize, usize),
    neighour_left: (usize, usize),
    neighour_right: (usize, usize),
    neighour_top_left: (usize, usize),
    neighour_top_right: (usize, usize),
    neighour_bottom_left: (usize, usize),
    neighour_bottom_right: (usize, usize),
}

impl Neighbours {
    pub fn to_vec(self) -> Vec<(usize, usize)> {
        [
            self.neighour_top,
            self.neighour_bottom,
            self.neighour_left,
            self.neighour_right,
            self.neighour_top_left,
            self.neighour_top_right,
            self.neighour_bottom_left,
            self.neighour_bottom_right,
        ]
        .to_vec()
    }
}

#[derive(Clone, Copy)]
pub struct Cell {
    x: usize,
    y: usize,
    alive: bool,
    neighbours: Neighbours,
    ignore: i64,
    heat: f32,
}

impl Cell {
    pub fn new(width: usize, height: usize, x: usize, y: usize, alive: bool) -> Self {
        let x1 = x as i64;
        let y1 = y as i64;

        let neighbours = Neighbours {
            neighour_top: fix_coords(width, height, (x1, y1 - 1)),
            neighour_bottom: fix_coords(width, height, (x1, y1 + 1)),
            neighour_left: fix_coords(width, height, (x1 - 1, y1)),
            neighour_right: fix_coords(width, height, (x1 + 1, y1)),
            neighour_top_left: fix_coords(width, height, (x1 - 1, y1 - 1)),
            neighour_top_right: fix_coords(width, height, (x1 + 1, y1 - 1)),
            neighour_bottom_left: fix_coords(width, height, (x1 - 1, y1 + 1)),
            neighour_bottom_right: fix_coords(width, height, (x1 + 1, y1 + 1)),
        };

        Self {
            x,
            y,
            alive,
            neighbours,
            ignore: 0,
            heat: 1.0,
        }
    }

    pub fn coords(&self) -> (usize, usize) {
        (self.x, self.y)
    }

    fn increase_heat(&mut self) {
        if self.heat < 1000.0 {
            self.heat += 2.0;
        }
    }

    fn decrease_heat(&mut self) {
        if self.heat > 1.0 {
            self.heat -= 0.5;
        }
    }

    pub fn heat(&self) -> f32 {
        self.heat
    }

    pub fn alive(&self) -> bool {
        self.alive
    }

    fn set_alive(&mut self, alive: bool, ignore: i64) {
        if !alive && self.alive && self.ignore > 0 {
            self.ignore -= 1;
            return;
        }

        self.ignore = ignore;

        self.alive = alive;
    }

    fn reset(&mut self) {
        self.alive = false;
        self.ignore = 0;
        self.heat = 1.0;
    }

    fn neighbours(self) -> Neighbours {
        self.neighbours
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(DEFAULT_WIDTH, DEFAULT_HEIGHT, 0, 0, false)
    }
}

#[derive(Clone)]
pub struct Universe {
    rows: Vec<Vec<Cell>>,
    paused: bool,
}

impl Universe {
    pub fn new(rows: Vec<Vec<Cell>>) -> anyhow::Result<Self> {
        let height = rows.len();

        if height == 0 {
            return Err(anyhow!("Not enough rows; expected > 0, got {}", height));
        }

        let width: usize = rows.first().unwrap().len();

        for (i, row) in rows.iter().enumerate() {
            let this_width = row.len();

            if this_width == 0 {
                return Err(anyhow!(
                    "Not enough cells in row {}; expected > 0, got {}",
                    i,
                    this_width
                ));
            }

            if this_width != width {
                return Err(anyhow!(
                    "Incorrent amount of cells in row {}; expected > {}, got {}",
                    i,
                    width,
                    this_width
                ));
            }
        }

        Ok(Self { rows, paused: true })
    }

    pub fn from_dimensions(width: usize, height: usize) -> anyhow::Result<Self> {
        let mut cells = vec![];

        for i in 0..height {
            let mut row = Vec::new();
            for j in 0..width {
                let cell = Cell::new(width, height, j, i, false);
                row.push(cell);
            }
            cells.push(row);
        }

        Self::new(cells)
    }

    pub fn summarize(&self) -> String {
        let mut output = String::new();

        for row in self.rows.iter() {
            for cell in row.iter() {
                let mut state = ".";

                if cell.alive() {
                    state = "*";
                }

                output.push_str(state);
            }
            output.push('\n');
        }

        output
    }

    pub fn rows(&self) -> Vec<Vec<Cell>> {
        self.rows.clone()
    }

    pub fn reset(&mut self) {
        for row in self.rows.iter_mut() {
            for cell in row.iter_mut() {
                cell.reset();
            }
        }
    }

    pub fn set_alive(&mut self, coords: (usize, usize), alive: bool, ignore: i64) {
        let x = coords.0;
        let y = coords.1;
        let cell = &mut self.rows[y][x];
        cell.set_alive(alive, ignore);
    }

    pub fn alive(&mut self, coords: (usize, usize)) -> bool {
        let x = coords.0;
        let y = coords.1;
        let cell = &mut self.rows[y][x];
        cell.alive()
    }

    pub fn increase_heat(&mut self, coords: (usize, usize)) {
        let x = coords.0;
        let y = coords.1;
        let cell = &mut self.rows[y][x];
        cell.increase_heat();
    }

    pub fn decrease_heat(&mut self, coords: (usize, usize)) {
        let x = coords.0;
        let y = coords.1;
        let cell = &mut self.rows[y][x];
        cell.decrease_heat();
    }

    pub fn tick(&mut self) {
        let rows = self.rows.clone();

        for row in rows.iter() {
            for cell in row.iter() {
                let this_cell = *cell;

                let mut alive_neighbours = 0;
                for coords in this_cell.neighbours().to_vec().iter() {
                    let neighbour_cell = rows[coords.1][coords.0];
                    if neighbour_cell.alive() {
                        alive_neighbours += 1;
                    }
                }

                let was_alive = self.alive(this_cell.coords());

                if !self.paused {
                    if this_cell.alive() && alive_neighbours < 2 {
                        self.set_alive(this_cell.coords(), false, 0);
                    }

                    if this_cell.alive() && alive_neighbours > 3 {
                        self.set_alive(this_cell.coords(), false, 0);
                    }

                    if !this_cell.alive() && alive_neighbours == 3 {
                        self.set_alive(this_cell.coords(), true, 0);
                    }
                }

                let is_alive = self.alive(this_cell.coords());

                if !was_alive && is_alive {
                    self.increase_heat(this_cell.coords());
                    continue;
                }

                self.decrease_heat(this_cell.coords());
            }
        }
    }

    pub fn pause(&mut self) {
        self.paused = !self.paused;
    }
}

impl Default for Universe {
    fn default() -> Self {
        let possible_universe = Self::from_dimensions(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        if possible_universe.is_err() {
            panic!("failed to create universe")
        }

        possible_universe.unwrap()
    }
}
