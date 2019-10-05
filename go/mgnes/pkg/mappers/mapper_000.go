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

package mappers

type Mapper000 struct {
	numPRGBanks uint8
	numCHRBanks uint8
}

func NewMapper000(numPRGBanks, numCHRBanks uint8) *Mapper000 {
	return &Mapper000{
		numPRGBanks: numPRGBanks,
		numCHRBanks: numCHRBanks,
	}
}

func (m *Mapper000) CpuMapRead(addr uint16) (mappedAddr uint32, flag bool) {
	// if PRGROM is 16KB
	//     CPU Address Bus          PRG ROM
	//     0x8000 -> 0xBFFF: Map    0x0000 -> 0x3FFF
	//     0xC000 -> 0xFFFF: Mirror 0x0000 -> 0x3FFF
	// if PRGROM is 32KB
	//     CPU Address Bus          PRG ROM
	//     0x8000 -> 0xFFFF: Map    0x0000 -> 0x7FFF
	if addr >= 0x8000 {
		if m.numPRGBanks > 1 {
			mappedAddr = uint32(addr & 0x7FFF)
		} else {
			mappedAddr = uint32(addr & 0x3FFF)
		}

		flag = true
	}
	return
}

func (m *Mapper000) CpuMapWrite(addr uint16) (mappedAddr uint32, flag bool) {
	if addr >= 0x8000 {
		if m.numPRGBanks > 1 {
			mappedAddr = uint32(addr & 0x7FFF)
		} else {
			mappedAddr = uint32(addr & 0x3FFF)
		}
		flag = true
	}
	return
}

func (m *Mapper000) PpuMapRead(addr uint16) (mappedAddr uint32, flag bool) {
	// There is no mapping required for PPU
	// PPU Address Bus          CHR ROM
	// 0x0000 -> 0x1FFF: Map    0x0000 -> 0x1FFF
	if addr <= 0x1FFF {
		mappedAddr = uint32(addr)
		flag = true
	}
	return
}

func (m *Mapper000) PpuMapWrite(addr uint16) (mappedAddr uint32, flag bool) {
	if addr <= 0x1FFF {
		if m.numCHRBanks == 0 {
			// Treat as RAM
			mappedAddr = uint32(addr)
			flag = true
		}
	}
	return
}
