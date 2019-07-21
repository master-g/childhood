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

package nes6502

const (
	// MemoryCapacity the size of memory that a 6502 can address
	MemoryCapacity = 65536

	// FlagNegative N
	FlagNegative uint8 = 0x80
	// FlagOverflow V
	FlagOverflow uint8 = 0x40
	// FlagBreak B
	FlagBreak uint8 = 0x10
	// FlagDecimal D
	FlagDecimal uint8 = 0x08
	// FlagInterrupt I
	FlagInterrupt uint8 = 0x04
	// FlagZero Z
	FlagZero uint8 = 0x02
	// FlagCarry C
	FlagCarry uint8 = 0x01
)

// NES6502 cpu context
type NES6502 struct {
	// A accumulator
	A uint8
	// X index register
	X uint8
	// Y index register
	Y uint8
	// S stack pointer
	S uint8
	// PC program counter
	PC uint16
	// P processor status register
	P uint8

	// memory on chip
	memory [MemoryCapacity]uint8

	// internal flag
	internalFlag uint8
	// interrupt flag
	interrupt bool

	// cpu cycles since reset
	cycles int
	// last executed opcode
	lastOpCode OpCode
}

const (
	internalFlagDirty         = 0x01
	internalFlagWaitInterrupt = 0x02
)

// NewNES6502 returns a NES 6502 cpu instance and sets its initial state to power up
func NewNES6502() *NES6502 {
	cpu := &NES6502{}

	return cpu
}

// PowerUp sets cpu to power up state
func (cpu *NES6502) PowerUp() {
	cpu.A = 0
	cpu.X = 0
	cpu.Y = 0
	cpu.S = 0xFD
	cpu.P = 0x34

	cpu.internalFlag = 0
	cpu.interrupt = false
	cpu.cycles = 0
}
