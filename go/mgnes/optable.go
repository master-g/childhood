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

func newInstructionSet() []*Instruction {
	lookup := []*Instruction{
		{"BRK", opBRK, amIMM, 7}, {"ORA", opORA, amIZX, 6}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 3}, {"ORA", opORA, amZP0, 3}, {"ASL", opASL, amZP0, 5}, {"???", opXXX, amIMP, 5}, {"PHP", opPHP, amIMP, 3}, {"ORA", opORA, amIMM, 2}, {"ASL", opASL, amIMP, 2}, {"???", opXXX, amIMP, 2}, {"???", opNOP, amIMP, 4}, {"ORA", opORA, amABS, 4}, {"ASL", opASL, amABS, 6}, {"???", opXXX, amIMP, 6},
		{"BPL", opBPL, amREL, 2}, {"ORA", opORA, amIZY, 5}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 4}, {"ORA", opORA, amZPX, 4}, {"ASL", opASL, amZPX, 6}, {"???", opXXX, amIMP, 6}, {"CLC", opCLC, amIMP, 2}, {"ORA", opORA, amABY, 4}, {"???", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 7}, {"???", opNOP, amIMP, 4}, {"ORA", opORA, amABX, 4}, {"ASL", opASL, amABX, 7}, {"???", opXXX, amIMP, 7},
		{"JSR", opJSR, amABS, 6}, {"AND", opAND, amIZX, 6}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"BIT", opBIT, amZP0, 3}, {"AND", opAND, amZP0, 3}, {"ROL", opROL, amZP0, 5}, {"???", opXXX, amIMP, 5}, {"PLP", opPLP, amIMP, 4}, {"AND", opAND, amIMM, 2}, {"ROL", opROL, amIMP, 2}, {"???", opXXX, amIMP, 2}, {"BIT", opBIT, amABS, 4}, {"AND", opAND, amABS, 4}, {"ROL", opROL, amABS, 6}, {"???", opXXX, amIMP, 6},
		{"BMI", opBMI, amREL, 2}, {"AND", opAND, amIZY, 5}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 4}, {"AND", opAND, amZPX, 4}, {"ROL", opROL, amZPX, 6}, {"???", opXXX, amIMP, 6}, {"SEC", opSEC, amIMP, 2}, {"AND", opAND, amABY, 4}, {"???", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 7}, {"???", opNOP, amIMP, 4}, {"AND", opAND, amABX, 4}, {"ROL", opROL, amABX, 7}, {"???", opXXX, amIMP, 7},
		{"RTI", opRTI, amIMP, 6}, {"EOR", opEOR, amIZX, 6}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 3}, {"EOR", opEOR, amZP0, 3}, {"LSR", opLSR, amZP0, 5}, {"???", opXXX, amIMP, 5}, {"PHA", opPHA, amIMP, 3}, {"EOR", opEOR, amIMM, 2}, {"LSR", opLSR, amIMP, 2}, {"???", opXXX, amIMP, 2}, {"JMP", opJMP, amABS, 3}, {"EOR", opEOR, amABS, 4}, {"LSR", opLSR, amABS, 6}, {"???", opXXX, amIMP, 6},
		{"BVC", opBVC, amREL, 2}, {"EOR", opEOR, amIZY, 5}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 4}, {"EOR", opEOR, amZPX, 4}, {"LSR", opLSR, amZPX, 6}, {"???", opXXX, amIMP, 6}, {"CLI", opCLI, amIMP, 2}, {"EOR", opEOR, amABY, 4}, {"???", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 7}, {"???", opNOP, amIMP, 4}, {"EOR", opEOR, amABX, 4}, {"LSR", opLSR, amABX, 7}, {"???", opXXX, amIMP, 7},
		{"RTS", opRTS, amIMP, 6}, {"ADC", opADC, amIZX, 6}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 3}, {"ADC", opADC, amZP0, 3}, {"ROR", opROR, amZP0, 5}, {"???", opXXX, amIMP, 5}, {"PLA", opPLA, amIMP, 4}, {"ADC", opADC, amIMM, 2}, {"ROR", opROR, amIMP, 2}, {"???", opXXX, amIMP, 2}, {"JMP", opJMP, amIND, 5}, {"ADC", opADC, amABS, 4}, {"ROR", opROR, amABS, 6}, {"???", opXXX, amIMP, 6},
		{"BVS", opBVS, amREL, 2}, {"ADC", opADC, amIZY, 5}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 4}, {"ADC", opADC, amZPX, 4}, {"ROR", opROR, amZPX, 6}, {"???", opXXX, amIMP, 6}, {"SEI", opSEI, amIMP, 2}, {"ADC", opADC, amABY, 4}, {"???", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 7}, {"???", opNOP, amIMP, 4}, {"ADC", opADC, amABX, 4}, {"ROR", opROR, amABX, 7}, {"???", opXXX, amIMP, 7},
		{"???", opNOP, amIMP, 2}, {"STA", opSTA, amIZX, 6}, {"???", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 6}, {"STY", opSTY, amZP0, 3}, {"STA", opSTA, amZP0, 3}, {"STX", opSTX, amZP0, 3}, {"???", opXXX, amIMP, 3}, {"DEY", opDEY, amIMP, 2}, {"???", opNOP, amIMP, 2}, {"TXA", opTXA, amIMP, 2}, {"???", opXXX, amIMP, 2}, {"STY", opSTY, amABS, 4}, {"STA", opSTA, amABS, 4}, {"STX", opSTX, amABS, 4}, {"???", opXXX, amIMP, 4},
		{"BCC", opBCC, amREL, 2}, {"STA", opSTA, amIZY, 6}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 6}, {"STY", opSTY, amZPX, 4}, {"STA", opSTA, amZPX, 4}, {"STX", opSTX, amZPY, 4}, {"???", opXXX, amIMP, 4}, {"TYA", opTYA, amIMP, 2}, {"STA", opSTA, amABY, 5}, {"TXS", opTXS, amIMP, 2}, {"???", opXXX, amIMP, 5}, {"???", opNOP, amIMP, 5}, {"STA", opSTA, amABX, 5}, {"???", opXXX, amIMP, 5}, {"???", opXXX, amIMP, 5},
		{"LDY", opLDY, amIMM, 2}, {"LDA", opLDA, amIZX, 6}, {"LDX", opLDX, amIMM, 2}, {"???", opXXX, amIMP, 6}, {"LDY", opLDY, amZP0, 3}, {"LDA", opLDA, amZP0, 3}, {"LDX", opLDX, amZP0, 3}, {"???", opXXX, amIMP, 3}, {"TAY", opTAY, amIMP, 2}, {"LDA", opLDA, amIMM, 2}, {"TAX", opTAX, amIMP, 2}, {"???", opXXX, amIMP, 2}, {"LDY", opLDY, amABS, 4}, {"LDA", opLDA, amABS, 4}, {"LDX", opLDX, amABS, 4}, {"???", opXXX, amIMP, 4},
		{"BCS", opBCS, amREL, 2}, {"LDA", opLDA, amIZY, 5}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 5}, {"LDY", opLDY, amZPX, 4}, {"LDA", opLDA, amZPX, 4}, {"LDX", opLDX, amZPY, 4}, {"???", opXXX, amIMP, 4}, {"CLV", opCLV, amIMP, 2}, {"LDA", opLDA, amABY, 4}, {"TSX", opTSX, amIMP, 2}, {"???", opXXX, amIMP, 4}, {"LDY", opLDY, amABX, 4}, {"LDA", opLDA, amABX, 4}, {"LDX", opLDX, amABY, 4}, {"???", opXXX, amIMP, 4},
		{"CPY", opCPY, amIMM, 2}, {"CMP", opCMP, amIZX, 6}, {"???", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"CPY", opCPY, amZP0, 3}, {"CMP", opCMP, amZP0, 3}, {"DEC", opDEC, amZP0, 5}, {"???", opXXX, amIMP, 5}, {"INY", opINY, amIMP, 2}, {"CMP", opCMP, amIMM, 2}, {"DEX", opDEX, amIMP, 2}, {"???", opXXX, amIMP, 2}, {"CPY", opCPY, amABS, 4}, {"CMP", opCMP, amABS, 4}, {"DEC", opDEC, amABS, 6}, {"???", opXXX, amIMP, 6},
		{"BNE", opBNE, amREL, 2}, {"CMP", opCMP, amIZY, 5}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 4}, {"CMP", opCMP, amZPX, 4}, {"DEC", opDEC, amZPX, 6}, {"???", opXXX, amIMP, 6}, {"CLD", opCLD, amIMP, 2}, {"CMP", opCMP, amABY, 4}, {"NOP", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 7}, {"???", opNOP, amIMP, 4}, {"CMP", opCMP, amABX, 4}, {"DEC", opDEC, amABX, 7}, {"???", opXXX, amIMP, 7},
		{"CPX", opCPX, amIMM, 2}, {"SBC", opSBC, amIZX, 6}, {"???", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"CPX", opCPX, amZP0, 3}, {"SBC", opSBC, amZP0, 3}, {"INC", opINC, amZP0, 5}, {"???", opXXX, amIMP, 5}, {"INX", opINX, amIMP, 2}, {"SBC", opSBC, amIMM, 2}, {"NOP", opNOP, amIMP, 2}, {"???", opSBC, amIMP, 2}, {"CPX", opCPX, amABS, 4}, {"SBC", opSBC, amABS, 4}, {"INC", opINC, amABS, 6}, {"???", opXXX, amIMP, 6},
		{"BEQ", opBEQ, amREL, 2}, {"SBC", opSBC, amIZY, 5}, {"???", opXXX, amIMP, 2}, {"???", opXXX, amIMP, 8}, {"???", opNOP, amIMP, 4}, {"SBC", opSBC, amZPX, 4}, {"INC", opINC, amZPX, 6}, {"???", opXXX, amIMP, 6}, {"SED", opSED, amIMP, 2}, {"SBC", opSBC, amABY, 4}, {"NOP", opNOP, amIMP, 2}, {"???", opXXX, amIMP, 7}, {"???", opNOP, amIMP, 4}, {"SBC", opSBC, amABX, 4}, {"INC", opINC, amABX, 7}, {"???", opXXX, amIMP, 7},
	}
	return lookup
}
