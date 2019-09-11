// Copyright © 2019 ${<OWNER>}
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

type Instruction struct {
	name   string
	op     func(cpu *MG6502) uint8
	am     func(cpu *MG6502) uint8
	cycles uint8
}

// Addressing modes ===========================================================
// The 6502 has a variety of addressing modes to access data in memory, some of
// which are direct and some are indirect. Each opcode contains information
// about which address mode should be employed to facilitate the instruction,
// in regards to where it reads/writes the data it uses. The address mode
// changes the number of bytes that makes up the full instruction, to make sure
// the program counter is at the correct location, the instruction is primed
// with the addresses it needs, and the number of clock cycles the instruction
// requires is calculated. These functions may adjust the number of cycles
// required depending upon where and how the memory is accessed, so they return
// the required adjustment

// Address Mode: Implied
// There is no additional data required for this instruction. The instruction
// does something very simple like sets a status bit. However, we will
// target the accumulator, for instructions like PHA
func amIMP(cpu *MG6502) uint8 {
	cpu.fetched = cpu.A
	return 0
}

// Address Mode: Immediate
// The instruction expects the next byte to be used as a value, so we'll prep
// the read address to point to the next byte
func amIMM(cpu *MG6502) uint8 {
	cpu.addrAbs = cpu.PC
	cpu.PC++
	return 0
}

// Address Mode: Zero Page
// To save program bytes, zero page addressing allows you to absolutely address
// a location in first 0xFF bytes of address range. Clearly this only requires
// one byte instead of the usual two.
func amZP0(cpu *MG6502) uint8 {
	cpu.addrAbs = uint16(cpu.read(cpu.PC))
	cpu.PC++
	cpu.addrAbs &= 0x00FF
	return 0
}

// Address Mode: Zero Page with X Offset
// Fundamentally the same as Zero Page addressing, but the contents of the X Register
// is added to the supplied single byte address. This is useful for iterating through
// ranges within the first page.
func amZPX(cpu *MG6502) uint8 {
	cpu.addrAbs = uint16(cpu.read(cpu.PC) + cpu.X)
	cpu.PC++
	cpu.addrAbs &= 0x00FF
	return 0
}

// Address Mode: Zero Page with Y Offset
// Same as above but uses Y Register for offset
func amZPY(cpu *MG6502) uint8 {
	cpu.addrAbs = uint16(cpu.read(cpu.PC) + cpu.Y)
	cpu.PC++
	cpu.addrAbs &= 0x00FF
	return 0
}

// Address Mode: Relative
// This address mode is exclusive to branch instructions. The address
// must reside within -128 to +127 of the branch instruction, i.e.
// you can't directly branch to any address in the addressable range.
func amREL(cpu *MG6502) uint8 {
	cpu.addrRel = uint16(cpu.read(cpu.PC))
	cpu.PC++
	if cpu.addrRel&0x80 > 0 {
		cpu.addrRel |= 0xFF00
	}
	return 0
}

// Address Mode: Absolute
// A full 16-bit address is loaded and used
func amABS(cpu *MG6502) uint8 {
	cpu.addrAbs = cpu.read16(cpu.PC)
	cpu.PC += 2
	return 0
}

// Address Mode: Absolute with X Offset
// Fundamentally the same as absolute addressing, but the contents of the X Register
// is added to the supplied two byte address. If the resulting address changes
// the page, an additional clock cycle is required
func amABX(cpu *MG6502) uint8 {
	addr := cpu.read16(cpu.PC)
	cpu.PC += 2
	cpu.addrAbs = addr
	cpu.addrAbs += uint16(cpu.X)

	if cpu.addrAbs&0xFF00 != addr&0xFF00 {
		// page changed
		return 1
	} else {
		return 0
	}
}

// Address Mode: Absolute with Y Offset
// Fundamentally the same as absolute addressing, but the contents of the Y Register
// is added to the supplied two byte address. If the resulting address changes
// the page, an additional clock cycle is required
func amABY(cpu *MG6502) uint8 {
	addr := cpu.read16(cpu.PC)
	cpu.PC += 2
	cpu.addrAbs += uint16(cpu.Y)

	if cpu.addrAbs&0xFF00 != addr&0xFF00 {
		// page changed
		return 1
	} else {
		return 0
	}
}

// Note: The next 3 address modes use indirection (aka Pointers)

// Address Mode: Indirect
// The supplied 16-bit address is read to get the actual 16-bit address. This is
// instruction is unusual in that it has a bug in the hardware! To emulate its
// function accurately, we also need to emulate this bug. If the low byte of the
// supplied address is 0xFF, then to read the high byte of the actual address
// we need to cross a page boundary. This doesn't actually work on the chip as
// designed, instead it wraps back around in the same page, yielding an invalid
// actual address
func amIND(cpu *MG6502) uint8 {
	var ptrLo, ptrHi, ptr uint16
	ptrLo = uint16(cpu.read(cpu.PC))
	cpu.PC++
	ptrHi = uint16(cpu.read(cpu.PC))
	cpu.PC++

	ptr = (ptrHi << 8) | ptrLo

	if ptrLo == 0x00FF {
		// simulate page boundary hardware bug
		cpu.addrAbs = uint16(cpu.read(ptr&0xFF00))<<8 | uint16(cpu.read(ptr+0))
	} else {
		cpu.addrAbs = uint16(cpu.read(ptr+1))<<8 | uint16(cpu.read(ptr+0))
	}

	return 0
}

// Address Mode: Indirect X
// The supplied 8-bit address is offset by X Register to index
// a location in page 0x00. The actual 16-bit address is read from this location
func amIZX(cpu *MG6502) uint8 {
	t := uint16(cpu.read(cpu.PC))
	cpu.PC++

	lo := uint16(cpu.read((t + uint16(cpu.X)) & 0x00FF))
	hi := uint16(cpu.read((t + uint16(cpu.X) + 1) & 0x00FF))

	cpu.addrAbs = (hi << 8) | lo

	return 0
}

// Address Mode: Indirect Y
// The supplied 8-bit address indexes a location in page 0x00. From
// here the actual 16-bit address is read, and the contents of Y
// Register is added to it to offset it. If the offset causes a change
// in page then an additional clock cycle is required
func amIZY(cpu *MG6502) uint8 {
	t := uint16(cpu.read(cpu.PC))
	cpu.PC++

	lo := uint16(cpu.read(t & 0x00FF))
	hi := uint16(cpu.read((t + 1) & 0x00FF))

	cpu.addrAbs = (hi << 8) | lo
	cpu.addrAbs += uint16(cpu.Y)

	if cpu.addrAbs&0xFF00 != (hi << 8) {
		return 1
	} else {
		return 0
	}
}

// Opcodes =====================================================================
// There are 56 "legitimate" opcodes provided by the 6502 CPU. I have not
// modelled "unofficial" opcodes. As each opcode is defined by 1 byte, there are
// potentially 256 possible codes. Codes are not used in a "switch case" style
// on a processor, instead they are responsible for switching individual parts
// of CPU circuits on and off. The opcodes listed here are official, meaning
// that the functionality of the chip when provided with these codes is as the
// developers intended it to be. Unofficial codes will of course also influence
// the CPU circuitry in interesting ways, and can be exploited to gain
// additional functionality!
//
// These functions return 0 normally, but some are capable of requiring more
// clock cycles when executed under certain conditions combined with certain
// addressing modes. If that is the case, they return 1.
//
// I have included detailed explanations of each function in the class
// implementation file. Note they are listed in alphabetical order here for ease
// of finding.

func opADC(cpu *MG6502) uint8 {
	return 0
}

func opAND(cpu *MG6502) uint8 {
	return 0
}

func opASL(cpu *MG6502) uint8 {
	return 0
}

func opBCC(cpu *MG6502) uint8 {
	return 0
}

func opBCS(cpu *MG6502) uint8 {
	return 0
}

func opBEQ(cpu *MG6502) uint8 {
	return 0
}

func opBIT(cpu *MG6502) uint8 {
	return 0
}

func opBMI(cpu *MG6502) uint8 {
	return 0
}

func opBNE(cpu *MG6502) uint8 {
	return 0
}

func opBPL(cpu *MG6502) uint8 {
	return 0
}

func opBRK(cpu *MG6502) uint8 {
	return 0
}

func opBVC(cpu *MG6502) uint8 {
	return 0
}

func opBVS(cpu *MG6502) uint8 {
	return 0
}

func opCLC(cpu *MG6502) uint8 {
	return 0
}

func opCLD(cpu *MG6502) uint8 {
	return 0
}

func opCLI(cpu *MG6502) uint8 {
	return 0
}

func opCLV(cpu *MG6502) uint8 {
	return 0
}

func opCMP(cpu *MG6502) uint8 {
	return 0
}

func opCPX(cpu *MG6502) uint8 {
	return 0
}

func opCPY(cpu *MG6502) uint8 {
	return 0
}

func opDEC(cpu *MG6502) uint8 {
	return 0
}

func opDEX(cpu *MG6502) uint8 {
	return 0
}

func opDEY(cpu *MG6502) uint8 {
	return 0
}

func opEOR(cpu *MG6502) uint8 {
	return 0
}

func opINC(cpu *MG6502) uint8 {
	return 0
}

func opINX(cpu *MG6502) uint8 {
	return 0
}

func opINY(cpu *MG6502) uint8 {
	return 0
}

func opJMP(cpu *MG6502) uint8 {
	return 0
}

func opJSR(cpu *MG6502) uint8 {
	return 0
}

func opLDA(cpu *MG6502) uint8 {
	return 0
}

func opLDX(cpu *MG6502) uint8 {
	return 0
}

func opLDY(cpu *MG6502) uint8 {
	return 0
}

func opLSR(cpu *MG6502) uint8 {
	return 0
}

func opNOP(cpu *MG6502) uint8 {
	return 0
}

func opORA(cpu *MG6502) uint8 {
	return 0
}

func opPHA(cpu *MG6502) uint8 {
	return 0
}

func opPHP(cpu *MG6502) uint8 {
	return 0
}

func opPLA(cpu *MG6502) uint8 {
	return 0
}

func opPLP(cpu *MG6502) uint8 {
	return 0
}

func opROL(cpu *MG6502) uint8 {
	return 0
}

func opROR(cpu *MG6502) uint8 {
	return 0
}

func opRTI(cpu *MG6502) uint8 {
	return 0
}

func opRTS(cpu *MG6502) uint8 {
	return 0
}

func opSBC(cpu *MG6502) uint8 {
	return 0
}

func opSEC(cpu *MG6502) uint8 {
	return 0
}

func opSED(cpu *MG6502) uint8 {
	return 0
}

func opSEI(cpu *MG6502) uint8 {
	return 0
}

func opSTA(cpu *MG6502) uint8 {
	return 0
}

func opSTX(cpu *MG6502) uint8 {
	return 0
}

func opSTY(cpu *MG6502) uint8 {
	return 0
}

func opTAX(cpu *MG6502) uint8 {
	return 0
}

func opTAY(cpu *MG6502) uint8 {
	return 0
}

func opTSX(cpu *MG6502) uint8 {
	return 0
}

func opTXA(cpu *MG6502) uint8 {
	return 0
}

func opTXS(cpu *MG6502) uint8 {
	return 0
}

func opTYA(cpu *MG6502) uint8 {
	return 0
}

// capture all "unofficial" opcodes with this function.
// It is functionally identical to a NOP
func opXXX(cpu *MG6502) uint8 {
	return 0
}
