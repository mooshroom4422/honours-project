use std::fs::File;
use std::io;
use gif::{Frame, Encoder, Repeat};
use std::borrow::Cow;

use crate::map::*;

// 0 -> empty (white)
// 1 -> wall (black)
// 2 -> agent (blue)
// 3 -> target (red)
// 4 -> agent and target (purple) (anomaly, shouldnt happened)
pub fn generate_frame(map: &Map, agents: &Vec<Agent>, targets: &Vec<Target>) -> Vec<u8> {
    let mut frame = vec![1; map.height*map.width];
    for y in (0..map.height).rev() {
        for x in 0..map.width {
            let mut c: u8 = 1;
            let ag = agents.into_iter()
                .any(|f| f.position == Point{x, y});
            let tr = targets.into_iter()
                .any(|f| f.position == Point{x, y});
            if ag && tr {
                c = 4;
            }
            else if ag {
                c = 2;
            }
            else if tr {
                c = 3;
            }
            else if map.valid_point(&Point{x, y}){
                c = 0;
            }
            frame[map.conv(x, map.height-y-1)] = c;

        }
    }

    frame
}

pub fn generate_gif(frames: &Vec<Vec<u8>>, map: &Map, file_path: &str) -> Result<(), io::Error> {
    let color_map = &[
            0xFF, 0xFF, 0xFF, // 0 -> white
            0x00, 0x00, 0x00, // 1 -> black
            0x00, 0x00, 0xFF, // 2 -> blue
            0xFF, 0x00, 0x00, // 3 -> red
            0xFF, 0x00, 0xFF, // 4 -> purple
        ];

    let mut file = match File::create(file_path) {
        Ok(f) => f,
        Err(err) => return Err(err),
    };

    let mut encoder = Encoder::new(&mut file, map.width as u16, map.height as u16, color_map).unwrap();
    encoder.set_repeat(Repeat::Infinite).unwrap();

    for frame in frames.iter() {
        let mut fr = Frame::default();
        fr.width = map.width as u16;
        fr.height = map.height as u16;
        fr.buffer = Cow::Borrowed(&*frame);
        encoder.write_frame(&fr).unwrap();
    }

    Ok(())
}
