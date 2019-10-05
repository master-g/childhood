// Copyright © 2019 mg
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in
// all copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN
// THE SOFTWARE.

package ines

import (
	"bytes"
	"errors"
	"fmt"
	"io"
)

const (
	// HeaderSize standard NES rom header is 16 bytes
	HeaderSize = 16
)

// MirroringDirection mirroring direction
type MirroringDirection int

// TVSystemType support TV system
type TVSystemType int

// TVCompatibleType TV compatible
type TVCompatibleType int

const (
	MirroringHorizontal MirroringDirection = 0
	MirroringVertical   MirroringDirection = 1

	TVSystemNTSC TVSystemType = 0
	TVSystemPAL  TVSystemType = 1

	TVCompatibleNTSC TVCompatibleType = 0
	TVCompatiblePAL  TVCompatibleType = 2
	TVCompatibleDual TVCompatibleType = 3
)

func (d MirroringDirection) String() string {
	if d == MirroringHorizontal {
		return "Horizontal"
	} else if d == MirroringVertical {
		return "Vertical"
	} else {
		return "N/A"
	}
}

func (t TVSystemType) String() string {
	if t == TVSystemNTSC {
		return "NTSC"
	} else if t == TVSystemPAL {
		return "PAL"
	} else {
		return "N/A"
	}
}

func (t TVCompatibleType) String() string {
	if t == TVCompatibleNTSC {
		return "NTSC"
	} else if t == TVCompatiblePAL {
		return "PAL"
	} else if t == TVCompatibleDual {
		return "DUAL"
	} else {
		return "N/A"
	}
}

// Header represents a standard iNES format header
type Header struct {
	Identifier [4]byte // Identifier must be ascii 'NES' and a MS-DOS character break
	PRG        uint8   // PRG size of PRG ROM in 16 KB units
	CHR        uint8   // CHR size of CHR ROM in 8KB units, 0 means CHR RAM only
	Flag6      uint8   // NNNN FTBM
	Flag7      uint8   // NNNN xxPV
	PRGRAM     uint8   // PRG RAM in 8KB units, 0 infers 8KB for compatibility
	Flag9      uint8   // xxxx xxxT
	Flag10     uint8   // xxBP xxTT
	padding    [5]byte // zero padding
}

var (
	standardIdentifier = []byte{0x4E, 0x45, 0x53, 0x1A}
	standardPadding    = []byte{0x00, 0x00, 0x00, 0x00, 0x00}
	magic2mapper       = map[int]string{
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
)

// NewHeader create a new header from data
func NewHeader(r io.Reader) (header *Header, err error) {
	buf := make([]byte, HeaderSize)
	n := 0
	n, err = io.ReadAtLeast(r, buf, HeaderSize)
	if n != HeaderSize {
		err = errors.New("invalid header size")
		return
	}
	if err != nil {
		return
	}
	header = &Header{}
	copy(header.Identifier[:], buf[:4])
	if !bytes.Equal(header.Identifier[:], standardIdentifier) {
		err = errors.New("invalid identifier")
		header = nil
		return
	}

	header.PRG = buf[4]
	header.CHR = buf[5]
	header.Flag6 = buf[6]
	header.Flag7 = buf[7]
	header.PRGRAM = buf[8]
	header.Flag9 = buf[9]
	header.Flag10 = buf[10]
	copy(header.padding[:], buf[10:])
	if !bytes.Equal(header.padding[:], standardPadding) {
		err = errors.New("invalid padding")
		header = nil
		return
	}

	return
}

// PRGROMSize returns PRG ROM size
func (h *Header) PRGROMSize() int {
	return int(h.PRG) * 16 * 1024
}

// CHRROMSize returns CHR ROM size
func (h *Header) CHRROMSize() int {
	return int(h.CHR) * 8 * 1024
}

// Mapper returns mapper number
func (h *Header) Mapper() uint8 {
	low4 := (h.Flag6 & 0xF0) >> 4
	high4 := h.Flag7 & 0xF0
	return low4 | high4
}

// Flag6
// --------
// 76543210
// NNNNFTBM
// ||||||||
// |||||||+- Mirroring. 0 = horizontal, 1 = vertical
// ||||||+-- SRAM at 6000-7FFFh battery backed. 0 = no, 1 = yes
// |||||+--- Trainer. 0 = no trainer present, 1 = 512 byte trainer at 7000-71FFh
// ||||+---- Four screen mode. 0 = no, 1 = yes. (When set, the M bit has no effect)
// ++++----- Lower 4 bits of the mapper number

// FourScreenMode returns true when F flag is set
func (h *Header) FourScreenMode() bool {
	return h.Flag6&0x08 != 0
}

// Trainer returns true when T flag is set
func (h *Header) Trainer() bool {
	return h.Flag6&0x04 != 0
}

// PersistentSRAM returns true when B flag is set
func (h *Header) PersistentSRAM() bool {
	return h.Flag6&0x02 != 0
}

// Mirroring returns mirroring direction
func (h *Header) Mirroring() MirroringDirection {
	return MirroringDirection(h.Flag6 & 0x01)
}

// Flag7
// --------
// 76543210
// NNNNSSPV
// ||||||||
// |||||||+- Vs. Unisystem. When set, this is a Vs. game
// ||||||+-- PlayChoice-10. When set this is a PC-10 Game (8KB of Hint Screen data stored after CHR data)
// ||||++--- If equal to 2, flags 8-15 are in NES 2.0 format
// ++++----- Upper 4 bits of the mapper number

// NES20 returns true when header is in iNES2.0 format
func (h *Header) NES20() bool {
	return h.Flag7&0x0C != 0
}

// PlayChoice10 returns true if header is a PC-10 game header
func (h *Header) PlayChoice10() bool {
	return h.Flag7&0x02 != 0
}

// Vs returns true is header is a Vs. game header
func (h *Header) Vs() bool {
	return h.Flag7&0x01 != 0
}

// Byte8
// --------
// PRGRAMSize returns size of PRG RAM
func (h *Header) PRGRAMSize() int {
	if h.PRGRAM == 0 {
		return 8
	} else {
		return int(h.PRGRAM) * 8
	}
}

// Flag9
// --------
// 76543210
// xxxxxxxT
// ||||||||
// |||||||+- TV system. 0 = NTSC, 1 = PAL
// +++++++-- Reserved, must be 0

// TVSystem returns TV system type defined in flag9
func (h *Header) TVSystem() TVSystemType {
	return TVSystemType(h.Flag9 & 0x01)
}

// Flag10
// --------
// 76543210
// xxBPxxTT
//   ||  ||
//   ||  ++- TV system. 0 = NTSC, 2 = PAL, 1,3 = dual compatible
//   |+----- PRG RAM. 0 = present, 1 = not present
//   +------ Bus conflict. 0 = no conflict, 1 = bus conflict

// TVCompatible returns TV system type in flag10
func (h *Header) TVCompatible() TVCompatibleType {
	f := h.Flag10 & 0x03
	if f == 1 || f == 3 {
		return TVCompatibleDual
	} else {
		return TVCompatibleType(f)
	}
}

// PRGRAMPresent returns PRG RAM present in flag10
func (h *Header) PRGRAMPresent() bool {
	return (h.Flag10 & 0x10) == 0
}

// BusConflict returns true if board has bus conflict
func (h *Header) BusConflict() bool {
	return (h.Flag10 & 0x20) != 0
}

func (h *Header) String() string {
	var ver string
	if h.NES20() {
		ver = "iNES2.0"
	} else {
		ver = "iNES1.0"
	}
	return fmt.Sprintf(`HDR: %v
VER: %v
PRG: %v %vKB
CHR: %v %vKB
MAP: %v %v
PRG: %v %vKB
4Screen: %v
Trainer: %v
PersistentSRAM: %v
Mirroring: %v
PlayChoice10: %v
VS Unisystem: %v
TV System: %v
TV Compatible: %v
BUS Conflict: %v`,
		string(h.Identifier[:]),
		ver,
		h.PRG, h.PRG*16,
		h.CHR, h.CHR*8,
		h.Mapper(), Magic2Mapper(int(h.Mapper())),
		h.PRGRAM, h.PRGRAMSize(),
		h.FourScreenMode(),
		h.Trainer(),
		h.PRGRAMPresent(),
		h.Mirroring(),
		h.PlayChoice10(),
		h.Vs(),
		h.TVSystem(),
		h.TVCompatible(),
		h.BusConflict(),
	)
}

func Magic2Mapper(mapperId int) string {
	if mapper, ok := magic2mapper[mapperId]; ok {
		return mapper
	} else {
		return "Unknown"
	}
}
