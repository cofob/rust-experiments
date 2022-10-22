use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use num::Complex;
use std::fs::File;
use std::io::{Error, ErrorKind, Write};
use std::str::FromStr;

/// Find the escape time for a given point in the complex plane.
fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z * z + c;
        // If the absolute value of z is greater than 2, then the point is
        // unbounded and we return the number of iterations it took to get
        // there.
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}

/// Parse the string `s` as a pair, like `"400x600"` or `"1.0,0.5"`.
///
/// # Examples
/// ```
/// assert_eq!(parse_pair("400x600", 'x'), Ok((400, 600)));
/// assert_eq!(parse_pair("1.0,0.5", ','), Ok((1.0, 0.5)));
/// ```
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    // Find position of separator
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

/// Parse a pair of floating-point numbers separated by a comma as a complex number.
fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

#[test]
fn test_parse_complex() {
    assert_eq!(
        parse_complex("1.25,-0.0625"),
        Some(Complex {
            re: 1.25,
            im: -0.0625
        })
    );
    assert_eq!(parse_complex(",-0.0625"), None);
}

/// Given the row and column of a pixel in the output image, return the corresponding point on the
/// complex plane.
fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 100),
            (25, 75),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 }
        ),
        Complex { re: -0.5, im: -0.5 }
    );
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    // Iterate over the rows of the image.
    for row in 0..bounds.1 {
        // Iterate over the columns of the image.
        for column in 0..bounds.0 {
            // Find the point in the complex plane that corresponds to this pixel in the output image.
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            // Compute the escape time for that point.
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 16,
                Some(count) => count as u8,
            };
        }
    }
}

#[test]
fn test_render() {
    let mut pixels = [0; 10 * 10];
    render(
        &mut pixels,
        (10, 10),
        Complex { re: 0.0, im: 0.0 },
        Complex { re: 0.0, im: 0.0 },
    );
    println!("{:?}", pixels);
    assert_eq!(pixels[0], 16);
    assert_eq!(pixels[1], 16);
    assert_eq!(pixels[2], 16);
    assert_eq!(pixels[3], 16);
}

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
fn write_image(
    filename: &str,
    pixels: &[u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    // Create a new file.
    let output = File::create(filename)?;

    // Create a new encoder that writes to the file we just created.
    let encoder = PngEncoder::new(output);
    match encoder.write_image(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::L8) {
        Ok(_) => (),
        Err(e) => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to write image: {:?}", e),
            ));
        }
    };

    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Check that we have the right number of arguments.
    if args.len() != 5 {
        writeln!(
            std::io::stderr(),
            "Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT"
        )
        .unwrap();
        writeln!(
            std::io::stderr(),
            "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
            args[0]
        )
        .unwrap();
        std::process::exit(1);
    }

    // Parse the arguments.
    let bounds = parse_pair(&args[2], 'x').expect("error parsing image dimensions");
    let upper_left = parse_complex(&args[3]).expect("error parsing upper left corner point");
    let lower_right = parse_complex(&args[4]).expect("error parsing lower right corner point");

    // Create a buffer of pixels.
    let mut pixels = vec![0; bounds.0 * bounds.1];

    // Render the Mandelbrot set into the buffer.
    let threads = num_cpus::get();
    let rows_per_band = bounds.1 / threads + 1;
    {
        let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();
        crossbeam::scope(|spawner| {
            for (i, band) in bands.into_iter().enumerate() {
                let top = rows_per_band * i;
                let height = band.len() / bounds.0;
                let band_bounds = (bounds.0, height);
                let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
                let band_lower_right =
                    pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);
                spawner.spawn(move |_| {
                    render(band, band_bounds, band_upper_left, band_lower_right);
                });
            }
        })
        .expect("Failed to render");
    }

    // Write the buffer as a PNG image.
    write_image(&args[1], &pixels, bounds).expect("error writing PNG file");
}
