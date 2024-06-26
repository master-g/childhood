mod palettes;

#[allow(unused_imports)]
pub use palettes::*;
use serde::Serialize;

#[derive(clap::ValueEnum, Debug, Clone, Default, Serialize)] // ArgEnum here
#[serde(rename_all = "kebab-case")]
pub enum Palette {
	#[default]
	P3DSVC,
	ASQRealityA,
	ASQRealityB,
	BMFFinal2,
	BMFFinal3,
	Commodore64,
	CompositeDirectFBX,
	Consumer,
	CRTNostalgiaBeta01,
	FBXCompositeDCFinal,
	FBXCompositeDCFinalSaturated,
	FBXNESUnsaturated,
	FBXYUVCustomized,
	Fceu13DefaultNitsuja,
	Fceu15NitsujaNew,
	FceuX,
	FX3NESPAL,
	Grayscale,
	Hybrid,
	NesCap,
	NESClassic,
	NESClassicBeta,
	NESClassicFBX,
	NESClassicFBXfs,
	NESRemixU,
	NintendulatorNTSC,
	NostalgiaFBX,
	NtscU,
	OriginalHardwareFBX,
	PVMtoDigitalFBXbeta01,
	PVMtoDigitalFBXbeta02,
	PVMtoDigitalFBXbeta03,
	PVMtoDigitalFBXbeta04,
	Raw,
	Rgb,
	Rockman9,
	Rockman921to2C,
	Sony,
	UnsaturatedFinal,
	UnsaturatedV4,
	UnsaturatedV5,
	UnsaturatedV6,
	UnsaturatedV7,
	WiiVC,
	WiiVCbrighter,
	Xvzgjw,
	Yuv,
	Yuvv3,
	YUVCorrected,
}

impl Palette {
	pub fn as_slice(&self) -> &[(u8, u8, u8)] {
		match self {
			Palette::P3DSVC => P3DSVC.as_slice(),
			Palette::ASQRealityA => ASQ_REALITY_A.as_slice(),
			Palette::ASQRealityB => ASQ_REALITY_B.as_slice(),
			Palette::BMFFinal2 => BMF_FINAL_2.as_slice(),
			Palette::BMFFinal3 => BMF_FINAL_3.as_slice(),
			Palette::Commodore64 => COMMODORE_64.as_slice(),
			Palette::CompositeDirectFBX => COMPOSITE_DIRECT_FBX.as_slice(),
			Palette::Consumer => CONSUMER.as_slice(),
			Palette::CRTNostalgiaBeta01 => CRT_NOSTALGIA_BETA_01.as_slice(),
			Palette::FBXCompositeDCFinal => FBX_COMPOSITE_DC_FINAL.as_slice(),
			Palette::FBXCompositeDCFinalSaturated => FBX_COMPOSITE_DC_FINAL_SATURATED.as_slice(),
			Palette::FBXNESUnsaturated => FBX_NES_UNSATURATED.as_slice(),
			Palette::FBXYUVCustomized => FBX_YUV_CUSTOMIZED.as_slice(),
			Palette::Fceu13DefaultNitsuja => FCEU13_DEFAULT_NITSUJA.as_slice(),
			Palette::Fceu15NitsujaNew => FCEU15_NITSUJA_NEW.as_slice(),
			Palette::FceuX => FCEUX.as_slice(),
			Palette::FX3NESPAL => FX3_NES_PAL.as_slice(),
			Palette::Grayscale => GRAYSCALE.as_slice(),
			Palette::Hybrid => HYBRID.as_slice(),
			Palette::NesCap => NES_CAP.as_slice(),
			Palette::NESClassic => NES_CLASSIC.as_slice(),
			Palette::NESClassicBeta => NES_CLASSIC_BETA.as_slice(),
			Palette::NESClassicFBX => NES_CLASSIC_FBX.as_slice(),
			Palette::NESClassicFBXfs => NES_CLASSIC_FBX_FS.as_slice(),
			Palette::NESRemixU => NES_REMIX_U.as_slice(),
			Palette::NintendulatorNTSC => NINTENDULATOR_NTSC.as_slice(),
			Palette::NostalgiaFBX => NOSTALGIA_FBX.as_slice(),
			Palette::NtscU => NTSCU.as_slice(),
			Palette::OriginalHardwareFBX => ORIGINAL_HARDWARE_FBX.as_slice(),
			Palette::PVMtoDigitalFBXbeta01 => PVM_TO_DIGITAL_FBX_BETA_01.as_slice(),
			Palette::PVMtoDigitalFBXbeta02 => PVM_TO_DIGITAL_FBX_BETA_02.as_slice(),
			Palette::PVMtoDigitalFBXbeta03 => PVM_TO_DIGITAL_FBX_BETA_03.as_slice(),
			Palette::PVMtoDigitalFBXbeta04 => PVM_TO_DIGITAL_FBX_BETA_04.as_slice(),
			Palette::Raw => RAW.as_slice(),
			Palette::Rgb => RGB.as_slice(),
			Palette::Rockman9 => ROCKMAN_9.as_slice(),
			Palette::Rockman921to2C => ROCKMAN_9_21_TO_2C.as_slice(),
			Palette::Sony => SONY.as_slice(),
			Palette::UnsaturatedFinal => UNSATURATED_FINAL.as_slice(),
			Palette::UnsaturatedV4 => UNSATURATED_V4.as_slice(),
			Palette::UnsaturatedV5 => UNSATURATED_V5.as_slice(),
			Palette::UnsaturatedV6 => UNSATURATED_V6.as_slice(),
			Palette::UnsaturatedV7 => UNSATURATED_V7.as_slice(),
			Palette::WiiVC => WII_VC.as_slice(),
			Palette::WiiVCbrighter => WII_VC_BRIGHTER.as_slice(),
			Palette::Xvzgjw => XVZGJW.as_slice(),
			Palette::Yuv => YUV.as_slice(),
			Palette::Yuvv3 => YUVV_3.as_slice(),
			Palette::YUVCorrected => YUV_CORRECTED.as_slice(),
		}
	}
}
