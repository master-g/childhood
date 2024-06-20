use clap::Args;
use image::{ImageBuffer, Rgba};
use tokio::io::AsyncReadExt;

use crate::{
	err::Error,
	palette::{Palette, RGB_SIZE},
};

const CHR_SIZE: usize = 8192; // process 8KB per time
const PAGE_SIZE_IN_BYTES: usize = 256 * 16; // 16*16 tiles, 16 bytes per tile

#[derive(Args, Debug)]
pub(super) struct ChrCommandArguments {
	#[arg(short, long, help = "path to the chr file")]
	src: String,

	#[arg(short, long, help = "output")]
	out: String,

	#[clap(short, long, default_value_t, value_enum)]
	palette: Palette,

	#[arg(short, long, help = "chr palette")]
	#[arg(default_value = "22271618")]
	chr: String,
}

pub(super) async fn exec(args: ChrCommandArguments) -> Result<(), Error> {
	let sprite_palette = hex::decode(&args.chr)?;
	let canvas_palette = args.palette.as_slice();
	// copy the sprite palette to the global palette
	let mut f = tokio::fs::File::open(&args.src).await?;

	let fp_output = std::path::Path::new(&args.out);
	let without_ext = fp_output.with_extension("");
	let extension = if let Some(ext) = fp_output.extension() {
		ext.to_str().unwrap().to_lowercase()
	} else {
		"png".to_string()
	};

	let mut file_no = 0;
	let mut buf = vec![0u8; CHR_SIZE];
	loop {
		let bytes_read = f.read(&mut buf).await?;
		if bytes_read == 0 {
			debug!("file read completed");
			break;
		}
		let img = draw_image(canvas_palette, &sprite_palette, &buf[..bytes_read]);
		let op = format!("{}_{:03}.{}", without_ext.to_str().unwrap(), file_no, extension);
		match extension.as_str() {
			"bmp" => img.save_with_format(op, image::ImageFormat::Bmp)?,
			"jpg" => img.save_with_format(op, image::ImageFormat::Jpeg)?,
			_ => img.save_with_format(op, image::ImageFormat::Png)?,
		}

		file_no += 1;
	}

	Ok(())
}

#[allow(clippy::cast_lossless)]
fn set_tile_pixel(y: usize, line: u8, buf: &mut [u32], add: bool) {
	let mirror = line.reverse_bits();
	for x in 0..8 {
		let c = (mirror >> x) & 0x1;
		let pos = y * 8 + x;
		if add {
			buf[pos] = buf[pos] * 2 + c as u32;
		} else {
			buf[pos] = c as u32;
		}
	}
}

#[allow(clippy::cast_possible_truncation)]
fn write_tile(
	canvas_palette: &[u8],
	sprite_palette: &[u8],
	img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
	page: usize,
	tx: usize,
	ty: usize,
	pixels: &[u32],
) {
	for y in 0..8 {
		for x in 0..8 {
			let pixel = pixels[y * 8 + x];
			let ox = (tx + page * 16) * 8 + x;
			let oy = ty * 8 + y;
			let palette_value = sprite_palette[pixel as usize];
			let cpi: usize = (palette_value * RGB_SIZE as u8) as usize;
			let r = canvas_palette[cpi];
			let g = canvas_palette[cpi + 1];
			let b = canvas_palette[cpi + 2];
			img.put_pixel(ox as u32, oy as u32, Rgba([r, g, b, 255]));
		}
	}
}

fn draw_image(
	canvas_palette: &[u8],
	sprite_palette: &[u8],
	buf: &[u8],
) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
	let mut img = ImageBuffer::<Rgba<u8>, Vec<u8>>::new(256, 128);

	let mut tile_data = vec![0u32; 64];

	for (i, &b) in buf.iter().enumerate() {
		let page = i / PAGE_SIZE_IN_BYTES;
		let ii = i % PAGE_SIZE_IN_BYTES;
		let tile_x = ii / 16 % 16;
		let tile_y = ii / 256;
		let ti = i % 16;

		if ti < 8 {
			// first pass
			set_tile_pixel(i % 8, b, &mut tile_data, false);
		} else {
			// second pass
			set_tile_pixel(i % 8, b, &mut tile_data, true);
		}

		if ti == 15 {
			// draw
			write_tile(canvas_palette, sprite_palette, &mut img, page, tile_x, tile_y, &tile_data);
		}
	}

	img
}
