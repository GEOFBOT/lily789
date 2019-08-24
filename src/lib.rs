use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

pub mod wks;

#[derive(Debug, PartialEq)]
struct SheetCell<'a> {
    row: u64,
    col: u64,
    value: SheetValue<'a>,
}

#[derive(Debug, PartialEq)]
pub enum SheetValue<'a> {
    Number(f64),
    Label(&'a str),
}

pub struct Sheet<'a> {
    name: &'a str,
    row_major: HashMap<u64, HashMap<u64, Rc<SheetValue<'a>>>>,
    col_major: HashMap<u64, HashMap<u64, Rc<SheetValue<'a>>>>,
}

impl<'a> fmt::Debug for Sheet<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Sheet named {}:\n", self.name)?;

        let max_row = self.row_major.keys().max();
        let max_col = self.col_major.keys().max();
        match (max_row, max_col) {
            (None, _) | (_, None) => write!(f, "[empty]"),
            (Some(max_row), Some(max_col)) => {
                for _ in 0..*max_col + 1 {
                    write!(f, "+---------------")?;
                }
                write!(f, "+\n")?;

                for r in 0..*max_row + 1 {
                    for c in 0..*max_col + 1 {
                        match self.get(r, c) {
                            Some(v) => {
                                write!(f, "|{:?}\t", *v)?;
                            }
                            None => {
                                write!(f, "|\t\t")?;
                            }
                        }
                    }

                    write!(f, "|\n")?;
                    for _ in 0..*max_col + 1 {
                        write!(f, "+---------------")?;
                    }
                    write!(f, "+\n")?;
                }
                Ok(())
            }
        }
    }
}

impl<'a> Sheet<'a> {
    pub fn name(&self) -> &'a str {
        self.name
    }

    pub fn new(name: &'a str) -> Sheet<'a> {
        Sheet {
            name,
            row_major: HashMap::new(),
            col_major: HashMap::new(),
        }
    }

    pub fn add(&mut self, row: u64, col: u64, val: SheetValue<'a>) {
        let rc = Rc::new(val);

        let r = self.row_major.entry(row).or_insert(HashMap::new());
        r.insert(col, Rc::clone(&rc));
        let c = self.col_major.entry(col).or_insert(HashMap::new());
        c.insert(row, Rc::clone(&rc));
    }

    pub fn del(&mut self, row: u64, col: u64) {
        match self.row_major.get_mut(&row) {
            Some(r) => {
                r.remove(&col);
            }
            _ => {}
        }

        match self.col_major.get_mut(&col) {
            Some(c) => {
                c.remove(&row);
            }
            _ => {}
        }
    }

    pub fn get(&self, row: u64, col: u64) -> Option<Rc<SheetValue<'a>>> {
        self.row_major
            .get(&row)
            .and_then(|r| r.get(&col))
            .map(|rc| Rc::clone(rc))
    }

    pub fn cells(&self) -> Vec<(u64, u64, Rc<SheetValue<'a>>)> {
        self.row_major
            .iter()
            .map(|(row, c)| c.iter().map(move |(col, v)| (*row, *col, Rc::clone(v))))
            .flatten()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        let mut s = Sheet::new("hello");
        s.add(1, 4, SheetValue::Number(2.));
        s.add(5, 5, SheetValue::Number(1.));
        s.add(2, 1, SheetValue::Label("test"));
        s.add(4, 3, SheetValue::Number(1.8));
        println!("{:?}", s);
    }
}
