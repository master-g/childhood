package main

var (
	mappers map[int]string
)

func init() {
	mappers = map[int]string{
		0:   "No Mapper",
		1:   "MMC1",
		2:   "UNROM",
		3:   "CNROM",
		4:   "MMC3",
		5:   "MMC5",
		6:   "FFE F4xxx",
		7:   "AOROM",
		8:   "FFE F3xxx",
		9:   "MMC2",
		10:  "MMC4",
		11:  "Colour Dreams",
		12:  "FFE F6xxx",
		13:  "CPROM",
		15:  "100-in-1",
		16:  "Bandai",
		17:  "FFE F8xxx",
		18:  "Jaleco SS8806",
		19:  "Namcot 106",
		20:  "Famicom Disk System",
		21:  "Konami VRC4-2A",
		22:  "Konami VRC4-1B",
		23:  "Konami VRC2B",
		24:  "Konami VRC6",
		25:  "Konami VRC4",
		26:  "Konami VRC6v",
		32:  "Irem G-101",
		33:  "Taito TC0190/TC0350",
		34:  "Nina-1",
		48:  "TC190V",
		64:  "Rambo-1",
		65:  "Irem H3001",
		66:  "74161/32",
		67:  "Sunsoft 3",
		68:  "Sunsoft 4",
		69:  "Sunsoft 5",
		70:  "74161/32",
		71:  "Camerica",
		78:  "74161/32",
		79:  "AVE",
		80:  "Taito X005",
		81:  "C075",
		82:  "Taito X1-17",
		83:  "PC-Cony",
		84:  "PasoFami",
		85:  "VRC7",
		88:  "Namco 118",
		90:  "PCJY??",
		91:  "HK-SF3",
		95:  "Namco 1xx",
		97:  "Irem 74161/32",
		99:  "Unisystem",
		119: "TQROM",
		159: "Bandai",
	}
}

func getMapper(num int) string {
	if name, ok := mappers[num]; ok {
		return name
	} else {
		return "Unknown"
	}
}
