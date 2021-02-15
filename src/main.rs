use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    fmt::Display,
    ops::{Deref, DerefMut},
    str::FromStr,
};

use anyhow::{bail, ensure, Context};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Cell {
    Tree,
    Tent,
    Grass,
    Empty,
}

impl Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Cell::Tree => write!(f, "T"),
            Cell::Tent => write!(f, "X"),
            Cell::Grass => write!(f, "."),
            Cell::Empty => write!(f, " "),
        }
    }
}

impl TryFrom<u8> for Cell {
    type Error = anyhow::Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            b'T' => Ok(Self::Tree),
            b'X' => Ok(Self::Tent),
            b'.' => Ok(Self::Grass),
            b' ' => Ok(Self::Empty),
            _ => bail!("Invalid cell character"),
        }
    }
}

struct State {
    dim: usize,
    cells: Vec<Cell>,
    col_tents: Vec<u8>,
    row_tents: Vec<u8>,
}

impl State {
    fn get_cell(&self, x: usize, y: usize) -> Cell {
        self.cells[x + y * self.dim]
    }

    fn set_cell(&mut self, x: usize, y: usize, value: Cell) {
        self.cells[x + y * self.dim] = value;
    }

    fn index_to_xy(&self, index: usize) -> (usize, usize) {
        (index % self.dim, index / self.dim)
    }

    fn fill_green(&mut self) {
        use Cell::*;

        for y in 0..self.dim {
            for x in 0..self.dim {
                if self.get_cell(x, y) != Empty
                    || x > 0 && self.get_cell(x - 1, y) == Tree
                    || y > 0 && self.get_cell(x, y - 1) == Tree
                    || x + 1 < self.dim && self.get_cell(x + 1, y) == Tree
                    || y + 1 < self.dim && self.get_cell(x, y + 1) == Tree
                {
                    continue;
                }
                self.set_cell(x, y, Grass);
            }
        }

        for x in 0..self.dim {
            if self.col_tents[x] == 0 {
                for y in 0..self.dim {
                    if self.get_cell(x, y) == Empty {
                        self.set_cell(x, y, Grass);
                    }
                }
            }
        }
        for y in 0..self.dim {
            if self.row_tents[y] == 0 {
                for x in 0..self.dim {
                    if self.get_cell(x, y) == Empty {
                        self.set_cell(x, y, Grass);
                    }
                }
            }
        }
    }

    fn solve(mut self) -> Self {
        self.fill_green();
        Backtrack::apply(self)
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " ")?;
        for c in &self.col_tents {
            write!(f, "{}", c)?;
        }
        for (i, col) in self.row_tents.iter().zip(self.cells.chunks(self.dim)) {
            writeln!(f)?;
            write!(f, "{}", i)?;
            for c in col {
                write!(f, "{}", c)?;
            }
        }
        Ok(())
    }
}

impl FromStr for State {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut lines = s.lines();
        let first = lines.next().context("Empty input")?;
        ensure!(first.is_ascii(), "First line is not ASCII");
        ensure!(
            first.as_bytes()[0] == b' ',
            "Top left corner is not a space"
        );
        let col_tents = first
            .as_bytes()
            .iter()
            .skip(1)
            .copied()
            .map(parse_ascii_digit)
            .collect::<anyhow::Result<Vec<u8>>>()?;
        let dim = col_tents.len();
        let mut row_tents = Vec::new();
        let mut cells = Vec::new();
        for (i, line) in lines.map(str::as_bytes).enumerate() {
            ensure!(!line.is_empty(), "Line {} is empty", i);
            ensure!(first.is_ascii(), "Line {} is not ASCII", i);
            ensure!(
                line.len() == dim + 1,
                "width != height in row {} ({} != {})",
                i,
                line.len() - 1,
                dim
            );
            row_tents.push(parse_ascii_digit(line[0])?);
            for &c in &line[1..] {
                cells.push(c.try_into()?);
            }
        }
        Ok(Self {
            dim,
            cells,
            col_tents,
            row_tents,
        })
    }
}

struct Backtrack {
    state: State,
    tree_count: usize,
    tent_count: usize,
    todo: HashSet<usize>,
    cords_stack: Vec<usize>,
    backtrack_stack: Vec<usize>,
}

impl Deref for Backtrack {
    type Target = State;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl DerefMut for Backtrack {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Backtrack {
    fn apply(state: State) -> State {
        let todo: HashSet<usize> = state
            .cells
            .iter()
            .enumerate()
            .filter(|(_, &c)| c == Cell::Empty)
            .map(|(i, _)| i)
            .collect();

        let tree_count = state.cells.iter().filter(|c| **c == Cell::Tree).count();
        let tent_count = state.cells.iter().filter(|c| **c == Cell::Tent).count();

        let mut backtrack = Backtrack {
            state,
            tree_count,
            tent_count,
            todo,
            cords_stack: Vec::new(),
            backtrack_stack: Vec::new(),
        };

        backtrack.run();
        backtrack.state
    }

    fn mark_green(&mut self, x: usize, y: usize) {
        let new_cord = x + y * self.dim;
        if self.todo.remove(&new_cord) {
            self.cords_stack.push(new_cord);
            self.cells[new_cord] = Cell::Grass;
        }
    }

    fn run(&mut self) {
        loop {
            while let Some(&cord) = self.todo.iter().next() {
                self.todo.remove(&cord);
                self.cords_stack.push(cord);
                self.backtrack_stack.push(self.cords_stack.len());

                self.cells[cord] = Cell::Tent;
                self.tent_count += 1;

                let (x, y) = self.index_to_xy(cord);

                self.col_tents[x] -= 1;
                if self.col_tents[x] == 0 {
                    for y in 0..self.dim {
                        self.mark_green(x, y);
                    }
                }

                self.row_tents[y] -= 1;
                if self.row_tents[y] == 0 {
                    for x in 0..self.dim {
                        self.mark_green(x, y);
                    }
                }

                for &dx in &[(-1_isize) as usize, 0, 1] {
                    let nx = x.wrapping_add(dx);
                    if nx == self.dim || nx == usize::MAX {
                        continue;
                    }
                    for &dy in &[(-1_isize) as usize, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let ny = y.wrapping_add(dy);
                        if ny == self.dim || ny == usize::MAX {
                            continue;
                        }
                        self.mark_green(nx, ny);
                    }
                }
            }

            if self.tent_count == self.tree_count {
                return;
            }

            if let Some(i) = self.backtrack_stack.pop() {
                for c in self.cords_stack.drain(i..) {
                    self.state.cells[c] = Cell::Empty;
                    self.todo.insert(c);
                }
                let cord = *self.cords_stack.last().unwrap();
                let (x, y) = self.index_to_xy(cord);
                self.cells[cord] = Cell::Grass;
                self.col_tents[x] += 1;
                self.row_tents[y] += 1;
                self.tent_count -= 1;
            } else {
                panic!("No solution");
            }
        }
    }
}

fn parse_ascii_digit(c: u8) -> anyhow::Result<u8> {
    ensure!(b'0' <= c && c <= b'9', "Invalid ASCII digit");
    Ok(c - b'0')
}

fn main() {
    let s = " 1202031\n\
             1       \n\
             2  T  T \n\
             1T      \n\
             2T   T T\n\
             1 T    T\n\
             1       \n\
             1    T  ";
    let s: State = s.parse().unwrap();
    println!("{}", s);
    println!("{}", s.solve());
}
