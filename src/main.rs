use image::{open, Pixel};
use std::fs::File;
use std::io::{self,Write, BufWriter};
use argh::FromArgs;
use ndarray::Array2;
use std::time::Instant;

fn h_map(x: usize, y: usize, width: i32, height: i32, radius: i32, src: &Array2::<u8>) -> f32 {
            let mut black_count: f32 = 0.0;
            for dy in -radius..=radius {
                for dx in -radius..=radius {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let distance: f32 = f32::powf(dx as f32,2.0) + f32::powf(dy as f32, 2.0);
                    if  distance > f32::powf(radius as f32,2.0) {
                        continue;
                    }
                    let nx: i32 = x as i32 + dx;
                    let ny: i32 = y as i32 + dy;
                    if nx >= 0 && ny >= 0 && nx < width && ny < height {
                        if src[[nx as usize,ny as usize]] == 0 {
                            black_count += 1.0/distance;
                            // println!("{}",img.get_pixel(nx as u32, ny as u32).to_luma().0[0]);
                        } else {
                            black_count -= 1.0/distance;
                        }
                    } else { // anything outside of the picture considered to be white.
                        if distance < f32::powf(radius as f32,2.0) {
                            black_count -= 1.0/distance;
                        }
                    }
                }
            }
            let height: f32;
            // let area: f32 = f32::powf(radius as f32,2.0) * 3.14; // how many pixels are expected to be in the circle
            let area: f32 = 2.0 * 3.14 * (radius as f32 - 1.0);
            if black_count < 0.0 { height = 0.0 } // bottom plateau
            else if black_count > area { height = radius as f32 }  // top plateau
            else { height = black_count / area }  // slopes
            return height*8.0;
}

// Compute normal vector for a triangle
fn normal(ax: [f32; 3], bx: [f32; 3], cx: [f32; 3]) -> [f32; 3] {
    let ux = [bx[0] - ax[0], bx[1] - ax[1], bx[2] - ax[2]];
    let vy = [cx[0] - ax[0], cx[1] - ax[1], cx[2] - ax[2]];
    [
        ux[1] * vy[2] - ux[2] * vy[1],
        ux[2] * vy[0] - ux[0] * vy[2],
        ux[0] * vy[1] - ux[1] * vy[0],
    ]
}

fn write_triangle(w: &mut BufWriter<File>, a: [f32; 3], b: [f32; 3], c: [f32; 3]) {
    let n = normal(a, b, c);
    writeln!(w, "  facet normal {} {} {}", n[0], n[1], n[2]).unwrap();
    writeln!(w, "    outer loop").unwrap();
    writeln!(w, "      vertex {} {} {}", a[0], a[1], a[2]).unwrap();
    writeln!(w, "      vertex {} {} {}", b[0], b[1], b[2]).unwrap();
    writeln!(w, "      vertex {} {} {}", c[0], c[1], c[2]).unwrap();
    writeln!(w, "    endloop").unwrap();
    writeln!(w, "  endfacet").unwrap();
}

#[derive(FromArgs)]
/// Yet another image to STL mesh converter, focused on creating STLs for CNC engraving.
/// Creates an STL based on a two-colour engraving-style image: black for the image and white for the background.
/// If a colour image is provided, it will be converted to black and white, treating anything darker than 50% as black and the rest as white.
/// Images with an alpha channel are treated as if backed by 100% white.
/// Pixels outside the picture border are considered white.
struct CmdArgs {
     /// input file name
     #[argh(positional)]
     input_file: String,

     /// output file name. Default is "output.stl"
     #[argh(option, short = 'o', default = "String::from(\"output.stl\")")]
     output_file: String,

     /// look around radius. Use smaller values for sharper edges and bigger for smoother. Default is 8 pixels. Doubling radius quadripls compute time.
     #[argh(option, short = 'r', default = "String::from(\"8\")" ) ]
     capture_radius: String,

     /// if specified code will generate 0.001 unit thick base underlying all other features. Default: Do Not Generate
     #[argh(switch, short = 'p')]
     generate_plane: bool
}


fn main() {
    let start_time = Instant::now();
    let cmd_args: CmdArgs = argh::from_env();
    let radius: i32 = cmd_args.capture_radius.parse().unwrap();
    // let generate_plane: bool = cmd_args.generate_plane.parse::<bool>().unwrap();

    print!("Reading and parcing input file..."); io::stdout().flush().unwrap();
    let img = open(cmd_args.input_file).unwrap().into_luma_alpha8();
    let (x_size, y_size) = img.dimensions();

    let mut input_values = Array2::<u8>::zeros((x_size as usize, y_size as usize));
    for ((x, y), val) in input_values.indexed_iter_mut() {
        let pixel = img.get_pixel(x as u32, y as u32).to_luma_alpha();
        if (pixel[0] as f32 / 255.0) * (pixel[1] as f32 / 255.0) > 0.5  {
          *val = 1
      } else {
          *val = 0
      }
    }
    println!("Done");
    print!("Creating heights map (and burning CPU in the process)..."); io::stdout().flush().unwrap();
    let mut heights_map = Array2::<f32>::zeros((x_size as usize, y_size as usize));

    let adj = if cmd_args.generate_plane { 0.001 } else { 0.0 };
    for ((x, y), val) in heights_map.indexed_iter_mut() {
        *val= h_map(x, y, x_size as i32, y_size as i32, radius, &input_values) + adj;
    }

    println!("Done");
    print!("Creating STL text file (and trashing disk in the process)..."); io::stdout().flush().unwrap();
    let file = File::create(cmd_args.output_file).unwrap();
    let mut writer = BufWriter::new(file);
    writeln!(writer, "solid surface").unwrap();

    for y in 0..y_size - 1 {
        for x in 0..x_size - 1 {
            // 1 2
            // 3 4
            let h1 = heights_map[[x as usize,y  as usize]];
            let h2 = heights_map[[(x + 1) as usize,y as usize]];
            let h3 = heights_map[[x as usize,(y + 1) as usize]];
            let h4 = heights_map[[(x+1) as usize,(y + 1) as usize]];
            let p1 = [x as f32, y as f32, h1 ];
            let p2 = [(x + 1) as f32, y as f32, h2 ];
            let p3 = [x as f32, (y + 1) as f32, h3];
            let p4 = [(x + 1) as f32, (y + 1) as f32, h4];
            let b1 = [x as f32, y as f32, 0.0 ];
            let b2 = [(x + 1) as f32, y as f32, 0.0 ];
            let b3 = [x as f32, (y + 1) as f32, 0.0 ];
            let b4 = [(x + 1) as f32, (y + 1) as f32, 0.0];

            if h1 != 0.0 || h2 != 0.0 || h3 != 0.0 || h4 != 0.0 {
               if h1 + h4 < h2 + h3 {  // use highest common edge
                   // common edge from 3 to 2
                   write_triangle(&mut writer, p1, p2, p3); // top
                   write_triangle(&mut writer, p2, p4, p3); // top
               }  else {
                   // common from 1 to 4
                   write_triangle(&mut writer, p1, p4, p3); // top
                   write_triangle(&mut writer, p1, p2, p4); // top
               }
               if ! cmd_args.generate_plane { // for plane version bottom will be just 2 triangles
                   write_triangle(&mut writer, b1, b2, b3); // bottom
                   write_triangle(&mut writer, b2, b4, b3); // bottom
               }
            }
        }
    }
    // doing sides
    for y in 0..y_size - 1 {  // going along Y axis
        for x in [0,x_size -1 ] { // not stricly correct STL as outbound vercto of one side will be pointing inside.
        if heights_map[[x as usize,y as usize]] != 0.0 {
           write_triangle(&mut writer,[x as f32,y as f32, heights_map[[x as usize,y as usize]]],
                                     [x as f32, (y + 1 )as f32, 0.0],
                                     [x as f32, y as f32, 0.0]);
       }
       if heights_map[[x as usize,(y + 1) as usize]] != 0.0 {
          write_triangle(&mut writer,[x as f32,y as f32, heights_map[[x as usize,y as usize]]],
                                    [x as f32,( y + 1 ) as f32, heights_map[[x as usize,(y + 1) as usize]]],
                                    [x as f32,( y + 1 ) as f32, 0.0]);
      }
     }
   }
   for x in 0..x_size - 1 { // going along X axis
       for y in [0,y_size - 1] { // not stricly correct STL as  normal vector of one side will be pointing inside.
       if heights_map[[x as usize,y as usize]] != 0.0 {
          write_triangle(&mut writer,[x as f32,y as f32, heights_map[[x as usize,y as usize]]],
                                    [(x + 1) as f32, y as f32, 0.0],
                                    [x as f32, y as f32, 0.0]);
      }
      if heights_map[[(x + 1 ) as usize,y as usize]] != 0.0 {
         write_triangle(&mut writer,[x as f32,y as f32, heights_map[[x as usize,y as usize]]],
                                   [(x + 1) as f32, y  as f32, heights_map[[x as usize,y as usize]]],
                                   [( x + 1 ) as f32, y  as f32, 0.0]);
     }
    }
  }
  if cmd_args.generate_plane { // Adding bottom in case if Plane is generated.
      write_triangle(&mut writer, [ 0.0, 0.0, 0.0 ],[ 0.0, ( y_size - 1) as f32, 0.0 ],[ ( x_size - 1) as f32, ( y_size - 1) as f32, 0.0 ]);
      write_triangle(&mut writer, [ 0.0, 0.0, 0.0 ],[ ( x_size - 1) as f32, ( y_size - 1) as f32, 0.0 ],[  ( x_size - 1) as f32 , 0.0, 0.0 ]);
  }
    writeln!(writer, "endsolid surface").unwrap();
  println!("Done.");
  println!(" Execution time {:?} seconds",start_time.elapsed());
}
