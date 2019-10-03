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

package main

import (
	"fmt"
	"log"
	"mgnes/pkg/mg6502"
	"strings"

	ui "github.com/gizak/termui/v3"
	"github.com/gizak/termui/v3/widgets"
)

var (
	cpu           *mg6502.MG6502
	reader        mg6502.Reader
	disassembly   *mg6502.Disassembly
	paragraphCPU  *widgets.Paragraph
	paragraphCode *widgets.Paragraph
	paragraphRam0 *widgets.Paragraph
	paragraphRam1 *widgets.Paragraph
	paragraphTips *widgets.Paragraph
)

func renderCpu(p *widgets.Paragraph) {
	sb := &strings.Builder{}
	flags := []uint8{
		mg6502.FlagNegative,
		mg6502.FlagOverflow,
		mg6502.FlagUnused,
		mg6502.FlagBreak,
		mg6502.FlagDecimal,
		mg6502.FlagInterrupt,
		mg6502.FlagZero,
		mg6502.FlagCarry,
	}
	symbols := []rune{'N', 'V', '-', 'B', 'D', 'I', 'Z', 'C'}

	sb.WriteString("STATUS: ")
	for i, f := range flags {
		sb.WriteRune('[')
		sb.WriteRune(symbols[i])
		sb.WriteRune(']')
		sb.WriteString("(fg:")
		if cpu.GetFlag(f) != 0 {
			sb.WriteString("green")
		} else {
			sb.WriteString("red")
		}
		sb.WriteString(") ")
	}
	sb.WriteRune('\n')
	sb.WriteString(fmt.Sprintf("PC: $0x%04X SP: $%04X", cpu.PC, cpu.SP))
	sb.WriteRune('\n')
	sb.WriteString(fmt.Sprintf("A: $%02X [%d]", cpu.A, cpu.A))
	sb.WriteRune('\n')
	sb.WriteString(fmt.Sprintf("X: $%02X [%d]", cpu.X, cpu.X))
	sb.WriteRune('\n')
	sb.WriteString(fmt.Sprintf("Y: $%02X [%d] ", cpu.Y, cpu.Y))

	p.Text = sb.String()
}

func renderRam(p *widgets.Paragraph, addr uint16, numRow, numCol int) {
	curAddr := addr
	sb := &strings.Builder{}
	for row := 0; row < numRow; row++ {
		sb.WriteString(fmt.Sprintf("$%04X:", curAddr))
		for col := 0; col < numCol; col++ {
			sb.WriteRune(' ')
			sb.WriteString(fmt.Sprintf("%02X", reader.CpuRead(curAddr, true)))
			curAddr++
		}
		sb.WriteRune('\n')
	}
	p.Text = sb.String()
}

func renderCode(p *widgets.Paragraph) {
	sb := strings.Builder{}
	pc := cpu.PC
	for i := pc - 6; i <= pc+34; i++ {
		if i > 0xFFFF {
			i = i % 0xFFFF
		}
		for j := 0; j < len(disassembly.Index); j++ {
			if disassembly.Index[j] == i {
				line := disassembly.Stringify(i, 32)
				if i == pc {
					sb.WriteString(fmt.Sprintf("[%s](fg:cyan)", line))
				} else {
					sb.WriteString(line)
				}
				sb.WriteRune('\n')
			}
		}
	}
	p.Text = sb.String()
}

func renderTips(p *widgets.Paragraph) {
	p.Text = "SPACE = Step Instruction    R = RESET    I = IRQ    N = NMI"
}

func draw() {
	renderRam(paragraphRam0, 0x0000, 16, 16)
	renderRam(paragraphRam1, 0x8000, 16, 16)
	renderCpu(paragraphCPU)
	renderCode(paragraphCode)
	renderTips(paragraphTips)

	ui.Render(paragraphRam0, paragraphRam1, paragraphCPU, paragraphCode, paragraphCode, paragraphTips)
}

func loadCPU() {
	// create cpu and bus
	cpu = mg6502.NewMG6502()
	if cpu == nil {
		log.Fatal("could not create 6502")
		return
	}

	bus := &PlainBus{
		mem: make([]uint8, 65536),
	}
	cpu.SetWriter(bus)
	cpu.SetReader(bus)
	reader = bus
	bus.Reset()

	// load bytecode
	codes := []byte{0xA2, 0x0A, 0x8E, 0x00, 0x00, 0xA2, 0x03, 0x8E, 0x01, 0x00, 0xAC, 0x00, 0x00, 0xA9, 0x00, 0x18, 0x6D, 0x01, 0x00, 0x88, 0xD0, 0xFA, 0x8D, 0x02, 0x00, 0xEA, 0xEA, 0xEA}
	var offset uint16 = 0x8000
	for i, b := range codes {
		bus.CpuWrite(offset+uint16(i), b)
	}

	// set reset vector
	bus.CpuWrite(0xFFFC, 0x00)
	bus.CpuWrite(0xFFFD, 0x80)

	// disassembly
	disassembly = cpu.Disassemble(0x0000, 0xFFFF)

	// reset
	cpu.Reset()
}

func initLayout() {
	// Ram
	paragraphRam0 = widgets.NewParagraph()
	paragraphRam0.Title = "RAM Page 0x00"
	paragraphRam0.SetRect(0, 0, 56, 18)

	paragraphRam1 = widgets.NewParagraph()
	paragraphRam1.Title = "RAM Page 0x80"
	paragraphRam1.SetRect(0, 18, 56, 36)

	// CPU
	paragraphCPU = widgets.NewParagraph()
	paragraphCPU.Title = "CPU"
	paragraphCPU.SetRect(56, 0, 56+34, 7)

	// Code
	paragraphCode = widgets.NewParagraph()
	paragraphCode.Title = "Disassembly"
	paragraphCode.SetRect(56, 7, 56+34, 7+29)

	// Tips
	paragraphTips = widgets.NewParagraph()
	paragraphTips.Title = "Tips"
	paragraphTips.SetRect(0, 36, 56+34, 39)
}

func main() {
	if err := ui.Init(); err != nil {
		log.Fatalf("failed to initialize termui: %v", err)
	}
	defer ui.Close()

	initLayout()
	loadCPU()

	draw()

	for e := range ui.PollEvents() {
		if e.Type == ui.KeyboardEvent {
			if e.ID == "q" || e.ID == "Q" || e.ID == "<C-c>" {
				break
			} else if e.ID == "<Space>" {
				cpu.Clock()
				for !cpu.Complete() {
					cpu.Clock()
				}
			} else if e.ID == "r" || e.ID == "R" {
				cpu.Reset()
			} else if e.ID == "i" || e.ID == "I" {
				cpu.IRQ()
			} else if e.ID == "n" || e.ID == "N" {
				cpu.NMI()
			}
			draw()
		}
	}
}
