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

func amIMP(cpu *MG6502) uint8 {
	return 0
}

func amIMM(cpu *MG6502) uint8 {
	return 0
}

func amZP0(cpu *MG6502) uint8 {
	return 0
}

func amZPX(cpu *MG6502) uint8 {
	return 0
}

func amZPY(cpu *MG6502) uint8 {
	return 0
}

func amREL(cpu *MG6502) uint8 {
	return 0
}

func amABS(cpu *MG6502) uint8 {
	return 0
}

func amABX(cpu *MG6502) uint8 {
	return 0
}

func amABY(cpu *MG6502) uint8 {
	return 0
}

func amIND(cpu *MG6502) uint8 {
	return 0
}

func amIZX(cpu *MG6502) uint8 {
	return 0
}

func amIZY(cpu *MG6502) uint8 {
	return 0
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
