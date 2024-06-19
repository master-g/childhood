use crate::err::Error;

#[allow(clippy::unused_async)]
pub(super) async fn init() -> Result<(), Error> {
	trace!("foo!");
	debug!("foo!");
	info!("foo!");
	warn!("foo!");
	error!("foo!");

	println!("foo!");

	generate_image();

	Ok(())
}

#[allow(clippy::cast_lossless)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn generate_image() {
	//! An example of generating julia fractals.
	let img_x = 800;
	let img_y = 800;

	let scale_x = 3.0 / img_x as f64;
	let scale_y = 3.0 / img_y as f64;

	// Create a new ImgBuf with width: imgx and height: imgy
	let mut imgbuf = image::ImageBuffer::new(img_x, img_y);

	// Iterate over the coordinates and pixels of the image
	for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
		let r = (0.3 * x as f64) as u8;
		let b = (0.3 * y as f64) as u8;
		*pixel = image::Rgb([r, 0, b]);
	}

	// A redundant loop to demonstrate reading image data
	for x in 0..img_x {
		for y in 0..img_y {
			let cx = y as f64 * scale_x - 1.5;
			let cy = x as f64 * scale_y - 1.5;

			let c = num_complex::Complex::new(-0.4, 0.6);
			let mut z = num_complex::Complex::new(cx, cy);

			let mut i = 0;
			while i < 255 && z.norm() <= 2.0 {
				z = z * z + c;
				i += 1;
			}

			let pixel = imgbuf.get_pixel_mut(x, y);
			let image::Rgb(data) = *pixel;
			*pixel = image::Rgb([data[0], i as u8, data[2]]);
		}
	}

	// Save the image as “fractal.png”, the format is deduced from the path
	imgbuf.save("fractal.png").unwrap();
}
