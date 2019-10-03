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

package pkg

// MG2C02 emulates NES' PPU unit (2C02 chip) from a software perspective
type MG2C02 struct {
	name    [2][1024]uint8
	pattern [2][4096]uint8
	palette [32]uint8

	scanline int16
	cycle    int16
}

func (ppu *MG2C02) CpuWrite(addr uint16, data uint8) {
	// ppu.addr & 0x0007 = data
	return
}

func (ppu *MG2C02) CpuRead(addr uint16, readonly bool) (data uint8) {
	// data = ppu.addr & 0x0007
	return
}

func (ppu *MG2C02) AttachCartridge(cart *Cartridge) {

}

func (ppu *MG2C02) Clock() {

}
