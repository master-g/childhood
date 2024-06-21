pub mod ines1;
pub mod ines2;

#[derive(Debug, PartialEq)]
pub enum Mirroring {
	Vertical,
	Horizontal,
	FourScreen,
}
