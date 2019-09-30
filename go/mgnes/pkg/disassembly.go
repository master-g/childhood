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

import (
	"strings"
)

// Disassembly represents disassembly of an 6502 instruction context
type Disassembly struct {
	// Index contains address list
	Index []uint16
	// Op maps addr to opcode name
	Op map[uint16]string
	// Desc maps addr to opcode addressing mode
	Desc map[uint16]string
}

// String implementation
func (d *Disassembly) Stringify(addr uint16, length int) string {
	op := d.Op[addr]
	desc := d.Desc[addr]

	sb := &strings.Builder{}
	sb.WriteString(op)
	if sb.Len()+len(desc) > length {
		sb.WriteRune(' ')
	} else {
		for sb.Len()+len(desc) < length {
			sb.WriteRune(' ')
		}
	}

	sb.WriteString(desc)

	return sb.String()
}
