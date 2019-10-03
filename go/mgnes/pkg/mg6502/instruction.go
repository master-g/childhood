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

package mg6502

// Instruction contains information of a 6502 opcode
type Instruction struct {
	name     string
	op       func(cpu *MG6502) uint8
	am       func(cpu *MG6502) uint8
	cycles   uint8
	addrMode int
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

// Note: Ive started with the two most complicated instructions to emulate, which
// ironically is addition and subtraction! Ive tried to include a detailed
// explanation as to why they are so complex, yet so fundamental. Im also NOT
// going to do this through the explanation of 1 and 2's complement.

// Instruction: Add with Carry In
// Function:    A = A + M + C
// Flags Out:   C, V, N, Z
//
// Explanation:
// The purpose of this function is to add a value to the accumulator and a carry bit. If
// the result is > 255 there is an overflow setting the carry bit. Ths allows you to
// chain together ADC instructions to add numbers larger than 8-bits. This in itself is
// simple, however the 6502 supports the concepts of Negativity/Positivity and Signed Overflow.
//
// 10000100 = 128 + 4 = 132 in normal circumstances, we know this as unsigned and it allows
// us to represent numbers between 0 and 255 (given 8 bits). The 6502 can also interpret
// this word as something else if we assume those 8 bits represent the range -128 to +127,
// i.e. it has become signed.
//
// Since 132 > 127, it effectively wraps around, through -128, to -124. This wraparound is
// called overflow, and this is a useful to know as it indicates that the calculation has
// gone outside the permissable range, and therefore no longer makes numeric sense.
//
// Note the implementation of ADD is the same in binary, this is just about how the numbers
// are represented, so the word 10000100 can be both -124 and 132 depending upon the
// context the programming is using it in. We can prove this!
//
//  10000100 =  132  or  -124
// +00010001 = + 17      + 17
//  ========    ===       ===     See, both are valid additions, but our interpretation of
//  10010101 =  149  or  -107     the context changes the value, not the hardware!
//
// In principle under the -128 to 127 range:
// 10000000 = -128, 11111111 = -1, 00000000 = 0, 00000000 = +1, 01111111 = +127
// therefore negative numbers have the most significant set, positive numbers do not
//
// To assist us, the 6502 can set the overflow flag, if the result of the addition has
// wrapped around. V <- ~(A^M) & A^(A+M+C) :D lol, let's work out why!
//
// Let's suppose we have A = 30, M = 10 and C = 0
//          A = 30 = 00011110
//          M = 10 = 00001010+
//     RESULT = 40 = 00101000
//
// Here we have not gone out of range. The resulting significant bit has not changed.
// So let's make a truth table to understand when overflow has occurred. Here I take
// the MSB of each component, where R is RESULT.
//
// A  M  R | V | A^R | A^M |~(A^M) |
// 0  0  0 | 0 |  0  |  0  |   1   |
// 0  0  1 | 1 |  1  |  0  |   1   |
// 0  1  0 | 0 |  0  |  1  |   0   |
// 0  1  1 | 0 |  1  |  1  |   0   |  so V = ~(A^M) & (A^R)
// 1  0  0 | 0 |  1  |  1  |   0   |
// 1  0  1 | 0 |  0  |  1  |   0   |
// 1  1  0 | 1 |  1  |  0  |   1   |
// 1  1  1 | 0 |  0  |  0  |   1   |
//
// We can see how the above equation calculates V, based on A, M and R. V was chosen
// based on the following hypothesis:
//       Positive Number + Positive Number = Negative Result -> Overflow
//       Negative Number + Negative Number = Positive Result -> Overflow
//       Positive Number + Negative Number = Either Result -> Cannot Overflow
//       Positive Number + Positive Number = Positive Result -> OK! No Overflow
//       Negative Number + Negative Number = Negative Result -> OK! NO Overflow

func opADC(cpu *MG6502) uint8 {
	// fetch the data that we are adding to the accumulator
	cpu.fetch()

	// Add is performed in 16-bit domain for emulation to capture
	// any carry bit, which will exist in bit 8 of the 16-bit word
	cpu.temp = uint16(cpu.A) + uint16(cpu.fetched) + uint16(cpu.GetFlag(FlagCarry))

	// The carry flag out exists in high byte bit 0
	cpu.SetFlag(FlagCarry, cpu.temp > 255)

	// The Zero flag is set if the result is 0
	cpu.SetFlag(FlagZero, (cpu.temp&0x00FF) == 0)

	// The singed Overflow flag is set based on all that up there!
	overflow := (^(uint16(cpu.A) ^ uint16(cpu.fetched)) & (uint16(cpu.A) ^ cpu.temp)) & 0x0080
	cpu.SetFlag(FlagOverflow, overflow != 0)

	// The negative flag is set to the most significant bit of the result
	cpu.SetFlag(FlagNegative, cpu.temp&0x80 != 0)

	// Load the result into the accumulator (it's 8-bit don't forget)
	cpu.A = uint8(cpu.temp & 0x00FF)

	// This instruction has the potential to require an additional clock cycle
	return 1
}

// Instruction: Bitwise Logic AND
// Function: A = A & M
// Flags Out: N, Z
func opAND(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.A &= cpu.fetched
	cpu.SetFlag(FlagZero, cpu.A == 0x00)
	cpu.SetFlag(FlagNegative, cpu.A&0x80 != 0)

	return 1
}

// Instruction: Arithmetic Shift Left
// Function: A = C <- (A << 1) <- 0
// Flags Out: N, Z, C
func opASL(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.fetched) << 1
	cpu.SetFlag(FlagCarry, cpu.temp&0xFF00 > 0)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x00)
	cpu.SetFlag(FlagNegative, cpu.temp&0x80 != 0)

	if cpu.lookup[cpu.opcode].addrMode == AddrModeIMP {
		cpu.A = uint8(cpu.temp & 0x00FF)
	} else {
		cpu.write(cpu.addrAbs, uint8(cpu.temp&0x00FF))
	}

	return 0
}

// Instruction: Branch if Carry Clear
// Function: if C == 0 { pc = address }
func opBCC(cpu *MG6502) uint8 {
	if cpu.GetFlag(FlagCarry) == 0 {
		cpu.cycles++
		cpu.addrAbs = cpu.PC + cpu.addrRel
		if cpu.addrAbs&0xFF00 != cpu.PC&0xFF00 {
			cpu.cycles++
		}

		cpu.PC = cpu.addrAbs
	}
	return 0
}

// Instruction: Branch if Carry Set
// Function: if C == 1 { pc = address }
func opBCS(cpu *MG6502) uint8 {
	if cpu.GetFlag(FlagCarry) == 1 {
		cpu.cycles++
		cpu.addrAbs = cpu.PC + cpu.addrRel

		if cpu.addrAbs&0xFF00 != cpu.PC&0xFF00 {
			cpu.cycles++
		}

		cpu.PC = cpu.addrAbs
	}
	return 0
}

// Instruction: Branch if Equal
// Function: if Z == 1 { pc = address }
func opBEQ(cpu *MG6502) uint8 {
	if cpu.GetFlag(FlagZero) == 1 {
		cpu.cycles++
		cpu.addrAbs = cpu.PC + cpu.addrRel

		if cpu.addrAbs&0xFF00 != cpu.PC&0xFF00 {
			cpu.cycles++
		}

		cpu.PC = cpu.addrAbs
	}
	return 0
}

// Instruction: Bit Test
// Function: Z = mem[addr] & A, S = mem[addr] & 10000000b, V = mem[addr] >> 010000000b
// Flags Out: Z, S, V
func opBIT(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.A & cpu.fetched)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x00)
	cpu.SetFlag(FlagNegative, cpu.fetched&(1<<7) != 0)
	cpu.SetFlag(FlagOverflow, cpu.fetched&(1<<6) != 0)
	return 0
}

// Instruction: Branch if Negative
// Function: if N == 1 { pc = addr }
func opBMI(cpu *MG6502) uint8 {
	if cpu.GetFlag(FlagNegative) == 1 {
		cpu.cycles++
		cpu.addrAbs = cpu.PC + cpu.addrRel

		if cpu.addrAbs&0xFF00 != cpu.PC&0xFF00 {
			cpu.cycles++
		}

		cpu.PC = cpu.addrAbs
	}
	return 0
}

// Instruction: Branch if Not Equal
// Function: if Z == 0 { pc = addr }
func opBNE(cpu *MG6502) uint8 {
	if cpu.GetFlag(FlagZero) == 0 {
		cpu.cycles++
		cpu.addrAbs = cpu.PC + cpu.addrRel
		if cpu.addrAbs&0xFF00 != cpu.PC&0xFF00 {
			cpu.cycles++
		}

		cpu.PC = cpu.addrAbs
	}
	return 0
}

// Instruction: Branch if Positive
// Function: if N == 0 { pc = addr }
func opBPL(cpu *MG6502) uint8 {
	if cpu.GetFlag(FlagNegative) == 0 {
		cpu.cycles++
		cpu.addrAbs = cpu.PC + cpu.addrRel

		if cpu.addrAbs&0xFF00 != cpu.PC&0xFF00 {
			cpu.cycles++
		}

		cpu.PC = cpu.addrAbs
	}
	return 0
}

// Instruction: Break
// Function: Program Sourced Interrupt
func opBRK(cpu *MG6502) uint8 {
	cpu.PC++

	cpu.SetFlag(FlagInterrupt, true)
	cpu.pushPC()

	cpu.SetFlag(FlagBreak, true)
	cpu.push(cpu.FLAG)
	cpu.SetFlag(FlagBreak, false)

	cpu.PC = cpu.read16(0xFFFE)

	return 0
}

// Instruction: Branch if Overflow Clear
// Function: if V == 0 { pc = address }
func opBVC(cpu *MG6502) uint8 {
	if cpu.GetFlag(FlagOverflow) == 0 {
		cpu.cycles++
		cpu.addrAbs = cpu.PC + cpu.addrRel

		if cpu.addrAbs&0xFF00 != cpu.PC&0xFF00 {
			cpu.cycles++
		}

		cpu.PC = cpu.addrAbs
	}
	return 0
}

// Instruction: Branch if Overflow Set
// Function: if V == 1 { pc = address }
func opBVS(cpu *MG6502) uint8 {
	if cpu.GetFlag(FlagOverflow) == 1 {
		cpu.cycles++
		cpu.addrAbs = cpu.PC + cpu.addrRel

		if cpu.addrAbs&0xFF00 != cpu.PC&0xFF00 {
			cpu.cycles++
		}

		cpu.PC = cpu.addrAbs
	}
	return 0
}

// Instruction: Clear Carry Flag
// Function: C = 0
func opCLC(cpu *MG6502) uint8 {
	cpu.SetFlag(FlagCarry, false)
	return 0
}

// Instruction: Clear Decimal Flag
// Function: D = 0
func opCLD(cpu *MG6502) uint8 {
	cpu.SetFlag(FlagDecimal, false)
	return 0
}

// Instruction: Disable Interrupts / Clear Interrupt Flag
func opCLI(cpu *MG6502) uint8 {
	cpu.SetFlag(FlagInterrupt, false)
	return 0
}

// Instruction: Clear Overflow Flag
// Function: V = 0
func opCLV(cpu *MG6502) uint8 {
	cpu.SetFlag(FlagOverflow, false)
	return 0
}

// Instruction: Compare Accumulator
// Function: C <- A >= M	Z <- (A - M) == 0
func opCMP(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.A) - uint16(cpu.fetched)
	cpu.SetFlag(FlagCarry, cpu.A >= cpu.fetched)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x0000)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	return 1
}

// Instruction: Compare X Register
// Function: C <- X >= M	Z <- (X - M) == 0
// Flags Out: N, C, Z
func opCPX(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.X) - uint16(cpu.fetched)
	cpu.SetFlag(FlagCarry, cpu.X >= cpu.fetched)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x0000)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	return 0
}

// Instruction: Compare Y Register
// Function: C <- Y >= M	Z <- (Y - M) == 0
// Flags Out: N, C, Z
func opCPY(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.Y) - uint16(cpu.fetched)
	cpu.SetFlag(FlagCarry, cpu.Y >= cpu.fetched)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x0000)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	return 0
}

// Instruction: Decrement Value at Memory Location
// Function: M = M - 1
// Flags Out: N, Z
func opDEC(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.fetched - 1)
	cpu.write(cpu.addrAbs, uint8(cpu.temp&0x00FF))
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x0000)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	return 0
}

// Instruction: Decrement X Register
// Function: X = X - 1
// Flags Out: N, Z
func opDEX(cpu *MG6502) uint8 {
	cpu.X--
	cpu.SetFlag(FlagZero, cpu.X == 0x00)
	cpu.SetFlag(FlagNegative, cpu.X&0x80 != 0)
	return 0
}

// Instruction: Decrement Y Register
// Function: Y = Y - 1
// Flags Out: N, Z
func opDEY(cpu *MG6502) uint8 {
	cpu.Y--
	cpu.SetFlag(FlagZero, cpu.Y == 0x00)
	cpu.SetFlag(FlagNegative, cpu.Y&0x80 != 0)
	return 0
}

// Instruction: Bitwise Logic XOR
// Function: A = A xor M
// Flags Out: N, Z
func opEOR(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.A ^= cpu.fetched
	cpu.SetFlag(FlagZero, cpu.A == 0x00)
	cpu.SetFlag(FlagNegative, cpu.A&0x80 != 0)
	return 1
}

// Instruction: Increment Value at Memory Location
// Function: M = M + 1
// Flags Out: N, Z
func opINC(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.fetched + 1)
	cpu.write(cpu.addrAbs, uint8(cpu.temp&0x00FF))
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x0000)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	return 0
}

// Instruction: Increment X Register
// Function: X = X + 1
// Flags Out: N, Z
func opINX(cpu *MG6502) uint8 {
	cpu.X++
	cpu.SetFlag(FlagZero, cpu.X == 0x00)
	cpu.SetFlag(FlagNegative, cpu.X&0x80 != 0)
	return 0
}

// Instruction: Increment Y Register
// Function: Y = Y + 1
// Flags Out: N, Z
func opINY(cpu *MG6502) uint8 {
	cpu.Y++
	cpu.SetFlag(FlagZero, cpu.Y == 0x00)
	cpu.SetFlag(FlagNegative, cpu.Y&0x80 != 0)
	return 0
}

// Instruction: Jump to Location
// Function: pc = address
func opJMP(cpu *MG6502) uint8 {
	cpu.PC = cpu.addrAbs
	return 0
}

// Instruction: Jump to Sub-Routine
// Function: Push current pc to stack, pc = address
func opJSR(cpu *MG6502) uint8 {
	cpu.PC--
	cpu.pushPC()
	cpu.PC = cpu.addrAbs
	return 0
}

// Instruction: Load The Accumulator
// Function: A = M
// Flags Out: N, Z
func opLDA(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.A = cpu.fetched
	cpu.SetFlag(FlagZero, cpu.A == 0)
	cpu.SetFlag(FlagNegative, cpu.A&0x80 != 0)
	return 0
}

// Instruction: Load The X Register
// Function: X = M
// Flags Out: N, Z
func opLDX(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.X = cpu.fetched
	cpu.SetFlag(FlagZero, cpu.X == 0)
	cpu.SetFlag(FlagNegative, cpu.X&0x80 != 0)
	return 0
}

// Instruction: Load The Y Register
// Function: Y = M
// Flags Out: N, Z
func opLDY(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.Y = cpu.fetched
	cpu.SetFlag(FlagZero, cpu.Y == 0)
	cpu.SetFlag(FlagNegative, cpu.Y&0x80 != 0)
	return 0
}

// Instruction: Logical Shift Right
// Function: 0 -> addr >> 1 -> C
func opLSR(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.SetFlag(FlagCarry, cpu.fetched&0x01 != 0)
	cpu.temp = uint16(cpu.fetched >> 1)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x0000)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	if cpu.lookup[cpu.opcode].addrMode == AddrModeIMP {
		cpu.A = uint8(cpu.temp & 0x00FF)
	} else {
		cpu.write(cpu.addrAbs, uint8(cpu.temp&0x00FF))
	}
	return 0
}

// Instruction: No Operation
func opNOP(cpu *MG6502) uint8 {
	// Sadly not all NOPs are equal, Ive added a few here
	// based on https://wiki.nesdev.com/w/index.php/CPU_unofficial_opcodes
	// and will add more based on game compatibility, and ultimately
	// I'd like to cover all illegal opcodes too
	switch cpu.opcode {
	case 0x1C, 0x3C, 0x5C, 0x7C, 0xDC, 0xFC:
		return 1
	}
	return 0
}

// Instruction: Bitwise Logic OR
// Function: A = A | M
// Flags Out: N, Z
func opORA(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.A |= cpu.fetched
	cpu.SetFlag(FlagZero, cpu.A == 0x00)
	cpu.SetFlag(FlagNegative, cpu.A&0x80 != 0)
	return 1
}

// Instruction: Push Accumulator to Stack
// Function: A -> stack
func opPHA(cpu *MG6502) uint8 {
	cpu.push(cpu.A)
	return 0
}

// Instruction: Push Status Register to Stack
// Function: status -> stack
// Note: Break flag is set to 1 before push
func opPHP(cpu *MG6502) uint8 {
	cpu.SetFlag(FlagBreak, true)
	cpu.SetFlag(FlagUnused, true)
	cpu.push(cpu.FLAG)
	cpu.SetFlag(FlagBreak, false)
	cpu.SetFlag(FlagUnused, false)
	return 0
}

// Instruction: Pop Accumulator off Stack
// Function: A <- stack
// Flags Out: N, Z
func opPLA(cpu *MG6502) uint8 {
	cpu.A = cpu.pop()
	cpu.SetFlag(FlagZero, cpu.A == 0x00)
	cpu.SetFlag(FlagNegative, cpu.A&0x80 != 0)
	return 0
}

// Instruction: Pop Status Register off Stack
// Function: Status <- stack
func opPLP(cpu *MG6502) uint8 {
	cpu.FLAG = cpu.pop()
	cpu.SetFlag(FlagUnused, true)
	return 0
}

// Instruction: Rotate Left
// Function: C <- address << 1 <- C
// Flags Out: N, Z, C
func opROL(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.fetched<<1) | uint16(cpu.GetFlag(FlagCarry))
	cpu.SetFlag(FlagCarry, cpu.temp&0xFF00 != 0)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x0000)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	if cpu.lookup[cpu.opcode].addrMode == AddrModeIMP {
		cpu.A = uint8(cpu.temp & 0x00FF)
	} else {
		cpu.write(cpu.addrAbs, uint8(cpu.temp&0x00FF))
	}
	return 0
}

// Instruction: Rotate Right
// Function: C -> address >> 1 -> C
func opROR(cpu *MG6502) uint8 {
	cpu.fetch()
	cpu.temp = uint16(cpu.fetched>>1) | uint16(cpu.GetFlag(FlagCarry)<<7)
	cpu.SetFlag(FlagCarry, cpu.fetched&0x01 != 0)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0x00)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	if cpu.lookup[cpu.opcode].addrMode == AddrModeIMP {
		cpu.A = uint8(cpu.temp & 0x00FF)
	} else {
		cpu.write(cpu.addrAbs, uint8(cpu.temp&0x00FF))
	}
	return 0
}

// Instruction: Return from Interrupt
// Function:
func opRTI(cpu *MG6502) uint8 {
	cpu.FLAG = cpu.pop()
	cpu.FLAG &= ^FlagBreak
	cpu.FLAG &= ^FlagUnused

	cpu.popPC()
	return 0
}

// Instruction: Return from Subroutine
func opRTS(cpu *MG6502) uint8 {
	cpu.popPC()
	cpu.PC++
	return 0
}

// Instruction: Subtraction with Borrow In
// Function:    A = A - M - (1 - C)
// Flags Out:   C, V, N, Z
//
// Explanation:
// Given the explanation for ADC above, we can reorganise our data
// to use the same computation for addition, for subtraction by multiplying
// the data by -1, i.e. make it negative
//
// A = A - M - (1 - C)  ->  A = A + -1 * (M - (1 - C))  ->  A = A + (-M + 1 + C)
//
// To make a signed positive number negative, we can invert the bits and add 1
// (OK, I lied, a little bit of 1 and 2s complement :P)
//
//  5 = 00000101
// -5 = 11111010 + 00000001 = 11111011 (or 251 in our 0 to 255 range)
//
// The range is actually unimportant, because if I take the value 15, and add 251
// to it, given we wrap around at 256, the result is 10, so it has effectively
// subtracted 5, which was the original intention. (15 + 251) % 256 = 10
//
// Note that the equation above used (1-C), but this got converted to + 1 + C.
// This means we already have the +1, so all we need to do is invert the bits
// of M, the data(!) therfore we can simply add, exactly the same way we did
// before.
func opSBC(cpu *MG6502) uint8 {
	cpu.fetch()

	// Operating in 16-bit domain to capture carry out

	// We can invert the bottom 8 bit with bitwise xor
	value := uint16(cpu.fetched) ^ 0x00FF

	// Notice this is exactly the same as addition from here
	cpu.temp = uint16(cpu.A) + value + uint16(cpu.GetFlag(FlagCarry))
	cpu.SetFlag(FlagCarry, cpu.temp&0xFF00 != 0)
	cpu.SetFlag(FlagZero, cpu.temp&0x00FF == 0)
	overflow := (cpu.temp ^ uint16(cpu.A)) & ((cpu.temp ^ value) & 0x0080)
	cpu.SetFlag(FlagOverflow, overflow != 0)
	cpu.SetFlag(FlagNegative, cpu.temp&0x0080 != 0)
	cpu.A = uint8(cpu.temp & 0x00FF)

	return 1
}

// Instruction: Set Carry Flag
// Function: C = 1
func opSEC(cpu *MG6502) uint8 {
	cpu.SetFlag(FlagCarry, true)
	return 0
}

// Instruction: Set Decimal Flag
// Function: D = 1
func opSED(cpu *MG6502) uint8 {
	cpu.SetFlag(FlagDecimal, true)
	return 0
}

// Instruction: Set Interrupt Flag / Enable Interrupts
// Function: I = 1
func opSEI(cpu *MG6502) uint8 {
	cpu.SetFlag(FlagInterrupt, true)
	return 0
}

// Instruction: Store Accumulator at Address
// Function: M = A
func opSTA(cpu *MG6502) uint8 {
	cpu.write(cpu.addrAbs, cpu.A)
	return 0
}

// Instruction: Store X Register at Address
// Function: M = X
func opSTX(cpu *MG6502) uint8 {
	cpu.write(cpu.addrAbs, cpu.X)
	return 0
}

// Instruction: Store Y Register at Address
// Function: M = Y
func opSTY(cpu *MG6502) uint8 {
	cpu.write(cpu.addrAbs, cpu.Y)
	return 0
}

// Instruction: Transfer Accumulator to X Register
// Function: X = A
// Flags Out: N, Z
func opTAX(cpu *MG6502) uint8 {
	cpu.X = cpu.A
	cpu.SetFlag(FlagZero, cpu.X == 0x00)
	cpu.SetFlag(FlagNegative, cpu.X&0x80 != 0)
	return 0
}

// Instruction: Transfer Accumulator to Y Register
// Function: Y = A
// Flags Out: N, Z
func opTAY(cpu *MG6502) uint8 {
	cpu.Y = cpu.A
	cpu.SetFlag(FlagZero, cpu.Y == 0x00)
	cpu.SetFlag(FlagNegative, cpu.Y&0x80 != 0)
	return 0
}

// Instruction: Transfer Stack Pointer to X Register
// Function: X = stack pointer
func opTSX(cpu *MG6502) uint8 {
	cpu.X = cpu.SP
	cpu.SetFlag(FlagZero, cpu.X == 0x00)
	cpu.SetFlag(FlagNegative, cpu.X&0x80 != 0)
	return 0
}

// Instruction: Transfer X Register to Accumulator
// Function: A = X
// Flags Out: N, Z
func opTXA(cpu *MG6502) uint8 {
	cpu.A = cpu.X
	cpu.SetFlag(FlagZero, cpu.A == 0x00)
	cpu.SetFlag(FlagNegative, cpu.A&0x80 != 0)
	return 0
}

// Instruction: Transfer X Register to Stack Pointer
// Function: stack pointer = X
func opTXS(cpu *MG6502) uint8 {
	cpu.SP = cpu.X
	return 0
}

// Instruction: Transfer Y Register to Accumulator
// Function: A = Y
// Flags Out: N, Z
func opTYA(cpu *MG6502) uint8 {
	cpu.A = cpu.Y
	cpu.SetFlag(FlagZero, cpu.A == 0x00)
	cpu.SetFlag(FlagNegative, cpu.A&0x80 != 0)
	return 0
}

// capture all "unofficial" opcodes with this function.
// It is functionally identical to a NOP
func opXXX(cpu *MG6502) uint8 {
	_ = cpu
	return 0
}
