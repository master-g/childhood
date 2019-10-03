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

const (
	// MirrorHorizontal for cartridge mirroring horizontal
	MirrorHorizontal = iota
	MirrorVertical
	MirrorOneScreenLo
	MirrorOneScreenHi
)

// Cartridge represents a NES cartridge from a software perspective
type Cartridge struct {
	Mirror int

	imageValid  bool
	mapperId    uint8
	numPRGBanks uint8
	numCHRBanks uint8

	memPRG []uint8
	memCHR []uint8

	mapper Mapper
}

func (cart *Cartridge) IsImageValid() bool {
	return cart.imageValid
}

func (cart *Cartridge) CpuRead(addr uint16) (data uint8, flag bool) {
	var mappedAddr uint32
	if mappedAddr, flag = cart.mapper.CpuMapRead(addr); flag {
		data = cart.memPRG[mappedAddr]
	}
	return
}

func (cart *Cartridge) CpuWrite(addr uint16, data uint8) (flag bool) {
	var mappedAddr uint32
	if mappedAddr, flag = cart.mapper.CpuMapWrite(addr); flag {
		cart.memPRG[mappedAddr] = data
	}
	return
}

func (cart *Cartridge) PpuRead(addr uint16) (data uint8, flag bool) {
	var mappedAddr uint32
	if mappedAddr, flag = cart.mapper.PpuMapRead(addr); flag {
		data = cart.memCHR[mappedAddr]
	}
	return
}

func (cart *Cartridge) PpuWrite(addr uint16, data uint8) (flag bool) {
	var mappedAddr uint32
	if mappedAddr, flag = cart.mapper.PpuMapWrite(addr); flag {
		cart.memCHR[mappedAddr] = data
	}
	return
}
