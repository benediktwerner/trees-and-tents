use std::{collections::HashSet, convert::{TryFrom, TryInto}, fmt::Display, str::FromStr};

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
    row_tents: Vec<u8>,
    col_tents: Vec<u8>,
}

impl State {
    fn get_cell(&self, x: usize, y: usize) -> Cell {
        self.cells[x + y * self.dim]
    }

    fn set_cell(&mut self, x: usize, y: usize, value: Cell) {
        self.cells[x + y * self.dim] = value;
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
    }

    fn backtrack(&mut self) {
        let mut todo: HashSet<usize> = self
            .cells
            .iter()
            .enumerate()
            .filter(|(_, c)| c == Cell::Empty)
            .map(|(i,_)|i)
            .collect();

        let mut cords_stack = Vec::new();
        let mut backtrack_stack: Vec<(usize, bool)> = Vec::new();

        while let Some(cord) = todo.iter().next() {
            backtrack_stack.push((cords_stack.len(), true));
            cords_stack.push(cord);

            self.cells[cord] = Cell::Tent;

            let x = cord % self.dim;
            let y = cord / self.dim;

            todo!();
            if self.row_tents[x] == 0 || self.row_tents {
            }
        }
    }

    fn solve(mut self) -> Self {
        self.fill_green();
        self.backtrack();
        self
    }
}

impl Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " ")?;
        for c in &self.row_tents {
            write!(f, "{}", c)?;
        }
        for (i, col) in self.col_tents.iter().zip(self.cells.chunks(self.dim)) {
            write!(f, "\n")?;
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
        ensure!(
            first.as_bytes()[0] == b' ',
            "Top left corner is not a space"
        );
        let row_tents = first
            .as_bytes()
            .iter()
            .skip(1)
            .copied()
            .map(parse_ascii_digit)
            .collect::<anyhow::Result<Vec<u8>>>()?;
        let dim = row_tents.len();
        let mut col_tents = Vec::new();
        let mut cells = Vec::new();
        for (i, line) in lines.map(str::as_bytes).enumerate() {
            ensure!(line.len() > 0, "Line {} is empty", i);
            ensure!(
                line.len() == dim + 1,
                "width != height in row {} ({} != {})",
                i,
                line.len() - 1,
                dim
            );
            col_tents.push(parse_ascii_digit(line[0])?);
            for &c in &line[1..] {
                cells.push(c.try_into()?);
            }
        }
        Ok(Self {
            dim,
            cells,
            row_tents,
            col_tents,
        })
    }
}

fn parse_ascii_digit(c: u8) -> anyhow::Result<u8> {
    ensure!(b'0' <= c && c <= b'9', "Invalid ASCII digit");
    Ok(c - b'0')
}

fn main() {
    let s = " 3030122\n\
             3 T    T\n\
             0  T    \n\
             1  T  T \n\
             3     T \n\
             1T    T \n\
             0  T  T \n\
             3 T     ";
    let s: State = s.parse().unwrap();
    println!("{}", s);
    println!("{}", s.solve());
}
