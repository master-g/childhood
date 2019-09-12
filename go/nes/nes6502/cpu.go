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

// NES6502 cpu context
type NES6502 struct {
	registers Registers // registers

	// memory on chip
	memory Memory

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
	cpu.registers.PowerUp()
	cpu.memory.reset()

	cpu.internalFlag = 0
	cpu.interrupt = false
	cpu.cycles = 0
}