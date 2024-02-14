use rusqlite::{Connection, Result};

use std::fs::File;
use std::io::{self, BufRead, Read};
use std::path::Path;

const NMAX: usize = 1005;

pub struct Database {
    conn: Connection,
}

const PATH = "maps";

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("example.db")?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS distances (
                 source_row INTEGER NOT NULL,
                 source_col INTEGER NOT NULL,
                 dest_row INTEGER NOT NULL,
                 dest_col INTEGER NOT NULL,
                 distance REAL NOT NULL,
                 map INTEGER NOT NULL,
                 PRIMARY KEY (source_row, source_col, dest_row, dest_col)
             )",
            [],
        )?;
        
        check_and_populate_data()?;

        Ok(Self { conn })
    }

    pub fn insert_distance(&self, source_row: i32, source_col: i32, dest_row: i32, dest_col: i32, distance: f64, map: i32) -> Result<()> {
        self.conn.execute("INSERT INTO distances (source_row, source_col, dest_row, dest_col, distance, map) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                          &[&source_row, &source_col, &dest_row, &dest_col, &distance, &map])?;
        Ok(())
    }

    pub fn get_distance(&self, source_row: i32, source_col: i32, dest_row: i32, dest_col: i32) -> Result<Option<f64>> {
        let mut stmt = self.conn.prepare("SELECT distance FROM distances WHERE source_row = ?1 AND source_col = ?2 AND dest_row = ?3 AND dest_col = ?4")?;
        let mut rows = stmt.query([&source_row, &source_col, &dest_row, &dest_col])?;

        if let Some(row) = rows.next()? {
            let distance: f64 = row.get(0)?;
            Ok(Some(distance))
        } else {
            Ok(None)
        }
    }

    pub fn check_and_populate_data(&self) -> Result<()> {
        let mut stmt = self.conn.prepare("SELECT COUNT(*) FROM distances")?;
        let count: i32 = stmt.query_row([], |row| row.get(0))?;

        if count == 0 {
            self.populate_sample_data()?;
        } 

        Ok(())
    }

    fn populate_sample_data(&self) -> Result<()> {
        for element in FILENAMES.iter() {
            read_data(element);
        }

        // for &(source_row, source_col, dest_row, dest_col, distance, map) in &sample_data {
        //     self.insert_distance(source_row, source_col, dest_row, dest_col, distance, map)?;
        // }

        Ok(())
    }

    fn algorithm(&self) -> Result<()>{

    }


    fn read_data(&self, file_path: &str) -> Result<()> {
        let mut nod: Vec<Vec<Vec<(i32, i32)>>> = vec![vec![vec![]; NMAX]; NMAX];
        let mut height: Option<usize> = None;
        let mut width: Option<usize> = None;
        let mut file = File::open(file_path)?;
        let mut grid: Vec<Vec<char>> = Vec::new();
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let lines: Vec<&str> = contents.lines().collect();

        if let Some(height_str) = lines.get(1) {
            if let Some(width_str) = lines.get(2) {
                if let Ok(h) = height_str.trim().split_whitespace().last().unwrap().parse() {
                    height = Some(h);
                }
                if let Ok(w) = width_str.trim().split_whitespace().last().unwrap().parse() {
                    width = Some(w);
                }
            } else {
                eprintln!("Error: Width line not found.");
            }
        } else {
            eprintln!("Error: Height line not found.");
        }

        let n = height.unwrap_or_else(|| {
            eprintln!("Error: Height value not found.");
            0 
        });

        let m = width.unwrap_or_else(|| {
            eprintln!("Error: Width value not found.");
            0 
        });

        for i in 4..lines.len() {
            let char_line: Vec<char> = lines[i].chars().collect();
            grid.push(char_line);
        }

        println!("The value of n is {}", n);
        println!("The value of m is {}", m);

        println!("The value of the grid is {}", grid);

        Ok(())
    }
}
