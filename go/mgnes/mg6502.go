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

import (
	"fmt"
	"strings"
)

const (
	// FlagNegative N
	FlagNegative uint8 = 0x80
	// FlagOverflow V
	FlagOverflow uint8 = 0x40
	// FlagUnused U
	FlagUnused uint8 = 0x20
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

	// Addressing Mode Unknown
	AddrModeUnknown = iota
	// Addressing Mode Implied
	AddrModeIMP
	// Addressing Mode Immediate
	AddrModeIMM
	// Addressing Mode Zero Page
	AddrModeZP0
	// Addressing Mode Zero Page with X Offset
	AddrModeZPX
	// Addressing Mode Zero Page with Y Offset
	AddrModeZPY
	// Addressing Mode Relative
	AddrModeREL
	// Addressing Mode Absolute
	AddrModeABS
	// Addressing Mode Absolute with X Offset
	AddrModeABX
	// Addressing Mode Absolute with Y Offset
	AddrModeABY
	// Addressing Mode Indirect
	AddrModeIND
	// Addressing Mode Indirect X
	AddrModeIZX
	// Addressing Mode Indirect Y
	AddrModeIZY
)

// MG6502 emulates a 6502 cpu from software perspective
type MG6502 struct {
	// registers

	// A accumulator
	A uint8
	// X register
	X uint8
	// Y register
	Y uint8
	// Stack pointer register
	SP uint8
	// Program counter register
	PC uint16
	// Flag status register
	FLAG uint8

	// bus
	bus *Bus

	// assistive variables
	fetched    uint8  // Represents the working input value to the ALU
	temp       uint16 // A convenience variable used everywhere
	addrAbs    uint16 // All used memory addresses end up in here
	addrRel    uint16 // Represents absolute address following a branch
	opcode     uint8  // Instruction byte
	cycles     uint8  // How many cycles the instruction has remaining
	clockCount uint32 // Global accumulation of the number of clocks

	lookup []*Instruction
}

// NewMG6502 creates and return a 6502 cpu reference
func NewMG6502() *MG6502 {
	cpu := &MG6502{
		A:          0,
		X:          0,
		Y:          0,
		SP:         0,
		PC:         0,
		FLAG:       0,
		bus:        nil,
		fetched:    0,
		temp:       0,
		addrAbs:    0,
		addrRel:    0,
		opcode:     0,
		cycles:     0,
		clockCount: 0,
		lookup:     newInstructionSet(),
	}

	return cpu
}

// Reset interrupt
// Force the 6502 into a known state. This is hard-wired inside the CPU.
// The registers are set to 0x00, the status register is cleared except for unused
// bit which remains at 1. An absolute address is read from location 0xFFFC
// which contains a second address that the program counter is set to. This
// allows the programmer to jump to a known and programmable location in the memory
// to start executing from. Typically the programmer would set the value at location
// 0xFFFC at compile time
func (cpu *MG6502) Reset() {
	// get interrupt vector
	cpu.PC = cpu.read16(0xFFFC)

	// clear register
	cpu.A = 0
	cpu.X = 0
	cpu.Y = 0
	cpu.SP = 0xFD
	cpu.FLAG = 0x00 | FlagUnused

	// clear internal stuff
	cpu.addrRel = 0
	cpu.addrAbs = 0
	cpu.fetched = 0

	// reset op time
	cpu.cycles = 8
}

// IRQ Interrupt Request
// Interrupt requests are a complex operation and only happen if the
// "disable interrupt" flag is unset. IRQs can happen at any time, but
// you don't want them to be destructive to the operation of the running
// program. Therefore the current instruction is allowed to finish and then
// the current program counter is stored on the stack. When the routine
// that services the interrupt has finished, the status register and
// program counter can be restored to how they where before it occurred.
// This is implemented by the "RTI" instruction. Once the IRQ has happened,
// in a similar way to a reset, a programmable address is read from hard coded
// location 0xFFFE, which is subsequently set to the program counter.
func (cpu *MG6502) IRQ() {
	// check interrupt disable flag
	if cpu.GetFlag(FlagInterrupt) != 0 {
		return
	}

	// push the program counter to the stack
	cpu.pushPC()

	// push status register
	cpu.SetFlag(FlagBreak, false)
	cpu.SetFlag(FlagUnused, true)
	cpu.SetFlag(FlagInterrupt, true)
	cpu.push(cpu.FLAG)

	// read new program counter vector
	cpu.PC = cpu.read16(0xFFFE)

	// IRQs take time
	cpu.cycles = 7
}

// NMI Non-Maskable Interrupt
// A non-maskable interrupt cannot be ignored. It behaves in exactly the
// same way as a regular IRQ, but reads the new program counter address
// form location 0xFFFA
func (cpu *MG6502) NMI() {
	cpu.pushPC()

	cpu.SetFlag(FlagBreak, false)
	cpu.SetFlag(FlagUnused, true)
	cpu.SetFlag(FlagInterrupt, true)
	cpu.push(cpu.FLAG)

	cpu.PC = cpu.read16(0xFFFA)

	cpu.cycles = 8
}

// Clock perform a clock cycle
func (cpu *MG6502) Clock() {
	if cpu.cycles == 0 {
		cpu.opcode = cpu.read(cpu.PC)

		instruction := cpu.lookup[cpu.opcode]

		logPC := cpu.PC

		// always set the unused flag to 1
		cpu.SetFlag(FlagUnused, true)
		// increment PC since we read the opcode
		cpu.PC++
		// get instruction cycle cost
		cpu.cycles = instruction.cycles
		// perform fetch of immediate data using the required addressing mode
		addressingCycles := instruction.am(cpu)
		// perform opcode
		executionCycles := instruction.op(cpu)

		// the address mode and opcode may altered the number of cycles
		// this instruction requires before its completed
		cpu.cycles += addressingCycles & executionCycles

		// always set the unused flag to 1
		cpu.SetFlag(FlagUnused, true)

		if logEnable {
			flagString := "NVUBDIZC"
			flagValues := []uint8{FlagNegative, FlagOverflow, FlagUnused, FlagBreak, FlagDecimal, FlagInterrupt, FlagZero, FlagCarry}

			sb := &strings.Builder{}
			for i, c := range flagString {
				if cpu.GetFlag(flagValues[i]) != 0 {
					sb.WriteRune(c)
				} else {
					sb.WriteRune('.')
				}
			}

			logger.Log(fmt.Sprintf("%10d:%02d PC:%04X %s A:%02X X:%02X Y:%02X %s STKP:%02X",
				cpu.clockCount, 0, logPC, "XXX", cpu.A, cpu.X, cpu.Y,
				sb.String(), cpu.SP))
		}
	}

	// use for logging
	cpu.clockCount++

	// decrement the number of cycles remaining for current instruction
	cpu.cycles--
}

// Complete indicate the current instruction has completed by returning true.
// This is a utility function to enable "step-by-step" execution, without manually
// clocking every cycle
func (cpu *MG6502) Complete() bool {
	return cpu.cycles == 0
}

// Attach CPU to bus
func (cpu *MG6502) Attach(bus *Bus) {
	cpu.bus = bus
}

// Disassemble a range of memory, with keys equivalent to instruction start
// locations in memory
// This is the disassembly function. Its workings are not required for emulation.
// It is merely a convenience function to turn the binary instruction code into
// human readable form. Its included as part of the emulator because it can take
// advantage of many of the CPUs internal operations to do this.
func (cpu *MG6502) Disassemble(start, end uint16) *Disassembly {
	addr := uint32(start)
	var value, lo, hi uint8
	var lineAddr uint16
	disassembly := &Disassembly{
		Index: []uint16{},
		Lines: make(map[uint16]string),
	}

	hex := func(n uint32, d uint8) []byte {
		s := []byte{'0', '0', '0', '0'}
		for i := d - 1; i != 0; i-- {
			s[i] = "0123456789ABCDEF"[n&0xF]
			n >>= 4
		}
		return s
	}

	// Starting at the specified address we read an instruction
	// byte, which in turn yields information from the lookup table
	// as to how many additional bytes we need to read and what the
	// addressing mode is. I need this info to assemble human readable
	// syntax, which is different depending upon the addressing mode

	// As the instruction is decoded, a std::string is assembled
	// with the readable output
	for addr <= uint32(end) {
		lineAddr = uint16(addr)

		sb := &strings.Builder{}
		// Prefix line with instruction address
		sb.WriteRune('$')
		sb.Write(hex(addr, 4))
		sb.WriteString(": ")

		// Read instruction, and get its mnemonic name
		opcode := cpu.bus.Read(uint16(addr), true)
		addr++
		sb.WriteString(cpu.lookup[opcode].name)
		sb.WriteRune(' ')

		// Get oprands from desired locations, and form the
		// instruction based upon its addressing mode. These
		// routines mimic the actual fetch routine of the
		// 6502 in order to get accurate data as part of the
		// instruction
		switch cpu.lookup[opcode].addrMode {
		case AddrModeIMP:
			sb.WriteString(" {IMP}")
		case AddrModeIMM:
			value = cpu.bus.Read(uint16(addr), true)
			addr++
			sb.WriteString("#$")
			sb.Write(hex(uint32(value), 2))
			sb.WriteString(" {IMM}")
		case AddrModeZP0:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = 0x00
			sb.WriteRune('$')
			sb.Write(hex(uint32(lo), 2))
			sb.WriteString(" {ZP0}")
		case AddrModeZPX:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = 0x00
			sb.WriteRune('$')
			sb.Write(hex(uint32(lo), 2))
			sb.WriteString(", X {ZPX}")
		case AddrModeZPY:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = 0x00
			sb.WriteRune('$')
			sb.Write(hex(uint32(lo), 2))
			sb.WriteString(", Y {ZPY}")
		case AddrModeIZX:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = 0x00
			sb.WriteString("($")
			sb.Write(hex(uint32(lo), 2))
			sb.WriteString(", X) {IZX}")
		case AddrModeIZY:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = 0x00
			sb.WriteString("($")
			sb.Write(hex(uint32(lo), 2))
			sb.WriteString(", Y) {IZY}")
		case AddrModeABS:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = cpu.bus.Read(uint16(addr), true)
			addr++
			sb.WriteRune('$')
			sb.Write(hex(uint32(hi)<<8|uint32(lo), 4))
			sb.WriteString(" {ABS}")
		case AddrModeABX:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = cpu.bus.Read(uint16(addr), true)
			addr++
			sb.WriteRune('$')
			sb.Write(hex(uint32(hi)<<8|uint32(lo), 4))
			sb.WriteString(", X {ABX}")
		case AddrModeABY:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = cpu.bus.Read(uint16(addr), true)
			addr++
			sb.WriteRune('$')
			sb.Write(hex(uint32(hi)<<8|uint32(lo), 4))
			sb.WriteString(", Y {ABY}")
		case AddrModeIND:
			lo = cpu.bus.Read(uint16(addr), true)
			addr++
			hi = cpu.bus.Read(uint16(addr), true)
			addr++
			sb.WriteString("($")
			sb.Write(hex(uint32(hi)<<8|uint32(lo), 4))
			sb.WriteString(") {IND}")
		case AddrModeREL:
			value = cpu.bus.Read(uint16(addr), true)
			addr++
			sb.WriteRune('$')
			sb.Write(hex(uint32(value), 2))
			sb.WriteString(" [$")
			sb.Write(hex(addr+uint32(value), 4))
			sb.WriteString("] {REL}")
		}

		disassembly.Index = append(disassembly.Index, lineAddr)
		disassembly.Lines[lineAddr] = sb.String()
	}

	return disassembly
}

// GetFlag returns the flag
func (cpu *MG6502) GetFlag(flag uint8) uint8 {
	if cpu.FLAG&flag > 0 {
		return 1
	} else {
		return 0
	}
}

// SetFlag sets the flag
func (cpu *MG6502) SetFlag(flag uint8, v bool) {
	if v {
		cpu.FLAG |= flag
	} else {
		cpu.FLAG &^= flag
	}
}

// push data byte to stack
func (cpu *MG6502) push(data uint8) {
	cpu.write(0x0100+uint16(cpu.SP), data)
	cpu.SP--
}

// pop data from stack
func (cpu *MG6502) pop() uint8 {
	cpu.SP++
	return cpu.read(0x0100 + uint16(cpu.SP))
}

// push program counter to the stack
func (cpu *MG6502) pushPC() {
	cpu.write(0x0100+uint16(cpu.SP), uint8((cpu.PC>>8)&0x00FF))
	cpu.SP--
	cpu.write(0x0100+uint16(cpu.SP), uint8(cpu.PC&0x00FF))
	cpu.SP--
}

// pop program counter from the stack
func (cpu *MG6502) popPC() {
	cpu.SP++
	cpu.PC = cpu.read16(0x0100 + uint16(cpu.SP))
	cpu.SP++
}

// communication with bus

// reads an 8-bit data from the bus, located at the specified 16-bit address
func (cpu *MG6502) read(addr uint16) uint8 {
	// In normal operation "read only" is set to false. This may seem odd. Some
	// devices on the bus may change state when they are read from, and this
	// is intentional under normal circumstances. However the disassembler will
	// want to read the data at an address without changing the state of the
	// devices on the bus
	return cpu.bus.Read(addr, false)
}

// read a 16-bit data from the bus, the lower 8-bit is read first
func (cpu *MG6502) read16(addr uint16) uint16 {
	var lo, hi uint16
	lo = uint16(cpu.read(addr))
	hi = uint16(cpu.read(addr + 1))
	return hi<<8 | lo
}

// writes a byte to the bus at the specified address
func (cpu *MG6502) write(addr uint16, data uint8) {
	cpu.bus.Write(addr, data)
}

// This function sources the data used by the instruction into
// a convenient numeric variable. Some instructions dont have to
// fetch data as the source is implied by the instruction. For example
// "INX" increments the X register. There is no additional data
// required. For all other addressing modes, the data resides at
// the location held within addr_abs, so it is read from there.
// Immediate address mode exploits this slightly, as that has
// set addr_abs = pc + 1, so it fetches the data from the
// next byte for example "LDA $FF" just loads the accumulator with
// 256, i.e. no far reaching memory fetch is required. "fetched"
// is a variable global to the CPU, and is set by calling this
// function. It also returns it for convenience.
func (cpu *MG6502) fetch() uint8 {
	if cpu.lookup[cpu.opcode].addrMode != AddrModeIMP {
		cpu.fetched = cpu.read(cpu.addrAbs)
	}
	return cpu.fetched
}
