use image::{self, GenericImageView, GenericImage, DynamicImage};

// to read file we need to use std::fs
use std::fs::File;
use std::io::{BufReader, BufRead};

//to calculate the time

use std::time::{Instant};

//const RH: u8 = 255;
static mut RH: u8 = 0;
static mut RL: u8 = 0;
static mut GH: u8 = 0;
static mut GL: u8 = 0;
static mut BH: u8 = 0;
static mut BL: u8 = 0;
static mut OFFSET: i32 = 0;
static mut SIZE_THRESHOLD: i32 = 0;
static mut COUNT_THRESHOLD: u32 = 0;
static mut PROX: u32 = 0;


/* @brief This function checks if a RGB-Format Pixel is inside a certain color range
			and returns true or false.
*/
unsafe fn rgb_filter(red: u8, green: u8, blue: u8) ->bool {
    if red >= RL && red <= RH && green >= GL && green <= GH && blue >= BL && blue <= BH {
        return true;
    }

    return false;
}


/* @brief This function converts a image into an u8 Vector, containing the RGB values
		  Vec[pixel_index + 0] = Red
		  Vec[pixel_index + 1] = Green
		  Vec[pixel_index + 0] = Blue

*/
fn img_to_vec(img: &DynamicImage)->Vec<u8> {
    let (width, height) = img.dimensions();
    let mut tvec: Vec<u8> = Vec::with_capacity((width * height) as usize);


    for h in 0..height {
        for w in 0.. width {
            let rgb = img.get_pixel(w, h);
            tvec.push(rgb[0]);
            tvec.push(rgb[1]);
            tvec.push(rgb[2]);
        }
    }

    return tvec;

}

/* @brief This function converts an RBG-Pixel-Vector into an Image */
fn vec_to_img(img: &mut DynamicImage, pixel_vector: &Vec<u8>) {
    let (width, height) = img.dimensions();

    for i in 0..height {
        for j in 0.. width {

            let red = pixel_vector[((i*width*3) + (j*3)) as usize];
            let green = pixel_vector[((i*width*3) + (j*3) + 1) as usize];
            let blue = pixel_vector[((i*width*3) + (j*3) + 2) as usize];

            img.put_pixel(j, i, image::Rgba([red,green,blue,255]));
        }
    }
}



/* @brief this function blurs a pixel. This can be made faster by using a sliding window approach*/
fn blur(pixel_vector: &mut Vec<u8>, width: i32, height: i32, offset: i32, row: i32, col: i32) {


    let mut red_sum: u32 = 0;
    let mut green_sum: u32 = 0;
    let mut blue_sum: u32 = 0;
    let mut number_of_pixels: u32 = 0;

    for i in row-offset..row+offset {
        for j in col-offset..col+offset {
            if i >= 0 && i < height && j >= 0 && j < width  {
                red_sum += pixel_vector[((i*width*3) + (j*3)) as usize] as u32;
                green_sum += pixel_vector[((i*width*3) + (j*3) + 1) as usize] as u32;
                blue_sum += pixel_vector[((i*width*3) + (j*3) + 2) as usize] as u32;
                number_of_pixels += 1;
            }
        }
    }

    if number_of_pixels == 0 {return;}


    pixel_vector[((row*width*3) + (col*3)) as usize] = (red_sum / number_of_pixels) as u8;
    pixel_vector[((row*width*3) + (col*3) + 1) as usize] = (green_sum / number_of_pixels) as u8;
    pixel_vector[((row*width*3) + (col*3) + 2) as usize] = (blue_sum / number_of_pixels) as u8;

}



/* @brief This function sets all pixels to black */
fn set_all_black(pixel_vector: &mut Vec<u8>) {
    for i in 0..pixel_vector.len() {
        pixel_vector[i] = 0;
    }
}

unsafe fn widen(cords: &mut Vec<(i32, u32, u32, u32, u32)>, img_width: u32, img_height: u32) {
    // return (area, top_row, bot_row, left_col, right_col);
    let (mut a, mut b): (u32, bool);
    for i in 0..cords.len() {
        (a, b) = cords[i].1.overflowing_sub(PROX);
        if b == false {cords[i].1 = a;}

        (a, b) = cords[i].3.overflowing_sub(PROX);
        if b == false {cords[i].3 = a;}

        if cords[i].2 + PROX > img_height {cords[i].2 = img_height;}
        else {cords[i].2 += PROX;}

        if cords[i].4 + PROX > img_width {cords[i].4 = img_width;}
        else {cords[i].4 += PROX;}
    }
}


/* @brief This function is used to highlight/expose the approved Regions */
/*
fn expose(res: &mut Vec<u8>, org_img: &Vec<u8>, img_width: u32, cords: &Vec<(i32, u32, u32, u32, u32)>) {

    let mut i = 0;

    while i < res.len() {

        let row = ((i as u32) /3) / img_width;
        let col = ((i as u32) /3) % img_width;

        for j in 0..cords.len() {
            if row > cords[j].1 && row < cords[j].2 && col > cords[j].3 && col < cords[j].4 {
                res[i] = org_img[i];
                res[i + 1] = org_img[i + 1];
                res[i + 2] = org_img[i + 2];
            }
        }
        i += 3;
    }
}
*/


/* @brief This function is used to highlight/expose the approved Regions but does so faster then expose() */
fn expose2(res: &mut Vec<u8>, org_img: &Vec<u8>, img_width: u32, img_height:u32, cords: &Vec<(i32, u32, u32, u32, u32)>) {


    let mut helper: Vec<Vec<u32>> = Vec::with_capacity((img_height) as usize);
    for  _i in 0..img_height{
        helper.push(vec![0; img_width as usize]);
    }


    for i in 0..cords.len() {
        for r in cords[i].1 .. cords[i].2 {
            for mut _c in cords[i].3 .. cords[i].4 {

                res[((r*(img_width)*3) + (_c *3)) as usize] = org_img[((r*(img_width)*3) + (_c *3)) as usize];
                res[((r*(img_width)*3) + (_c *3) + 1) as usize] = org_img[((r*(img_width)*3) + (_c *3) + 1) as usize];
                res[((r*(img_width)*3) + (_c *3) + 2) as usize] = org_img[((r*(img_width)*3) + (_c *3) + 2) as usize];

                if helper[r as usize][_c as usize] == 0 {
                    helper[r as usize][_c as usize] = cords[i].4;

                }

                else {
                    _c = helper[r as usize][_c as usize];
                }
            }
        }
    }

}


/*
	@brief Just a normal DFS
	Usecase: explores connected areas that are "on fire"
		     and returns the total area, and the coordinates of the region
*/
fn dfs(pixel_vector: &Vec<u8>, visited: &mut Vec<Vec<u8>>, width: u32, height: u32, start_row: u32, start_col: u32)->(i32, u32, u32, u32, u32) {

    visited[start_row as usize][start_col as usize] = 1;

    let mut area = 0;
    let mut top_row = start_row;
    let mut bot_row = start_row;
    let mut left_col = start_col;
    let mut right_col = start_col;

    let moves_row:[i32; 4] = [-1, 0, 1, 0];
    let moves_col:[i32; 4] = [0, 1, 0, -1];

    let mut stack: Vec<Vec<u32>> = Vec::new();
    stack.push(vec![start_row, start_col]);


    while stack.len() > 0 {

        area += 1;

        let temp: Vec<u32> = stack.pop().unwrap();
        let row = temp[0];
        let col = temp[1];

        if row < top_row {top_row = row}
        if row > bot_row {bot_row = row}
        if col < left_col {left_col = col}
        if col > right_col {right_col = col}


        for i in 0..4 {
            let nrow: i32 = row as i32 + moves_row[i];
            let ncol: i32 = col as i32 + moves_col[i];

            if nrow >= 0 && nrow < height as i32 && ncol >= 0 && ncol < width as i32 && visited[nrow as usize][ncol as usize] == 0 {
                if pixel_vector[((nrow*(width as i32)*3) + (ncol*3)) as usize] == 255 {
                    stack.push(vec![nrow as u32, ncol as u32]);
                    visited[nrow as usize][ncol as usize] = 1;
                }
            }
        }

    }

    return (area, top_row, bot_row, left_col, right_col);
}



/*
	@brief This fucntion filters the pixel that are fire
	fire = white
	no fire = black
*/

unsafe fn filter(pixel_vector: &mut Vec<u8>, _img_width: u32, _img_height: u32) {
    let mut _total_fire_pixel = 0;

    let n = pixel_vector.len() as u32;
    let mut i: u32 = 0;


    while i < n {
        // calculate offset of each pixel beacuse of the rgb values (3 values in one pixel)
        //let row: u32 = (i/3) / img_width; is never used
        //let col: u32 = (i/3) % img_width; is never used


        let res: bool = rgb_filter(pixel_vector[(i+0) as usize], pixel_vector[(i+1) as usize], pixel_vector[(i+2) as usize]);
        if res == false {
            //img.put_pixel(col, row, image::Rgba([0,0,0,255]));
            pixel_vector[(i+0) as usize] = 0;
            pixel_vector[(i+1) as usize] = 0;
            pixel_vector[(i+2) as usize] = 0;
        }

        else {
            //img.put_pixel(col, row, image::Rgba([255,255,255,255]));
            pixel_vector[(i+0) as usize] = 255;
            pixel_vector[(i+1) as usize] = 255;
            pixel_vector[(i+2) as usize] = 255;
            _total_fire_pixel += 1;
        }

        i += 3;
    }

}


/* @brief this function uses all the other functions for preprocessing, filtering, exploring and exposing and returns the result */
unsafe fn get_fire_probability(pv: &mut Vec<u8>, img_width: u32, img_height: u32) ->f32 {

    let mut pixel_vector: Vec<u8> = pv.clone();
    let org_pv: Vec<u8> = pv.clone();
    set_all_black(pv);

    println!("cloning done");

    /* blur all pixels in the image */
    for i in 0..img_height{
        for j in 0..img_width {
            blur(&mut pixel_vector, img_width as i32, img_height as i32, OFFSET, i as i32, j as i32);
        }
    }

    println!("blurring done");

    // filter image -> fire = white, no fire = black
    filter(&mut pixel_vector, img_width, img_height);

    println!("filter done");


    /* init the visited vector
        visited keeps track of already explored fire area
    */
    let mut visited: Vec<Vec<u8>> = Vec::with_capacity(img_height as usize);
    for _i in 0..img_height{
        visited.push(vec![0; img_width as usize]);
    }


    let mut fire_area = 0; // the total fire-areas in pixel
    let mut count = 0; 	// the number of not connected fire-areas
    let mut cords: Vec<(i32, u32, u32, u32, u32)> = Vec::new(); // coordinates of recognized areas

    //use dfs to explore all the fire-area
    for i in 0..img_height{
        for j in 0..img_width {
            // only explore area if it has not been visited yet and if it is fire
            if visited[i as usize][j as usize] == 0 && pixel_vector[((i*(img_width)*3) + (j*3)) as usize] == 255 {
                let res =  dfs(&pixel_vector, &mut visited, img_width, img_height, i, j);

                // only count as a fire-area if it exceeds the SIZE_THRESHOLD size
                if res.0 > SIZE_THRESHOLD {
                    fire_area += res.0;
                    cords.push(res);
                    //println!("{} {} {} {} {}", res.0, res.1, res.2, res.3, res.4);
                    count += 1;
                }
            }
        }
    }

    println!("dfs done");

    // highlight/expose all recognized areas
    widen(&mut cords, img_width, img_height);
    expose2(pv, &org_pv, img_width, img_height, &cords);

    println!("expoes done");


    println!("{}, {}", fire_area, count);

    if count == 0 { return 0.0;}
    if count >= COUNT_THRESHOLD { return 1.0;}

    return (COUNT_THRESHOLD / count) as f32;

//	println!("{:.2}%", (fire_area as f64)/((img_width * img_height) as f64) * (count as f64) * (SIZE_THRESHOLD as f64));
}


fn read_file(file_path: &str) -> Vec<String> {
    let file = match File::open(file_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Error opening file: {}", e);
            return vec![];
        }
    };
    let mut lines = Vec::new();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        match line {
            Ok(line) => lines.push(line),
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                return vec![];
            }
        }
    }
    lines
}


fn parse_config(lines: Vec<String>) {
    unsafe {
        let num: u8 = lines[0].trim().parse().unwrap();
        RH = num;
        let num: u8 = lines[1].trim().parse().unwrap();
        RL = num;
        let num: u8 = lines[2].trim().parse().unwrap();
        GH = num;
        let num: u8 = lines[3].trim().parse().unwrap();
        GL = num;
        let num: u8 = lines[4].trim().parse().unwrap();
        BH = num;
        let num: u8 = lines[5].trim().parse().unwrap();
        BL = num;
        let num: i32 = lines[6].trim().parse().unwrap();
        OFFSET = num;
        let num: i32 = lines[7].trim().parse().unwrap();
        SIZE_THRESHOLD = num;
        let num: u32 = lines[8].trim().parse().unwrap();
        COUNT_THRESHOLD = num;
        let num: u32 = lines[9].trim().parse().unwrap();
        PROX = num;
    }
}


fn main() {
    let start = Instant::now();
    let file_path = "src/ImageData";
    let lines = read_file(file_path);  // read file and save it in lines
    // println!("{}", lines[0]);
    parse_config(lines);

    let file_name: &str = "test5.jpg";
    let mut img = image::open(file_name).unwrap();
    let (img_width, img_height) = img.dimensions();

    let mut pixel_vector: Vec<u8> = img_to_vec(&img);
    let res: f32 = unsafe { get_fire_probability(&mut pixel_vector, img_width, img_height) };

    println!("Fire Percentage {}%", res*100.0);

    vec_to_img(&mut img, &pixel_vector);
    img.save("res_wild.png").unwrap();
    let duration = start.elapsed();
    println!("Time elapsed is: {:?}", duration);
}




#[cfg(test)]
mod tests {
    use super::*;
    

    #[test]
    fn color_range_test() {
        let file_path = "src/ImageData";
        let lines = read_file(file_path);  // read file and save it in lines
        // println!("{}", lines[0]);
        parse_config(lines);
        unsafe {
            use rand::Rng;
            let mut rng = rand::thread_rng();
            assert!(rgb_filter(rng.gen_range(200..255), rng.gen_range(50..255),rng.gen_range(0..95)));
        }
    }


    #[test]
    fn test_img_to_vec() {
        let img = image::open("test.jpg").unwrap();
        let vec = img_to_vec(&img);
        assert_eq!(vec.len() as u32, img.dimensions().0 * img.dimensions().1 * 3);
    }


    #[test]
    fn test_blur() {
        let mut img = image::open("test.jpg").unwrap();
        let mut pixel_vector = img_to_vec(&img);
        let (width, height) = img.dimensions();
        blur(&mut pixel_vector, width as i32, height as i32, 1, 1, 1);
        vec_to_img(&mut img, &pixel_vector);
        let blurred_pixel = img.get_pixel(1, 1);
        // check if the blurred pixel values are not equal to the original
        assert!(blurred_pixel[0] != 0 || blurred_pixel[1] != 0 || blurred_pixel[2] != 0);
    }


    #[test]
    fn test_set_all_black() {
        let mut img = image::open("test.jpg").unwrap();
        let mut pixel_vector = img_to_vec(&img);
        set_all_black(&mut pixel_vector);
        vec_to_img(&mut img, &pixel_vector);
        let (width, height) = img.dimensions();
        for i in 0..height {
            for j in 0.. width {
                let black_pixel = img.get_pixel(j, i);
                // check if all pixels are black
                assert_eq!(black_pixel, image::Rgba([0, 0, 0, 255]));
            }
        }
    }


    #[test]
    fn test_get_fire_probability() {
        let mut pv = vec![0, 0, 0, 255, 255, 255, 0, 0, 0];
        let img_width = 3;
        let img_height = 1;
        let expected = 1.0;
        unsafe{
            let result = get_fire_probability(&mut pv, img_width, img_height);
            assert_eq!(result, expected);
        }
        //assert_eq!(result, expected);
    }


    

}
