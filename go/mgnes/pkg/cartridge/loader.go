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

package cartridge

import (
	"errors"
	"io"
	"io/ioutil"
	"mgnes/pkg/ines"
	"mgnes/pkg/mappers"
)

// Load cartridge from io.Reader
func Load(reader io.Reader) (cart *Cartridge, err error) {
	if reader == nil {
		err = errors.New("invalid reader")
		return
	}

	var header *ines.Header
	header, err = ines.NewHeader(reader)
	if header == nil {
		err = errors.New("invalid iNES header")
		return
	}

	if header.Trainer() {
		var discarded int64
		discarded, err = io.CopyN(ioutil.Discard, reader, 512)
		if discarded != 512 {
			err = errors.New("invalid iNES header with trainer flag set")
			return
		}
		if err != nil {
			return
		}
	}

	memPRG := make([]uint8, header.PRGROMSize())
	memCHR := make([]uint8, header.CHRROMSize())

	n := 0
	n, err = reader.Read(memPRG)
	if n != header.PRGROMSize() {
		err = errors.New("invalid PRG data")
		return
	}
	if err != nil {
		return
	}

	n, err = reader.Read(memCHR)
	if n != header.CHRROMSize() {
		err = errors.New("invalid CHR data")
		return
	}
	if err != nil {
		return
	}

	cart = &Cartridge{
		Mirroring:   header.Mirroring(),
		imageValid:  true,
		mapperId:    header.Mapper(),
		numPRGBanks: header.PRG,
		numCHRBanks: header.CHR,
		memPRG:      memPRG,
		memCHR:      memCHR,
		mapper:      mappers.Create(header),
	}

	return
}
