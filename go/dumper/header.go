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

type MirroringDirection int

const (
	MirroringHorizontal MirroringDirection = 0
	MirroringVertical   MirroringDirection = 1
)

type Header struct {
	Identifier [4]byte // Identifier must be ascii 'NES' and a MS-DOS character break
	PRG        uint8   // PRG size of PRG ROM in 16 KB units
	CHR        uint8   // CHR size of CHR ROM in 8KB units, 0 means CHR RAM only
	Flag6      uint8   // NNNN FTBM
	Flag7      uint8   // NNNN xxPV
}

var (
	standardIdentifier = []byte{0x4E, 0x45, 0x53, 0x1A}
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

	return h
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

func (h *Header) String() string {
	var ver string
	if h.NES20() {
		ver = "iNES2.0"
	} else {
		ver = "iNES1.0"
	}
	return fmt.Sprintf(`VER: %v
PRG: %v %vKB
CHR: %v %vKB
MAP: %v %v`,
		ver, h.PRG, h.PRG*16, h.CHR, h.CHR*8, h.Mapper(), getMapper(int(h.Mapper())))
}
