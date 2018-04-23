package main

import (
	"bytes"
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
)

// NewHeader create a new header from data
func NewHeader(r io.Reader) *Header {
	buf := make([]byte, HeaderSize)
	n, err := r.Read(buf)
	if n != HeaderSize || err != nil {
		return nil
	}
	h := &Header{}
	copy(h.Identifier[:], buf[:4])
	if bytes.Compare(h.Identifier[:], standardIdentifier) != 0 {
		return nil
	}

	h.PRG = buf[4]
	h.CHR = buf[5]
	h.Flag6 = buf[6]
	h.Flag7 = buf[7]
	h.PRGRAM = buf[8]
	h.Flag9 = buf[9]
	h.Flag10 = buf[10]
	copy(h.padding[:], buf[10:])
	if bytes.Compare(h.padding[:], standardPadding) != 0 {
		return nil
	}

	return h
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
		h.Mapper(), getMapper(int(h.Mapper())),
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
