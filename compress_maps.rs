use std::fs::File;
use std::io::{self, BufRead, Read};
use std::path::Path;

const NMAX: usize = 1005;

fn main() -> io::Result<()> {
    let mut nod: Vec<Vec<Vec<(i32, i32)>>> = vec![vec![vec![]; NMAX]; NMAX];
    let mut height: Option<usize> = None;
    let mut width: Option<usize> = None;
    let file_path = "maps/small.map";
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

    // for row in grid.iter() {
    //     for &ch in row.iter() {
    //         print!("{}", ch);
    //     }
    //     println!();
    // }

    for i in 0..n{
        for j in 0..m{
            
        }
    }

    Ok(())
}
