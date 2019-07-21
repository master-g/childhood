// Copyright © 2019 MG
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

// OpCode represent an opcode for MOS 6502
type OpCode uint8

const (
	OpMnemonicADC = "ADC"
	OpMnemonicAND = "AND"
	OpMnemonicASL = "ASL"
	OpMnemonicBCC = "BCC"
	OpMnemonicBCS = "BCS"
	OpMnemonicBEQ = "BEQ"
	OpMnemonicBIT = "BIT"
	OpMnemonicBMI = "BMI"
	OpMnemonicBNE = "BNE"
	OpMnemonicBPL = "BPL"
	OpMnemonicBRK = "BRK"
	OpMnemonicBVC = "BVC"
	OpMnemonicBVS = "BVS"
	OpMnemonicCLC = "CLC"
	OpMnemonicCLD = "CLD"
	OpMnemonicCLI = "CLI"
	OpMnemonicCLV = "CLV"
	OpMnemonicCMP = "CMP"
	OpMnemonicCPX = "CPX"
	OpMnemonicCPY = "CPY"
	OpMnemonicDEC = "DEC"
	OpMnemonicDEX = "DEX"
	OpMnemonicDEY = "DEY"
	OpMnemonicEOR = "EOR"
	OpMnemonicINC = "INC"
	OpMnemonicINX = "INX"
	OpMnemonicINY = "INY"
	OpMnemonicJMP = "JMP"
	OpMnemonicJSR = "JSR"
	OpMnemonicLDA = "LDA"
	OpMnemonicLDX = "LDX"
	OpMnemonicLDY = "LDY"
	OpMnemonicLSR = "LSR"
	OpMnemonicNOP = "NOP"
	OpMnemonicORA = "ORA"
	OpMnemonicPHA = "PHA"
	OpMnemonicPHP = "PHP"
	OpMnemonicPLA = "PLA"
	OpMnemonicPLP = "PLP"
	OpMnemonicROL = "ROL"
	OpMnemonicROR = "ROR"
	OpMnemonicRTI = "RTI"
	OpMnemonicRTS = "RTS"
	OpMnemonicSBC = "SBC"
	OpMnemonicSEC = "SEC"
	OpMnemonicSED = "SED"
	OpMnemonicSEI = "SEI"
	OpMnemonicSTA = "STA"
	OpMnemonicSTX = "STX"
	OpMnemonicSTY = "STY"
	OpMnemonicTAX = "TAX"
	OpMnemonicTAY = "TAY"
	OpMnemonicTSX = "TSX"
	OpMnemonicTXA = "TXA"
	OpMnemonicTXS = "TXS"
	OpMnemonicTYA = "TYA"
	OpMnemonicERR = "ERR"

	// opcode might have multiple addressing mode, except implicit mode, opcode
	// with below addressing mode will have postfix correspondingly
	//
	// ABS: absolute mode, next two bytes are the low/high byte of aan absolute
	// 		memory address
	// ABX: absolute, X, next two bytes are added to the value in register X to
	// 		get the memory address
	// ABY: absolute, Y, same as ABX, except the value of register is used
	// 		instead of X
	// ACC: accumulator, act on the value in the accumulator
	// IMM: immediate, next byte is a constant to be used instead of a lookup
	// IND: indirect, next two bytes are an absolute memory address of the
	// 	 	lower nibble of a memory address. That byte and the byte after will
	// 		be loaded and the address made of those two bytes will be used
	// INX: indirect, X, add X to the following byte, modulo 0xFF, and
	//		lookup two bytes starting at that location. Those two bytes from
	//		the memory address will be used
	// INY: indirect, Y, look up two bytes starting at address in the following
	// 		byte, and Y modulo 0xFFFF, and use the result as an address
	// REL: relative, next byte contains a signed offset fro the current PC.
	// ZPG: zero-page, next byte is the low bits of the memory address
	// ZPX: zero-page, X, add next byte to X modulo 0xFF and use that as a
	//		memory address
	opADCABS OpCode = 0x6D
	opADCABX OpCode = 0x7D
	opADCABY OpCode = 0x79
	opADCIMM OpCode = 0x69
	opADCINX OpCode = 0x61
	opADCINY OpCode = 0x71
	opADCZPG OpCode = 0x65
	opADCZPX OpCode = 0x75

	opANDABS OpCode = 0x2D
	opANDABX OpCode = 0x3D
	opANDABY OpCode = 0x39
	opANDIMM OpCode = 0x29
	opANDINX OpCode = 0x21
	opANDINY OpCode = 0x31
	opANDZPG OpCode = 0x25
	opANDZPX OpCode = 0x35

	opASLABS OpCode = 0x0E
	opASLABX OpCode = 0x1E
	opASLACC OpCode = 0x0A
	opASLZP  OpCode = 0x06
	opASLZPX OpCode = 0x16

	opBCCREL OpCode = 0x90
	opBCSREL OpCode = 0xB0
	opBEQREL OpCode = 0xF0

	opBITABS OpCode = 0x2C
	opBITZPG OpCode = 0x24

	opBMIREL OpCode = 0x30
	opBNEREL OpCode = 0xD0
	opBPLREL OpCode = 0x10

	opBRK OpCode = 0x00

	opBVCREL OpCode = 0x50
	opBVSREL OpCode = 0x70

	opCLC OpCode = 0x18
	opCLD OpCode = 0xD8
	opCLI OpCode = 0x58
	opCLV OpCode = 0xB8

	opCMPABS OpCode = 0xCD
	opCMPABX OpCode = 0xDD
	opCMPABY OpCode = 0xD9
	opCMPIMM OpCode = 0xC9
	opCMPINX OpCode = 0xC1
	opCMPINY OpCode = 0xD1
	opCMPZPG OpCode = 0xC5
	opCMPZPX OpCode = 0xD5

	opCPXABS OpCode = 0xEC
	opCPXIMM OpCode = 0xE0
	opCPXZPG OpCode = 0xE4

	opCPYABS OpCode = 0xCC
	opCPYIMM OpCode = 0xC0
	opCPYZPG OpCode = 0xC4

	opDECABS OpCode = 0xCE
	opDECABX OpCode = 0xDE
	opDECZPG OpCode = 0xC6
	opDECZPX OpCode = 0xD6

	opDEX OpCode = 0xCA
	opDEY OpCode = 0x88

	opEORABS OpCode = 0x4D
	opEORABX OpCode = 0x5D
	opEORABY OpCode = 0x59
	opEORIMM OpCode = 0x49
	opEORINX OpCode = 0x41
	opEORINY OpCode = 0x51
	opEORZPG OpCode = 0x45
	opEORZPX OpCode = 0x55

	opINCABS OpCode = 0xEE
	opINCABX OpCode = 0xFE
	opINCZPG OpCode = 0xE6
	opINCZPX OpCode = 0xF6

	opINX OpCode = 0xE8
	opINY OpCode = 0xC8

	opJMPABS OpCode = 0x4C
	opJMPIND OpCode = 0x6C

	opJSRAB OpCode = 0x20

	opLDAABS OpCode = 0xAD
	opLDAABX OpCode = 0xBD
	opLDAABY OpCode = 0xB9
	opLDAIMM OpCode = 0xA9
	opLDAINX OpCode = 0xA1
	opLDAINY OpCode = 0xB1
	opLDAZPG OpCode = 0xA5
	opLDAZPX OpCode = 0xB5

	opLDXABS OpCode = 0xAE
	opLDXABY OpCode = 0xBE
	opLDXIMM OpCode = 0xA2
	opLDXZPG OpCode = 0xA6
	opLDXZPY OpCode = 0xB6

	opLDYABS OpCode = 0xAC
	opLDYABX OpCode = 0xBC
	opLDYIMM OpCode = 0xA0
	opLDYZPG OpCode = 0xA4
	opLDYZPX OpCode = 0xB4

	opLSRABS OpCode = 0x4E
	opLSRABX OpCode = 0x5E
	opLSRACC OpCode = 0x4A
	opLSRZPG OpCode = 0x46
	opLSRZPX OpCode = 0x56

	opORAIMM OpCode = 0x09
	opORAZPG OpCode = 0x05
	opORAZPX OpCode = 0x15
	opORAABS OpCode = 0x0D
	opORAABX OpCode = 0x1D
	opORAABY OpCode = 0x19
	opORAINX OpCode = 0x01
	opORAINY OpCode = 0x11

	opNOP OpCode = 0xEA

	opPHA OpCode = 0x48
	opPHP OpCode = 0x08
	opPLA OpCode = 0x68
	opPLP OpCode = 0x28

	opROLABS OpCode = 0x2E
	opROLABX OpCode = 0x3E
	opROLACC OpCode = 0x2A
	opROLZPG OpCode = 0x26
	opROLZPX OpCode = 0x36

	opRORABS OpCode = 0x6E
	opRORABX OpCode = 0x7E
	opRORACC OpCode = 0x6A
	opRORZPG OpCode = 0x66
	opRORZPX OpCode = 0x76

	opRTI OpCode = 0x40
	opRTS OpCode = 0x60

	opSBCIMM OpCode = 0xE9
	opSBCZPG OpCode = 0xE5
	opSBCZPX OpCode = 0xF5
	opSBCABS OpCode = 0xED
	opSBCABX OpCode = 0xFD
	opSBCABY OpCode = 0xF9
	opSBCINX OpCode = 0xE1
	opSBCINY OpCode = 0xF1

	opSEC OpCode = 0x38
	opSED OpCode = 0xF8
	opSEI OpCode = 0x78

	opSTAABS OpCode = 0x8D
	opSTAABX OpCode = 0x9D
	opSTAABY OpCode = 0x99
	opSTAINX OpCode = 0x81
	opSTAINY OpCode = 0x91
	opSTAZPG OpCode = 0x85
	opSTAZPX OpCode = 0x95

	opSTXZPG OpCode = 0x86
	opSTXZPY OpCode = 0x96
	opSTXABS OpCode = 0x8E

	opSTYZPG OpCode = 0x84
	opSTYZPX OpCode = 0x94
	opSTYABS OpCode = 0x8C

	opTAX OpCode = 0xAA
	opTAY OpCode = 0xA8
	opTSX OpCode = 0xBA
	opTXA OpCode = 0x8A
	opTXS OpCode = 0x9A
	opTYA OpCode = 0x98

	opWAI OpCode = 0xCB
)

var (
	// http://www.thealmightyguru.com/Games/Hacking/Wiki/index.php/6502_Opcodes
	// HexToMnemonic holds mnemonic representation of an opcode, indexed by hex value of the opcode
	HexToMnemonic = []string{
		OpMnemonicBRK, // 0x00
		OpMnemonicORA, // 0x01
		OpMnemonicERR, // 0x02
		OpMnemonicERR, // 0x03
		OpMnemonicERR, // 0x04
		OpMnemonicORA, // 0x05
		OpMnemonicASL, // 0x06
		OpMnemonicERR, // 0x07
		OpMnemonicPHP, // 0x08
		OpMnemonicORA, // 0x09
		OpMnemonicASL, // 0x0A
		OpMnemonicERR, // 0x0B
		OpMnemonicERR, // 0x0C
		OpMnemonicORA, // 0x0D
		OpMnemonicASL, // 0x0E
		OpMnemonicERR, // 0x0F
		OpMnemonicBPL, // 0x10
		OpMnemonicORA, // 0x11
		OpMnemonicERR, // 0x12
		OpMnemonicERR, // 0x13
		OpMnemonicERR, // 0x14
		OpMnemonicORA, // 0x15
		OpMnemonicASL, // 0x16
		OpMnemonicERR, // 0x17
		OpMnemonicCLC, // 0x18
		OpMnemonicORA, // 0x19
		OpMnemonicERR, // 0x1A
		OpMnemonicERR, // 0x1B
		OpMnemonicERR, // 0x1C
		OpMnemonicORA, // 0x1D
		OpMnemonicASL, // 0x1E
		OpMnemonicERR, // 0x1F
		OpMnemonicJSR, // 0x20
		OpMnemonicAND, // 0x21
		OpMnemonicERR, // 0x22
		OpMnemonicERR, // 0x23
		OpMnemonicBIT, // 0x24
		OpMnemonicAND, // 0x25
		OpMnemonicROL, // 0x26
		OpMnemonicERR, // 0x27
		OpMnemonicPLP, // 0x28
		OpMnemonicAND, // 0x29
		OpMnemonicROL, // 0x2A
		OpMnemonicERR, // 0x2B
		OpMnemonicBIT, // 0x2C
		OpMnemonicAND, // 0x2D
		OpMnemonicROL, // 0x2E
		OpMnemonicERR, // 0x2F
		OpMnemonicBMI, // 0x30
		OpMnemonicAND, // 0x31
		OpMnemonicERR, // 0x32
		OpMnemonicERR, // 0x33
		OpMnemonicERR, // 0x34
		OpMnemonicAND, // 0x35
		OpMnemonicROL, // 0x36
		OpMnemonicERR, // 0x37
		OpMnemonicSEC, // 0x38
		OpMnemonicAND, // 0x39
		OpMnemonicERR, // 0x3A
		OpMnemonicERR, // 0x3B
		OpMnemonicERR, // 0x3C
		OpMnemonicAND, // 0x3D
		OpMnemonicROL, // 0x3E
		OpMnemonicERR, // 0x3F
		OpMnemonicRTI, // 0x40
		OpMnemonicEOR, // 0x41
		OpMnemonicERR, // 0x42
		OpMnemonicERR, // 0x43
		OpMnemonicERR, // 0x44
		OpMnemonicEOR, // 0x45
		OpMnemonicLSR, // 0x46
		OpMnemonicERR, // 0x47
		OpMnemonicPHA, // 0x48
		OpMnemonicEOR, // 0x49
		OpMnemonicLSR, // 0x4A
		OpMnemonicERR, // 0x4B
		OpMnemonicJMP, // 0x4C
		OpMnemonicEOR, // 0x4D
		OpMnemonicLSR, // 0x4E
		OpMnemonicERR, // 0x4F
		OpMnemonicBVC, // 0x50
		OpMnemonicEOR, // 0x51
		OpMnemonicERR, // 0x52
		OpMnemonicERR, // 0x53
		OpMnemonicERR, // 0x54
		OpMnemonicEOR, // 0x55
		OpMnemonicLSR, // 0x56
		OpMnemonicERR, // 0x57
		OpMnemonicCLI, // 0x58
		OpMnemonicEOR, // 0x59
		OpMnemonicERR, // 0x5A
		OpMnemonicERR, // 0x5B
		OpMnemonicERR, // 0x5C
		OpMnemonicEOR, // 0x5D
		OpMnemonicLSR, // 0x5E
		OpMnemonicERR, // 0x5F
		OpMnemonicRTS, // 0x60
		OpMnemonicADC, // 0x61
		OpMnemonicERR, // 0x62
		OpMnemonicERR, // 0x63
		OpMnemonicERR, // 0x64
		OpMnemonicADC, // 0x65
		OpMnemonicROR, // 0x66
		OpMnemonicERR, // 0x67
		OpMnemonicPLA, // 0x68
		OpMnemonicADC, // 0x69
		OpMnemonicROR, // 0x6A
		OpMnemonicERR, // 0x6B
		OpMnemonicJMP, // 0x6C
		OpMnemonicADC, // 0x6D
		OpMnemonicROR, // 0x6E
		OpMnemonicERR, // 0x6F
		OpMnemonicBVS, // 0x70
		OpMnemonicADC, // 0x71
		OpMnemonicERR, // 0x72
		OpMnemonicERR, // 0x73
		OpMnemonicERR, // 0x74
		OpMnemonicADC, // 0x75
		OpMnemonicROR, // 0x76
		OpMnemonicERR, // 0x77
		OpMnemonicSEI, // 0x78
		OpMnemonicADC, // 0x79
		OpMnemonicERR, // 0x7A
		OpMnemonicERR, // 0x7B
		OpMnemonicERR, // 0x7C
		OpMnemonicADC, // 0x7D
		OpMnemonicROR, // 0x7E
		OpMnemonicERR, // 0x7F
		OpMnemonicERR, // 0x80
		OpMnemonicSTA, // 0x81
		OpMnemonicERR, // 0x82
		OpMnemonicERR, // 0x83
		OpMnemonicSTY, // 0x84
		OpMnemonicSTA, // 0x85
		OpMnemonicSTX, // 0x86
		OpMnemonicERR, // 0x87
		OpMnemonicDEY, // 0x88
		OpMnemonicERR, // 0x89
		OpMnemonicTXA, // 0x8A
		OpMnemonicERR, // 0x8B
		OpMnemonicSTY, // 0x8C
		OpMnemonicSTA, // 0x8D
		OpMnemonicSTX, // 0x8E
		OpMnemonicERR, // 0x8F
		OpMnemonicBCC, // 0x90
		OpMnemonicSTA, // 0x91
		OpMnemonicERR, // 0x92
		OpMnemonicERR, // 0x93
		OpMnemonicSTY, // 0x94
		OpMnemonicSTA, // 0x95
		OpMnemonicSTX, // 0x96
		OpMnemonicERR, // 0x97
		OpMnemonicTYA, // 0x98
		OpMnemonicSTA, // 0x99
		OpMnemonicTXS, // 0x9A
		OpMnemonicERR, // 0x9B
		OpMnemonicERR, // 0x9C
		OpMnemonicSTA, // 0x9D
		OpMnemonicERR, // 0x9E
		OpMnemonicERR, // 0x9F
		OpMnemonicLDY, // 0xA0
		OpMnemonicLDA, // 0xA1
		OpMnemonicLDX, // 0xA2
		OpMnemonicERR, // 0xA3
		OpMnemonicLDY, // 0xA4
		OpMnemonicLDA, // 0xA5
		OpMnemonicLDX, // 0xA6
		OpMnemonicERR, // 0xA7
		OpMnemonicTAY, // 0xA8
		OpMnemonicLDA, // 0xA9
		OpMnemonicTAX, // 0xAA
		OpMnemonicERR, // 0xAB
		OpMnemonicLDY, // 0xAC
		OpMnemonicLDA, // 0xAD
		OpMnemonicLDX, // 0xAE
		OpMnemonicERR, // 0xAF
		OpMnemonicBCS, // 0xB0
		OpMnemonicLDA, // 0xB1
		OpMnemonicERR, // 0xB2
		OpMnemonicERR, // 0xB3
		OpMnemonicLDY, // 0xB4
		OpMnemonicLDA, // 0xB5
		OpMnemonicLDX, // 0xB6
		OpMnemonicERR, // 0xB7
		OpMnemonicCLV, // 0xB8
		OpMnemonicLDA, // 0xB9
		OpMnemonicTSX, // 0xBA
		OpMnemonicERR, // 0xBB
		OpMnemonicLDY, // 0xBC
		OpMnemonicLDA, // 0xBD
		OpMnemonicLDX, // 0xBE
		OpMnemonicERR, // 0xBF
		OpMnemonicCPY, // 0xC0
		OpMnemonicCMP, // 0xC1
		OpMnemonicERR, // 0xC2
		OpMnemonicERR, // 0xC3
		OpMnemonicCPY, // 0xC4
		OpMnemonicCMP, // 0xC5
		OpMnemonicDEC, // 0xC6
		OpMnemonicERR, // 0xC7
		OpMnemonicINY, // 0xC8
		OpMnemonicCMP, // 0xC9
		OpMnemonicDEX, // 0xCA
		OpMnemonicERR, // 0xCB
		OpMnemonicCPY, // 0xCC
		OpMnemonicCMP, // 0xCD
		OpMnemonicDEC, // 0xCE
		OpMnemonicERR, // 0xCF
		OpMnemonicBNE, // 0xD0
		OpMnemonicCMP, // 0xD1
		OpMnemonicERR, // 0xD2
		OpMnemonicERR, // 0xD3
		OpMnemonicERR, // 0xD4
		OpMnemonicCMP, // 0xD5
		OpMnemonicDEC, // 0xD6
		OpMnemonicERR, // 0xD7
		OpMnemonicCLD, // 0xD8
		OpMnemonicCMP, // 0xD9
		OpMnemonicERR, // 0xDA
		OpMnemonicERR, // 0xDB
		OpMnemonicERR, // 0xDC
		OpMnemonicCMP, // 0xDD
		OpMnemonicDEC, // 0xDE
		OpMnemonicERR, // 0xDF
		OpMnemonicCPX, // 0xE0
		OpMnemonicSBC, // 0xE1
		OpMnemonicERR, // 0xE2
		OpMnemonicERR, // 0xE3
		OpMnemonicCPX, // 0xE4
		OpMnemonicSBC, // 0xE5
		OpMnemonicINC, // 0xE6
		OpMnemonicERR, // 0xE7
		OpMnemonicINX, // 0xE8
		OpMnemonicSBC, // 0xE9
		OpMnemonicNOP, // 0xEA
		OpMnemonicERR, // 0xEB
		OpMnemonicCPX, // 0xEC
		OpMnemonicSBC, // 0xED
		OpMnemonicINC, // 0xEE
		OpMnemonicERR, // 0xEF
		OpMnemonicBEQ, // 0xF0
		OpMnemonicSBC, // 0xF1
		OpMnemonicERR, // 0xF2
		OpMnemonicERR, // 0xF3
		OpMnemonicERR, // 0xF4
		OpMnemonicSBC, // 0xF5
		OpMnemonicINC, // 0xF6
		OpMnemonicERR, // 0xF7
		OpMnemonicSED, // 0xF8
		OpMnemonicSBC, // 0xF9
		OpMnemonicERR, // 0xFA
		OpMnemonicERR, // 0xFB
		OpMnemonicERR, // 0xFC
		OpMnemonicSBC, // 0xFD
		OpMnemonicINC, // 0xFE
		OpMnemonicERR, // 0xFF
	}
)
