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

package mgnes

const (
	// MemoryCapacity the size of memory that a 6502 cpu can address
	MemoryCapacity = 65536
)

// Memory interface definition
type Memory interface {
	Reset()
	Read(addr uint16) (value uint8)
	Write(addr uint16, value uint8) (oldValue uint8)
}

// PlainMemory 64KB of plain bytes
type PlainMemory [MemoryCapacity]uint8

// NewPlainMemory create and returns a plain memory reference
func NewPlainMemory() *PlainMemory {
	mem := &PlainMemory{}
	mem.Reset()
	return mem
}

func (m *PlainMemory) Reset() {
	for i := 0; i < len(m); i++ {
		m[i] = 0xFF
	}
}

func (m *PlainMemory) Read(addr uint16) (value uint8) {
	return m[int(addr)%MemoryCapacity]
}

func (m *PlainMemory) Write(addr uint16, value uint8) (oldValue uint8) {
	oldValue = m[int(addr)%MemoryCapacity]
	m[int(addr)%MemoryCapacity] = value

	return
}
